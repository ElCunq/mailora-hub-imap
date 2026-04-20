# Mailora Hub IMAP - Teknik Dokümantasyon

**Sürüm:** v0.3.0  
**Tarih:** 05 Aralık 2025  
**Durum:** Aktif Geliştirme

---

## 1. Giriş

### 1.1 Vizyon
Mailora Hub IMAP, modern e-posta istemcilerinin ihtiyaç duyduğu hız, güvenilirlik ve gizliliği sağlayan, yerel öncelikli (local-first) bir e-posta arka uç (backend) servisidir. Outlook veya Thunderbird gibi geleneksel masaüstü istemcilerinin sunduğu deneyimi, daha düşük kaynak kullanımı ve daha güçlü veri denetimi ile sunmayı hedefler.

### 1.2 Problem Tanımı
Mevcut e-posta istemcileri genellikle iki uçtan birine kayar:
1.  **Bulut Tabanlı:** Verilerinizi üçüncü taraf sunucularda saklar, gizlilik endişeleri yaratır ve çevrimdışı erişim kısıtlıdır.
2.  **Geleneksel Masaüstü:** Hantal, kaynak tüketen ve modern senkronizasyon özelliklerinden (Unified Inbox, anlık bildirimler) yoksun olabilirler.

### 1.3 Çözüm
Mailora Hub, Rust dilinin performansını ve güvenliğini kullanarak, doğrudan IMAP/SMTP bağlantıları üzerinden çalışan, verileri yerel SQLite veritabanında önbellekleyen ve RESTful API üzerinden modern arayüzlere servis eden hibrit bir çözümdür.

---

## 2. Sistem Mimarisi

### 2.1 Yüksek Seviye Genel Bakış
Sistem, dört ana katmandan oluşur:
1.  **Frontend (İstemci):** Kullanıcı arayüzü (Web UI veya Tauri uygulaması).
2.  **API Katmanı (Axum):** İstemci ile haberleşen REST API.
3.  **Servis Katmanı (Core):** İş mantığı, senkronizasyon motoru ve veri işleme.
4.  **Veri Katmanı (SQLite):** Yerel veri saklama ve önbellekleme.

### 2.2 Bileşen Diyagramı

```mermaid
graph TD
    User[Kullanıcı (Web/Tauri)] -->|HTTP REST| API[API Katmanı (Axum)]
    
    subgraph "Mailora Backend"
        API --> AccountSvc[Hesap Servisi]
        API --> MessageSvc[Mesaj Servisi]
        API --> SyncSvc[Senkronizasyon Servisi]
        
        SyncSvc -->|IMAP| IMAPClient[IMAP İstemcisi (async-imap)]
        MessageSvc -->|SMTP| SMTPClient[SMTP İstemcisi (lettre)]
        
        SyncSvc --> DB[(SQLite Veritabanı)]
        MessageSvc --> DB
        AccountSvc --> DB
        
        Background[Arka Plan İşçileri]
        Background -->|IDLE| IMAPClient
        Background -->|Backfill| DB
    end
    
    IMAPClient -->|IMAP/TLS| ExternalMail[Dış E-posta Sağlayıcıları (Gmail, Outlook...)]
    SMTPClient -->|SMTP/TLS| ExternalMail
```

### 2.3 Veri Akışı (Data Flow)

#### E-posta Alma (Fetch)
1.  **Tetikleme:** Kullanıcı isteği veya Arka Plan Zamanlayıcısı (Scheduler).
2.  **Bağlantı:** `async-imap` ile sağlayıcıya bağlanılır.
3.  **Delta Senkronizasyonu:** Yerel `last_uid` ile sunucudaki `UIDNEXT` karşılaştırılır.
4.  **Meta Veri İndirme:** Yeni mesajların başlıkları (Envelope) ve yapı bilgisi (BodyStructure) indirilir.
5.  **Kaydetme:** Veriler `messages` tablosuna yazılır.
6.  **Gövde İndirme (Lazy):** Kullanıcı mesajı açtığında veya önbellekleme politikasına göre gövde (Body) indirilir, sanitize edilir ve `message_bodies` tablosuna yazılır.

#### E-posta Gönderme (Send)
1.  **Oluşturma:** Kullanıcı arayüzden mesajı oluşturur.
2.  **Kuyruklama:** Mesaj geçici olarak işlenir.
3.  **Gönderim:** `lettre` kütüphanesi ile SMTP sunucusuna iletilir.
4.  **Senkronizasyon:** Gönderilen mesaj, IMAP "Sent" klasörüne kaydedilir (Append) veya sunucunun otomatik kaydetmesi beklenir.
5.  **Veritabanı:** Gönderilen mesaj yerel veritabanına işlenir.

---

## 3. Veritabanı Şeması

Sistem, verileri yerel bir SQLite veritabanında saklar. Ana tablolar şunlardır:

### 3.1 `accounts` (Hesaplar)
Kullanıcıların eklediği e-posta hesaplarını tutar.
*   `id`: Benzersiz hesap kimliği (UUID veya String).
*   `email`: E-posta adresi.
*   `provider`: Sağlayıcı türü (gmail, outlook, custom).
*   `imap_host`, `imap_port`: IMAP sunucu bilgileri.
*   `smtp_host`, `smtp_port`: SMTP sunucu bilgileri.
*   `encrypted_password`: Şifrelenmiş parola.

### 3.2 `messages` (Mesajlar)
E-postaların meta verilerini ve başlık bilgilerini tutar.
*   `id`: Yerel benzersiz ID.
*   `account_id`: Hangi hesaba ait olduğu.
*   `remote_uid`: Sunucudaki UID.
*   `folder`: Bulunduğu klasör (INBOX, Sent, vb.).
*   `subject`: Konu başlığı.
*   `from_addr`, `to_addr`: Gönderen ve alıcılar.
*   `date`: Gönderim tarihi.
*   `flags`: Okundu, Silindi, Bayraklı durumları.
*   `has_attachments`: Ek içerip içermediği (Boolean).

### 3.3 `message_bodies` (Mesaj Gövdeleri)
E-posta içeriklerinin (HTML ve Düz Metin) önbelleğidir.
*   `account_id`, `remote_uid`, `folder`: Bileşik anahtar.
*   `plain_text`: Düz metin içeriği.
*   `html_body`: Sanitize edilmiş HTML içeriği.
*   `last_updated`: Son güncelleme zamanı (TTL için).

### 3.4 `attachments` (Ekler)
E-posta eklerinin meta verilerini ve (opsiyonel olarak) içeriğini tutar.
*   `id`: Benzersiz ek ID.
*   `message_id`: Bağlı olduğu mesaj.
*   `filename`: Dosya adı.
*   `content_type`: MIME türü.
*   `size`: Dosya boyutu.
*   `content_id`: Inline görseller için CID.
*   `is_inline`: Satır içi (inline) olup olmadığı.
*   `data`: Dosya içeriği (BLOB - Opsiyonel/Küçük dosyalar için).
*   `file_path`: Dosya sistemindeki yolu (Büyük dosyalar için).

---

## 4. Teknik Özellikler

### 4.1 Teknoloji Yığını
*   **Backend:** Rust (Edition 2021)
*   **Web Framework:** Axum 0.7
*   **Asenkron Runtime:** Tokio
*   **Veritabanı Sürücüsü:** SQLx (SQLite)
*   **IMAP:** `async-imap` + `tokio-native-tls`
*   **SMTP:** `lettre`
*   **MIME Parsing:** `mailparse`
*   **HTML Sanitization:** `ammonia`

### 4.2 Performans Hedefleri (NFRs)
*   **Listeleme Hızı:** 100 mesaj < 3.5sn (Gmail), < 2sn (Standart).
*   **Veri Verimliliği:** Mesaj başına < 80KB indirme (ekler hariç).
*   **Bellek Kullanımı:** IDLE modunda < 150MB RAM.
*   **Ölçeklenebilirlik:** 50.000+ mesajlık posta kutularında donma olmadan çalışma.

---

## 5. Temel Özellikler ve Uygulama Detayları

### 5.1 Hesap Yönetimi
*   **Şifreleme:** Parolalar veritabanında şifreli saklanır (basit XOR/Base64 şimdilik, OS Keychain planlanıyor).
*   **Sağlayıcılar:** Gmail, Outlook ve özel IMAP sunucuları desteklenir.

### 5.2 Senkronizasyon Motoru
*   **Delta Sync:** Sadece son kalınan UID'den sonraki mesajlar indirilir.
*   **IDLE:** Sunucudan gelen anlık bildirimlerle (PUSH) yeni mesajlar anında algılanır.
*   **Backfill:** Eski mesajların ek bilgileri arka planda tamamlanır.

### 5.3 Mesaj İşleme
*   **Parsing:** `mailparse` ile MIME yapısı (Multipart/Alternative, Mixed) çözümlenir.
*   **Sanitizasyon:** `ammonia` kütüphanesi ile HTML içeriğindeki zararlı scriptler (`<script>`, `<iframe>`, `on*` eventleri) temizlenir.
*   **Karakter Setleri:** UTF-8, ISO-8859-1, Windows-1254 gibi farklı kodlamalar ve `RFC 2047` başlık kodlamaları desteklenir.

### 5.4 Ek (Attachment) Yönetimi
*   **Algılama:** `Content-Disposition` (attachment/inline) ve MIME türüne göre ekler tespit edilir.
*   **İndirme:** İstemci talep ettiğinde (On-Demand) veya küçük dosyalar için otomatik indirilir.
*   **Akış (Streaming):** Büyük dosyalar bellek şişkinliği yaratmadan parça parça (chunk) indirilebilir.

### 5.5 Birleşik Gelen Kutusu (Unified Inbox)
*   Farklı hesaplardaki (Gmail, Outlook) "INBOX" klasörleri sanal bir görünümde birleştirilir.
*   Tarihe göre sıralanır ve tek bir liste olarak sunulur.

---

## 6. API Referansı (Özet)

### Hesaplar
*   `GET /accounts`: Kayıtlı hesapları listele.
*   `POST /accounts`: Yeni hesap ekle.
*   `DELETE /accounts/:id`: Hesabı sil.

### Mesajlar
*   `GET /messages/:account_id/:folder`: Klasördeki mesajları listele.
*   `GET /unified/inbox`: Birleşik gelen kutusunu listele.
*   `GET /test/body/:account_id/:uid`: Mesaj gövdesini getir.

### Ekler
*   `GET /attachments`: Mesajın eklerini listele.
*   `GET /attachments/download`: Eki indir.

### Senkronizasyon
*   `POST /sync/:account_id`: Manuel senkronizasyonu tetikle.
*   `POST /sync/:account_id/backfill-attachments`: Ek taramasını başlat.

---

## 7. Geliştirme Kılavuzu

### 7.1 Kurulum
1.  Rust ve Cargo'yu yükleyin (`rustup`).
2.  Projeyi klonlayın.
3.  `cargo build` ile derleyin.

### 7.2 Çalıştırma
*   Geliştirme modu: `cargo run`
*   Sunucu `127.0.0.1:3030` adresinde başlar.
*   Arayüz: `http://127.0.0.1:3030/app.html`

### 7.3 Test Stratejisi
*   **Birim Testleri:** `cargo test`
*   **Entegrasyon Testleri:** `tests/` klasöründeki senaryolar.

---

## 8. Operasyonel Kılavuz

### 8.1 Loglama
*   `tracing` kütüphanesi kullanılır.
*   Log seviyesi `RUST_LOG` çevre değişkeni ile ayarlanabilir (örn: `RUST_LOG=debug`).
*   Loglar standart çıktıya (stdout) ve dosyaya (`mailora-imap.log`) yazılır.

### 8.2 Sorun Giderme
*   **"Body alınamadı":** Genellikle klasör adı uyuşmazlığı veya bağlantı kopması. Loglarda `fetch_message_body` hatalarını kontrol edin.
*   **"Ek yok":** MIME parsing hatası veya desteklenmeyen `Content-Disposition`. Backfill işlemini tetikleyin.

---

## 9. Gelecek Planları (Roadmap)

*   **Tam Metin Arama (FTS):** SQLite FTS5 modülü ile hızlı içerik arama.
*   **OAuth2:** Gmail ve Outlook için modern kimlik doğrulama.
*   **Kalıcı Kuyruklar:** Giden e-postalar için `Outbox` mekanizması.
*   **Masaüstü Paketleme:** Tauri ile `.exe`, `.dmg`, `.deb` paketleri.

---

## 10. Sözlük

*   **UID:** Unique Identifier (IMAP sunucusundaki mesaj kimliği).
*   **MIME:** Multipurpose Internet Mail Extensions (E-posta format standardı).
*   **IDLE:** IMAP sunucusunun yeni mesajları anında bildirmesini sağlayan komut.
*   **Backfill:** Eksik verilerin sonradan tamamlanması işlemi.
*   **Sanitizasyon:** Güvenlik amacıyla verinin temizlenmesi.
