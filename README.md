# Mailora Hub IMAP/SMTP

Bu sistem, OAuth2 ve diğer karmaşık kimlik doğrulama yöntemleri olmadan, sadece IMAP/SMTP (app password) ile e-posta hesaplarını yönetmek için tasarlanmıştır.

## Özellikler
- Çoklu e-posta hesabı ekleme
- IMAP ile mesaj senkronizasyonu
- SMTP ile e-posta gönderme
- Basit ve sade arayüz
- SQLite veritabanı

## Kurulum
1. Rust ve Cargo kurulu olmalı.
2. Ortam değişkenlerini ayarlayın (örnek `.env` dosyası):
   - SMTP_SERVER
   - SMTP_USERNAME
   - SMTP_PASSWORD
3. `cargo run` ile başlatın.

## Kullanım
- Hesap eklerken e-posta ve uygulama şifresi girin.
- Mesajları senkronize edin, SMTP ile test maili gönderin.

## Notlar
- OAuth2, Outlook, Gmail OAuth2, Yahoo OAuth2 desteği kaldırıldı.
- Kod tabanı ve arayüz sadeleştirildi.

---
Son güncelleme: 23 Ekim 2025
