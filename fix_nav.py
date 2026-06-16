import re
import os
import glob

files = [
    'static/calendar.html', 'static/sheets.html', 'static/admin.html',
    'static/Example/calendar.html', 'static/Example/sheets.html', 'static/Example/admin.html',
    'static/index.html', 'static/Example/index.html'
]

# Delete mock pages from original section
for df in ['static/admin_demo.html', 'static/calendar_demo.html', 'static/sheets_demo.html']:
    if os.path.exists(df):
        os.remove(df)

for fpath in files:
    if not os.path.exists(fpath): continue
    
    with open(fpath, 'r', encoding='utf-8') as f:
        c = f.read()
        
    # Remove "Navigasyon" and all its <a class="nav-link">...</a> below it
    c = re.sub(r'<div class="section-label">\s*Navigasyon\s*</div>', '', c)
    c = re.sub(r'<a[^>]*class="nav-link[^>]*>.*?</a>', '', c, flags=re.DOTALL)
    
    # Identify if it is mock
    is_mock = 'Example/' in fpath
    base_url = '/static/Example' if is_mock else '/static'
    
    launcher = f'''<div style="position:relative;">
                    <button class="icon-btn" onclick="document.getElementById('app-launcher').classList.toggle('open')" title="Uygulamalar">▦</button>
                    <!-- Apps Dropdown -->
                    <div id="app-launcher" class="apps-dropdown">
                        <a href="{base_url}/index.html" class="app-icon">
                            <span class="app-icon-img">✉️</span>
                            <span class="app-icon-label">E-posta</span>
                        </a>
                        <a href="{base_url}/calendar.html" class="app-icon">
                            <span class="app-icon-img">📅</span>
                            <span class="app-icon-label">Takvim</span>
                        </a>
                        <a href="{base_url}/sheets.html" class="app-icon">
                            <span class="app-icon-img">📊</span>
                            <span class="app-icon-label">Tablo</span>
                        </a>
                        <a href="{base_url}/admin.html" class="app-icon">
                            <span class="app-icon-img">🛡️</span>
                            <span class="app-icon-label">Admin</span>
                        </a>
                    </div>
                </div>'''

    # If already has app-launcher wrapper, replace it
    if '<div style="position:relative;">' in c and 'id="app-launcher"' in c:
        c = re.sub(r'<div style="position:relative;">\s*<button class="icon-btn"[^>]*' + r"app-launcher" + r'.*?</div>\s*</div>', launcher, c, flags=re.DOTALL)
    else:
        # Some files like calendar.html don't have it, inject before theme-btn
        c = re.sub(r'(<button class="icon-btn" id="theme-btn")', launcher + r'\n                \1', c)

    with open(fpath, 'w', encoding='utf-8') as f:
        f.write(c)

print("Navigasyon moved successfully.")
