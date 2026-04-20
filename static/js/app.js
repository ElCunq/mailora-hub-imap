// Mailora v2 — App Entry Point
import { store, ACTION } from './store.js';
import { dataSource } from './data-source.js';
import { mountSidebar } from './components/sidebar.js';
import { mountMessageList } from './components/message-list.js';
import { mountPreview } from './components/message-preview.js';
import { mountCompose, handleFileInput, sendEmail, closeCompose } from './components/compose-modal.js';
import { mountAnalytics } from './components/analytics.js';
import { toggleFocus } from './features/focus.js';

// Mount all components
mountSidebar();
mountMessageList();
mountPreview();
mountCompose();
mountAnalytics();

// Load initial data
async function init() {
    try {
        const accounts = await dataSource.getAccounts();
        store.dispatch({ type: ACTION.SET_ACCOUNTS, payload: accounts });

        if (accounts.length > 0) {
            const accId = accounts[0].id;
            const folders = await dataSource.getFolders(accId);
            store.dispatch({ type: ACTION.SET_FOLDERS, payload: folders });

            const messages = await dataSource.getMessages(accId, 'INBOX');
            store.dispatch({ type: ACTION.SET_MESSAGES, payload: messages });
        }
    } catch (e) {
        console.error("Init error:", e);
    }
    // Apply saved theme
    document.documentElement.setAttribute('data-theme', store.getState().theme);
}

// Watch store changes to reload data
let lastAcc = null;
let lastFold = null;
store.subscribe('selectedAccountId', async (accId) => {
    if (!accId || accId === lastAcc) return;
    lastAcc = accId;
    const folders = await dataSource.getFolders(accId);
    store.dispatch({ type: ACTION.SET_FOLDERS, payload: folders });
    const msgs = await dataSource.getMessages(accId, store.getState().selectedFolder);
    store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
});

store.subscribe('selectedFolder', async (folder) => {
    if (!folder || folder === lastFold) return;
    lastFold = folder;
    const accId = store.getState().selectedAccountId;
    if (accId) {
        const msgs = await dataSource.getMessages(accId, folder);
        store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
    }
});

// Sync handler
async function handleSync() {
    const accId = store.getState().selectedAccountId;
    if (!accId) return;
    try {
        await dataSource.syncAccount(accId);
        const msgs = await dataSource.getMessages(accId, store.getState().selectedFolder);
        store.dispatch({ type: ACTION.SET_MESSAGES, payload: msgs });
    } catch (e) { console.error("Sync error:", e); }
}

init();

// Global event bindings
window.mailora = {
    compose: () => store.dispatch({ type: ACTION.TOGGLE_COMPOSE }),
    closeCompose,
    sendEmail,
    handleFileInput,
    toggleAnalytics: () => store.dispatch({ type: ACTION.TOGGLE_ANALYTICS }),
    toggleTheme: () => store.dispatch({ type: ACTION.TOGGLE_THEME }),
    toggleFocus,
    search: (q) => store.dispatch({ type: ACTION.SET_SEARCH, payload: q }),
    sync: handleSync,
};

// Keyboard shortcuts
document.addEventListener('keydown', e => {
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
    const msgs = store.getVisibleMessages();
    const s = store.getState();
    const idx = msgs.findIndex(m => m.id === s.selectedMessageId);
    if (e.key === 'j' && idx < msgs.length - 1) store.dispatch({ type: ACTION.SELECT_MESSAGE, payload: msgs[idx + 1]?.id });
    else if (e.key === 'k' && idx > 0) store.dispatch({ type: ACTION.SELECT_MESSAGE, payload: msgs[idx - 1]?.id });
    else if (e.key === 'c') store.dispatch({ type: ACTION.TOGGLE_COMPOSE });
    else if (e.key === '/' || e.key === 'f') { e.preventDefault(); document.getElementById('search')?.focus(); }
});
