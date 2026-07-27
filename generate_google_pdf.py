#!/usr/bin/env python3
"""Convert resume markdown to a Google-themed PDF with color-highlighted metrics."""

import re
import sys
import weasyprint

MD_FILE = sys.argv[1] if len(sys.argv) > 1 else "/Users/arpanpathak/Projects/rust/jobsense-parker/ArpanPathak_Google_Android.md"
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
            else:
                html.append(f'<p class="bullet">\u2022 {item}</p>')

        else:
            if in_project_box:
                html.append(f'<p class="project-text">{line}</p>')
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


body_html = convert(md)

CSS = """
@page {
    size: letter;
    margin: 0.5in 0.65in;
}
body {
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    font-size: 10pt;
    line-height: 1.32;
    color: #1a1a1a;
}

.name {
    font-size: 24pt;
    font-weight: 700;
    color: #111;
    margin-bottom: 2px;
    letter-spacing: 0.5px;
}

.project-box {
    float: right;
    width: 48%;
    margin-left: 14px;
    margin-bottom: 10px;
    border: 1.5px solid #1A73E8;
    border-radius: 8px;
    padding: 8px 11px;
    background: #f0f6ff;
    box-shadow: 4px 4px 14px rgba(0, 0, 0, 0.15), 1px 1px 4px rgba(0, 0, 0, 0.08);
}
.project-box p:first-child {
    font-size: 8.5pt;
    font-weight: 700;
    color: #1A73E8;
    text-transform: uppercase;
    letter-spacing: 1.2px;
    margin: 0 0 3px 0;
}
p.project-text {
    font-size: 8pt;
    line-height: 1.28;
    margin: 2px 0;
    color: #222;
}
p.project-text strong {
    font-size: 8.5pt;
    color: #1A73E8;
}
p.bullet-sm {
    margin: 1px 0 1px 12px;
    font-size: 8pt;
    line-height: 1.28;
    text-indent: -7px;
    color: #222;
}
p.bullet-sm strong {
    font-size: 8pt;
    color: #1A73E8;
}
.project-box a {
    color: #1A73E8;
    font-size: 8pt;
}

.section-heading {
    font-size: 11pt;
    font-weight: 700;
    color: #111;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    border-bottom: 2px solid #1A73E8;
    padding-bottom: 3px;
    margin-top: 14px;
    margin-bottom: 7px;
}

.job-header {
    font-size: 10.5pt;
    margin: 9px 0 2px 0;
    line-height: 1.3;
}
.job-title {
    font-weight: 700;
    color: #111;
}
.job-company {
    font-weight: 600;
    color: #1A73E8;
    font-size: 10pt;
}
.job-date {
    font-weight: 400;
    color: #5F6368;
    font-size: 9.5pt;
}

p {
    margin: 2px 0;
    font-size: 9.5pt;
    line-height: 1.32;
}

p.bullet {
    margin: 1px 0 1px 16px;
    font-size: 9.5pt;
    line-height: 1.32;
    text-indent: -8px;
}

strong {
    font-weight: 700;
    color: #111;
}

a {
    color: #1A73E8;
    text-decoration: none;
}

span.metric {
    color: #B8860B;
    font-weight: 700;
}
"""

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
print(f"PDF generated: {PDF_FILE}")
