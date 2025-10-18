# IMAP Facts (Mailora Hub IMAP)

- Gmail UID is per-mailbox; same message can have different UIDs across labels.
- Modified UTF-7 mailbox names appear in LIST; use them verbatim for SELECT.
- UID SEARCH returns a set; use it to derive exact UIDs for fetching.
- BODYSTRUCTURE can be multipart; prefer text/plain then text/html if plain missing.
- Some servers delay FETCH immediately after EXISTS; NOOP/sleep helps stabilize.

---

# IMAP Sunucu Farkları (Kısa Notlar)

Gmail
- Etiket tabanlı model; bazı komutlara X-GM-EXT uzantıları eşlik eder.
- All Mail arşivi büyük olabilir; listeleme/arama dikkatli yapılmalı.
- ESEARCH ve IDLE genellikle iyi desteklenir.

Outlook/O365
- Modern IMAP uygulaması yaygın; bazı ortamlarda IDLE sınırlanabilir.
- Büyük klasörlerde arama/yanıt süreleri değişken olabilir; zaman aşımlarına hazırlıklı olun.

Dovecot
- Standartlara uyumlu ve öngörülebilir davranış; CONDSTORE/IDLE desteği sık görülür.

Fastmail
- Geniş IMAP özelliği desteği; ESEARCH/IDLE iyi çalışır.

Genel İpuçları
- UID/UIDVALIDITY takibi zorunlu; klasör değişimlerinde yeniden açma stratejisi hazır olmalı.
- BODYSTRUCTURE’ı tek sefer alıp yalnız gerekli BODY[section] parçalarını indirin.
- Hata ve zaman aşımı durumlarında NOOP/yeniden deneme ve ölçümleme.
