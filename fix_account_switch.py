import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/static/js/app.js"
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

replacement = """store.subscribe('selectedAccountId', async (accId) => {
    if (!accId || accId === lastAcc) return;
    lastAcc = accId;
    
    // Always reset to INBOX when switching accounts
    store.dispatch({ type: ACTION.SELECT_FOLDER, payload: 'INBOX' });
    lastFold = 'INBOX'; // Update lastFold so folder subscriber doesn't double fetch if it's the same

    if (accId === 'unified') {
        store.dispatch({ type: ACTION.SET_FOLDERS, payload: ['INBOX', 'Sent', 'Drafts', 'Trash', 'Spam', 'Outbox'] });
        const msgs = await dataSource.getUnifiedInbox('INBOX');
        store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
    } else {
        const folders = await dataSource.getFolders(accId);
        if (!folders.includes('Outbox')) {
            folders.push('Outbox');
        }
        store.dispatch({ type: ACTION.SET_FOLDERS, payload: folders });
        const msgs = await dataSource.getMessages(accId, 'INBOX');
        store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
    }
});"""

content = re.sub(r"store\.subscribe\('selectedAccountId', async \(accId\) => \{.*?(?=\}\);\n\nstore\.subscribe\('selectedFolder')", replacement + "\n", content, flags=re.DOTALL)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
print("Account switch fixed!")
