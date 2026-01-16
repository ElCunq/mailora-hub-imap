# Milestone 1 Status (Güncel)

Güncelleme: 2025-11-05  
Sürüm Dalı: v0.2.0

## Karar
- ✅ Stalwart/JMAP/async-smtp tamamen kaldırıldı; yalın IMAP/SMTP mimarisi
- ✅ Test UI ile doğrulama; tam UI Faz 2

## Durum
- ✅ IMAP bağlantı ve mesaj gövde testleri çalışıyor
- ✅ SMTP (lettre) test endpoint’i JSON uyumlu
- ⚠️ Deprecation: ClientId::new (gelecekte Domain ile sadeleştirilecek)

## Faz 1 Hedefleri
1) Sent APPEND + UID yakalama
2) UID cursors + delta senkron
3) Flags iki yönlü + Trash/Sent/Junk rol eşlemesi
4) Dayanıklılık: timeout/backoff, single-flight
5) Güvenlik: at-rest secret, log redaction

## Kabul Kriterleri
- APPEND sonrası Gmail/Outlook’ta tek kopya; UID DB’de
- Delta sync tekrar yaratmaz; UIDVALIDITY değişiminde reset
- Flags iki yönlü 10 sn içinde tutarlı
- Tüm hatalar JSON ve kod/hint içerir

## Takvim (öneri)
- Sprint 1: (1)(2)(5)
- Sprint 2: (3)(4)
