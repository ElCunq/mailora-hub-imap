import re

with open('static/admin.html', 'r', encoding='utf-8') as f:
    html = f.read()

new_script = """
        const authToken = localStorage.getItem('auth_token');
        const authRole = localStorage.getItem('auth_role');
        const authUser = localStorage.getItem('auth_user');

        if (!authToken || authRole !== 'Admin') {
            window.location.href = 'login.html';
        }

        const apiFetch = (url, options = {}) => {
            if (!options.headers) options.headers = {};
            options.headers['Authorization'] = authToken;
            options.headers['Content-Type'] = 'application/json';
            return fetch(url, options);
        };

        let systemLogs = [];

        async function loadDashboardStats() {
            try {
                const res = await apiFetch('/admin/stats');
                if (!res.ok) throw new Error('Stats API failed');
                const stats = await res.json();
                
                // Update Dashboard Cards
                document.querySelector('.stat-box.blue .stat-number').textContent = stats.total_emails.toLocaleString();
                document.querySelector('.stat-box.purple .stat-number').textContent = stats.total_users;
                document.querySelector('.stat-box.orange .stat-number').textContent = stats.spam_blocked.toLocaleString();
                document.querySelector('.stat-box.red .stat-number').textContent = stats.error_count;
                
                // Update Logs
                systemLogs = stats.recent_logs || [];
                document.getElementById('log-badge').textContent = systemLogs.length;
                renderLogs('all');
            } catch (e) {
                console.error(e);
            }
        }

        function renderLogs(filter = 'all') {
            const list = document.getElementById('log-list');
            const filtered = filter === 'all' ? systemLogs : systemLogs.filter(l => {
                if (filter === 'error') return l.action.toLowerCase().includes('error') || (l.details && l.details.toLowerCase().includes('error'));
                if (filter === 'success') return l.action.toLowerCase().includes('success') || (l.details && l.details.toLowerCase().includes('success'));
                return true; // Simplified for demo
            });
            
            if(filtered.length === 0) {
                list.innerHTML = '<div style="padding:10px; color:var(--text-muted); font-size:12px;">Log bulunamadı</div>';
                return;
            }

            list.innerHTML = filtered.map(l => {
                const isErr = l.action.toLowerCase().includes('error') || (l.details && l.details.toLowerCase().includes('error'));
                const level = isErr ? 'error' : 'info';
                const timeStr = l.created_at.split('.')[0].replace('T', ' ');
                return `
                <div class="log-row">
                    <span class="log-time">${timeStr}</span>
                    <span class="log-level ${level}">${level}</span>
                    <span class="log-source">[${l.action}]</span>
                    <span class="log-msg">${l.details || ''} (Kullanıcı: ${l.user_id || '-'})</span>
                </div>
            `}).join('');
        }

        function filterLogs(level, btn) {
            document.querySelectorAll('.log-filter-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            renderLogs(level);
        }

        async function loadUsers() {
            try {
                const res = await apiFetch('/admin/users');
                const users = await res.json();
                const tbody = document.getElementById('user-table-body');
                tbody.innerHTML = '';
                
                if(users.length === 0) {
                    tbody.innerHTML = '<tr><td colspan="4" style="text-align:center">Kullanıcı bulunamadı</td></tr>';
                    return;
                }

                users.forEach(u => {
                    const color = u.role === 'Admin' ? '#ef4444' : '#3b82f6';
                    const tr = document.createElement('tr');
                    tr.innerHTML = `
                        <td>
                            <div class="user-name-cell">
                                <div class="user-avatar" style="background:${color}">${u.username[0].toUpperCase()}</div>
                                <div>
                                    <div style="font-weight:600">${u.username}</div>
                                    <div class="user-email">ID: ${u.id}</div>
                                </div>
                            </div>
                        </td>
                        <td>
                            <select class="form-select" onchange="updateRole(${u.id}, this.value)">
                                <option value="Member" ${u.role === 'Member' ? 'selected' : ''}>Member</option>
                                <option value="Admin" ${u.role === 'Admin' ? 'selected' : ''}>Admin</option>
                            </select>
                        </td>
                        <td>
                            <button class="form-btn secondary" onclick="manageAccounts(${u.id}, '${u.username}')">Yetkileri Yönet</button>
                        </td>
                        <td>
                            <button class="form-btn danger" onclick="deleteUser(${u.id}, '${u.username}')">🗑️ Sil</button>
                        </td>
                    `;
                    tbody.appendChild(tr);
                });
            } catch (e) {
                console.error(e);
            }
        }

        async function updateRole(userId, newRole) {
            const res = await apiFetch(`/admin/users/${userId}/role`, {
                method: 'PATCH',
                body: JSON.stringify({ role: newRole })
            });
            if (!res.ok) alert('Rol güncellenemedi');
        }

        async function deleteUser(userId, username) {
            if(!confirm(`DİKKAT: ${username} adlı kullanıcıyı tamamen silmek istediğinize emin misiniz? Bu işlem geri alınamaz!`)) return;
            const res = await apiFetch(`/admin/users/${userId}`, { method: 'DELETE' });
            if(res.ok) {
                loadUsers();
                loadDashboardStats();
            } else {
                alert('Kullanıcı silinemedi');
            }
        }

        let currentEditingUserId = null;

        async function manageAccounts(userId, username) {
            currentEditingUserId = userId;
            document.getElementById('modal-title').textContent = `${username} - Mail Kutusu Yetkileri`;
            document.getElementById('accounts-modal').style.display = 'flex';

            const [allRes, assignedRes] = await Promise.all([
                apiFetch('/accounts'),
                apiFetch(`/admin/users/${userId}/accounts`)
            ]);

            const all = await allRes.json();
            const assigned = await assignedRes.json();

            const listEl = document.getElementById('modal-accounts');
            listEl.innerHTML = '';

            all.forEach(acc => {
                const isAssigned = assigned.includes(acc.id);
                const chip = document.createElement('div');
                chip.className = `account-chip ${isAssigned ? 'assigned' : ''}`;
                chip.innerHTML = `
                    <span>${acc.email}</span>
                    <button onclick="toggleAccount('${acc.id}', ${isAssigned})">${isAssigned ? 'Kaldır' : 'Yetki Ver'}</button>
                `;
                listEl.appendChild(chip);
            });
        }

        async function toggleAccount(accountId, currentlyAssigned) {
            const method = currentlyAssigned ? 'DELETE' : 'POST';
            const url = currentlyAssigned
                ? `/admin/users/${currentEditingUserId}/accounts/${accountId}`
                : `/admin/users/${currentEditingUserId}/accounts`;

            const res = await apiFetch(url, {
                method,
                body: JSON.stringify({ account_id: accountId })
            });

            if (res.ok) {
                manageAccounts(currentEditingUserId, document.getElementById('modal-title').textContent.split(' - ')[0]);
            } else {
                alert('İşlem başarısız');
            }
        }

        function closeModal() {
            document.getElementById('accounts-modal').style.display = 'none';
        }

        async function loadAccountsTable() {
            try {
                const res = await apiFetch('/accounts');
                const accounts = await res.json();
                const tbody = document.getElementById('accounts-table-body');
                tbody.innerHTML = '';

                if(accounts.length === 0) {
                    tbody.innerHTML = '<tr><td colspan="3" style="text-align:center">Hesap bulunamadı</td></tr>';
                    return;
                }

                accounts.forEach(a => {
                    const color = a.color || '#3b82f6';
                    const tr = document.createElement('tr');
                    tr.innerHTML = `
                        <td>
                            <div style="font-weight:600">${a.email}</div>
                            <div style="font-size:11px; color:var(--text-muted)">ID: ${a.id}</div>
                        </td>
                        <td>
                            <div style="display:flex; align-items:center; gap:8px;">
                                <div style="width:20px; height:20px; border-radius:4px; background:${color}"></div>
                                <span style="font-family:monospace; font-size:12px;">${color}</span>
                            </div>
                        </td>
                        <td style="display:flex; gap:8px; align-items:center;">
                            <button class="form-btn secondary" onclick="openEditAccount('${a.id}', '${a.email}', '${color}', '${a.carddav_url || ''}', '${a.caldav_url || ''}')">Düzenle</button>
                            <button class="form-btn danger" onclick="deleteAccount('${a.id}', '${a.email}')">🗑️ Sil</button>
                        </td>
                    `;
                    tbody.appendChild(tr);
                });
            } catch(e) {
                console.error(e);
            }
        }

        function openEditAccount(id, email, color, carddavUrl, caldavUrl) {
            document.getElementById('edit-acc-id').value = id;
            document.getElementById('edit-acc-email').value = email;
            document.getElementById('edit-acc-color').value = color;
            document.getElementById('color-hex').textContent = color;
            document.getElementById('edit-acc-carddav').value = carddavUrl || '';
            document.getElementById('edit-acc-caldav').value = caldavUrl || '';
            document.getElementById('edit-account-modal').style.display = 'flex';
        }

        // Live hex update
        document.getElementById('edit-acc-color').addEventListener('input', (e) => {
            document.getElementById('color-hex').textContent = e.target.value;
        });

        async function saveAccountSettings() {
            const id = document.getElementById('edit-acc-id').value;
            const color = document.getElementById('edit-acc-color').value;
            const carddavUrl = document.getElementById('edit-acc-carddav').value.trim();
            const caldavUrl = document.getElementById('edit-acc-caldav').value.trim();

            const res = await apiFetch(`/accounts/${id}`, {
                method: 'PATCH',
                body: JSON.stringify({ 
                    color: color,
                    carddav_url: carddavUrl || null,
                    caldav_url: caldavUrl || null
                })
            });

            if (res.ok) {
                closeEditModal();
                loadAccountsTable();
            } else {
                alert('Kaydedilemedi');
            }
        }

        function closeEditModal() {
            document.getElementById('edit-account-modal').style.display = 'none';
        }

        async function deleteAccount(id, email) {
            if(!confirm(`DİKKAT: ${email} hesabını tamamen silmek istediğinize emin misiniz?`)) return;
            const res = await apiFetch(`/accounts/${id}`, { method: 'DELETE' });
            if(res.ok) {
                loadAccountsTable();
            } else {
                alert('Hesap silinemedi');
            }
        }

        // Sidebar Navigation
        function showSection(section, btn) {
            ['dashboard', 'logs', 'users', 'accounts', 'ai'].forEach(s => {
                const el = document.getElementById('section-' + s);
                if (el) el.style.display = s === section ? 'block' : 'none';
            });
            document.querySelectorAll('.admin-subnav-item').forEach(b => b.classList.remove('active'));
            if (btn) btn.classList.add('active');
            else {
                document.querySelectorAll('.admin-subnav-item').forEach(b => { 
                    if (b.onclick.toString().includes(section)) b.classList.add('active'); 
                });
            }

            document.querySelectorAll('.admin-menu-item').forEach(m => m.classList.remove('active'));
            document.querySelectorAll('.admin-menu-item').forEach(m => { 
                if (m.onclick.toString().includes(section)) m.classList.add('active'); 
            });

            // Mock Data for AI Section as it doesn't have real endpoint yet
            if (section === 'ai') {
                renderBarChart('ai-daily-chart', [
                    { label: '1', value: 42 }, { label: '2', value: 58 }, { label: '3', value: 35 }, { label: '4', value: 67 }, { label: '5', value: 89 },
                    { label: '6', value: 74 }, { label: '7', value: 62 }, { label: '8', value: 91 }, { label: '9', value: 53 }, { label: '10', value: 78 },
                    { label: '11', value: 85 }, { label: '12', value: 96 }, { label: '13', value: 72 }, { label: '14', value: 108 }
                ], ['#3b82f6', '#8b5cf6']);
            }
        }

        function refreshLogs() {
            loadDashboardStats();
        }

        function exportReport() {
            const report = `Mailora Admin Raporu\nTarih: ${new Date().toLocaleString('tr-TR')}\n(Bu özellik yapım aşamasındadır)\n`;
            const blob = new Blob([report], { type: 'text/plain' });
            const a = document.createElement('a');
            a.href = URL.createObjectURL(blob);
            a.download = `mailora_rapor_${new Date().toISOString().split('T')[0]}.txt`;
            a.click();
        }

        function toggleTheme() {
            const html = document.documentElement;
            const next = html.dataset.theme === 'dark' ? 'light' : 'dark';
            html.dataset.theme = next;
            document.getElementById('theme-btn').textContent = next === 'dark' ? '🌙' : '☀️';
        }

        // Mock Rendering functions for dashboard charts (since we don't have real timeseries data yet)
        function renderBarChart(id, data, colors) {
            const chart = document.getElementById(id);
            if(!chart) return;
            const max = Math.max(...data.map(d => d.value));
            chart.innerHTML = data.map((d, i) => {
                const h = (d.value / max) * 100;
                const color = colors ? colors[i % colors.length] : 'var(--accent-blue)';
                return `<div class="bar-col">
                    <div class="bar-fill" style="height:${h}%;background:${color}" data-val="${d.value}"></div>
                    <span class="bar-label">${d.label}</span>
                </div>`;
            }).join('');
        }

        function renderDonut() {
            const data = [
                { label: 'İş/Proje', value: 28, color: '#3b82f6' },
                { label: 'Finans', value: 18, color: '#10b981' },
                { label: 'Pazarlama', value: 22, color: '#8b5cf6' },
                { label: 'Kişisel', value: 15, color: '#f59e0b' },
                { label: 'Diğer', value: 17, color: '#94a3b8' },
            ];
            const total = data.reduce((s, d) => s + d.value, 0);
            let offset = 0;
            const svg = document.getElementById('donut-chart');
            const legend = document.getElementById('donut-legend');
            if(!svg || !legend) return;

            svg.innerHTML = data.map(d => {
                const pct = (d.value / total) * 100;
                const dash = `${pct * 2.51327} ${251.327 - pct * 2.51327}`;
                const seg = `<circle cx="50" cy="50" r="40" fill="none" stroke="${d.color}" stroke-width="14"
                    stroke-dasharray="${dash}" stroke-dashoffset="${-offset * 2.51327}"
                    style="transition:stroke-dasharray 0.6s" />`;
                offset += pct;
                return seg;
            }).join('') + '<circle cx="50" cy="50" r="28" fill="var(--bg-secondary)"/><text x="50" y="48" text-anchor="middle" fill="var(--text-primary)" font-size="10" font-weight="700">' + total + '</text><text x="50" y="58" text-anchor="middle" fill="var(--text-muted)" font-size="6">Toplam</text>';

            legend.innerHTML = data.map(d => `
                <div class="donut-legend-item">
                    <div class="donut-legend-dot" style="background:${d.color}"></div>
                    <span>${d.label}</span>
                    <span class="donut-legend-val">${d.value}%</span>
                </div>
            `).join('');
        }

        // Initialize App
        async function init() {
            await loadDashboardStats();
            await loadUsers();
            await loadAccountsTable();
            
            // Render mock charts
            renderBarChart('traffic-chart', [
                { label: 'Pzt', value: 186 }, { label: 'Sal', value: 215 }, { label: 'Çar', value: 178 }, { label: 'Per', value: 245 },
                { label: 'Cum', value: 198 }, { label: 'Cmt', value: 87 }, { label: 'Pzr', value: 52 }
            ], ['#3b82f6', '#3b82f6', '#3b82f6', '#8b5cf6', '#3b82f6', '#64748b', '#64748b']);
            renderBarChart('ai-perf-chart', [
                { label: 'Konu v3', value: 96.3 }, { label: 'Spam', value: 87 }, { label: 'Duygu', value: 89 },
                { label: 'Çeviri', value: 92 }, { label: 'NER', value: 88 }, { label: 'Smart Reply', value: 79 }
            ], ['#10b981', '#f59e0b', '#3b82f6', '#8b5cf6', '#ef4444', '#f59e0b']);
            renderDonut();
        }

        init();
"""

script_pattern = re.compile(r'<script>.*?</script>', re.DOTALL)
new_html = script_pattern.sub(f'<script>\n{new_script}\n    </script>', html)

with open('static/admin.html', 'w', encoding='utf-8') as f:
    f.write(new_html)

print("Patching complete!")
