#!/usr/bin/env python3
"""Convert Arpan's NVIDIA resume markdown to a clean PDF (original resume style)."""

import re
import weasyprint

MD_FILE = "/Users/arpanpathak/Projects/rust/jobsense-parker/Arpan_Pathak_NVIDIA_Resume.md"
PDF_FILE = "/Users/arpanpathak/Projects/rust/jobsense-parker/Arpan_Pathak_NVIDIA_Resume.pdf"

with open(MD_FILE, "r") as f:
    md = f.read()

def convert(md_text):
    """Convert markdown to simple clean HTML matching original resume style."""
    lines = md_text.split("\n")
    html = []
    in_ul = False
    
    for line in lines:
        stripped = line.strip()
        
        # H1 - Name
        if line.startswith("# ") and not line.startswith("## ") and not line.startswith("### "):
            if in_ul:
                html.append("</ul>"); in_ul = False
            html.append(f'<div class="name">{line[2:]}</div>')
        
        # H2 - Section headings (SUMMARY, EXPERIENCE, SKILLS, EDUCATION)
        elif line.startswith("## "):
            if in_ul:
                html.append("</ul>"); in_ul = False
            html.append(f'<div class="section-heading">{line[3:]}</div>')
        
        # H3 - Job entries: "Title | Company — Location | Date"
        elif line.startswith("### "):
            if in_ul:
                html.append("</ul>"); in_ul = False
            content = line[4:]
            parts = [p.strip() for p in content.split("|")]
            if len(parts) >= 2:
                title = parts[0]
                rest = " | ".join(parts[1:])
                html.append(f'<div class="job-header"><span class="job-title">{title}</span> <span class="job-meta">{rest}</span></div>')
            else:
                html.append(f'<div class="job-header">{content}</div>')
        
        # Empty line
        elif not stripped or stripped == "---":
            if in_ul:
                html.append("</ul>"); in_ul = False
            continue
        
        # Bullet points - use • text directly instead of HTML lists
        elif stripped.startswith("- "):
            if in_ul:
                html.append("</ul>"); in_ul = False
            item = stripped[2:]
            html.append(f'<p class="bullet">\u2022 {item}</p>')
        
        # Regular line (contact info, skills lines, etc.)
        else:
            if in_ul:
                html.append("</ul>"); in_ul = False
            html.append(f'<p>{line}</p>')
    
    if in_ul:
        html.append("</ul>")
    
    result = "\n".join(html)
    # Convert **bold** to <strong>
    result = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', result)
    return result

body_html = convert(md)

CSS = """
@page {
    size: letter;
    margin: 0.65in 0.75in;
}
body {
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    font-size: 10.5pt;
    line-height: 1.3;
    color: #222;
}
.name {
    font-size: 20pt;
    font-weight: 700;
    color: #000;
    margin-bottom: 6px;
}
.section-heading {
    font-size: 10.5pt;
    font-weight: 700;
    color: #000;
    text-transform: uppercase;
    letter-spacing: 1px;
    border-bottom: 1px solid #555;
    padding-bottom: 1px;
    margin-top: 12px;
    margin-bottom: 6px;
}
.job-header {
    font-size: 10.5pt;
    margin: 8px 0 2px 0;
    line-height: 1.3;
}
.job-title {
    font-weight: 700;
    color: #000;
}
.job-meta {
    font-weight: 400;
    color: #555;
    font-size: 10pt;
}
p {
    margin: 3px 0;
    font-size: 10pt;
    line-height: 1.3;
}
p.bullet {
    margin: 1px 0 1px 14px;
    font-size: 10pt;
    line-height: 1.3;
    text-indent: -6px;
}
strong {
    font-weight: 700;
    color: #000;
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
