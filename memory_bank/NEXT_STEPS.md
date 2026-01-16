# NEXT STEPS – Faz 1 (v0.2.0)

Tarih: 2025-11-05
Durum: IMAP/SMTP temel akışlar çalışıyor; test UI ile doğrulandı. Tam arayüz Faz 2’ye ertelendi.

Öncelikler (Sıra ile)
1) SMTP sonrası Sent’e APPEND
- Gönderilen her iletinin IMAP “Sent” klasörüne APPEND edilmesi.
- InternalDate = gönderim zamanı; Flags = (\\Seen)
- Sağlayıcı davranışı: Çift kayıt önleme (özellikle Gmail’de server-side kopya kontrolü), Message-Id ile deduplikasyon.
- APPEND sonucunda UID/UIDVALIDITY yakalanıp DB’ye işlenecek (events/messages).

2) IMAP delta senkron (UID cursors)
- Folder başına UIDVALIDITY + last_uid persist.
- UIDVALIDITY değişiminde full reset, aksi halde sadece yeni UID’lerin meta/body’si alınır.
- Expunge ve flags değişimlerinin tespiti ve işlenmesi.

3) Bayrakların iki yönlü senkronu
- REST uçları: okundu/işaretli/sil (Trash’a taşı), kalıcı silme opsiyonel.
- IMAP STORE/EXPUNGE ile güncelleme; UI’ye JSON durum dönüşü.

4) Klasör rol eşlemesi
- SPECIAL-USE keşfi ve sağlayıcıya özel eşlemeler (Gmail [Gmail]/Sent Mail, Outlook Sent Items, Yahoo Sent, vb.).
- Override/düzeltme tablosu ve kalıcı seçim.

5) Dayanıklılık ve gözlemlenebilirlik
- Timeout’lar, exponential backoff + jitter; single-flight (aynı klasörde eşzamanlı sync engeli).
- Structured logs: request-id, account_id, folder tag’leri; PII redaction.

6) Güvenlik
- Parola at-rest koruma (basit secret key ile şifreleme veya OS keychain); log redaction.

Kabul Kriterleri
- Sent APPEND: Gmail/Outlook testlerinde tek kopya; UID yakalanıp DB’de görünür.
- Delta sync: art arda çağrılarda tekrarsız kayıt; UIDVALIDITY değişiminde temiz reset.
- Flags: 10 sn içinde iki yönlü yansır; Trash/Sent akışları doğru klasörlerle.
- Hata modeli: Tüm API uçları JSON ve hata kodları ile döner.

Sprint Planı (öneri)
- Sprint 1: (1) Sent APPEND, (2) Delta sync, (6) Güvenlik/log redaction
- Sprint 2: (3) İki yönlü bayrak, (4) Rol eşleme, (5) Dayanıklılık

Notlar
- Tam mail UI (liste/görünüm/compose) Faz 2’de; şu an test UI yeterli.
- lettre ClientId deprecation uyarısı, fonksiyonelliği etkilemiyor; ileride ClientId::Domain ile sadeleştirilecek.

## 2025-11-06 Güncellemesi
- Gmail: Sent UID hemen çözümleme henüz güvenilir değil (bekleyen IMAP indekslemesi). Arka planda tamamlama (60s) ve sonraki senkronizasyon yedekleme önlemleri alındı.
- Sonraki adımlar:
  - Faz 2 görevlerine geçiş yapın (örn. mesaj gövdesi önbelleğe alma, ek listeleme ve birleşik gelen kutusu düzenleme).
  - İsteğe bağlı: pending_uid öğelerini yeniden kontrol etmek için periyodik bir iş ekleyin.
