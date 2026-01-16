# Son Durum: Sadece Düz IMAP/SMTP

- Tüm özelleştirilmiş bağlantı türleri (wrapper TLS, farklı mekanizmalar, port mantığı) koddan kaldırıldı.
- Sadece Gmail için önerilen 587 + STARTTLS + LOGIN mekanizması ile SMTP bağlantısı sunuluyor.
- IMAP tarafında da sade ve doğrudan bağlantı kullanılacak.
- Kaldırılan kodlar yorum satırı olarak src/smtp/mod.rs içinde tutuldu.

## Gelecek için Notlar
- İleride wrapper TLS (465), farklı port/mekanizma, Mailtrap/Ethereal gibi test sunucuları veya Stalwart backend entegrasyonu eklenmek istenirse, yorum satırındaki kodlar referans olarak kullanılabilir.
- memory_bank/ klasöründe önceki planlar ve alternatifler arşivlendi.

## Kodda Son Hal
- Sadece tek, deterministik SMTP bağlantısı (Gmail uyumlu)
- Tüm kimlik bilgisi normalize ediliyor
- SNI, EHLO, TLS ayarları sabit
- Debug log aktif

---
Son güncelleme: 2025-10-27
