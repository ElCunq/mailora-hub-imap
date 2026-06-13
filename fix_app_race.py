import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/static/js/app.js"
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

replacement = """let lastAcc = null;
let lastFold = null;
let fetchGeneration = 0;

store.subscribe('selectedAccountId', async (accId) => {
    if (!accId || accId === lastAcc) return;
    lastAcc = accId;
    
    const currentGen = ++fetchGeneration;
    
    // Always reset to INBOX when switching accounts
    store.dispatch({ type: ACTION.SELECT_FOLDER, payload: 'INBOX' });
    lastFold = 'INBOX'; 

    if (accId === 'unified') {
        store.dispatch({ type: ACTION.SET_FOLDERS, payload: ['INBOX', 'Sent', 'Drafts', 'Trash', 'Spam', 'Outbox'] });
        const msgs = await dataSource.getUnifiedInbox('INBOX');
        if (currentGen === fetchGeneration) store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
    } else {
        const folders = await dataSource.getFolders(accId);
        if (!folders.includes('Outbox')) {
            folders.push('Outbox');
        }
        if (currentGen === fetchGeneration) store.dispatch({ type: ACTION.SET_FOLDERS, payload: folders });
        const msgs = await dataSource.getMessages(accId, 'INBOX');
        if (currentGen === fetchGeneration) store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
    }
});

store.subscribe('selectedFolder', async (folder) => {
    if (!folder || folder === lastFold) return;
    lastFold = folder;
    
    const currentGen = ++fetchGeneration;
    const accId = store.getState().selectedAccountId;
    
    if (accId) {
        if (accId === 'unified') {
            const msgs = await dataSource.getUnifiedInbox(folder);
            if (currentGen === fetchGeneration) store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
        } else {
            const msgs = await dataSource.getMessages(accId, folder);
            if (currentGen === fetchGeneration) store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
        }
    }
});"""

content = re.sub(
    r"let lastAcc = null;\nlet lastFold = null;\nstore\.subscribe\('selectedAccountId', async \(accId\) => \{.*?\n\}\);\n\nstore\.subscribe\('selectedFolder', async \(folder\) => \{.*?\n\}\);", 
    replacement, 
    content, 
    flags=re.DOTALL
)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
print("app.js race fixed!")
