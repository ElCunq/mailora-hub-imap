# IMAP/SMTP-only Destek

Son Güncelleme: 2025-11-05

Özet
- OAuth2, JMAP ve Stalwart entegrasyonları kaldırıldı.
- SMTP sadece `lettre` ile; async-smtp tamamen çıkarıldı.
- UI tarafında sadece test sayfası var; tam arayüz Faz 2’ye ertelendi.

Ne Değişti
- `async-smtp` bağımlılığı ve `/test/async-smtp/:account_id` uçları kaldırıldı.
- Stalwart API bağlantısı ve ilgili UI formları silindi.
- JMAP proxy rotaları ve durumu kaldırıldı.
- `/test/smtp/:account_id` JSON kontratı garanti altına alındı.

Sonraki Adımlar (Faz 1)
- Sent’e APPEND + UID takibi
- UID cursors & delta sync
- Bayraklar iki yönlü + rol eşlemesi
- Dayanıklılık & güvenlik sertleştirmeleri
