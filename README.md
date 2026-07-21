# Mailora v3 — Mailcow Hub & E-posta Yönetim Sistemi

**Mailora v3**, genel amaçlı bir PIM (takvim, hesap tablosu vs.) veya üretkenlik uygulaması yerine tamamen **Mailcow entegrasyonuna odaklanmış**, kurumsal seviyede **Role-Based Access Control (RBAC)** destekli, yüksek performanslı bir e-posta altyapısı ve merkezi yönetim sistemidir.

Bu repository (`v2.0` branch), Mailora'nın **sadece e-posta senkronizasyonu (IMAP)**, **güvenilir ve kuyruk tabanlı gönderim (SMTP Outbox)** ve **Mailcow domain/mailbox yönetim modüllerini** içerir.

---

## 🚀 Öne Çıkan Özellikler & Mimari

### 1. Rol ve Yetki Yönetimi (RBAC)
Sistemde katı ve hiyerarşik bir rol & izin mimarisi bulunmaktadır:
- **SuperAdmin**: Sistemdeki tüm Mailcow sunucularını ekleyebilir, yönetebilir, keşif (discovery) tetikleyebilir, tüm domain ve mail kutularını görebilir/yönetebilir.
- **DomainAdmin**: Yalnızca kendisine atanan `domain_id` kapsamındaki mail kutularını listeyebilir, kullanıcılara yetki atayabilir, kota ve senkronizasyon durumlarını inceleyebilir.
- **User / Member**: Yalnızca kendisine (`mailbox_assignments` tablosu üzerinden) açıkça tanımlanan mail kutularına erişebilir ve izin verilen işlemleri (`View`, `Read`, `Reply`, `Send`, `SendAs`, `MarkRead`, `Move`, `Delete`, `Manage`) gerçekleştirebilir.

### 2. Mailcow Domain & Mail Kutusu Keşfi (Discovery Engine)
- API anahtarı (Read/Write) ile yapılandırılan Mailcow sunucularına (`mailcow_instances`) otomatik bağlantı sağlar.
- `api/v1/get/domain/all` ve `api/v1/get/mailbox/all` endpoint'lerinden çekilen verilerle yerel veritabanını atomik `UPSERT` işlemleriyle senkronize eder.
- Silinmiş veya pasife alınmış domain ve mail kutuları otomatik olarak `active = 0` ve `deleted_at` alanlarıyla işaretlenir; fiziksel veri kaybı önlenir (soft-delete).

### 3. Senkronizasyon Motoru (Sync Engine & Concurrency Control)
- **Job Lock Mekanizması (`job_locks`)**: Aynı mail kutusunun veya klasörün eşzamanlı (concurrency) senkronize edilmesini engeller.
- **UIDVALIDITY Sıfırlama Koruması**: IMAP sunucusunda UIDVALIDITY değişirse, eski UID eşleşmelerini temizleyerek `sync_state` kaydını güvenle sıfırlar.
- **Klasör Tipleri (`folder_kind`)**: `Inbox`, `Sent`, `Drafts`, `Trash`, `Spam`, `Archive`, `Custom` otomatik olarak algılanır.

### 4. Outbox & Güvenilir SMTP Gönderimi
- Gönderilecek e-postalar doğrudan senkron olarak gönderilip bloklanmak yerine `outbox` tablosuna `queued` durumunda yazılır.
- Arka planda çalışan worker, eksponansiyel geri çekilme (exponential backoff) stratejisiyle başarısız (`failed`) iletileri otomatik olarak yeniden dener (en fazla 5 deneme).
- Her gönderim denemesi ve SMTP yanıtı (`250 OK` vb.) ayrıntılı biçimde `send_audit` tablosunda kayıt altına alınır.

### 5. Yapılandırılmış Audit Raporlama (Audit Logging)
- Kullanıcı girişlerinden yetki atamalarına, mail kutusu keşfinden e-posta gönderim raporlarına kadar her eylem `audit_logs`, `send_audit` ve `sync_runs` tablolarında denetlenir.
- Gelişmiş filtreleme ve sayfalama imkanı sağlar.

---

## 🛠 Teknik Katman & Veritabanı (SQLite)

- **Veritabanı**: SQLite (FTS5 tam metin arama desteği ile)
- **Veri Tutarlılığı (Consistency)**: Tüm ekleme ve güncellemeler `ON CONFLICT DO UPDATE` (upsert) yaklaşımıyla yürütülür.

### Temel Tablolar (`migrations/20260720000000_mailora_v3_schema.sql`)
- `users`: Sistem kullanıcıları (`role`: `SuperAdmin`, `DomainAdmin`, `User`)
- `mailcow_instances`: Tanımlı Mailcow sunucuları
- `domains` & `mailboxes`: Keşfedilen Mailcow domainleri ve mail kutuları
- `domain_admin_assignments` & `mailbox_assignments`: Çok-çoğa rol ve yetki eşleştirmeleri
- `messages` & `messages_fts`: Senkronize edilen e-postalar ve tam metin arama dizini
- `outbox`: E-posta gönderim kuyruğu (`queued`, `processing`, `sent`, `failed`)
- `audit_logs`, `send_audit`, `sync_runs`: Denetim ve sistem logları

---

## 📡 REST API & Endpoint Yapısı

### Mailcow v3 Management (`/api/v1/mailcow`)
- `GET /api/v1/mailcow/instances` — Tanımlı Mailcow sunucularını listeler
- `POST /api/v1/mailcow/instances` — Yeni Mailcow sunucusu ekler (`SuperAdmin` gerektirir)
- `POST /api/v1/mailcow/discovery/run/:id` — Belirli bir sunucuda alan adı/mailbox keşfini tetikler
- `POST /api/v1/mailcow/discovery/run-all` — Tüm sunucularda eşzamanlı keşif başlatır

### RBAC & Yetkilendirme (`/api/v1/rbac`)
- `GET /api/v1/rbac/domains` — Kullanıcının yetkili olduğu domainleri listeler
- `GET /api/v1/rbac/mailboxes` — Yetkili olunan mail kutularını listeler
- `GET /api/v1/rbac/users` — Kullanıcıları listeler (`SuperAdmin`)
- `POST /api/v1/rbac/users/:id/role` — Kullanıcı rolünü günceller (`SuperAdmin`, `DomainAdmin`, `User`)
- `POST /api/v1/rbac/domains/:domain_id/admins/:user_id` — Belirtilen domain'e `DomainAdmin` atar
- `DELETE /api/v1/rbac/domains/:domain_id/admins/:user_id` — Domain Admin yetkisini kaldırır
- `POST /api/v1/rbac/mailboxes/:mailbox_id/assignments` — Kullanıcıya belirli mail kutusu izinlerini tanır (`View`, `Read`, `Send` vb.)

### Outbox & Audit (`/outbox`, `/api/v3/audit`)
- `GET /outbox` — Kuyruktaki ve gönderilmiş iletileri listeler
- `POST /outbox/:id/retry` — Başarısız iletinin yeniden denenmesini sağlar
- `GET /api/v3/audit/logs` — Genel sistem ve güvenlik denetim logları
- `GET /api/v3/audit/send` — SMTP gönderim raporları ve hata kodları
- `GET /api/v3/audit/sync-runs` — IMAP klasör senkronizasyon süre ve sonuçları

---

## 💻 Kurulum & Çalıştırma

### 1. Docker ile Hızlı Kurulum (Önerilen)
```bash
git clone https://github.com/ElCunq/mailora-hub-imap.git
cd mailora-hub-imap
git checkout v2.0

# Docker Compose ile arka planda başlatın
docker-compose up -d --build
```
Uygulama `http://localhost:3030` üzerinde çalışacaktır. Veritabanı ve kalıcı veriler `./data` dizininde tutulur.

### 2. Geliştirici Ortamı (Cargo)
```bash
# Bağımlılıkları kontrol et ve derle
cargo check --workspace

# Testleri çalıştır (3 Birim Test + 17 Entegrasyon Testi)
cargo test --workspace

# Uygulamayı çalıştır
cargo run
```

---

## 🖥 Arayüz Bileşenleri (`/static`)
- **`/static/index.html`** — Temiz, modern ve yüksek performanslı E-posta Uygulaması. Rol ve yetkilere göre sekmeleri (Gelen Kutusu, Gönderilenler vb.) dinamik gösterir.
- **`/static/admin.html`** — Mailora Hub Yönetim Paneli. SuperAdmin ve DomainAdmin rolündeki yöneticiler için Mailcow sunucularını bağlama, keşif başlatma, rol & izin yönetimi yapma ve Outbox/Audit raporlarını anlık izleme imkanı sunar.

---
*Mailora v3 — Mailcow Entegrasyonlu E-posta Yönetim Hub'ı (`v2.0` Branch)*
