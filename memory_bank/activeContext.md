# Active Context

Tarih: 2025-12-16
Branch: v1.0.0 (origin/v1.0.0)

## Mevcut Durum
Proje **v1.0.0** sürümüne ulaştı. Temel e-posta işlevleri (okuma, gönderme, ekler, çoklu hesap) tamamlandı ve Docker desteği eklendi.

### Son Tamamlananlar (v0.3.0 -> v1.0.0)
- **Docker Desteği:** `Dockerfile` ve `docker-compose.yml` eklendi. Multi-stage build ile optimize edildi.
- **Ek (Attachment) Yönetimi:**
  - Meta veri kaydı ve `backfill` mekanizması.
  - İndirme ve listeleme endpoint'leri.
  - Inline görsel (CID) ve RFC 2231/2047 dosya adı desteği.
- **UI İyileştirmeleri:**
  - Sandbox uyarıları giderildi.
  - Eksik fonksiyonlar (`loadAttachments`, `resolveFolderName`) eklendi.
  - Okundu/Silindi işaretleme ve otomatik yenileme.
- **Dokümantasyon:** `PROJECT_MASTER_REPORT.md` ve `MAILORA_HUB_IMAP_DOCUMENTATION.md` oluşturuldu.

## Odak
- Docker deployment süreçlerinin stabilizasyonu (Coolify entegrasyonu).
- Performans optimizasyonları (indeksleme).

## Sıradaki İşler (Faz 3)
- **Tam Metin Arama (FTS):** SQLite FTS5 entegrasyonu.
- **Kalıcı Kuyruklar:** Giden e-postalar için `Outbox` mekanizması.
- **Hata Yönetimi:** Standartlaştırılmış hata zarfları.
- **JMAP (Opsiyonel):** Büyük posta kutuları için performans artışı.

## Kararlar
- **Docker Build:** `sqlx` prepare adımı build sırasında `sqlx-cli` kurularak çözüldü.
- **Veritabanı:** SQLite WAL modu ile performans artışı hedefleniyor.
