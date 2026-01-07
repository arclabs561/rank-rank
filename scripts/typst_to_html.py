#!/usr/bin/env python3
# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "markdown>=3.4.0",
#   "beautifulsoup4>=4.11.0",
# ]
# ///
"""
Convert Typst PDF to HTML using a template-based approach.

Since Typst doesn't have direct HTML export, we:
1. Extract content from Typst source
2. Convert to HTML using markdown-like structure
3. Apply styling
"""

import re
import sys
from pathlib import Path
from typing import List, Dict

def parse_typst_content(typst_path: Path) -> Dict[str, any]:
    """Parse Typst file and extract structured content."""
    content = typst_path.read_text()
    
    # Extract title
    title_match = re.search(r'^= (.+)$', content, re.MULTILINE)
    title = title_match.group(1) if title_match else "Documentation"
    
    # Extract sections
    sections = []
    current_section = None
    current_content = []
    
    for line in content.split('\n'):
        # Heading level 1
        if line.startswith('== '):
            if current_section:
                sections.append({
                    'title': current_section,
                    'content': '\n'.join(current_content)
                })
            current_section = line[3:].strip()
            current_content = []
        # Heading level 2
        elif line.startswith('=== '):
            if current_content:
                current_content.append(f"\n### {line[4:].strip()}\n")
        # Code blocks
        elif line.startswith('```'):
            current_content.append(line)
        else:
            current_content.append(line)
    
    if current_section:
        sections.append({
            'title': current_section,
            'content': '\n'.join(current_content)
        })
    
    return {
        'title': title,
        'sections': sections,
        'raw': content
    }

def typst_to_html(typst_path: Path, output_path: Path):
    """Convert Typst source to HTML."""
    parsed = parse_typst_content(typst_path)
    
    html_parts = []
    html_parts.append("""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: 'Linux Libertine', 'Times New Roman', serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2em;
            line-height: 1.6;
            color: #333;
        }}
        h1 {{
            border-bottom: 2px solid #333;
            padding-bottom: 0.3em;
        }}
        h2 {{
            margin-top: 2em;
            border-bottom: 1px solid #ccc;
            padding-bottom: 0.2em;
        }}
        h3 {{
            margin-top: 1.5em;
            color: #555;
        }}
        code {{
            background: #f4f4f4;
            padding: 0.2em 0.4em;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
        }}
        pre {{
            background: #f4f4f4;
            padding: 1em;
            border-radius: 5px;
            overflow-x: auto;
        }}
        pre code {{
            background: none;
            padding: 0;
        }}
        .math {{
            font-style: italic;
        }}
        .center {{
            text-align: center;
        }}
    </style>
</head>
<body>""".format(title=parsed['title']))
    
    html_parts.append(f"<h1>{parsed['title']}</h1>")
    
    for section in parsed['sections']:
        html_parts.append(f"<h2>{section['title']}</h2>")
        
        # Convert content (simplified - would need more sophisticated parsing)
        content = section['content']
        # Convert code blocks
        content = re.sub(r'```(\w+)?\n(.*?)```', r'<pre><code>\2</code></pre>', content, flags=re.DOTALL)
        # Convert inline code
        content = re.sub(r'`([^`]+)`', r'<code>\1</code>', content)
        # Convert math (simplified)
        content = re.sub(r'\$([^$]+)\$', r'<span class="math">\1</span>', content)
        # Convert bold
        content = re.sub(r'\*([^*]+)\*', r'<strong>\1</strong>', content)
        # Convert line breaks
        content = content.replace('\n\n', '</p><p>')
        content = f'<p>{content}</p>'
        
        html_parts.append(content)
    
    html_parts.append("</body></html>")
    
    # Ensure output directory exists
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text('\n'.join(html_parts))
    print(f"✅ Generated HTML: {output_path}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: typst_to_html.py <input.typ> <output.html>")
        sys.exit(1)
    
    typst_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    
    if not typst_path.exists():
        print(f"❌ Typst file not found: {typst_path}")
        sys.exit(1)
    
    typst_to_html(typst_path, output_path)

