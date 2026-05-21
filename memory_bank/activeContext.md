# Active Context

Tarih: 2026-05-21
Branch: v1.1.0

## Mevcut Durum
Proje **Auto-Discovery (Magic Login)**, **UI Unifikasyonu (Dark Theme)**, **Kurumsal (Enterprise) Mod (RBAC)**, **Unified Inbox (Tüm Gelen Kutuları)** ve premium **add_account.html** entegrasyonları ile başarıyla güçlendirilmiş ve kararlı bir şekilde çalışmaktadır. 
Yalnızca **Takvim otomatik eşzamanlama / CalDAV otomasyonu** (takvim kısmının otomatik olması) fonksiyonu çalışmamaktadır ve kararsız durumdadır.

### Son Geliştirmeler
- **Unified Inbox (Başarılı & Kararlı):**
  - Tüm hesapların iletilerini tek bir çatı altında toplayan "Tüm Hesaplar" görünümü (`app.js`, `mock.js`, `sidebar.js`) entegre edildi.
  - İletilerin sol tarafına, hangi hesaptan geldiklerini gösteren renkli hesap rozetleri (badges) başarıyla eklendi.
- **Premium add_account.html (Başarılı & Kararlı):**
  - Eski manuel ve otomatik keşif mantığı, premium karanlık tema standartlarında yeniden oluşturuldu ve stabil bir şekilde çalışmaktadır.
- **Takvim Otomasyonu (Çalışmıyor / Kararsız):**
  - CalDAV / Takvim kısmının otomatik olarak sunucuyla eşzamanlanması veya otonom keşif döngüsü çalışmamaktadır.

## Odak
- Takvim eşzamanlama otomasyonundaki (takvim kısmının otomatik olması) hataları tespit edip çözmek.
- CalDAV otomatik entegrasyonunu ve veri akışını stabilize etmek.

## Sıradaki İşler (Faz 3 - Enterprise & PIM)
- **Takvim Otomasyonu Düzeltmeleri:** Takvim kısmının otomatik olarak çalışmamasını gidermek, CalDAV senkronizasyonunu stabilize etmek.
- **Admin Paneli:** Kullanıcıları listeleme, silme ve sistem loglarını (`event_logs`) görüntüleme.
- **Olay Günlüğü:** `LOGIN`, `FETCH`, `SEND` gibi önemli eylemleri veritabanına kaydetme.
- **Tam Metin Arama (FTS):** SQLite FTS5 entegrasyonu.
- **Kalıcı Kuyruklar:** Giden e-postalar için `Outbox` mekanizması.

## Kararlar
- **RBAC:** Basit bir JWT benzeri token ("id:role") yapısı şimdilik yeterli görüldü (MVP için). İleride gerçek JWT'ye geçilecek.
- **Kayıt Politikası:** "Herkese açık" kayıt politikası benimsendi (ilk üye Admin, diğerleri Member).
- **Unified Tasarım:** Tüm hesaplar görünümündeyken mesajların hangi hesap orijinli olduğunu belirtmek için renkli rozet yapısı tercih edildi.


