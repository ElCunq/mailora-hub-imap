import os
import re

STATIC_DIR = "/home/cunq/Desktop/mailora-hub-imap/static"

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
        
    # Replace href="filename.html" -> href="/static/filename.html"
    # and href="./filename.html" -> href="/static/filename.html"
    content = re.sub(r'href="(?:\./)?([^/"]+\.html)"', r'href="/static/\1"', content)
    
    # Replace window.location.href = 'filename.html' -> window.location.href = '/static/filename.html'
    content = re.sub(r'location\.href\s*=\s*[\'"](?:\./)?([^/\'"]+\.html)[\'"]', r"location.href = '/static/\1'", content)
    
    # Replace window.location.href = '/filename.html' -> window.location.href = '/static/filename.html'
    # but be careful not to replace if it's already /static/
    content = re.sub(r'location\.href\s*=\s*[\'"]/(?!static/)([^/\'"]+\.html)[\'"]', r"location.href = '/static/\1'", content)
    
    # Replace href="/filename.html" -> href="/static/filename.html"
    content = re.sub(r'href="/(?!static/)([^/"]+\.html)"', r'href="/static/\1"', content)

    # Some redirects might be in js files or embedded scripts
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

for root, _, files in os.walk(STATIC_DIR):
    for file in files:
        if file.endswith('.html') or file.endswith('.js'):
            process_file(os.path.join(root, file))

print("Fixed all redirects!")
