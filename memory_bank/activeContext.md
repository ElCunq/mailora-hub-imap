# Active Context

Tarih: 2025-11-05
Branch: v0.2.0 (origin/v0.2.0)

Odak
- IMAP/SMTP-only mimari kesinleşti; OAuth2/JMAP/Stalwart kaldırıldı.
- Test UI ile uçtan uca doğrulama yapılıyor; tam arayüz Faz 2’ye ertelendi.
- SMTP test endpoint’i tüm durumlarda JSON dönüyor (UI parse hatası çözüldü).

Sıradaki İşler (Faz 1)
- Sent APPEND (lettre sonrası IMAP APPEND) + UID yakalama
- UID cursors (UIDVALIDITY + last_uid) ve delta senkron
- Flags iki yönlü + Trash/Sent/Junk rol eşlemesi
- Timeout/backoff ve structured logging (request-id, redaction)
- Parolaların at-rest korunması (basit secret veya OS keychain)

Kararlar
- Tam mail UI → Faz 2
- Gmail/Outlook/Yahoo için rol eşleme tablosu eklenecek
- FTS5 ve unified inbox Faz 1 sonuna doğru planlanacak (opsiyonel)
