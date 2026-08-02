#!/usr/bin/env python3
"""Convert resume markdown to a Google-themed PDF with color-highlighted metrics."""

import re
import sys
import weasyprint

MD_FILE = sys.argv[1] if len(sys.argv) > 1 else "/Users/arpanpathak/Projects/rust/jobsense-parker/ArpanPathak_Google_Android.md"
THEME_NAME = sys.argv[2] if len(sys.argv) > 2 else "google"
PDF_FILE = MD_FILE.replace(".md", ".pdf")

with open(MD_FILE, "r") as f:
    md = f.read()


def highlight_metrics(text):
    """Wrap metrics and numbers in colored spans."""
    text = re.sub(r'(\$\d+[.,]?\d*[KMB]?[/]?(month|year)?)', r'<span class="metric">\1</span>', text)
    text = re.sub(r'(\d+[.,]?\d*\s*(ops/s|ms|QPS|TPM|%|writes/second|hourly transactions))', r'<span class="metric">\1</span>', text)
    text = re.sub(r'(\d+\s*(days?|hours?|minutes?))', r'<span class="metric">\1</span>', text)
    text = re.sub(r'(\d+[KMB]\++?)(?!\s*(days?|hours?|minutes?|ops/s|ms|QPS|TPM|%))', r'<span class="metric">\1</span>', text)
    return text


def convert(md_text):
    """Convert markdown to clean HTML with proper link rendering."""
    lines = md_text.split("\n")
    html = []
    in_project_box = False

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

        elif line.startswith("## "):
            html.append(f'<div class="section-heading">{line[3:]}</div>')

        elif line.startswith("### "):
            content = line[4:]
            parts = [p.strip() for p in content.split("|")]
            title = parts[0] if len(parts) >= 1 else content
            if len(parts) >= 3:
                company = parts[1]
                date = parts[2]
                html.append(f'<div class="job-header"><span class="job-title">{title}</span> <span class="job-company">{company}</span> <span class="job-date">{date}</span></div>')
            elif len(parts) == 2:
                rest = parts[1]
                html.append(f'<div class="job-header"><span class="job-title">{title}</span> <span class="job-company">{rest}</span></div>')
            else:
                html.append(f'<div class="job-header">{content}</div>')

        elif not stripped or stripped == "---":
            continue

        elif stripped.startswith("- "):
            item = stripped[2:]
            if in_project_box:
                html.append(f'<p class="bullet-sm">\u2022 {item}</p>')
            elif item.startswith("TechStack"):
                label, _, rest = item.partition(":")
                html.append(f'<p class="techstack-box"><span class="techstack-label">{label}</span>:{rest}</p>')
            else:
                html.append(f'<p class="bullet">\u2022 {item}</p>')

        else:
            if in_project_box:
                html.append(f'<p class="project-text">{line}</p>')
            else:
                cls = "skill-group" if "**: " in line else ""
                html.append(f'<p class="{cls}">{line}</p>')

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


body_html = convert(md)

CSS = """
@page {
    size: letter;
    margin: 0.5in 0.65in;
    background: __BG__;
}
body {
    font-family: 'Avenir Next', 'Avenir', 'Helvetica Neue', Helvetica, Arial, sans-serif;
    font-size: 10pt;
    line-height: 1.38;
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

.project-box {
    float: right;
    width: 48%;
    margin-left: 14px;
    margin-bottom: 10px;
    border: 1.5px solid __PROJECT__;
    border-radius: 8px;
    padding: 8px 11px;
    background: __BOX_BG__;
    box-shadow: __SHADOW__;
}
.project-box p:first-child {
    font-size: 8.5pt;
    font-weight: 700;
    color: __PROJECT__;
    text-transform: uppercase;
    letter-spacing: 1.2px;
    margin: 0 0 3px 0;
}
p.project-text {
    font-size: 8pt;
    line-height: 1.28;
    margin: 2px 0;
    color: __MUTED__;
}
p.project-text strong {
    font-size: 8.5pt;
    color: __PROJECT__;
}
p.bullet-sm {
    margin: 1px 0 1px 12px;
    font-size: 8pt;
    line-height: 1.28;
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
    letter-spacing: 0.6px;
    border-bottom: 2px solid __ACCENT__;
    padding: 3px 8px;
    margin-top: 14px;
    margin-bottom: 7px;
    border-radius: 4px;
    box-shadow: __HEADER_SHADOW__;
}

.job-header {
    font-size: 10.5pt;
    margin: 9px 0 2px 0;
    line-height: 1.3;
    padding: 3px 8px;
    border-radius: 6px;
    box-shadow: __HEADER_SHADOW__;
}
.job-title {
    font-style: italic;
    font-weight: 600;
    color: __NAME__;
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
}

p {
    margin: 2px 0;
    font-size: 9.5pt;
    line-height: 1.32;
}

p.skill-group {
    margin: 6px 0 3px 0;
}

p.bullet {
    margin: 1px 0 1px 16px;
    font-size: 9.5pt;
    line-height: 1.32;
    text-indent: -8px;
}

p.techstack-box {
    margin: 4px 0 8px 16px;
    line-height: 1.38;
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
    text-decoration: none;
}

span.metric {
    color: __METRIC__;
    font-weight: 700;
}
"""

THEMES = {
    "google": {
        "BG": "#ffffff", "TEXT": "#1a1a1a", "NAME": "#111111", "STRONG": "#111111",
        "ACCENT": "#1A73E8", "LINK": "#1A73E8", "BOX_BG": "#f0f6ff",
        "DATE": "#5F6368", "METRIC": "#B8860B", "MUTED": "#222222",
        "TECH": "#188038", "PROJECT": "#1A73E8", "TECH_LABEL": "#5F6368",
        "SHADOW": "4px 4px 14px rgba(0, 0, 0, 0.15), 1px 1px 4px rgba(0, 0, 0, 0.08)",
        "HEADER_SHADOW": "0 1px 3px rgba(0, 0, 0, 0.12)",
    },
    "nightowl": {
        "BG": "#011627", "TEXT": "#d6deeb", "NAME": "#ffffff", "STRONG": "#e8eef5",
        "ACCENT": "#ec4899", "LINK": "#f472b6", "BOX_BG": "#152238",
        "DATE": "#8fa3b8", "METRIC": "#f472b6", "MUTED": "#c8d3de",
        "TECH": "#22d3ee", "PROJECT": "#c792ea", "TECH_LABEL": "#8fa3b8",
        "SHADOW": "4px 4px 14px rgba(0, 0, 0, 0.45), 1px 1px 4px rgba(0, 0, 0, 0.3)",
        "HEADER_SHADOW": "0 1px 4px rgba(0, 0, 0, 0.5)",
    },
    "pink": {
        "BG": "#ffffff", "TEXT": "#1a1a1a", "NAME": "#111111", "STRONG": "#111111",
        "ACCENT": "#d63384", "LINK": "#d63384", "BOX_BG": "#fdf0f6",
        "DATE": "#6b7280", "METRIC": "#d63384", "MUTED": "#333333",
        "TECH": "#0f766e", "PROJECT": "#a21caf", "TECH_LABEL": "#6b7280",
        "SHADOW": "4px 4px 14px rgba(0, 0, 0, 0.12), 1px 1px 4px rgba(0, 0, 0, 0.06)",
        "HEADER_SHADOW": "0 1px 3px rgba(0, 0, 0, 0.08)",
    },
    "vivid": {
        "BG": "#ffffff", "TEXT": "#1f2937", "NAME": "#111827", "STRONG": "#1f2937",
        "ACCENT": "#6366f1", "LINK": "#3b82f6", "BOX_BG": "#eef2ff",
        "DATE": "#9ca3af", "METRIC": "#f59e0b", "MUTED": "#4b5563",
        "TECH": "#0d9488", "PROJECT": "#7c3aed", "TECH_LABEL": "#6b7280",
        "SHADOW": "0 2px 6px rgba(31, 41, 55, 0.08)",
        "HEADER_SHADOW": "0 1px 4px rgba(31, 41, 55, 0.12)",
    },
}

theme = THEMES.get(THEME_NAME, THEMES["google"])
for token, value in theme.items():
    CSS = CSS.replace("__" + token + "__", value)

full_html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<style>{CSS}</style>
</head>
<body>
{body_html}
</body>
</html>
"""

weasyprint.HTML(string=full_html).write_pdf(PDF_FILE)
print(f"PDF generated: {PDF_FILE} (theme: {THEME_NAME})")
