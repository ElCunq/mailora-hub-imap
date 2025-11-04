# IMAP/SMTP-only Destek
Bu projede OAuth2 ile giriş ve senkronizasyon tamamen kaldırılmıştır. Artık sadece IMAP/SMTP (uygulama şifresi) ile e-posta hesapları eklenebilir ve senkronize edilebilir.

## Kaldırılanlar
- OAuth2 ile ilgili tüm backend kodları
- UI'daki OAuth2 butonları ve açıklamalar
- Veritabanındaki OAuth2 alanları
- Outlook, Gmail OAuth2, Yahoo OAuth2 desteği

## Kalanlar
- Sadece IMAP/SMTP (app password) ile giriş ve senkronizasyon
- Temizlenmiş kod tabanı ve arayüz

## Kullanım
Hesap eklerken e-posta adresinizi ve uygulama şifrenizi girin. OAuth2 ile ilgili hiçbir alan veya seçenek yoktur.

---
Son güncelleme: 23 Ekim 2025
