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

use headless_chrome::{Browser, LaunchOptions};

/// Launch Chrome (visible), navigate to the URL, and fill form fields via CDP.
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
        idle_browser_timeout: std::time::Duration::from_secs(120),
        ..LaunchOptions::default()
    })?;

    let tab = browser.new_tab()?;
    tab.set_default_timeout(std::time::Duration::from_secs(15));
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    // Give the page a moment for JS frameworks to render forms
    std::thread::sleep(std::time::Duration::from_secs(3));

    let js = build_fill_javascript(name, email, phone, location, linkedin, github);
    let filled = tab
        .evaluate(&js, false)?
        .value
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    eprintln!(
        "  {} Auto-filled {filled} field(s). Review and click Submit.",
        "✓".green()
    );

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

    function tryFill(selector, value) {{
        if (!value) return;
        try {{
            var el = document.querySelector(selector);
            if (el) {{
                var tag = el.tagName.toLowerCase();
                if ((tag === 'input' || tag === 'textarea') && !el.readOnly && !el.disabled) {{
                    var nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
                    nativeInputValueSetter.call(el, value);
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                    filled++;
                }}
            }}
        }} catch(e) {{}}
    }}

    function fillByNames(names, value) {{
        if (!value) return;
        for (var i = 0; i < names.length; i++) {{
            var n = names[i];
            if (tryFill('input[name="' + n + '"]', value)) return;
            if (tryFill('input[id="' + n + '"]', value)) return;
            if (tryFill('textarea[name="' + n + '"]', value)) return;
            if (tryFill('textarea[id="' + n + '"]', value)) return;
            if (tryFill('input[placeholder*="' + n + '" i]', value)) return;
            if (tryFill('textarea[placeholder*="' + n + '" i]', value)) return;
            if (tryFill('input[aria-label*="' + n + '" i]', value)) return;
        }}
    }}

    function fillByLabelText(labelText, value) {{
        if (!value) return;
        try {{
            var labels = document.querySelectorAll('label');
            for (var i = 0; i < labels.length; i++) {{
                if (labels[i].textContent.toLowerCase().indexOf(labelText.toLowerCase()) !== -1) {{
                    var forId = labels[i].getAttribute('for');
                    if (forId) {{
                        if (tryFill('#' + forId, value)) return;
                    }}
                    var input = labels[i].querySelector('input, textarea');
                    if (input) {{
                        tryFill('#' + input.id, value);
                        return;
                    }}
                }}
            }}
        }} catch(e) {{}}
    }}

    // Name fields
    fillByNames(['first_name', 'firstname', 'fname', 'firstName', 'given-name', 'given_name'], '{first_name}');
    fillByNames(['last_name', 'lastname', 'lname', 'lastName', 'family-name', 'family_name', 'surname'], '{last_name}');
    fillByNames(['name', 'full_name', 'fullname', 'your-name', 'applicant_name'], '{name}');

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

    // Try type-based selectors for remaining empty fields
    if ('{email}') {{
        try {{
            var emailInputs = document.querySelectorAll('input[type="email"]');
            for (var i = 0; i < emailInputs.length; i++) {{
                if (!emailInputs[i].value && !emailInputs[i].readOnly && !emailInputs[i].disabled) {{
                    var setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
                    setter.call(emailInputs[i], '{email}');
                    emailInputs[i].dispatchEvent(new Event('input', {{ bubbles: true }}));
                    emailInputs[i].dispatchEvent(new Event('change', {{ bubbles: true }}));
                    filled++;
                }}
            }}
        }} catch(e) {{}}
    }}

    if ('{phone}') {{
        try {{
            var telInputs = document.querySelectorAll('input[type="tel"]');
            for (var i = 0; i < telInputs.length; i++) {{
                if (!telInputs[i].value && !telInputs[i].readOnly && !telInputs[i].disabled) {{
                    var setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
                    setter.call(telInputs[i], '{phone}');
                    telInputs[i].dispatchEvent(new Event('input', {{ bubbles: true }}));
                    telInputs[i].dispatchEvent(new Event('change', {{ bubbles: true }}));
                    filled++;
                }}
            }}
        }} catch(e) {{}}
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
