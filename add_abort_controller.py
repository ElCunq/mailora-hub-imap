import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/static/js/api.js"

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Update getMessage signature to accept signal
if 'export async function getMessage(accountId, uid, folder, signal)' not in content:
    content = content.replace("export async function getMessage(accountId, uid, folder) {", "export async function getMessage(accountId, uid, folder, signal) {")

# Update apiFetch call inside getMessage to use signal
content = re.sub(
    r'const r = await apiFetch\(`/test/body/\$\{encodeURIComponent\(accountId\)\}/\$\{uid\}\?folder=\$\{encodeURIComponent\(resolvedFolder\)\}`\);',
    r'const r = await apiFetch(`/test/body/${encodeURIComponent(accountId)}/${uid}?folder=${encodeURIComponent(resolvedFolder)}`, { signal });',
    content
)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
print("api.js getMessage updated!")

file_path2 = "/home/cunq/Desktop/mailora-hub-imap/static/js/components/message-preview.js"
with open(file_path2, 'r', encoding='utf-8') as f:
    content2 = f.read()

# Add abortController logic
if 'let currentAbortController = null;' not in content2:
    content2 = content2.replace("let currentRenderId = 0;", "let currentRenderId = 0;\nlet currentAbortController = null;")

replacement2 = """    const renderId = ++currentRenderId;
    if (currentAbortController) {
        currentAbortController.abort();
    }
    currentAbortController = new AbortController();
    const signal = currentAbortController.signal;"""

content2 = content2.replace("    const renderId = ++currentRenderId;", replacement2)

content2 = re.sub(
    r'bodyData\s*=\s*await dataSource\.getMessage\(msg\.accountId,\s*msg\.uid,\s*msg\.folder\);',
    r'bodyData = await dataSource.getMessage(msg.accountId, msg.uid, msg.folder, signal);',
    content2
)

with open(file_path2, 'w', encoding='utf-8') as f:
    f.write(content2)
print("message-preview.js updated!")
