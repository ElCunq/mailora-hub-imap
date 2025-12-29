# System Patterns

Architecture:
- Axum HTTP server exposing /login, /diff, /body, /folders endpoints.
- IMAP client (async-imap) per request; simple credential store; no long-lived pooled sessions yet.

Key decisions:
- Multi-folder cursor with per-folder last_uid; folder emitted in change events.
- Body fetch pipeline with retries and multiple section fallbacks.

Component relationships:
- routes/* -> imap/sync.rs for IMAP ops; services/diff_service.rs for cursors and wire types.

Critical paths:
- /diff incremental: SEARCH ALL > last_uid → UID FETCH → minimal fallback.
- /body: SELECT → UID FETCH meta → choose body section → UID FETCH body.

---

# Mimari Genel Bakış

Bileşenler
- Client (Rust, Tauri UI)
- IMAP Motoru: IDLE, delta takip (UIDNEXT, UIDVALIDITY, HIGHESTMODSEQ), BODYSTRUCTURE → hedefli FETCH
- SMTP Submission: 587/465, PIPELINING
- JMAP (opsiyonel): Enterprise modda hızlı listeleme/sorgu
- Unified Index: SQLite tablosu (messages)
- Event Logger: SQLite tablosu (events); Enterprise modda sunucu event API senkronu
- **Discovery Service:** Email domain üzerinden ISPDB, DNS SRV ve heuristiklerle IMAP/SMTP sunucu tespiti (`src/services/discovery_service.rs`)
- RBAC Görünüm Katmanı: admin vs üye maskesi
- Embedded Server (Personal): Stalwart binary (IMAP=1143, SMTP=1025/1587), loopback

Kritik Akışlar
1) İlk Kurulum Sihirbazı
- Mod seçimi → Personal: Stalwart çıkar/konfigüre et, port keşfi, health-check → hesap ekle
- Enterprise: Uçlar ve kimlik bilgileri, RBAC oturumu → görünüm şekillenir

2) Senkronizasyon
- INBOX için 1× IDLE + talep anında 1× FETCH kanalı
- Önce liste/meta (ENVELOPE/BODYSTRUCTURE), içerik gerektiğinde seçici BODY.PEEK[section]

3) Unified Inbox
- Tüm hesaplardan meta → messages tablosu
- Listeleme tek sorgu; flag/okunma IMAP STORE ile ilgili hesaba yansıtılır

4) Event Log & RBAC
- events: direction(IN/OUT), mailbox, actor, peer, subject, ts
- Admin: actor açık; Üye: actor maskeli
- Enterprise: sunucu event API ile çift yönlü senkron

Tasarım İlkeleri
- “Önce meta” sonra ihtiyaç duyulan içeriği indir (lazy fetch)
- Ağ turları minimize: SEARCH + iki dalga FETCH üst sınırı
- Hata semantiklerini türlendirme ve UI’ye taşıma
