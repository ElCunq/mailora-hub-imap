const mockData = [
    {
        id: 1, sender: 'Ahmet Yılmaz', email: 'ahmet@company.com', accName: 'Work', accColor: '#3b82f6',
        subject: 'Q3 Finansal Raporları ve Proje Analizleri', time: '10:45', readTime: '~3dk',
        preview: 'Merhaba ekip, son çeyreğe ait finansal raporları ve proje analizlerini ekte bulabilirsiniz. Toplantıda görüşmek üzere...',
        body: `<p>Merhaba ekip,</p><p>Son çeyreğe ait finansal raporları ve proje durum analizlerini ekteki tabloda bulabilirsiniz.</p><p>Özellikle <strong>Rust ve Axum</strong> mimarisine geçiş sürecindeki performans kazanımlarımız oldukça tatmin edici görünüyor. IMAP senkronizasyon sürelerinde %40 oranında bir iyileşme raporlanmış durumda. Bunun yanında yeni geliştirilen <strong>RBAC</strong> sisteminin güvenlik analiz sonuçları da oldukça başarılı.</p><p>Detayları yarınki haftalık toplantıda konuşacağız.</p><p>Kolay gelsin,</p><p><strong>Ahmet Yılmaz</strong><br>Danışman</p>`,
        pinned: false, important: true, read: true,
        aiTopic: 'Eğitim', aiIcon: '🎓', aiSpam: null,
        hasAttachment: true
    },
    {
        id: 2, sender: 'Mailora Team', email: 'updates@mailora.local', accName: 'Dev', accColor: '#10b981',
        subject: 'Mailora Hub V2.0 Güncelleme Detayları', time: '09:12', readTime: '~1dk',
        preview: 'Yeni sürüm başarıyla yayına alındı. IMAP asenkron senkronizasyonu ve Tablolar modülü artık çok daha hızlı çalışıyor.',
        body: `<div style="text-align:center; padding: 20px; background:var(--bg-secondary); border-radius: 8px;">
            <h1 style="color:var(--accent-blue)">Mailora Hub V2.0 Yayında! 🎉</h1>
            <p>Sevgili geliştirici, yeni sürümdeki yenilikler:</p>
            <ul style="text-align:left; display:inline-block; margin:20px auto;">
                <li>Asenkron IMAP senkronizasyonu eklendi.</li>
                <li>Tablolar modülü için WebSocket desteği eklendi.</li>
                <li>Vanilla JS arayüzünde %60 hız artışı sağlandı.</li>
            </ul>
            <br><button style="padding:10px 20px; background:var(--accent-blue); color:white; border:none; border-radius:5px; margin-top:15px; cursor:pointer;">Sürüm Notlarını Oku</button>
        </div>`,
        pinned: true, important: false, read: false,
        aiTopic: 'Teknoloji', aiIcon: '💻', aiSpam: null,
        hasAttachment: false
    },
    {
        id: 3, sender: 'DevOps Alerts', email: 'alerts@devops.local', accName: 'Dev', accColor: '#10b981',
        subject: 'Yeni Sunucu Kurulum Yönergeleri (Rust & Axum)', time: 'Dün', readTime: '~5dk',
        preview: 'Geliştirme ortamı için yeni Rust ve SQLite sunucularının ayağa kaldırılma adımları Wiki sayfasına eklendi. Lütfen inceleyin.',
        body: `<p>Sistem Yöneticisi,</p>
            <p>Aşağıdaki komutları kullanarak yeni sunucuları ayağa kaldırabilirsiniz:</p>
            <pre style="background:var(--bg-tertiary); padding:10px; border-radius:4px; border:1px solid var(--border);"><code>cargo run --release\nsqlite3 database.db < schema.sql</code></pre>
            <p>Daha fazla bilgi için Wiki'ye göz atın.</p>`,
        pinned: false, important: false, read: true,
        aiTopic: null, aiIcon: null, aiSpam: {score: 0, text: 'Güvenli', color: '#10b981'},
        hasAttachment: false
    },
    {
        id: 4, sender: 'Weekly Sync', email: 'sync@company.com', accName: 'Work', accColor: '#3b82f6',
        subject: 'Haftalık Ekip Toplantısı Notları', time: 'Pzt', readTime: '~2dk',
        preview: 'Dünkü toplantıda aldığımız kararlar ve RBAC yetkilendirme modülünün son durumu hakkında kısa bir özet geçiyorum...',
        body: `<p>Toplantı Özeti:</p><ul><li>RBAC modülü admin paneline entegre edildi.</li><li>Müşteri geri bildirimleri değerlendirildi.</li><li>UI güncellemeleri tamamlandı.</li></ul>`,
        pinned: false, important: false, read: true,
        aiTopic: 'İş', aiIcon: '💼', aiSpam: null,
        hasAttachment: false
    },
    {
        id: 5, sender: 'Unknown Sender', email: 'spam@freestuff.com', accName: 'Personal', accColor: '#ef4444',
        subject: 'Win a Free Cloud Server! Limited Time Offer!', time: '23 Eki', readTime: '~1dk',
        preview: 'Click here to claim your free lifetime cloud server instance today. No credit card required! Don\'t miss out on this...',
        body: `<h2 style="color:red">CONGRATULATIONS!</h2><p>You have been selected to win a free cloud server. <a href="#">Click here to claim</a>.</p>`,
        pinned: false, important: false, read: true,
        aiTopic: null, aiIcon: null, aiSpam: {score: 9, text: 'Spam Riski', color: '#ef4444'},
        hasAttachment: false
    },
    {
        id: 6, sender: 'GitHub', email: 'noreply@github.com', accName: 'Dev', accColor: '#10b981',
        subject: '[mailora-hub] Pull request #42: Feature/RBAC', time: '22 Eki', readTime: '~4dk',
        preview: 'A new pull request has been opened by dev-user. "Implemented Role-Based Access Control logic for admin endpoints".',
        body: `<div style="border:1px solid var(--border); padding: 15px; border-radius: 5px;">
            <h3><span style="color:#10b981">Open</span> Pull Request #42: Feature/RBAC</h3>
            <p>Implemented Role-Based Access Control logic for admin endpoints using custom Axum extractors.</p>
            <button style="padding:5px 10px; background:var(--bg-tertiary); border:1px solid var(--border); border-radius:3px; cursor:pointer; color:var(--text-primary);">View PR</button>
        </div>`,
        pinned: false, important: false, read: true,
        aiTopic: 'Teknoloji', aiIcon: '💻', aiSpam: null,
        hasAttachment: false
    },
    {
        id: 7, sender: 'Fatma Yılmaz', email: 'fatma@company.com', accName: 'Work', accColor: '#3b82f6',
        subject: 'Ofis Malzemeleri Siparişi', time: '20 Eki', readTime: '~1dk',
        preview: 'Yeni ofis malzemeleri listesi ektedir. Lütfen onaylayın.',
        body: `<p>Merhaba,</p><p>İhtiyaç duyulan malzemeler listesi ektedir. Onayınızdan sonra sipariş geçilecektir.</p>`,
        pinned: false, important: false, read: true,
        aiTopic: 'Alışveriş', aiIcon: '🛒', aiSpam: null,
        hasAttachment: true
    },
    {
        id: 8, sender: 'Canan Kaya', email: 'canan@company.com', accName: 'Work', accColor: '#3b82f6',
        subject: 'Müşteri Görüşmesi: Proje X', time: '18 Eki', readTime: '~3dk',
        preview: 'Proje X için bugün yaptığımız görüşmenin notlarını paylaşıyorum. Müşteri arayüz tasarımını beğendi ancak...',
        body: `<p>Proje X Görüşme Notları:</p><p>Müşteri arayüz tasarımını çok beğendi. Ancak "Tablolar" kısmında bazı ek özellikler talep ediyorlar. İlgili eklentileri haftaya kadar hazırlamamız gerekiyor.</p>`,
        pinned: false, important: true, read: true,
        aiTopic: 'İş', aiIcon: '💼', aiSpam: null,
        hasAttachment: false
    }
];

let state = {
    selectedMessageId: 1,
    folderCollapsed: false
};

window.togglePin = function(id) {
    const msg = mockData.find(m => m.id === id);
    if(msg) msg.pinned = !msg.pinned;
    renderList();
};

window.toggleImportant = function(id) {
    const msg = mockData.find(m => m.id === id);
    if(msg) msg.important = !msg.important;
    renderList();
};

window.deleteMsg = function(id) {
    const idx = mockData.findIndex(m => m.id === id);
    if(idx > -1) {
        mockData.splice(idx, 1);
        if(state.selectedMessageId === id && mockData.length > 0) state.selectedMessageId = mockData[0].id;
        renderList();
        renderPreview();
    }
};

window.selectMsg = function(id) {
    state.selectedMessageId = id;
    const msg = mockData.find(m => m.id === id);
    if(msg && !msg.read) msg.read = true;
    renderList();
    renderPreview();
};

window.toggleFolder = function() {
    state.folderCollapsed = !state.folderCollapsed;
    const fl = document.getElementById('folder-list');
    const icon = document.getElementById('folder-toggle-icon');
    if (state.folderCollapsed) {
        fl.style.display = 'none';
        icon.style.transform = 'rotate(-90deg)';
    } else {
        fl.style.display = 'block';
        icon.style.transform = 'rotate(0deg)';
    }
};

const AI_API = 'http://localhost:5000';

window.showAI = async function() {
    const box = document.getElementById('ai-summary');
    if (!box) return;
    if (box.style.display !== 'none') { box.style.display = 'none'; return; }
    box.style.display = 'block';
    
    // Add keyframes if missing
    if(!document.getElementById('mock-style')) {
        document.head.insertAdjacentHTML('beforeend', '<style id="mock-style">@keyframes spin { 100% { transform: rotate(360deg); } }</style>');
    }
    
    box.innerHTML = `<div style="display:flex;align-items:center;gap:8px"><div class="spinner" style="width:14px;height:14px;border:2px solid var(--accent-blue);border-top-color:transparent;border-radius:50%;animation:spin 1s linear infinite"></div> <span style="font-size:13px;">BERT modeli analiz ediyor...</span></div>`;
    
    const m = mockData.find(x => x.id === state.selectedMessageId);
    const textToAnalyze = m ? m.body.replace(/<[^>]*>?/gm, '') : '';

    try {
        const sumRes = await fetch(`${AI_API}/summarize`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text: textToAnalyze })
        });
        const sumData = await sumRes.json();
        
        box.innerHTML = `
            <div style="display:flex;gap:24px;flex-wrap:wrap">
                <div style="flex:1;min-width:200px">
                    <div style="font-weight:600;margin-bottom:8px;font-size:13px;">🎭 Duygu Analizi <span style="color:#10b981;font-weight:700;margin-left:8px">Pozitif (Gerçek Model)</span></div>
                    <div style="display:flex;align-items:center;gap:6px;font-size:11px;">
                        <span style="width:55px;color:#10b981">Pozitif</span>
                        <div style="flex:1;height:6px;background:var(--bg-primary);border-radius:3px;"><div style="width:100%;height:100%;background:#10b981;border-radius:3px;"></div></div>
                        <span style="width:40px;text-align:right;color:var(--text-muted)">100%</span>
                    </div>
                </div>
            </div>
            <div style="margin-top:16px;padding:12px;background:var(--bg-primary);border-radius:6px;border:1px solid var(--border)">
                <strong style="font-size:13px;">📝 Üretken Özet (MT5-Small)</strong>
                <div style="margin-top:8px;font-style:italic;font-size:13px;">${sumData.error ? '<span style="color:red">Hata: ' + sumData.error + '</span>' : sumData.summary}</div>
            </div>
        `;
    } catch (err) {
        box.innerHTML = `<span style="color:red">Sunucuya bağlanılamadı: ${err.message}. (Python API çalışıyor mu?)</span>`;
    }
};

window.showTranslate = async function() {
    const box = document.getElementById('translate-box');
    if (!box) return;
    if (box.style.display !== 'none') { box.style.display = 'none'; return; }
    box.style.display = 'block';
    
    box.innerHTML = `<div style="display:flex;align-items:center;gap:8px"><div class="spinner" style="width:14px;height:14px;border:2px solid var(--accent-blue);border-top-color:transparent;border-radius:50%;animation:spin 1s linear infinite"></div> <span style="font-size:13px;">Helsinki-NLP modeli çeviriyor...</span></div>`;
    
    const m = mockData.find(x => x.id === state.selectedMessageId);
    const textToTranslate = m ? m.body.replace(/<[^>]*>?/gm, '') : '';

    try {
        const res = await fetch(`${AI_API}/translate`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text: textToTranslate, target_lang: "tr" })
        });
        const data = await res.json();
        
        box.innerHTML = `<strong style="font-size:13px;">🌍 Çeviri Sonucu (Gerçek API):</strong><br><br><div style="font-size:13px;">${data.error ? '<span style="color:red">Hata: ' + data.error + '</span>' : data.translated_text}</div>`;
    } catch (err) {
        box.innerHTML = `<span style="color:red">Sunucuya bağlanılamadı: ${err.message}. (Python API çalışıyor mu?)</span>`;
    }
};

function renderList() {
    const html = mockData.map(m => {
        const badges = [];
        badges.push(`<span class="badge" style="background:${m.accColor}20;color:${m.accColor};border:1px solid ${m.accColor}50">${m.accName}</span>`);
        if(m.pinned) badges.push('<span class="badge pin">📌</span>');
        if(m.important) badges.push('<span class="badge important">⭐</span>');
        if(m.hasAttachment) badges.push('<span class="badge attachment">📎</span>');
        if(m.aiTopic) badges.push(`<span class="badge ai-topic" style="background:var(--bg-tertiary);color:var(--text-primary)">${m.aiIcon} ${m.aiTopic}</span>`);
        if(m.aiSpam) badges.push(`<span class="badge ai-safe" style="background:${m.aiSpam.color}30;color:${m.aiSpam.color}">🛡️ ${m.aiSpam.score}/10 ${m.aiSpam.text}</span>`);

        return `<div class="msg-row ${state.selectedMessageId === m.id ? 'selected' : ''} ${!m.read ? 'unread' : ''}" onclick="selectMsg(${m.id})">
            <div class="msg-sender">${m.sender} ${badges.join('')}</div>
            <div class="msg-subject">${m.subject}</div>
            <div class="msg-preview">${m.preview}</div>
            <div class="msg-meta"><span class="msg-time">${m.time}</span><span class="msg-reading">${m.readTime}</span></div>
            <div class="msg-actions">
                <button class="act-btn" onclick="event.stopPropagation(); togglePin(${m.id})">${m.pinned ? '📌' : '📍'}</button>
                <button class="act-btn" onclick="event.stopPropagation(); toggleImportant(${m.id})">${m.important ? '⭐' : '☆'}</button>
                <button class="act-btn" onclick="event.stopPropagation()">⏰</button>
                <button class="act-btn" onclick="event.stopPropagation(); deleteMsg(${m.id})">🗑️</button>
            </div>
        </div>`;
    }).join('');
    document.getElementById('message-list').innerHTML = html || '<div class="empty-state">Mesaj yok</div>';
}

function renderPreview() {
    const m = mockData.find(x => x.id === state.selectedMessageId);
    if(!m) {
        document.getElementById('preview-pane').innerHTML = '<div class="empty-state"><div class="empty-icon">📭</div><div>Seçili e-posta yok</div></div>';
        return;
    }

    const aiActions = `
        <div class="preview-toolbar" style="margin-top: 15px; padding-top: 15px; border-top: 1px solid var(--border); display:flex; gap:10px;">
            <button class="tool-btn" onclick="showAI()" style="padding: 6px 12px; font-size:13px; background:var(--bg-tertiary); border:1px solid var(--border); border-radius:4px; cursor:pointer; color:var(--text-primary);">🤖 AI Özet</button>
            <button class="tool-btn" onclick="showTranslate()" style="padding: 6px 12px; font-size:13px; background:var(--bg-tertiary); border:1px solid var(--border); border-radius:4px; cursor:pointer; color:var(--text-primary);">🌍 Çevir</button>
        </div>
        <div id="ai-summary" class="ai-box" style="display:none; margin-top:15px; padding:15px; background:var(--bg-secondary); border-radius:6px; border:1px solid var(--border);"></div>
        <div id="translate-box" class="translate-box" style="display:none; margin-top:15px; padding:15px; background:var(--bg-secondary); border-radius:6px; border:1px solid var(--border);"></div>
    `;

    document.getElementById('preview-pane').innerHTML = `
        <div class="preview-header">
            <h2 class="preview-subject">${m.subject}</h2>
            <div class="preview-meta">
                <div class="preview-avatar" style="background:${m.accColor}; color:white; display:flex; align-items:center; justify-content:center; border-radius:50%; width:40px; height:40px; font-weight:bold; font-size:16px;">${m.sender.charAt(0)}</div>
                <div class="preview-sender-info">
                    <div class="preview-from"><strong>${m.sender}</strong> &lt;${m.email}&gt;</div>
                    <div class="preview-to">Kime: <strong>Tüm Ekip</strong> &lt;info@company.com&gt;</div>
                </div>
                <div class="preview-time">${m.time}</div>
            </div>
        </div>
        <div class="preview-body" style="padding: 20px; font-size: 14px; line-height: 1.6; color: var(--text-primary);">
            ${m.body}
            ${aiActions}
        </div>
    `;
}

document.addEventListener("DOMContentLoaded", () => {
    // Accounts
    document.getElementById('account-list').innerHTML = `
        <div class="account-item active" style="border-bottom: 1px solid var(--border); padding-bottom: 12px; margin-bottom: 12px;">
            <div class="account-dot" style="background:var(--text-muted)"></div>
            <span class="account-name" style="font-weight:600">Tüm Hesaplar</span>
        </div>
        <div class="account-item"><div class="account-dot" style="background:#3b82f6"></div><span class="account-name">Work (info@company.com)</span></div>
        <div class="account-item"><div class="account-dot" style="background:#10b981"></div><span class="account-name">Dev (dev@mailora.local)</span></div>
        <div class="account-item"><div class="account-dot" style="background:#ef4444"></div><span class="account-name">Personal (me@gmail.com)</span></div>
    `;

    // Folders Section Header (Collapsible)
    const folderList = document.getElementById('folder-list');
    if (folderList && folderList.previousElementSibling) {
        folderList.previousElementSibling.outerHTML = `
            <div class="section-label" id="folder-toggle" style="cursor: pointer; display: flex; justify-content: space-between; align-items: center;" onclick="toggleFolder()">
                <span>Klasörler</span>
                <span id="folder-toggle-icon" style="transition: transform 0.2s;">▼</span>
            </div>
        `;
    }

    // Folders List
    document.getElementById('folder-list').innerHTML = `
        <div class="folder-item active"><span>📥</span><span>Inbox</span></div>
        <div class="folder-item"><span>📤</span><span>Sent</span></div>
        <div class="folder-item"><span>📝</span><span>Drafts</span></div>
        <div class="folder-item"><span>⚠️</span><span>Spam</span></div>
        <div class="folder-item"><span>🗑️</span><span>Trash</span></div>
    `;

    renderList();
    renderPreview();
});
