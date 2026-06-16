import json

file_path = 'static/Example/js/mock-app.js'
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Add accounts
account_str_old = """        <div class="account-item" style="cursor:pointer;" onclick="selectAccount('Personal')"><div class="account-dot" style="background:#ef4444"></div><span class="account-name">Personal (me@gmail.com)</span></div>"""
account_str_new = account_str_old + """
        <div class="account-item" style="cursor:pointer;" onclick="selectAccount('School')"><div class="account-dot" style="background:#8b5cf6"></div><span class="account-name">School (student@university.edu)</span></div>
        <div class="account-item" style="cursor:pointer;" onclick="selectAccount('Finance')"><div class="account-dot" style="background:#f59e0b"></div><span class="account-name">Finance (finance@company.com)</span></div>"""

content = content.replace(account_str_old, account_str_new)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
