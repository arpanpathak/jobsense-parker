#!/usr/bin/env python3
"""Convert Arpan's NVIDIA resume markdown to a beautiful PDF using WeasyPrint."""

import re
import weasyprint

MD_FILE = "/Users/arpanpathak/Projects/rust/jobsense-parker/Arpan_Pathak_NVIDIA_Resume.md"
PDF_FILE = "/Users/arpanpathak/Projects/rust/jobsense-parker/Arpan_Pathak_NVIDIA_Resume.pdf"

with open(MD_FILE, "r") as f:
    md = f.read()

def md_to_html(md_text):
    """Convert a limited subset of markdown to styled HTML."""
    lines = md_text.split("\n")
    html_parts = []
    in_table = False
    in_ul = False
    table_header = []
    table_rows = []

    for line in lines:
        # Headings
        if line.startswith("## "):
            if in_ul:
                html_parts.append("</ul>")
                in_ul = False
            html_parts.append(f'<h2>{line[3:]}</h2>')
        elif line.startswith("### "):
            if in_ul:
                html_parts.append("</ul>")
                in_ul = False
            # Parse "Title | Company — Location | Date"
            content = line[4:]
            parts = [p.strip() for p in content.split("|")]
            if len(parts) >= 2:
                title = parts[0]
                rest = " | ".join(parts[1:])
                html_parts.append(f'<h3><span class="job-title">{title}</span> <span class="job-meta">{rest}</span></h3>')
            else:
                html_parts.append(f'<h3>{content}</h3>')
        # Horizontal rule
        elif line.strip() == "---":
            if in_ul:
                html_parts.append("</ul>")
                in_ul = False
            # horizontal line between sections
        # Table
        elif "|" in line and line.strip().startswith("|"):
            cells = [c.strip() for c in line.split("|")[1:-1]]  # ignore leading/trailing empty
            if not in_table:
                # Check if next line is separator row
                in_table = True
                table_header = cells
                html_parts.append('<table>')
            elif all(re.match(r'^[-:\s]+$', c) for c in cells):
                # separator row, skip
                pass
            else:
                if table_header:
                    html_parts.append('<thead><tr>' + ''.join(f'<th>{c}</th>' for c in table_header) + '</tr></thead>')
                    table_header = None
                html_parts.append('<tr>' + ''.join(f'<td>{c}</td>' for c in cells) + '</tr>')
        else:
            if in_table:
                in_table = False
                html_parts.append('</table>')
            if not line.strip():
                if in_ul:
                    html_parts.append("</ul>")
                    in_ul = False
                continue
            # Unordered list items
            if line.strip().startswith("- "):
                if not in_ul:
                    html_parts.append("<ul>")
                    in_ul = True
                # Bold/italic inline formatting
                item = line.strip()[2:]
                html_parts.append(f"<li>{item}</li>")
            else:
                if in_ul:
                    html_parts.append("</ul>")
                    in_ul = False
                # Regular paragraph with inline formatting
                html_parts.append(f"<p>{line}</p>")

    if in_ul:
        html_parts.append("</ul>")
    if in_table:
        html_parts.append("</table>")

    # Post-process inline formatting: **bold**
    result = "\n".join(html_parts)
    result = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', result)
    return result

body_html = md_to_html(md)

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

h1 {
    font-size: 22pt;
    font-weight: 700;
    color: #000;
    margin: 0 0 2px 0;
    letter-spacing: 0.5px;
}

/* Contact info block */
.contact-block {
    margin: 0 0 10px 0;
    font-size: 9pt;
    color: #333;
    line-height: 1.5;
}
.contact-block .visa-note {
    font-weight: 600;
    color: #c0392b;
    margin-top: 2px;
}

h2 {
    font-size: 10.5pt;
    font-weight: 700;
    color: #000;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid #333;
    padding-bottom: 1px;
    margin: 10px 0 4px 0;
}

/* First h2 shouldn't have top margin */
h2:first-of-type {
    margin-top: 0;
}

h3 {
    font-size: 10.5pt;
    font-weight: 700;
    margin: 8px 0 3px 0;
    color: #1a1a1a;
}
h3 .job-title {
    color: #000;
}
h3 .job-meta {
    font-weight: 400;
    font-size: 9.5pt;
    color: #555;
    float: right;
}

ul {
    margin: 2px 0 4px 0;
    padding-left: 18px;
    list-style-type: disc;
}
li {
    margin-bottom: 2px;
    font-size: 9.5pt;
    line-height: 1.35;
}

p {
    margin: 3px 0;
    font-size: 9.5pt;
    line-height: 1.35;
}

/* Summary paragraph */
h2:first-of-type + p {
    margin-bottom: 4px;
}

/* Infra line (italic) */
li:last-child, p:has(strong:only-child) {
    /* handled below */
}

/* Style the "Infra:" lines */
li:has(strong) {
    list-style-type: none;
    margin-left: -18px;
    font-size: 8.5pt;
    color: #555;
    margin-top: 1px;
    margin-bottom: 6px;
}

table {
    width: 100%;
    border-collapse: collapse;
    font-size: 9pt;
    margin: 4px 0;
}
th {
    text-align: left;
    font-weight: 700;
    width: 22%;
    vertical-align: top;
    padding: 2px 6px 2px 0;
    color: #1a1a1a;
    border: none;
}
td {
    text-align: left;
    vertical-align: top;
    padding: 2px 0;
    border: none;
}
tr:first-child th,
tr:first-child td {
    padding-top: 0;
}

strong {
    font-weight: 700;
    color: #000;
}
"""

# Split into header contact block and rest
body_parts = body_html.split("</h1>", 1)
if len(body_parts) == 2:
    header = body_parts[0] + "</h1>"
    rest = body_parts[1]
    # Wrap contact lines after h1
    contact_lines = []
    remaining = []
    lines = rest.split("\n")
    in_contact = True
    for l in lines:
        if in_contact and (l.strip().startswith("<p>") or l.strip().startswith("<strong")):
            contact_lines.append(l)
        else:
            in_contact = False
            remaining.append(l)
    contact_block = "\n".join(contact_lines)
    # Replace the contact lines with a styled block
    contact_html = '<div class="contact-block">\n'
    for cl in contact_lines:
        stripped = re.sub(r'^<p>|</p>$', '', cl.strip())
        if stripped:
            if "Visa" in stripped:
                contact_html += f'<div class="visa-note">{stripped}</div>\n'
            else:
                contact_html += f'<div>{stripped}</div>\n'
    contact_html += '</div>\n'
    body_html = header + "\n" + contact_html + "\n".join(remaining)

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

# Generate PDF
weasyprint.HTML(string=full_html).write_pdf(PDF_FILE)
print(f"PDF generated: {PDF_FILE}")
