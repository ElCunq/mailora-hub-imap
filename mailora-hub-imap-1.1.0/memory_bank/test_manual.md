# Manual Test Plan

## 1. Auto-Discovery
- [ ] **Gmail:** Enter `user@gmail.com` + App Password. Verify `imap.gmail.com` is found.
- [ ] **Outlook:** Enter `user@outlook.com`. Verify `outlook.office365.com` is found.
- [ ] **Unknown Domain:** Enter `user@unknown.com`. Verify "Could not find settings" message.
- [ ] **Manual Override:** Toggle "Manuel Kurulum" and enter details manually.

## 2. Account Management
- [ ] **Authentication:** Verify login with valid credentials creates account.
- [ ] **Listing:** Added account appears in sidebar.
- [ ] **Deletion:** Removing account clears it from sidebar and DB.
- [ ] **Edit Settings:** Changing display name in "Ayarlar" works (`PATCH /accounts`).

## 3. Messaging
- [ ] **Sync:** Inbox shows messages. Unread count matches server.
- [ ] **View:** Body content (HTML/Text) renders correctly.
- [ ] **Attachments:** Can list and download attachments.
- [ ] **Send:** Can send email to self; appears in Sent folder.
- [ ] **Flags:** Mark as read/unread updates UI and persists.

## 4. UI/UX
- [ ] **Dark Mode:** All pages (Login, App, Settings) use unified dark theme.
- [ ] **Responsiveness:** Sidebar toggles correctly on mobile view.
- [ ] **Feedback:** Success/Error toasts appear for actions (e.g. "Hesap eklendi").
