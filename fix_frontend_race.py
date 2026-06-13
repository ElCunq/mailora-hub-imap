import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/static/js/components/message-preview.js"

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Add currentRenderId variable
if 'let currentRenderId = 0;' not in content:
    content = content.replace("export function mountPreview() {", "let currentRenderId = 0;\nexport function mountPreview() {")

# Add renderId check in the try block
if 'const renderId = ++currentRenderId;' not in content:
    content = content.replace("async function render() {", "async function render() {\n    const renderId = ++currentRenderId;")

# The `bodyData = await dataSource.getMessage(...)` is around line 32
# Replace:
# bodyData = await dataSource.getMessage(msg.accountId, msg.uid, msg.folder);
# attData = await dataSource.getAttachments(msg.accountId, msg.uid, msg.folder);
# With a check after await:
replacement = """            bodyData = await dataSource.getMessage(msg.accountId, msg.uid, msg.folder);
            if (renderId !== currentRenderId) return;
            attData = await dataSource.getAttachments(msg.accountId, msg.uid, msg.folder);
            if (renderId !== currentRenderId) return;"""
            
content = re.sub(
    r'bodyData\s*=\s*await dataSource\.getMessage\(msg\.accountId,\s*msg\.uid,\s*msg\.folder\);\s*attData\s*=\s*await dataSource\.getAttachments\(msg\.accountId,\s*msg\.uid,\s*msg\.folder\);',
    replacement,
    content
)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
print("Frontend racing fixed!")
