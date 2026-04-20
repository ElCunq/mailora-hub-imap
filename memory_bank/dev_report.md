# Mailora Hub — Geliştirme Raporu (Tez Dosyası)

> **Son Güncelleme:** 2026-03-09
> Bu dosya her sprint sonunda otomatik güncellenir. Tez yazımında referans olarak kullanılabilir.

---

## Proje Özeti

**Mailora Hub**, IMAP/SMTP protokolleri üzerine inşa edilmiş, yerel depolama öncelikli (offline-first) bir e-posta istemci uygulamasıdır. Uygulama, bağımsız bir Rust (`axum`) backend ve vanilla HTML/CSS/JS frontend mimarisi kullanır. PIM (Kişisel Bilgi Yönetimi) modülü eklenerek uygulama, Outlook ve Gmail gibi platformlara kapsamlı bir alternatif hâline getirilmektedir.

**Teknoloji Yığını:**
- **Backend:** Rust, Axum (HTTP), SQLx + SQLite (DB), Tokio (async)
- **Frontend:** Vanilla HTML5, CSS3, JavaScript (ES2022)
- **Protokoller:** IMAP, SMTP, CardDAV (RFC 6352), CalDAV (RFC 4791), vCard 3.0/4.0, iCalendar (RFC 5545)
- **Deployment:** Docker + Coolify (VDS)

---

## Mimari Yapı

```
mailora-hub-imap/
├── migrations/            ← SQLite migration dosyaları
├── src/
│   ├── lib.rs             ← Kütüphane kök modülü
│   ├── main.rs            ← Axum sunucu başlatıcı
│   ├── config/            ← Uygulama ayarları
│   ├── db/                ← SQLx bağlantı havuzu
│   ├── imap/              ← IMAP sync, IDLE, folder tree
│   ├── models/            ← Rust veri modelleri (struct)
│   ├── pim/               ← vCard/iCal parser, WebDAV client
│   ├── routes/            ← HTTP endpoint handler'ları
│   ├── services/          ← İş mantığı katmanı
│   ├── smtp/              ← E-posta gönderim
│   ├── rbac/              ← Rol tabanlı erişim kontrolü
│   └── telemetry/         ← Loglama ve izleme
└── static/                ← Frontend HTML/CSS/JS dosyaları
```

---

## Geliştirme Geçmişi

### ✅ v1.0–v1.5 — Temel E-posta İstemcisi (Önceki Oturumlar)

| Özellik | Durum |
|---|---|
| IMAP hesap ekleme + çoklu hesap desteği | ✅ |
| E-posta listesi, thread görünümü, HTML/plain render | ✅ |
| Ek dosyalar (attachment) görüntüleme ve yükleme | ✅ |
| E-posta gönderme (SMTP + outbox servisi) | ✅ |
| Tam metin arama (FTS5) | ✅ |
| Klasör ağacı + IMAP IDLE (gerçek zamanlı bildirim) | ✅ |
| Bayrak yönetimi (Okundu/Yıldız/Silindi) | ✅ |
| Hesap renklendirme (account colors) | ✅ |
| RBAC — Rol tabanlı erişim kontrolü | ✅ |
| Otomatik keşif (IMAP/SMTP SRV + .well-known) | ✅ |
| Dark/Light tema navigasyon | ✅ |
| Veritabanı bakım görevi (VACUUM, temizlik) | ✅ |
| Snooze (ertele) özelliği | ✅ |
| Ayarlar sayfası (kimlik doğruma, hesap yönetimi) | ✅ |
| Docker yapılandırması (multistage + cargo-chef cache) | ✅ |
| Coolify VDS deployment | ✅ |

---

### ✅ v1.6 — PIM: Kişiler (Contacts) — 2026-03-09

#### Sprint 1: Backend

##### Veritabanı Şeması
**Dosya:** `migrations/20260309000000_create_contacts.sql`

| Tablo | Sütunlar (Özet) | Amaç |
|---|---|---|
| `contacts` | id, account_id, full_name, first_name, last_name, company, title, sync_status, vcard_uid, etag, href, raw_vcard… | Ana kişi kaydı |
| `contact_emails` | id, contact_id, email, label, is_primary | Çoklu e-posta |
| `contact_phones` | id, contact_id, phone, label, is_primary | Çoklu telefon |
| `contact_addresses` | id, contact_id, label, street, city, region, postal_code, country | Posta adresi |
| `contact_social` | id, contact_id, service, url | Sosyal medya |
| `contact_groups` | id, account_id, name, color | Grup tanımı |
| `contact_group_members` | contact_id, group_id | N:M ilişki |
| `carddav_sync_state` | account_id, addressbook_url, last_synced_at | Sync durumu |
| `contact_conflicts` | id, contact_id, local_data, remote_data, resolved | Çakışma kaydı |
| `contacts_fts` | FTS5 sanal tablo (unicode61 tokenizer) | Tam metin arama |

FTS için INSERT/UPDATE/DELETE trigger'ları eklendi.

**Dosya:** `migrations/20260309000001_add_pim_urls.sql`
- `accounts` tablosuna `carddav_url` ve `caldav_url` kolonları eklendi.

##### Rust Veri Modelleri
**Dosya:** `src/models/contact.rs`

```
Contact           → contacts tablosu (sqlx::FromRow)
ContactEmail      → contact_emails
ContactPhone      → contact_phones
ContactAddress    → contact_addresses
ContactSocial     → contact_social
ContactGroup      → contact_groups
ContactFull       → Birleştirilmiş tam veri (API detay yanıtı)
ContactSummary    → Hafif liste verisi (photo, primary_email, primary_phone)
ContactRequest    → POST/PUT gövdesi
ContactQuery      → GET query parametreleri
EmailEntry / PhoneEntry / AddressEntry / SocialEntry → Alt-kayıt request struct'ları
```

##### vCard Parser & Serializer
**Dosya:** `src/pim/vcard.rs`

- vCard 3.0 ve 4.0 parse desteği
- CRLF + whitespace line unfolding
- TYPE parametre çıkarımı (WORK, HOME, CELL, FAX…)
- Desteklenen property'ler: `FN`, `N`, `ORG`, `TITLE`, `EMAIL`, `TEL`, `ADR`, `BDAY`, `NOTE`, `PHOTO` (base64), `UID`, `REV`, `URL`, `CATEGORIES`, `X-SOCIALPROFILE`, `X-LINKEDIN`
- `parse_vcard(raw: &str) → Option<ParsedVCard>`: Yapısal ayrıştırma
- `serialize_vcard(...) → String`: vCard 3.0 çıktısı (line folding dahil)
- **4 birim testi** (temel parse, ORG parse, line folding, serialize) ✅

##### WebDAV HTTP Client
**Dosya:** `src/pim/dav_client.rs`

| Metod | HTTP | Açıklama |
|---|---|---|
| `propfind(path, depth)` | PROPFIND | Koleksiyon içeriği |
| `report_addressbook(path)` | REPORT (addressbook-query) | href+etag listesi |
| `get(path)` | GET | Tek kaynak indir |
| `put(path, body, etag)` | PUT + If-Match/If-None-Match | Oluştur/Güncelle |
| `delete(path, etag)` | DELETE | Kaydı sil |
| `discover_principal()` | PROPFIND (current-user-principal) | Principal URL keşfi |

Namespace-agnostic multistatus (207) XML parser eklendi.

##### Contact Service
**Dosya:** `src/services/contact_service.rs`

| Fonksiyon | Açıklama |
|---|---|
| `list_contacts(pool, query)` | FTS5 veya SQL sorgusu, filtre+sıralama |
| `get_contact(pool, id)` | JOIN ile tam veri |
| `create_contact(pool, req)` | Kişi + alt kayıtlar, `pending_create` |
| `update_contact(pool, id, req)` | Alt kayıtları sil/yeniden-ekle |
| `delete_contact(pool, id)` | Soft-delete (`pending_delete`) |
| `suggest_contacts(pool, q, account_id)` | Autocomplete (To: alanı) |
| `list_groups / create_group` | Grup yönetimi |
| `add_to_group / remove_from_group` | N:M üyelik |
| `import_vcf(pool, account_id, vcf_str)` | .vcf parse → DB, UID çakışması kontrolü |
| `export_vcf(pool, account_id, group_id)` | DB→vCard 3.0 string |

##### CardDAV Senkronizasyon Servisi
**Dosya:** `src/services/carddav_service.rs`

**Bidireksiyonel senkronizasyon algoritması (sync_addressbook):**
```
1. REPORT/PROPFIND → Sunucu {href: etag} haritası
2. SQL → Local {href: (id, etag)} haritası
3. DIFF (çekme yönü):
   a. Sunucuda yeni/değişmiş → GET → parse vCard → DB upsert
   b. Local da değişmişse → ÇAKIŞMA → contact_conflicts'e kaydet
   c. Sunucu silmiş → local DELETE (pending değilse)
4. PUSH pending_create → PUT (If-None-Match: *)
5. PUSH pending_update → PUT (If-Match: mevcut-etag)
6. PUSH pending_delete → DELETE → local sil
7. carddav_sync_state güncelle
```

**RFC 6764 Auto-Discovery:**
```
1. Bilinen provider (Gmail, iCloud, Fastmail, Outlook/365)
2. https://{domain}/.well-known/carddav → HEAD probe
3. http fallback
```

##### REST API Endpoint'leri
**Dosya:** `src/routes/contacts.rs`

| Metod | Path | Açıklama |
|---|---|---|
| GET | `/contacts` | Liste (FTS, grup, favori filtresi) |
| POST | `/contacts` | Yeni kişi oluştur |
| GET | `/contacts/:id` | Tam detay |
| PUT | `/contacts/:id` | Güncelle |
| DELETE | `/contacts/:id` | Soft-delete |
| GET | `/contacts/suggest` | E-posta autocomplete |
| GET | `/contacts/groups` | Grup listesi |
| POST | `/contacts/groups` | Yeni grup oluştur |
| POST | `/contacts/:id/groups/:gid` | Gruba ekle |
| DELETE | `/contacts/:id/groups/:gid` | Gruptan çıkar |
| POST | `/contacts/import` | .vcf içe aktar (raw Bytes) |
| GET | `/contacts/export` | .vcf dışa aktar |
| POST | `/sync/carddav/:account_id` | Manuel CardDAV sync tetikle |

##### Modül Entegrasyonu
- `src/lib.rs` → `pub mod pim;`
- `src/main.rs` → `mod pim;`
- `src/models/mod.rs` → `pub mod contact;`
- `src/services/mod.rs` → `pub mod contact_service; pub mod carddav_service;`
- `src/routes/mod.rs` → `pub mod contacts;` + tüm route tanımları

---

#### Sprint 2: Frontend

##### Adres Defteri Arayüzü
**Dosya:** `static/contacts.html`

**Bileşenler:**
- **Üst Navigasyon:** E-posta / Kişiler / Takvim sekmeleri, sync butonu
- **Sol Panel (Sidebar):** Tüm kişiler / Favoriler / Son Güncelleme / Grup listesi (renkli)
- **Orta Panel (Liste):** Alfabetik gruplandırma, avatar/inisyal, FTS live arama (300ms debounce), çakışma badge'i
- **Sağ Panel (Detay):** Büyük avatar, isim, unvan/şirket, sync durumu badge'i; e-posta/telefon/adres/sosyal alanlar; E-posta Gönder kısayolu
- **Modal (Oluştur/Düzenle):** Dinamik email/telefon/adres satırı ekleme/silme, validasyon
- **Modal (Yeni Grup):** İsim + renk seçici
- Favori toggle, kişi silme (confirm dialog)
- `.vcf` import (FileReader API) ve export (window.open)
- Manuel CardDAV sync tetikleyici + sonuç gösterimi
- Toast bildirim sistemi (success/error, 3.5s)

**Tasarım İlkeleri:** Dark theme (#0f1117 bg), Inter font, CSS custom properties, animated modals, micro-transitions.

---

#### Doğrulama Sonuçları

| Test | Sonuç |
|---|---|
| `cargo check` (lib + bin) | ✅ Hata yok |
| `cargo test pim::vcard` | ✅ 4/4 test geçti |

---

### ✅ v1.6 — Sprint 3: E-posta Autocomplete + Zamanlanmış Sync — 2026-03-09

#### Zamanlanmış CardDAV Sync
**Dosya:** `src/services/scheduler.rs`

- Mevcut 60 saniyelik tick döngüsüne CardDAV senkronizasyon logic'i eklendi
- `carddav_sync_state` tablosundan `last_synced_at` okunarak 15 dakika geçmişse sync tetiklenir
- Her hesap için bağımsız `tokio::spawn` ile çalışır (e-posta sync'ini bloklamaz)
- CardDAV URL yoksa sessizce atlanır (log spam önlenir)
- `SyncResult` (synced/created/updated/deleted/conflicts) `tracing::info!` ile loglanır

**Geçen süre tabanlı kontrol algoritması:**
```
current_time - last_synced_at >= 900 saniye  →  sync başlat
```

#### E-posta `To:` Alanı Kişi Autocomplete
**Dosya:** `static/preview.html`

- `compose-to` input alanı `position:relative` wrapper içine alındı
- `contact-suggestions` dropdown div'i eklendi
- `oninput` → 200ms debounce → `GET /contacts/suggest?q={token}` API çağrısı
- Sonuçlar: avatar (initials), isim, e-posta ile listelendi
- **Klavye navigasyonu:** ↑/↓ ile gezinme, Enter/Tab ile seçim, Escape ile kapat
- **Multiple recipients:** virgül/noktalı virgül ile ayrılmış, her token ayrı ayrı aranır
- Seçim formatı: `Ad Soyad <email@domain.com>, `
- Dışarı tıklayınca dropdown kapanır
- `openCompose(toAddress?)` fonksiyonu güncellendi: `contacts.html`'den yönlendirme desteği

#### Doğrulama
| Test | Sonuç |
|---|---|
| `cargo check` (lib + bin) | ✅ Hata yok |

---

### ✅ v1.6 — Sprint 4: Conflict UI & Duplicate Detection — 2026-03-09

#### Çakışma ve Çift Kayıt Yönetimi (Backend)
**Dosyalar:** `src/models/contact.rs`, `src/services/contact_service.rs`, `src/routes/contacts.rs`

- **Çakışma Endpointleri:** `GET /contacts/conflicts`, `POST /contacts/conflicts/:id/resolve` eklendi.
- **Çift Kayıt (Duplicate) Endpointleri:** `GET /contacts/duplicates`, `POST /contacts/duplicates/merge` eklendi.
- Çakışmalar `contact_conflicts` tablosundan listelenir.
- Çift kayıt tespiti: E-posta adresleri veya Ad-Soyad bilgileri birebir aynı olanlar gruplanır.
- Birleştirme (Merge) işlemi: İkinci kaydı `pending_delete` durumuna çeker (ön yüz birleştirmeyi kendisi yapar).

#### PIM Arayüz Güncellemeleri (Frontend)
**Dosya:** `static/contacts.html`

- `notifications-area`: Liste görünümü üzerine uyarı kutucukları (banner) yerleştirildi.
- Çakışma tespiti `checkIssues()` ile otomatik yapılır ve uyarı olarak (`issue-banner conflict`) yansıtılır.
- Tıklandığında yerel ve uzak değişiklikler arasında seçim (Yerel veya Merged/Remote) yaptırılır.
- Duplikat (Çift) Kayıtlar bulunduğunda mavi uyarı banner'ı (`issue-banner dup`) çıkar. Tıklanarak birleştirilir.
- `app.js` deki fallback yönlendirmesi için `GET /.well-known/carddav` route'u eklendi.

#### Doğrulama
| Test | Sonuç |
|---|---|
| `cargo check` (lib + bin) | ✅ Hata yok |

---

## 🔜 Planlanan Adımlar

### v1.7 — Takvim (CalDAV) — Yakın Vadeli

| Bileşen | Kapsam |
|---|---|
| DB şeması | `calendars`, `calendar_events`, `event_attendees`, `event_alarms`, `event_exceptions`, `notification_queue` |
| iCal parser | `pim/ical.rs` — VEVENT, VALARM, VTIMEZONE |
| CalDAV sync | `services/caldav_service.rs` — calendar-query REPORT |
| RRULE | DAILY/WEEKLY/MONTHLY/YEARLY + EXDATE + COUNT/UNTIL |
| Timezone | UTC depolama, `Intl.DateTimeFormat` ile frontend dönüşümü |
| UI | `calendar.html` — ay/hafta/gün görünümleri, etkinlik modal'ı |
| iMIP | Toplantı daveti e-posta akışı (RSVP accept/decline) |
| Bildirimler | SSE → browser Notification API (alarm pipeline) |

### v1.8 — Görevler / To-Do (VTODO) — Orta Vadeli

- Basit görev listesi, CalDAV üzerinden senkronizasyon
- Due date, öncelik, tamamlandı durumu

### DevOps / Kararlılık

| Görev | Açıklama |
|---|---|
| CardDAV sync retry/backoff | Exponential backoff, offline kuyruğu |

---

*Bu dosya, her sprint tamamlandığında `memory_bank/dev_report.md` olarak güncellenir.*
