#!/usr/bin/env python3
"""Convert resume markdown to a modern, clean PDF with proper link rendering."""

import re
import weasyprint

MD_FILE = "/Users/arpanpathak/Projects/rust/jobsense-parker/Arpan_Pathak_NVIDIA_Resume.md"
PDF_FILE = "/Users/arpanpathak/Projects/rust/jobsense-parker/Arpan_Pathak_NVIDIA_Resume.pdf"

with open(MD_FILE, "r") as f:
    md = f.read()


def convert(md_text):
    """Convert markdown to clean HTML with proper link rendering."""
    lines = md_text.split("\n")
    html = []

    for line in lines:
        stripped = line.strip()

        # H1 - Name
        if line.startswith("# ") and not line.startswith("## ") and not line.startswith("### "):
            html.append(f'<div class="name">{line[2:]}</div>')

        # H2 - Section headings
        elif line.startswith("## "):
            html.append(f'<div class="section-heading">{line[3:]}</div>')

        # H3 - Job entries: "Title | Company — Location | Date"
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
            html.append(f'<p class="bullet">\u2022 {item}</p>')

        # Regular line
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

/* ---- Contact info ---- */
.contact-line {
    font-size: 9pt;
    color: #444;
    margin: 1px 0;
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
