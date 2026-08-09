#!/usr/bin/env python3
"""Convert resume markdown to a Google-themed PDF with color-highlighted metrics."""

import re
import sys
import weasyprint

MD_FILE = sys.argv[1] if len(sys.argv) > 1 else "/Users/arpanpathak/Projects/rust/jobsense-parker/ArpanPathak_Google_Android.md"
THEME_NAME = sys.argv[2] if len(sys.argv) > 2 else "google"
NO_ICONS = "--no-icons" in sys.argv
PDF_FILE = MD_FILE.replace(".md", ".pdf")

with open(MD_FILE, "r") as f:
    md = f.read()


def highlight_metrics(text):
    """Wrap metrics and numbers in colored spans. Percentages get their own class."""
    # Percentages -> gold
    text = re.sub(r'(\d+[.,]?\d*\s*%)', r'<span class="metric-pct">\1</span>', text)
    # Dollar amounts
    text = re.sub(r'(\$\d+[.,]?\d*[KMB]?[/]?(month|year)?)', r'<span class="metric">\1</span>', text)
    # Numbers with performance units (excluding % which is handled above)
    text = re.sub(r'(\d+[.,]?\d*\s*(ops/s|ms|QPS|TPM|writes/second|hourly transactions))', r'<span class="metric">\1</span>', text)
    # Time spans
    text = re.sub(r'(\d+\s*(days?|hours?|minutes?))', r'<span class="metric">\1</span>', text)
    # Numbers with K/M/B suffix
    text = re.sub(r'(\d+[KMB]\++?)(?!\s*(days?|hours?|minutes?|ops/s|ms|QPS|TPM|%))', r'<span class="metric">\1</span>', text)
    return text


def svg_icon(name, color, size=11):
    """Return a clean inline SVG icon (feather-style stroke icons)."""
    paths = {
        "target": '<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/>',
        "briefcase": '<rect x="2" y="7" width="20" height="14" rx="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/>',
        "tool": '<path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>',
        "cap": '<path d="M22 10L12 5 2 10l10 5 10-5z"/><path d="M6 12v5c0 1.66 2.69 3 6 3s6-1.34 6-3v-5"/>',
        "rocket": '<path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/>',
        "mail": '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/>',
        "phone": '<path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/>',
        "pin": '<path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/>',
        "linkedin": '<path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4V8h4v1.5A5.98 5.98 0 0 1 16 8z"/><rect x="2" y="9" width="4" height="12"/><circle cx="4" cy="4" r="2"/>',
        "github": '<path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>',
        "code": '<polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/>',
        "shield": '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>',
        "cpu": '<rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/>',
        "globe": '<circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>',
        "lock": '<rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',
        "server": '<rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/>',
        "plug": '<path d="M12 22v-5"/><path d="M9 8V2"/><path d="M15 8V2"/><path d="M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8z"/>',
        "zap": '<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>',
        "cloud": '<path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/>',
        "database": '<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>',
        "layers": '<polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/>',
        "camera": '<path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/>',
        "book": '<rect x="3" y="4" width="18" height="16" rx="2"/><line x1="12" y1="4" x2="12" y2="20"/>',
        "activity": '<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>',
    }
    p = paths.get(name, "")
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="{size}" height="{size}" '
        f'fill="none" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" '
        f'style="vertical-align:-1.5px;margin-right:4px">{p}</svg>'
    )


def convert(md_text, icon_color="#0f766e", project_color="#0f766e", muted_color="#475569", tech_color="#0f766e", skill_color="#b45309", show_icons=True):
    """Convert markdown to clean HTML with proper link rendering."""
    lines = md_text.split("\n")
    html = []
    in_project_box = False
    contact_open = False
    heading_icons = {"SUMMARY": "target", "EXPERIENCE": "briefcase", "SKILLS": "tool", "EDUCATION": "cap"}
    skill_icons = {
        "Languages": "code",
        "Systems Programming & Performance": "cpu",
        "Networking & Protocols": "globe",
        "Network & Cloud Security": "lock",
        "Distributed Systems & Streaming": "server",
        "APIs & Serialization": "plug",
        "ML Inference & AI": "zap",
        "GPU & Parallel Computing": "cpu",
        "Cloud & Infra": "cloud",
        "Observability": "activity",
        "Data & Storage": "database",
        "Core CS": "layers",
    }

    for line in lines:
        stripped = line.strip()

        if stripped == "::: project-box":
            in_project_box = True
            html.append('<div class="project-box">')
            continue

        if stripped == ":::" and in_project_box:
            in_project_box = False
            html.append('</div>')
            continue

        if line.startswith("# ") and not line.startswith("## ") and not line.startswith("### "):
            html.append(f'<div class="name">{line[2:]}</div>')
            contact_open = True
            html.append('<div class="contact">')

        elif line.startswith("## "):
            title = line[3:]
            ic = heading_icons.get(title.upper(), "target")
            icon = svg_icon(ic, icon_color, 12) if show_icons else ""
            html.append(f'<div class="section-heading">{icon}{title}</div>')

        elif line.startswith("### "):
            content = line[4:]
            parts = [p.strip() for p in content.split("|")]
            title = parts[0] if len(parts) >= 1 else content
            if len(parts) >= 3:
                company = parts[1]
                date = parts[2]
                html.append(f'<div class="job-header"><span class="job-title">{title},</span> <span class="job-company">{company}</span> <span class="job-date">{date}</span></div>')
            elif len(parts) == 2:
                rest = parts[1]
                html.append(f'<div class="job-header"><span class="job-title">{title},</span> <span class="job-company">{rest}</span></div>')
            else:
                html.append(f'<div class="job-header">{content}</div>')

        elif not stripped or stripped == "---":
            if stripped == "---" and contact_open:
                contact_open = False
                html.append('</div>')
            continue

        elif stripped.startswith("- "):
            item = stripped[2:]
            if in_project_box:
                html.append(f'<p class="bullet-sm">\u2022 {item}</p>')
            elif item.startswith("TechStack"):
                label, _, rest = item.partition(":")
                icon = svg_icon("code", tech_color, 10) if show_icons else ""
                html.append(f'<p class="techstack-box">{icon}<span class="techstack-label">{label}</span>:{rest}</p>')
            else:
                html.append(f'<p class="bullet">\u2022 {item}</p>')

        else:
            if in_project_box:
                cls = "project-text"
                if stripped == "PERSONAL PROJECTS":
                    if show_icons:
                        line = svg_icon("rocket", project_color, 11) + line
                elif "github" in line and "[" in line:
                    if "Book" in line or "Physics" in line:
                        if show_icons:
                            line = svg_icon("book", project_color, 15) + line
                        cls = "project-text book-line"
                    elif "Driving-CivicSense" in line or "Vision" in line:
                        if show_icons:
                            line = svg_icon("camera", project_color, 10) + line
                            line = line.replace("[", svg_icon("github", project_color, 10) + "[", 1)
                    elif show_icons:
                        line = line.replace("[", svg_icon("github", project_color, 10) + "[", 1)
                html.append(f'<p class="{cls}">{line}</p>')
            elif contact_open:
                ico = "pin"
                if "Email:" in line:
                    ico = "mail"
                elif "Phone:" in line:
                    ico = "phone"
                elif "LinkedIn:" in line:
                    ico = "linkedin"
                elif "GitHub:" in line:
                    ico = "github"
                elif line.startswith("**Visa"):
                    ico = "shield"
                html.append(f'<div class="contact-item"><span class="contact-ico">{svg_icon(ico, muted_color, 11) if show_icons else ""}</span><span class="contact-txt">{line}</span></div>')
            else:
                if line.startswith("**") and "**: " in line:
                    label = line.split(":", 1)[0].strip("*")
                    ic = skill_icons.get(label, "layers")
                    icon = svg_icon(ic, skill_color, 10) if show_icons else ""
                    line = icon + line
                    html.append(f'<p class="skill-group">{line}</p>')
                elif "github" in line and "[" in line and ("Book" in line or "Physics" in line):
                    if show_icons:
                        line = svg_icon("book", project_color, 15) + line
                    html.append(f'<p class="project-text">{line}</p>')
                elif "Driving-CivicSense" in line or "Vision" in line:
                    if show_icons:
                        line = svg_icon("camera", project_color, 10) + line
                    html.append(f'<p>{line}</p>')
                else:
                    html.append(f'<p>{line}</p>')

    result = "\n".join(html)

    result = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', result)
    result = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', r'<a href="\2">\1</a>', result)

    lines_out = []
    for line in result.split("\n"):
        if not line.strip().startswith("<div class=") and "project" not in line:
            line = highlight_metrics(line)
        lines_out.append(line)
    result = "\n".join(lines_out)

    return result


CSS = """
@page {
    size: letter;
    margin: 0.35in 0.55in;
    background: __BG__;
}
body {
    font-family: 'Avenir Next', 'Avenir', 'Helvetica Neue', Helvetica, Arial, sans-serif;
    font-size: 10pt;
    line-height: 1.28;
    color: __TEXT__;
    background: __BG__;
}

.name {
    font-size: 24pt;
    font-weight: 600;
    color: __NAME__;
    margin-bottom: 2px;
    letter-spacing: 0.5px;
}

.contact {
    margin: 2px 0 4px 0;
}
.contact-item {
    display: flex;
    align-items: center;
    margin: 1px 0;
    font-size: 9.5pt;
    line-height: 1.45;
}
.contact-ico {
    flex: 0 0 16px;
    text-align: center;
    margin-right: 5px;
}
.no-icons .contact-ico {
    display: none;
}
.contact-ico svg {
    vertical-align: middle;
}
.contact-txt {
    flex: 1;
}

.project-box {
    float: right;
    width: 48%;
    margin-left: 14px;
    margin-bottom: 10px;
    border: 1.5px solid __PROJECT__;
    border-radius: 8px;
    padding: 9px 12px;
    background: __BOX_BG__;
    box-shadow: __SHADOW__;
}
.project-box p:first-child {
    font-size: 8.5pt;
    font-weight: 700;
    color: __PROJECT__;
    text-transform: uppercase;
    letter-spacing: 1.2px;
    margin: 0 0 4px 0;
}
p.project-text {
    font-size: 8pt;
    line-height: 1.3;
    margin: 3px 0;
    color: __MUTED__;
}
p.project-text strong {
    font-size: 8.5pt;
    color: __PROJECT__;
}
p.book-line {
    margin: 6px 0 6px 2px;
    padding: 5px 10px;
    border: 1px solid __CHIP_BORDER__;
    border-left: 4px solid __PROJECT__;
    border-radius: 8px;
    background: __CHIP_BG__;
    font-size: 7.5pt;
    line-height: 1.34;
}
p.book-line svg {
    vertical-align: -2px;
    margin-right: 4px;
}
p.book-line strong {
    font-size: 8pt;
    color: __PROJECT__;
}
p.bullet-sm {
    margin: 1.5px 0 1.5px 12px;
    font-size: 8pt;
    line-height: 1.3;
    text-indent: -7px;
    color: __MUTED__;
}
p.bullet-sm strong {
    font-size: 8pt;
    color: __PROJECT__;
}
.project-box a {
    color: __LINK__;
    font-size: 8pt;
}

.section-heading {
    font-size: 11pt;
    font-weight: 600;
    color: __NAME__;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    border-bottom: 2px solid __ACCENT__;
    padding: 4px 10px;
    margin-top: 18px;
    margin-bottom: 8px;
    border-radius: 4px;
    box-shadow: __HEADER_SHADOW__;
    background: linear-gradient(90deg, __HEADING_BG__ 55%, transparent);
}

.job-header {
    display: flex;
    align-items: baseline;
    column-gap: 6px;
    font-size: 10.5pt;
    margin: 8px 0 3px 0;
    line-height: 1.28;
    padding: 3px 8px;
    border-radius: 6px;
    border-left: 3px solid __ACCENT__;
    background: __HEADER_BG__;
    box-shadow: __HEADER_SHADOW__;
}
.job-title {
    font-style: italic;
    font-weight: 600;
    color: __STRONG__;
}
.job-company {
    font-weight: 600;
    color: __ACCENT__;
    font-size: 10pt;
}
.job-date {
    font-weight: 400;
    color: __DATE__;
    font-size: 9.5pt;
    margin-left: auto;
    white-space: nowrap;
}

p {
    margin: 1px 0;
    font-size: 9.5pt;
    line-height: 1.3;
}

p.skill-group {
    margin: 4px 0 2px 0;
}

p.skill-group strong {
    color: __SKILL__;
}

p.bullet {
    margin: 1px 0 1px 16px;
    font-size: 9.5pt;
    line-height: 1.26;
    text-indent: -8px;
}

p.techstack-box {
    margin: 3px 0 5px 16px;
    line-height: 1.32;
}
span.techstack-label {
    font-style: italic;
    font-weight: 600;
    color: __TECH_LABEL__;
}
p.techstack-box strong {
    color: __TECH__;
    font-weight: 600;
}

strong {
    font-style: italic;
    font-weight: 600;
    color: __STRONG__;
}

a {
    color: __LINK__;
    text-decoration: underline;
}

span.metric {
    color: __METRIC__;
    font-weight: 700;
    background: __METRIC_BG__;
    padding: 0 3px;
    border-radius: 3px;
}

span.metric-pct {
    color: __METRIC_PCT__;
    font-weight: 700;
    background: __METRIC_PCT_BG__;
    padding: 0 3px;
    border-radius: 3px;
}

/* ATS single-column mode: slightly denser so content fits 2 pages */
.no-icons p {
    font-size: 9pt;
    line-height: 1.24;
}
.no-icons p.bullet {
    font-size: 9pt;
    line-height: 1.22;
}
.no-icons .job-header {
    font-size: 10pt;
    margin: 6px 0 2px 0;
}
.no-icons .section-heading {
    margin-top: 9px;
    padding: 3px 10px;
    margin-bottom: 5px;
}
.no-icons .contact-item {
    font-size: 9pt;
}
.no-icons .skill-group {
    margin: 3px 0 2px 0;
}
"""

THEMES = {
    "google": {
        "BG": "#ffffff", "TEXT": "#1a1a1a", "NAME": "#111111", "STRONG": "#111111",
        "ACCENT": "#1A73E8", "LINK": "#1A73E8", "BOX_BG": "#f0f6ff",
        "DATE": "#5F6368", "METRIC": "#B45309", "METRIC_PCT": "#B91C1C", "METRIC_BG": "#FEF3C7", "METRIC_PCT_BG": "#FEE2E2", "MUTED": "#222222",
        "TECH": "#188038", "PROJECT": "#1A73E8", "TECH_LABEL": "#5F6368",
        "SKILL": "#1A73E8",
        "SHADOW": "4px 4px 14px rgba(0, 0, 0, 0.15), 1px 1px 4px rgba(0, 0, 0, 0.08)",
        "HEADER_SHADOW": "0 1px 3px rgba(0, 0, 0, 0.12)",
        "HEADING_BG": "#f0f6ff", "HEADER_BG": "#f8f9fa",
        "CHIP_BG": "#dce8fb", "CHIP_BORDER": "#b8cdf5",
    },
    "nightowl": {
        "BG": "#011627", "TEXT": "#d6deeb", "NAME": "#ffffff", "STRONG": "#e8eef5",
        "ACCENT": "#ec4899", "LINK": "#f472b6", "BOX_BG": "#152238",
        "DATE": "#8fa3b8", "METRIC": "#f472b6", "METRIC_PCT": "#f472b6", "METRIC_BG": "#2a1a2e", "METRIC_PCT_BG": "#2a1a2e", "MUTED": "#c8d3de",
        "TECH": "#22d3ee", "PROJECT": "#c792ea", "TECH_LABEL": "#8fa3b8",
        "SKILL": "#ff5874",
        "SHADOW": "4px 4px 14px rgba(0, 0, 0, 0.45), 1px 1px 4px rgba(0, 0, 0, 0.3)",
        "HEADER_SHADOW": "0 1px 4px rgba(0, 0, 0, 0.5)",
        "HEADING_BG": "#152238", "HEADER_BG": "#0e2737",
        "CHIP_BG": "#1e2a4a", "CHIP_BORDER": "#33415e",
    },
    "pink": {
        "BG": "#ffffff", "TEXT": "#1a1a1a", "NAME": "#111111", "STRONG": "#111111",
        "ACCENT": "#d63384", "LINK": "#d63384", "BOX_BG": "#fdf0f6",
        "DATE": "#6b7280", "METRIC": "#d63384", "METRIC_PCT": "#d63384", "METRIC_BG": "#fbe4f0", "METRIC_PCT_BG": "#fbe4f0", "MUTED": "#333333",
        "TECH": "#0f766e", "PROJECT": "#a21caf", "TECH_LABEL": "#6b7280",
        "SKILL": "#d63384",
        "SHADOW": "4px 4px 14px rgba(0, 0, 0, 0.12), 1px 1px 4px rgba(0, 0, 0, 0.06)",
        "HEADER_SHADOW": "0 1px 3px rgba(0, 0, 0, 0.08)",
        "HEADING_BG": "#fdf0f6", "HEADER_BG": "#fff7fb",
        "CHIP_BG": "#fbe4f0", "CHIP_BORDER": "#f3c0da",
    },
    "vivid": {
        "BG": "#ffffff", "TEXT": "#1f2937", "NAME": "#111827", "STRONG": "#1f2937",
        "ACCENT": "#1f2937", "LINK": "#0f766e", "BOX_BG": "#f0fdfa",
        "DATE": "#64748b", "METRIC": "#475569", "METRIC_PCT": "#b45309", "METRIC_BG": "#e2e8f0", "METRIC_PCT_BG": "#fef3c7", "MUTED": "#475569",
        "TECH": "#0f766e", "PROJECT": "#0f766e", "TECH_LABEL": "#64748b",
        "SKILL": "#0f766e",
        "SHADOW": "0 2px 6px rgba(15, 23, 42, 0.10)",
        "HEADER_SHADOW": "0 1px 4px rgba(15, 23, 42, 0.12)",
        "HEADING_BG": "#f1f5f9", "HEADER_BG": "#f1f5f9",
        "CHIP_BG": "#d9efea", "CHIP_BORDER": "#a9d8cd",
    },
}

theme = THEMES.get(THEME_NAME, THEMES["google"])
for token, value in theme.items():
    CSS = CSS.replace("__" + token + "__", value)

body_html = convert(md, icon_color=theme["ACCENT"], project_color=theme["PROJECT"],
                    muted_color=theme["MUTED"], tech_color=theme["TECH"], skill_color=theme["SKILL"],
                    show_icons=not NO_ICONS)

body_class = " class=\"no-icons\"" if NO_ICONS else ""
full_html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<style>{CSS}</style>
</head>
<body{body_class}>
{body_html}
</body>
</html>
"""

weasyprint.HTML(string=full_html).write_pdf(PDF_FILE)
print(f"PDF generated: {PDF_FILE} (theme: {THEME_NAME})")
