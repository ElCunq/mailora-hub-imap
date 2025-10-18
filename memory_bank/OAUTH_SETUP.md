================================================
🎯 AZURE + GOOGLE CLOUD SETUP REHBERİ
================================================

## MICROSOFT AZURE (Outlook için)

### 1. Azure Portal
https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade

### 2. New Registration
- Name: **Mailora Hub**
- Supported accounts: **Personal Microsoft accounts**
- Redirect URI: `http://localhost:3030/oauth/callback`

### 3. API Permissions (Önemli!)
Microsoft Graph → Delegated permissions:
- [x] Mail.Read
- [x] Mail.ReadWrite  
- [x] Mail.Send
- [x] IMAP.AccessAsUser.All
- [x] SMTP.Send
- [x] offline_access

**Grant admin consent** butonuna tıkla!

### 4. Client Secret
Certificates & secrets → New client secret:
- Description: "Mailora Secret"
- Expires: 24 months
- **VALUE'yu hemen kopyala!**

### 5. Application ID
Overview → **Application (client) ID** kopyala

---

## GOOGLE CLOUD (Gmail için)

### 1. Google Cloud Console
https://console.cloud.google.com/

### 2. New Project
- Project name: **Mailora Hub**
- CREATE

### 3. Enable Gmail API
- APIs & Services → Enable APIs and Services
- "Gmail API" ara → ENABLE

### 4. OAuth Consent Screen
- User Type: **External**
- App name: **Mailora Hub**
- User support email: [senin email]
- Developer email: [senin email]
- Scopes ekle:
  - .../auth/gmail.readonly
  - .../auth/gmail.send
  - .../auth/gmail.modify
- Test users: [senin email ekle]

### 5. Create Credentials
- Create Credentials → OAuth client ID
- Application type: **Web application**
- Name: "Mailora Web"
- Authorized redirect URIs:
  - `http://localhost:3030/oauth/callback`
- CREATE

### 6. Client ID/Secret
- Popup'ta görünür
- **Client ID** ve **Client secret** kopyala

---

## 📝 .ENV DOSYASINA EKLE:

```env
# Microsoft (Outlook)
MICROSOFT_CLIENT_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
MICROSOFT_CLIENT_SECRET=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Google (Gmail)
GOOGLE_CLIENT_ID=xxxxxxxxx.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=GOCSPX-xxxxxxxxxxxxxxxxxxxxx
```

---

## 🚀 SERVER RESTART

```bash
pkill -f mailora-hub-imap
./target/release/mailora-hub-imap &
```

---

## ✅ TEST

1. UI'da: http://localhost:3030/static/app.html
2. "Hesap Ekle" → Provider: Outlook
3. Email gir: cenkorfa@hotmail.com
4. "🔐 OAuth2 ile Giriş" (artık gözükecek)
5. Microsoft login popup → İzin ver
6. ✅ Hesap otomatik eklenir!

================================================
