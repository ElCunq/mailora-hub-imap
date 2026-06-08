import os
import glob
import re

static_dir = 'static'
html_files = glob.glob(os.path.join(static_dir, '*.html'))

replacements = [
    (r'href=["\']index\.html["\']', 'href="/"'),
    (r'href=["\']login\.html["\']', 'href="/static/login.html"'),
    (r'href=["\']admin\.html["\']', 'href="/static/admin.html"'),
    (r'href=["\']add_account\.html["\']', 'href="/static/add_account.html"'),
    (r'href=["\']calendar\.html["\']', 'href="/static/calendar.html"'),
    (r'href=["\']contacts\.html["\']', 'href="/static/contacts.html"'),
    (r"window\.location\.href\s*=\s*['\"]index\.html['\"]", "window.location.href = '/'"),
    (r"window\.location\.href\s*=\s*['\"]login\.html['\"]", "window.location.href = '/static/login.html'"),
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
