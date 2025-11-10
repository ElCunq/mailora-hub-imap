# Active Context

Tarih: 2025-11-10
Branch: v0.2.1 (origin/v0.2.1)

Odak
- Unified Inbox: `/unified/inbox?folder=...` ile hesaplar arası birleşik liste.
- Başlangıçta tam senkron (non-Gmail) ve arkaplan scheduler çalışıyor.
- Basit 3-panelli web istemci: hesap ekle (host/port), klasör seç, unified toggle, liste/önizleme/compose.

Sıradaki İşler (Faz 2 – v0.2.x)
- Attachments hattı (liste/indir) ve UI entegrasyonu.
- Unified arama endpoint’i + UI arayüzü.
- Kalıcı kuyruklar (outbox, sent finalize) ve retry/backoff.
- Error envelope standardizasyonu.
- Scheduler jitter/backoff + IDLE reconnect.
- İndeksler ve performans.

Kararlar
- Gmail özelleştirmeleri şimdilik dışarıda; hesap listesi ve scheduler Gmail’i atlar.
- DB sorguları dinamik SQL ile esnek.
- Metrics ikinci planda.
