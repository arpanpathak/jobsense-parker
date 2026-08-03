//! # Auto-Apply Module
//!
//! **Actually** fills job application forms using Chrome DevTools Protocol.
//! Launches Chrome in visible mode, navigates to the job URL, and injects
//! JavaScript that detects and fills form fields with your profile data.
//!
//! No AppleScript. No browser detection. Works on macOS, Linux, Windows.
//!
//! # Flow
//!
//! 1. User presses `a` on a job
//! 2. Load profile from preferences (name, email, phone, etc.)
//! 3. If incomplete, prompt user to set it up first
//! 4. Launch Chrome visible → navigate to URL → inject fill JS
//! 5. User sees the filled form, reviews, clicks Submit

use colored::Colorize;

use crate::cli::open_url;
use crate::storage;

// ─── Public API ────────────────────────────────────────────────────────────

/// Auto-apply to a job: open the URL and fill the application form.
///
/// Spawns a Chrome window, navigates to the job URL, and uses CDP to
/// inject JavaScript that fills common form fields from your profile.
/// The Chrome window stays open for you to review and click Submit.
pub fn auto_apply(url: &str, title: &str, company: Option<&str>) -> bool {
    let prefs = match storage::load_preferences() {
        Ok(p) => p,
        Err(_) => {
            println!("  {} Could not load profile.", "!".red());
            return false;
        }
    };

    if prefs.full_name.is_none() || prefs.email.is_none() {
        println!();
        println!(
            "  {} Profile incomplete — need at least name and email for auto-fill.",
            "!".yellow()
        );
        println!(
            "  {} Use 'Set profile' in the menu first, then try again.",
            "→".cyan()
        );
        println!();
        let _ = open_url(url);
        return false;
    }

    let company_str = company
        .map(|c| format!(" @ {}", c.cyan()))
        .unwrap_or_default();
    println!(
        "  {} Auto-filling application for '{}'{}",
        "→".green(),
        title.bright_white(),
        company_str
    );

    let url = url.to_string();
    let name = prefs.full_name.clone().unwrap_or_default();
    let email = prefs.email.clone().unwrap_or_default();
    let phone = prefs.phone.clone().unwrap_or_default();
    let location = prefs.preferred_location.clone().unwrap_or_default();
    let linkedin = prefs.linkedin_url.clone().unwrap_or_default();
    let github = prefs.github_url.clone().unwrap_or_default();

    // Run Chrome automation in a separate thread (headless_chrome is sync)
    std::thread::spawn(move || {
        if let Err(e) = run_chrome_fill(&url, &name, &email, &phone, &location, &linkedin, &github)
        {
            eprintln!("  {} Auto-fill failed: {}. Apply manually.", "!".red(), e);
            // Fallback: just open the URL
            let _ = open_url(&url);
        }
    });

    println!(
        "  {} Chrome opened with form fields filled. Review and click Submit. Good luck!",
        "✓".green()
    );
    true
}

// ─── Chrome CDP Automation ───────────────────────────────────────────────

use headless_chrome::{Browser, LaunchOptions, Tab};

/// Launch Chrome (visible), navigate to the URL, and fill form fields via CDP.
///
/// After filling, the browser is kept ALIVE so the user can review the form
/// and click Submit. Dropping the [`Browser`] closes Chrome immediately — that
/// is why the old window vanished right after filling. This function blocks in
/// a background thread (holding `browser`) until the user closes the window
/// (Chrome exits → CDP disconnects → `get_version()` fails) or a 10-minute cap.
fn run_chrome_fill(
    url: &str,
    name: &str,
    email: &str,
    phone: &str,
    location: &str,
    linkedin: &str,
    github: &str,
) -> anyhow::Result<()> {
    let browser = Browser::new(LaunchOptions {
        headless: false, // Show the browser window!
        sandbox: false,  // Avoid sandbox issues on some systems
        window_size: Some((1280, 900)),
        // Long enough that the event loop doesn't time out while the user
        // reviews the form (we poll below, so this is just a safety margin).
        idle_browser_timeout: std::time::Duration::from_secs(600),
        ..LaunchOptions::default()
    })?;

    let tab = browser.new_tab()?;
    tab.set_default_timeout(std::time::Duration::from_secs(20));
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    let js = build_fill_javascript(name, email, phone, location, linkedin, github);

    // Run the fill a few times: single-page apps (Greenhouse, Lever, ...)
    // render form sections in stages, so a single pass can miss fields.
    let mut filled: u64 = 0;
    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        filled = tab
            .evaluate(&js, false)
            .ok()
            .and_then(|r| r.value)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if filled > 0 {
            break;
        }
    }

    // ── Typing pass (real CDP input) ──────────────────────────────────
    // The JS pass above sets `.value` directly, which some frameworks
    // (Greenhouse, Lever, React ATS apps) wipe on re-render. Real keystrokes
    // via `Input.dispatchKeyEvent` update the framework's state, so typed
    // values STICK. Only empty fields are typed into (fields the JS pass
    // already filled — and that stuck — are left alone).
    let mut name_parts = name.split_whitespace();
    let first_name = name_parts.next().unwrap_or("");
    let last_name = name_parts.collect::<Vec<_>>().join(" ");

    let mut typed = 0u32;
    for (field, value) in [
        ("first_name", first_name),
        ("last_name", last_name.as_str()),
        ("email", email),
        ("phone", phone),
        ("location", location),
        ("linkedin", linkedin),
        ("github", github),
    ] {
        if value.is_empty() {
            continue;
        }
        if type_into_field(&tab, field, value)? {
            typed += 1;
        }
    }
    eprintln!(
        "  {} Filled {filled} field(s) via JS, {typed} via typing. Review and click Submit — the window stays open.",
        "✓".green()
    );

    // ── Keep the browser alive for review ──────────────────────────────
    // Dropping `browser` calls Browser::Close and kills Chrome. Block this
    // background thread (which holds `browser`) until the user closes the
    // window: Chrome exits → the CDP connection drops → get_version() fails.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(600);
    while std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_secs(2));
        if browser.get_version().is_err() {
            break; // user closed the Chrome window
        }
    }

    Ok(())
}

/// Build the JavaScript that finds and fills form fields.
fn build_fill_javascript(
    name: &str,
    email: &str,
    phone: &str,
    location: &str,
    linkedin: &str,
    github: &str,
) -> String {
    // Split name into first/last
    let name_parts: Vec<&str> = name.split_whitespace().collect();
    let first_name = name_parts.first().copied().unwrap_or("");
    let last_name = if name_parts.len() > 1 {
        name_parts[1..].join(" ")
    } else {
        String::new()
    };

    format!(
        r#"(function() {{
    'use strict';
    var filled = 0;

    function setValue(el, value) {{
        if (!el || el.readOnly || el.disabled) return false;
        var tag = el.tagName.toLowerCase();
        var proto = (tag === 'textarea')
            ? window.HTMLTextAreaElement.prototype
            : window.HTMLInputElement.prototype;
        var setter = Object.getOwnPropertyDescriptor(proto, 'value').set;
        if (!setter) return false;
        setter.call(el, value);
        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
        el.dispatchEvent(new Event('blur', {{ bubbles: true }}));
        filled++;
        return true;
    }}

    function tryFill(selector, value) {{
        if (!value) return false;
        try {{
            var el = document.querySelector(selector);
            return setValue(el, value);
        }} catch(e) {{ return false; }}
    }}

    // Matches exact names AND substrings — Greenhouse names fields
    // "job_application[first_name]", so `name*="first_name"` is required.
    function fillByNames(names, value) {{
        if (!value) return;
        for (var i = 0; i < names.length; i++) {{
            var n = names[i];
            if (tryFill('input[name="' + n + '"]', value)) return;
            if (tryFill('input[name*="' + n + '"]', value)) return;
            if (tryFill('textarea[name*="' + n + '"]', value)) return;
            if (tryFill('input[id*="' + n + '"]', value)) return;
            if (tryFill('textarea[id*="' + n + '"]', value)) return;
            if (tryFill('input[placeholder*="' + n + '" i]', value)) return;
            if (tryFill('textarea[placeholder*="' + n + '" i]', value)) return;
            if (tryFill('input[aria-label*="' + n + '" i]', value)) return;
            if (tryFill('textarea[aria-label*="' + n + '" i]', value)) return;
        }}
    }}

    function fillByLabelText(labelText, value) {{
        if (!value) return;
        try {{
            var labels = document.querySelectorAll('label');
            for (var i = 0; i < labels.length; i++) {{
                if (labels[i].textContent.toLowerCase().indexOf(labelText.toLowerCase()) !== -1) {{
                    var forId = labels[i].getAttribute('for');
                    if (forId) {{ if (setValue(document.getElementById(forId), value)) return; }}
                    var input = labels[i].querySelector('input, textarea');
                    if (input) {{ if (setValue(input, value)) return; }}
                }}
            }}
        }} catch(e) {{}}
    }}

    // A single "Full name" field, but NEVER first/last name fields
    // (substring "name" matches both, so skip them explicitly).
    function fillFullName(value) {{
        if (!value) return;
        var fields = document.querySelectorAll('input[name], textarea[name]');
        for (var i = 0; i < fields.length; i++) {{
            var nm = (fields[i].name || '').toLowerCase();
            if (nm.indexOf('first_name') !== -1 || nm.indexOf('last_name') !== -1) continue;
            if (nm === 'name' || nm.indexOf('full_name') !== -1 || nm.indexOf('fullname') !== -1
                || nm.indexOf('your-name') !== -1 || nm.indexOf('applicant_name') !== -1) {{
                if (setValue(fields[i], value)) return;
            }}
        }}
    }}

    // Name fields
    fillByNames(['first_name', 'firstname', 'fname', 'firstName', 'given-name', 'given_name'], '{first_name}');
    fillByNames(['last_name', 'lastname', 'lname', 'lastName', 'family-name', 'family_name', 'surname'], '{last_name}');
    fillFullName('{name}');

    // Email
    fillByNames(['email', 'e-mail', 'emailAddress', 'email_address', 'emailaddress', 'applicant_email'], '{email}');
    fillByLabelText('email', '{email}');

    // Phone
    fillByNames(['phone', 'phoneNumber', 'phone_number', 'phonenumber', 'telephone', 'tel', 'mobile', 'cell'], '{phone}');
    fillByLabelText('phone', '{phone}');

    // Location
    fillByNames(['location', 'city', 'locality', 'location_city'], '{location}');

    // Social / links
    fillByNames(['linkedin', 'linkedin_url', 'linkedin-url', 'linkedinUrl', 'linkedinurl'], '{linkedin}');
    fillByNames(['github', 'github_url', 'github-url', 'githubUrl', 'portfolio', 'website', 'url'], '{github}');

    // Type/autocomplete-based fallbacks for remaining empty fields
    // (React ATS apps like Greenhouse rely on these attributes).
    if ('{email}') {{
        var emailInputs = document.querySelectorAll('input[type="email"], input[autocomplete="email"]');
        for (var i = 0; i < emailInputs.length; i++) {{
            if (!emailInputs[i].value && setValue(emailInputs[i], '{email}')) break;
        }}
    }}

    if ('{phone}') {{
        var telInputs = document.querySelectorAll('input[type="tel"], input[autocomplete="tel"]');
        for (var i = 0; i < telInputs.length; i++) {{
            if (!telInputs[i].value && setValue(telInputs[i], '{phone}')) break;
        }}
    }}

    if ('{first_name}') {{
        var givenInputs = document.querySelectorAll('input[autocomplete="given-name"]');
        for (var i = 0; i < givenInputs.length; i++) {{
            if (setValue(givenInputs[i], '{first_name}')) break;
        }}
    }}

    if ('{last_name}') {{
        var familyInputs = document.querySelectorAll('input[autocomplete="family-name"]');
        for (var i = 0; i < familyInputs.length; i++) {{
            if (setValue(familyInputs[i], '{last_name}')) break;
        }}
    }}

    return filled;
}})()"#,
        first_name = first_name,
        last_name = last_name,
        name = name,
        email = email,
        phone = phone,
        location = location,
        linkedin = linkedin,
        github = github,
    )
}

/// Find a form field matching `field` (name / autocomplete / placeholder /
/// aria-label / label text), focus it, and type `value` with real CDP input.
///
/// Returns `true` if a field was found and typed into.
///
/// Real keystrokes (`Input.dispatchKeyEvent`) are indistinguishable from user
/// input, so ANY framework — React, Greenhouse, Lever, Backbone — updates its
/// state model and keeps the value. (The JS `.value` setter in
/// [`build_fill_javascript`] gets wiped by such frameworks on re-render.)
fn type_into_field(tab: &Tab, field: &str, value: &str) -> anyhow::Result<bool> {
    // Common autocomplete aliases per field (Greenhouse marks first/last name
    // inputs with autocomplete="given-name"/"family-name", not name attrs).
    let autocomplete_hint = match field {
        "first_name" => "input[autocomplete*='given-name'], textarea[autocomplete*='given-name']",
        "last_name" => "input[autocomplete*='family-name'], textarea[autocomplete*='family-name']",
        "email" => "input[type='email']",
        "phone" => "input[type='tel']",
        _ => "",
    };

    let find_js = format!(
        r#"(function() {{
            'use strict';
            var key = "{field}";
            var sels = [
                'input[name="{field}"], textarea[name="{field}"]',
                'input[name*="{field}"], textarea[name*="{field}"]',
                'input[autocomplete*="{field}"], textarea[autocomplete*="{field}"]',
                'input[type="{field}"], textarea[type="{field}"]',
                'input[placeholder*="{field}" i], textarea[placeholder*="{field}" i]',
                'input[aria-label*="{field}" i], textarea[aria-label*="{field}" i]'
            ];
            var hint = "{autocomplete_hint}";
            if (hint) sels.push(hint);
            for (var i = 0; i < sels.length; i++) {{
                var els = document.querySelectorAll(sels[i]);
                for (var j = 0; j < els.length; j++) {{
                    var el = els[j];
                    if (el.readOnly || el.disabled || el.value) continue;
                    el.focus();
                    el.click && el.click();
                    return true;
                }}
            }}
            var labels = document.querySelectorAll('label');
            for (var k = 0; k < labels.length; k++) {{
                if (labels[k].textContent.toLowerCase().indexOf(key.toLowerCase()) !== -1) {{
                    var forId = labels[k].getAttribute('for');
                    var el = forId ? document.getElementById(forId) : labels[k].querySelector('input, textarea');
                    if (el && !el.value && !el.readOnly && !el.disabled) {{
                        el.focus();
                        el.click && el.click();
                        return true;
                    }}
                }}
            }}
            return false;
        }})()"#,
        field = field,
        autocomplete_hint = autocomplete_hint,
    );

    let focused = tab
        .evaluate(&find_js, false)?
        .value
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !focused {
        return Ok(false);
    }

    // Type the value with real key events, then Tab to blur — this fires the
    // framework's change handler (and autosave, in Greenhouse's case).
    tab.type_str(value)?;
    tab.press_key("Tab")?;
    Ok(true)
}
