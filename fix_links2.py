import os
import glob
import re

static_dir = 'static'
html_files = glob.glob(os.path.join(static_dir, '*.html'))

replacements = [
    (r'href=["\']/["\']', 'href="/static/index.html"'),
    (r"window\.location\.href\s*=\s*['\"]/['\"]", "window.location.href = '/static/index.html'"),
]

for file_path in html_files:
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    new_content = content
    for pattern, repl in replacements:
        new_content = re.sub(pattern, repl, new_content)
        
    if new_content != content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"Fixed links in {file_path}")

print("Done")
