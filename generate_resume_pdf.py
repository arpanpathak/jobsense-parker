#!/usr/bin/env python3
"""Convert resume markdown to a modern, clean PDF with project box support."""

import re
import sys
import weasyprint

MD_FILE = sys.argv[1] if len(sys.argv) > 1 else "/Users/arpanpathak/Projects/rust/jobsense-parker/Arpan_Pathak_NVIDIA_Resume.md"
PDF_FILE = MD_FILE.replace(".md", ".pdf")

with open(MD_FILE, "r") as f:
    md = f.read()


def convert(md_text):
    """Convert markdown to clean HTML with proper link rendering."""
    lines = md_text.split("\n")
    html = []
    in_project_box = False

    for line in lines:
        stripped = line.strip()

        # Project box start
        if stripped == "::: project-box":
            in_project_box = True
            html.append('<div class="project-box">')
            continue

        # Project box end
        if stripped == ":::" and in_project_box:
            in_project_box = False
            html.append('</div>')
            continue

        # H1 - Name
        if line.startswith("# ") and not line.startswith("## ") and not line.startswith("### "):
            html.append(f'<div class="name">{line[2:]}</div>')

        # H2 - Section headings
        elif line.startswith("## "):
            html.append(f'<div class="section-heading">{line[3:]}</div>')

        # H3 - Job entries
        elif line.startswith("### "):
            content = line[4:]
            parts = [p.strip() for p in content.split("|")]
            if len(parts) >= 2:
                title = parts[0]
                rest = " | ".join(parts[1:])
                html.append(f'<div class="job-header"><span class="job-title">{title}</span> <span class="job-meta">{rest}</span></div>')
            else:
                html.append(f'<div class="job-header">{content}</div>')

        # Empty / separator
        elif not stripped or stripped == "---":
            continue

        # Bullet points
        elif stripped.startswith("- "):
            item = stripped[2:]
            cls = "bullet-sm" if in_project_box else "bullet"
            html.append(f'<p class="{cls}">\u2022 {item}</p>')

        # Regular line
        else:
            if in_project_box:
                html.append(f'<p class="project-text">{line}</p>')
            else:
                html.append(f'<p>{line}</p>')

    result = "\n".join(html)

    # Convert **bold** to <strong>
    result = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', result)

    # Convert [text](url) to <a href="url">text</a>
    result = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', r'<a href="\2">\1</a>', result)

    return result


body_html = convert(md)

CSS = """
@page {
    size: letter;
    margin: 0.6in 0.7in;
}
body {
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    font-size: 10pt;
    line-height: 1.35;
    color: #1a1a1a;
}

/* ---- Name ---- */
.name {
    font-size: 22pt;
    font-weight: 700;
    color: #111;
    margin-bottom: 4px;
}

/* ---- Project Box ---- */
.project-box {
    float: right;
    width: 48%;
    margin-left: 12px;
    margin-bottom: 8px;
    border: 1.5px solid #76B900;
    border-radius: 6px;
    padding: 7px 10px;
    background: #f6faf0;
    box-shadow: 3px 3px 8px rgba(0, 0, 0, 0.12);
}
.project-box .section-heading {
    font-size: 8.5pt;
    font-weight: 700;
    color: #76B900;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin: 0 0 3px 0;
    padding: 0;
    border: none;
}
p.project-text {
    font-size: 8pt;
    line-height: 1.3;
    margin: 2px 0;
    color: #333;
}
p.project-text strong {
    font-size: 8.5pt;
}
p.bullet-sm {
    margin: 1px 0 1px 12px;
    font-size: 8pt;
    line-height: 1.3;
    text-indent: -7px;
    color: #333;
}
p.bullet-sm strong {
    font-size: 8pt;
}
.project-box a {
    color: #76B900;
    font-size: 8pt;
}

/* ---- Section Headings ---- */
.section-heading {
    font-size: 11pt;
    font-weight: 700;
    color: #111;
    text-transform: uppercase;
    letter-spacing: 0.3px;
    border-bottom: 1.5px solid #76B900;
    padding-bottom: 2px;
    margin-top: 14px;
    margin-bottom: 6px;
}

/* ---- Job Header ---- */
.job-header {
    font-size: 10.5pt;
    margin: 8px 0 2px 0;
}
.job-title {
    font-weight: 700;
    color: #111;
}
.job-meta {
    font-weight: 400;
    color: #666;
    font-size: 9.5pt;
}

/* ---- Paragraphs ---- */
p {
    margin: 2px 0;
    font-size: 9.5pt;
    line-height: 1.35;
}

/* ---- Bullet points ---- */
p.bullet {
    margin: 1px 0 1px 16px;
    font-size: 9.5pt;
    line-height: 1.35;
    text-indent: -8px;
}

/* ---- Bold ---- */
strong {
    font-weight: 700;
    color: #111;
}

/* ---- Links ---- */
a {
    color: #76B900;
    text-decoration: none;
}
a:hover {
    text-decoration: underline;
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
