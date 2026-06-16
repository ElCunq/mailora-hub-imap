import re

file_path = 'static/Example/admin.html'
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Remove sidebar and subnav items
content = re.sub(r'<div class="admin-menu-item" onclick="showSection\(\'rbac\'\)">.*?</div>\n', '', content)
content = re.sub(r'<button class="admin-subnav-item" onclick="showSection\(\'rbac\',this\)">.*?</button>\n', '', content)

# 2. Remove section-rbac block
content = re.sub(r'<!-- RBAC Section -->.*?<div id="section-ai"', '<!-- AI Reports Section -->\n            <div id="section-ai"', content, flags=re.DOTALL)

# 3. Update table headers in users section
old_thead = '<tr><th>Kullanıcı</th><th>Rol</th><th>Durum</th><th>Son Giriş</th><th>E-posta Sayısı</th></tr>'
new_thead = '<tr><th>Kullanıcı</th><th>Rol</th><th>Yetkiler</th><th>İşlemler</th></tr>'
content = content.replace(old_thead, new_thead)

# 4. Update renderUsers function
old_renderUsers = r"""        function renderUsers() {
            document.getElementById('user-table-body').innerHTML = users.map(u => `
                <tr>
                    <td><div class="user-name-cell">
                        <div class="user-avatar" style="background:${u.color}">${u.name[0]}</div>
                        <div><div>${u.name}</div><div class="user-email">${u.email}</div></div>
                    </div></td>
                    <td><span class="role-badge ${u.role}">${u.role.charAt(0).toUpperCase() + u.role.slice(1)}</span></td>
                    <td><span class="status-dot ${u.online ? 'online' : 'offline'}"></span>${u.online ? 'Çevrimiçi' : 'Çevrimdışı'}</td>
                    <td style="color:var(--text-muted)">${u.lastLogin}</td>
                    <td><strong>${u.emails}</strong></td>
                </tr>
            `).join('');
        }"""

new_renderUsers = r"""        function renderUsers() {
            document.getElementById('user-table-body').innerHTML = users.map((u, index) => {
                const color = u.role === 'admin' ? '#ef4444' : '#3b82f6';
                const roleUpper = u.role.charAt(0).toUpperCase() + u.role.slice(1);
                return `
                <tr>
                    <td>
                        <div class="user-name-cell">
                            <div class="user-avatar" style="background:${color}">${u.name[0]}</div>
                            <div>
                                <div style="font-weight:600">${u.name}</div>
                                <div class="user-email">${u.email}</div>
                            </div>
                        </div>
                    </td>
                    <td>
                        <select style="padding:6px; border:1px solid var(--border); border-radius:6px; background:var(--bg-tertiary); color:var(--text-primary); cursor:pointer;">
                            <option value="Member" ${u.role !== 'admin' ? 'selected' : ''}>Member</option>
                            <option value="Admin" ${u.role === 'admin' ? 'selected' : ''}>Admin</option>
                        </select>
                    </td>
                    <td>
                        <button style="padding:6px 12px; border:1px solid var(--border); border-radius:6px; background:var(--bg-secondary); color:var(--text-primary); cursor:pointer; font-weight:500;">Yetkileri Yönet</button>
                    </td>
                    <td>
                        <button style="padding:6px 12px; border:none; border-radius:6px; background:var(--accent-red); color:white; cursor:pointer; font-weight:500;">🗑️ Sil</button>
                    </td>
                </tr>
                `;
            }).join('');
        }"""

content = content.replace(old_renderUsers, new_renderUsers)

# 5. Fix showSection
content = content.replace("['dashboard', 'logs', 'users', 'ai', 'rbac'].forEach", "['dashboard', 'logs', 'users', 'ai'].forEach")
content = content.replace("? 'kullanıcı' : section === 'rbac' ? 'rbac' : 'ai'", "? 'kullanıcı' : 'ai'")

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)

print("Done")
