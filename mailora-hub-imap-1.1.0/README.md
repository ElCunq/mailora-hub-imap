# Mailora Hub IMAP/SMTP

Bu sistem, OAuth2 ve diğer karmaşık kimlik doğrulama yöntemleri olmadan, sadece IMAP/SMTP (app password) ile e-posta hesaplarını yönetmek için tasarlanmıştır.

## Özellikler
- **Çoklu Hesap:** Gmail, Outlook ve özel IMAP sunucuları.
- **Senkronizasyon:** IMAP IDLE ile anlık bildirimler, delta sync.
- **Mesaj Okuma:** HTML/Plain text desteği, güvenli görüntüleme (sanitizasyon).
- **Ekler:** Ekleri listeleme, indirme, inline görsel desteği.
- **SMTP:** E-posta gönderme.
- **Veritabanı:** SQLite ile yerel depolama.
- **Docker:** Kolay kurulum ve dağıtım.

## Kurulum

### Docker ile Çalıştırma (Önerilen)
1. Docker ve Docker Compose'un kurulu olduğundan emin olun.
2. Projeyi klonlayın.
3. `docker-compose up -d` komutunu çalıştırın.
4. Uygulama `http://localhost:3030` adresinde çalışacaktır.

### Manuel Kurulum
1. Rust ve Cargo kurulu olmalı.
2. `cargo run` ile başlatın.

## Kullanım
- Arayüz üzerinden (`/app.html`) hesap ekleyin (E-posta ve Uygulama Şifresi).
- Mesajları okuyun, ekleri indirin, yeni e-posta gönderin.

## Notlar
- OAuth2 desteği şimdilik devre dışıdır (App Password kullanın).
- Veritabanı `data/` klasöründe saklanır (Docker volume).

---
Son güncelleme: 16 Aralık 2025 (v1.0.0)
