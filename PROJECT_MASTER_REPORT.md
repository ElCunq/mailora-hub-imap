# Mailora Hub IMAP - Proje Ana Raporu

**Tarih:** 05 Aralık 2025  
**Sürüm:** v0.3.0  
**Durum:** Aktif Geliştirme (Faz 2 - İyileştirme ve Kararlılık)

---

## 1. Yönetici Özeti

**Mailora Hub IMAP**, yerel öncelikli (local-first), yüksek performanslı ve gizlilik odaklı bir e-posta arka uç (backend) servisidir. Rust dili ile geliştirilen bu proje, modern e-posta istemcileri için güvenilir bir IMAP/SMTP köprüsü görevi görür.

Projenin temel amacı, birden fazla e-posta sağlayıcısını (Gmail, Outlook, Yahoo, vb.) tek bir birleşik arayüzde (Unified Inbox) toplamak, verileri yerel SQLite veritabanında önbelleklemek ve çevrimdışı erişim imkanı sunmaktır.

Şu anki aşamada (v0.3.0), temel e-posta okuma, gönderme, ek (attachment) yönetimi ve çoklu hesap desteği tamamlanmış olup, performans optimizasyonları ve kullanıcı deneyimi iyileştirmeleri üzerinde çalışılmaktadır.

---

## 2. Proje Kimliği ve Hedefler

### Vizyon
Outlook veya Thunderbird benzeri masaüstü deneyimini, daha düşük kaynak kullanımı ve daha güçlü veri denetimi ile sunmak.

### Temel Hedefler
1.  **Güvenilir Senkronizasyon:** Çoklu klasör ve UID tabanlı delta senkronizasyonu.
2.  **Birleşik Gelen Kutusu (Unified Inbox):** Tüm hesaplardan gelen e-postaların tek bir zaman çizelgesinde gösterilmesi.
3.  **Performans:** 100 iletiyi 3.5 saniyenin altında listeleme ve düşük bellek kullanımı.
4.  **Yerellik:** Verilerin kullanıcının cihazında (SQLite) saklanması.

### Kapsam Dışı (Non-Goals)
*   Tam kapsamlı bir sunucu tarafı MTA (Mail Transfer Agent) olmak.
*   Karmaşık spam filtreleme (istemci tarafı basit kurallar hariç).
*   Şimdilik tam kapsamlı JMAP desteği (öncelik IMAP).

---

## 3. Mimari ve Teknoloji Yığını

### Teknoloji Yığını
*   **Dil:** Rust (Güvenlik ve performans için).
*   **Web Framework:** Axum (REST API).
*   **Veritabanı:** SQLite (sqlx ile, WAL modunda).
*   **IMAP İstemcisi:** `async-imap` (Tokio tabanlı).
*   **SMTP İstemcisi:** `lettre`.
*   **MIME İşleme:** `mailparse`.
*   **HTML Sanitizasyon:** `ammonia`.
*   **Frontend (Test/Demo):** Vanilla JS + HTML (`static/app.html`).

### Mimari Bileşenler
1.  **API Katmanı:** RESTful endpoint'ler (`/sync`, `/messages`, `/accounts`, `/attachments`).
2.  **Servis Katmanı:** İş mantığı (Senkronizasyon, Mesaj Gövdesi Getirme, Ek Yönetimi).
3.  **Veri Katmanı:** SQLite veritabanı (`messages`, `accounts`, `attachments`, `message_bodies` tabloları).
4.  **Arka Plan İşçileri (Workers):**
    *   **IDLE Watcher:** Gerçek zamanlı e-posta bildirimleri.
    *   **Sync Scheduler:** Periyodik senkronizasyon.
    *   **Backfill Worker:** Eski mesajların ek bilgilerini tamamlama.

---

## 4. Mevcut Durum (v0.3.0)

Proje şu anda **v0.3.0** sürümündedir. Son yapılan çalışmalarla birlikte sistem kararlılığı artırılmış ve eksik özellikler tamamlanmıştır.

### Tamamlanan Özellikler (Features Completed)
*   **Hesap Yönetimi:** Ekleme, silme, listeleme (Gmail, Outlook, Custom).
*   **Temel IMAP Senkronizasyonu:** Klasör listeleme, mesaj başlıklarını çekme.
*   **Mesaj Okuma:**
    *   HTML ve Düz Metin (Plain Text) gövde desteği.
    *   Güvenli HTML görüntüleme (Sanitizasyon).
    *   Önbellekleme (Caching) mekanizması.
*   **Ek (Attachment) Yönetimi:**
    *   Eklerin meta verilerini (isim, boyut, tür) veritabanına kaydetme.
    *   Geriye dönük tarama (Backfill) ile eski mesajların eklerini bulma.
    *   Ekleri indirme ve görüntüleme.
    *   Büyük dosyalar ve `inline` görseller için destek.
*   **SMTP Gönderim:** E-posta gönderme ve "Gönderilmiş Öğeler" (Sent) klasörüne kaydetme.
*   **Kullanıcı Arayüzü (UI):**
    *   Okundu/Okunmadı işaretleme.
    *   Mesaj silme.
    *   Ekleri listeleme ve indirme.
    *   Otomatik yenileme.

### Son Değişiklikler (Changelog - v0.3.0)
*   **Derleme Hataları:** Tüm Rust derleme uyarıları (warnings) temizlendi.
*   **Ek İyileştirmeleri:** `RFC 2231/2047` dosya adı kodlaması desteği eklendi. Klasörler arası yedek (fallback) tarama mekanizması kuruldu.
*   **UI Düzeltmeleri:** Sandbox uyarıları giderildi, eksik fonksiyonlar (`loadAttachments`, `resolveFolderName`) eklendi.
*   **Veritabanı:** `attachments` tablosu genişletildi (`content_id`, `is_inline`, `data` sütunları eklendi).

---

## 5. Geçmiş İlerleme Raporu

### Milestone 1: Temel Bağlantı (Tamamlandı)
*   IMAP/SMTP bağlantıları sağlandı.
*   Veritabanı şeması oluşturuldu.
*   Stalwart bağımlılığı kaldırılarak doğrudan bağlantı mimarisine geçildi.

### Milestone 2: Birleşik Yapı (Tamamlandı)
*   Çoklu hesap desteği eklendi.
*   Unified Inbox (Birleşik Gelen Kutusu) mantığı kuruldu.
*   Arka plan senkronizasyon servisi devreye alındı.

### Faz 2 (Şu Anki Aşama)
*   Ek yönetimi ve UI entegrasyonu tamamlandı.
*   Performans iyileştirmeleri ve hata ayıklama devam ediyor.

---

## 6. Yol Haritası (Gelecek Planları)

### Kısa Vadeli (Sıradaki Adımlar)
1.  **Performans İndeksleri:** Veritabanı sorgularını hızlandırmak için eksik indekslerin eklenmesi.
2.  **Arama (Search):** Tam metin arama (Full-Text Search - FTS) entegrasyonu.
3.  **Hata Yönetimi:** Hata zarflarının (Error Envelopes) standartlaştırılması.
4.  **Kalıcı Kuyruklar:** Giden kutusu (Outbox) ve gönderim sonrası işlemler için kalıcı kuyruk yapısı.

### Orta Vadeli
1.  **JMAP Desteği:** Büyük posta kutularında daha verimli listeleme için opsiyonel JMAP desteği.
2.  **Güvenlik Denetimi:** TLS ve veri saklama güvenliğinin gözden geçirilmesi.
3.  **Paketleme:** Tauri entegrasyonu ile masaüstü uygulaması olarak paketleme.

### Uzun Vadeli
1.  **Spam Filtreleme:** İstemci tarafı kural motoru ve basit spam tespiti.
2.  **Telemetri:** Gelişmiş loglama ve yönetici paneli.

---

## 7. Bilinen Sorunlar (Known Issues)

*   **Gmail UID Gecikmesi:** Gmail API'si bazen yeni gönderilen mesajların UID'sini hemen döndürmeyebiliyor, bu da "Sent" klasöründe kısa süreli senkronizasyon gecikmesine yol açabilir.
*   **Büyük Posta Kutuları:** 50.000+ mesaj içeren kutularda ilk senkronizasyon (Initial Sync) uzun sürebilir.
*   **UI Kısıtlamaları:** Mevcut `app.html` sadece test ve geliştirme amaçlıdır, nihai ürün değildir.

---

*Bu rapor, projenin `memory_bank/` klasöründeki veriler ve son geliştirme oturumları baz alınarak oluşturulmuştur.*
