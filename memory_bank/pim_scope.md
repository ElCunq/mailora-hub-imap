# PIM Scope Document — Mailora Hub
# Kişisel Bilgi Yöneticisi (Kişiler + Takvim)

**Oluşturulma Tarihi:** 2026-03-09  
**Son Güncelleme:** 2026-03-09  
**Hedef Sürüm:** v1.6 (Contacts), v1.7 (Calendar), v1.8 (Tasks)  
**Vizyon:** Outlook ve Gmail'e gerçek alternatif — e-posta, kişiler ve takvim tek çatı altında.

---

## 1. Genel Vizyon ve Hedefler

PIM modülü, uygulamayı salt bir e-posta istemcisinden gerçek bir **Kişisel Bilgi Yöneticisi**ne dönüştürür.

### 1.1 Neden PIM?

| Kullanıcı Beklentisi | Outlook | Gmail | Mailora Hedefi |
|---|---|---|---|
| E-posta | ✅ | ✅ | ✅ Var |
| Kişi Rehberi | ✅ | ✅ | v1.6 |
| Takvim & Toplantılar | ✅ | ✅ | v1.7 |
| Görevler / To-do | ✅ | ✅ | v1.8 |
| Çevrimdışı Çalışma | Kısmi | Kısmi | v1.6+ |
| Gizlilik (veriler lokal) | ❌ | ❌ | ✅ Core değer |

### 1.2 Tasarım İlkeleri

1. **Çevrimdışı öncelikli (Offline-first):** Her şey önce yerel DB'ye yazılır, sync sonra gelir.
2. **Standart protokoller:** vCard, iCal, CardDAV, CalDAV — vendor lock-in yok.
3. **Kademeli karmaşıklık:** Temel özellikler önce, edge case'ler sonra.
4. **Hata toleransı:** Sync başarısız olsa bile uygulama çalışmaya devam eder.
5. **RBAC uyumlu:** Her PIM verisi kullanıcıya ait; kurumsal ortamda rol bazlı erişim.

---

## 2. Kişiler (Contacts / Address Book)

### 2.1 Kullanıcı Deneyimi (UX)

#### Ana Görünüm — contacts.html
```
┌──────────────────────────────────────────────────────────────┐
│ 🔍 [Ara: Ad, e-posta, şirket...]         [+ Yeni Kişi] [↑↓] │
├─────────────────────┬────────────────────────────────────────┤
│ GRUPLAR             │  DETAY PANELİ                          │
│ ─────────────────── │  ─────────────────────────────────────  │
│ 🌐 Tüm Kişiler (142)│  [Avatar] Ahmet Yılmaz                 │
│ ⭐ Favoriler (8)    │  ahmet@ornek.com • +90 555 111 22 33   │
│ 🕐 Son İletişim     │  Ornek A.Ş. — Genel Müdür              │
│ ─────────────────── │                                        │
│ 💼 İş              │  [ E-posta Gönder ] [ Arama Yap ]      │
│ 👨‍👩‍👧 Aile            │                                        │
│ 🎓 Okul             │  ─────────────────────────────────────  │
│ [+ Grup Ekle]       │  📧 ahmet@ornek.com (İş, Birincil)     │
│                     │  📧 ahmet@gmail.com (Kişisel)          │
│ A                   │  📞 +90 555 111 22 33 (Mobil)          │
│ Ahmet Yılmaz   ›   │  🏠 Ataşehir, İstanbul                 │
│ Ayşe Demir     ›   │  🎂 15 Mayıs 1985                      │
│ B                   │  📌 NOT: Proje yöneticisi             │
│ Burak Kaya     ›   │                                        │
│ ...                 │  [ Düzenle ] [ Gruba Ekle ] [ Sil ]   │
└─────────────────────┴────────────────────────────────────────┘
```

#### Kişi Ekle / Düzenle Formu
- **Zorunlu:** Tam ad (veya şirket)
- **Opsiyonel:** Her diğer alan
- **Dinamik:** E-posta, telefon ve adres alanları — "+" ile yeni satır ekle
- **Kaydet:** Anlık yerel kayıt, arkaplanda sync

#### Kişi Grubu Yönetimi
- Sürükle-bırak ile gruplara atama
- Çoklu seçim + toplu gruplama
- "Tüm Kişiler" silinemeyen sistem grubu
- Grup rengi → liste başlığında gösterilir

#### Arama ve Filtreleme
- Anlık filtre (debounce 300ms, min 1 karakter)
- Arama kapsamı: `full_name`, `company`, `email`, `phone`, `note`
- Filtreler: Grup, Hesap (hangi kurumu), Favoriler, Son güncelleme

#### Duplicate Tespit ve Birleştirme
- Otomatik: Aynı e-posta adresi veya ad+şirket kombinasyonu varsa uyarı
- Manuel: "Kişileri Birleştir" modal — hangi alan korunacağını kullanıcı seçer
- Merge stratejisi: En yeni `updated_at` kayıt öncelikli, ama kullanıcı override edebilir

#### Import / Export
- **Import:** `.vcf` dosyası sürükle-bırak veya dosya seç. Tek kişi veya bulk.
- **Export:** Seçili kişiler veya tümü → `.vcf` indir.
- **CSV:** basit ad/email/telefon import için CSV desteği (v1.6.1)

---

### 2.2 Veri Modeli — Kişi Alanları

| Alan | vCard Property | Tip | Zorunlu | Açıklama |
|---|---|---|---|---|
| Yerel UUID | — | TEXT PK | ✅ | SQLite birincil anahtar |
| CardDAV UID | `UID` | TEXT | hayır | Sunucu tarafı senkron kimliği |
| Tam Ad | `FN` | TEXT | ✅ | Görünen ad |
| Bölünmüş Ad | `N` | TEXT[] | hayır | [Soyad, Ad, İkinci Ad, Ön ek, Son ek] |
| Şirket | `ORG` | TEXT | hayır | Şirket; departman |
| Unvan | `TITLE` | TEXT | hayır | İş unvanı |
| E-postalar | `EMAIL` | TEXT[] | hayır | Çoklu, tipler: work/home/other |
| Telefonlar | `TEL` | TEXT[] | hayır | Çoklu, tipler: mobile/work/home/fax |
| Adresler | `ADR` | Struct[] | hayır | Çoklu |
| Doğum Tarihi | `BDAY` | DATE | hayır | YYYY-MM-DD |
| Not | `NOTE` | TEXT | hayır | Serbest |
| Fotoğraf | `PHOTO` | BLOB/URL | hayır | Base64 veya harici URL |
| Sosyal Medya | `X-*` | TEXT[] | hayır | LinkedIn, Twitter, GitHub vb. |
| Web Sitesi | `URL` | TEXT | hayır | |
| Cinsiyet | `GENDER` | TEXT | hayır | vCard 4.0 |
| Dil | `LANG` | TEXT | hayır | BCP 47 |
| Zaman Dilimi | `TZ` | TEXT | hayır | |
| Favori | — | INTEGER | hayır | 0/1, yerel |
| Kaynak Hesap | — | TEXT FK | ✅ | account_id |
| ETag | — | TEXT | hayır | CardDAV değişiklik tespiti |
| CardDAV href | — | TEXT | hayır | Sunucudaki yol |
| Ham vCard | — | TEXT | hayır | Parse edilemeyen alanlar için |
| Sync Durumu | — | TEXT | hayır | `local`, `synced`, `conflict`, `pending_delete` |
| Oluşturulma | — | DATETIME | ✅ | |
| Güncellenme | — | DATETIME | ✅ | |

---

### 2.3 Veritabanı Şeması (SQLite)

```sql
-- Ana kişi tablosu
CREATE TABLE contacts (
    id          TEXT PRIMARY KEY,
    account_id  TEXT NOT NULL,
    vcard_uid   TEXT,
    etag        TEXT,
    href        TEXT,
    full_name   TEXT NOT NULL,
    first_name  TEXT,
    last_name   TEXT,
    middle_name TEXT,
    prefix      TEXT,
    suffix      TEXT,
    company     TEXT,
    department  TEXT,
    title       TEXT,
    note        TEXT,
    birthday    TEXT,               -- YYYY-MM-DD
    photo_data  TEXT,               -- data:image/jpeg;base64,... veya URL
    website_url TEXT,
    gender      TEXT,
    language    TEXT,
    timezone    TEXT,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    sync_status TEXT NOT NULL DEFAULT 'local',
                                    -- 'local' | 'synced' | 'conflict' | 'pending_delete'
    raw_vcard   TEXT,
    synced_at   TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);
CREATE INDEX idx_contacts_account ON contacts(account_id);
CREATE INDEX idx_contacts_name ON contacts(last_name, first_name);

-- E-postalar (bire-çok)
CREATE TABLE contact_emails (
    id          TEXT PRIMARY KEY,
    contact_id  TEXT NOT NULL,
    email       TEXT NOT NULL,
    label       TEXT NOT NULL DEFAULT 'other', -- 'work' | 'home' | 'other'
    is_primary  INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
CREATE INDEX idx_contact_emails_email ON contact_emails(email);

-- Telefon numaraları (bire-çok)
CREATE TABLE contact_phones (
    id          TEXT PRIMARY KEY,
    contact_id  TEXT NOT NULL,
    phone       TEXT NOT NULL,
    label       TEXT NOT NULL DEFAULT 'other', -- 'mobile' | 'work' | 'home' | 'fax'
    is_primary  INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Adresler (bire-çok)
CREATE TABLE contact_addresses (
    id          TEXT PRIMARY KEY,
    contact_id  TEXT NOT NULL,
    label       TEXT NOT NULL DEFAULT 'other',
    street      TEXT,
    city        TEXT,
    region      TEXT,
    postal_code TEXT,
    country     TEXT,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Sosyal medya / URL'ler
CREATE TABLE contact_social (
    id          TEXT PRIMARY KEY,
    contact_id  TEXT NOT NULL,
    service     TEXT NOT NULL,     -- 'linkedin' | 'twitter' | 'github' | 'custom'
    url         TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Kişi grupları
CREATE TABLE contact_groups (
    id          TEXT PRIMARY KEY,
    account_id  TEXT NOT NULL,
    name        TEXT NOT NULL,
    color       TEXT,
    vcard_kind  TEXT,              -- vCard 4.0 KIND:group UID
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    UNIQUE (account_id, name)
);

-- Kişi ↔ Grup ilişkisi
CREATE TABLE contact_group_members (
    contact_id  TEXT NOT NULL,
    group_id    TEXT NOT NULL,
    PRIMARY KEY (contact_id, group_id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES contact_groups(id) ON DELETE CASCADE
);

-- CardDAV adres defteri senkronizasyon durumu
CREATE TABLE carddav_sync_state (
    account_id          TEXT NOT NULL,
    addressbook_url     TEXT NOT NULL,
    display_name        TEXT,
    sync_token          TEXT,
    ctag                TEXT,       -- Bazı sunucular ctag kullanır
    last_synced_at      TEXT,
    PRIMARY KEY (account_id, addressbook_url)
);

-- Kişi conflict log (silinmez, inceleme için tutulur)
CREATE TABLE contact_conflicts (
    id              TEXT PRIMARY KEY,
    contact_id      TEXT NOT NULL,
    local_data      TEXT NOT NULL,  -- JSON snapshot - yerel hali
    remote_data     TEXT NOT NULL,  -- JSON snapshot - sunucu hali
    detected_at     TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at     TEXT,
    resolution      TEXT            -- 'local_wins' | 'remote_wins' | 'merged'
);

-- FTS5 tam metin arama indeksi
CREATE VIRTUAL TABLE contacts_fts USING fts5(
    contact_id UNINDEXED,
    full_name,
    company,
    note,
    content='contacts',
    content_rowid='rowid',
    tokenize='unicode61'
);

-- FTS'i güncel tutmak için trigger'lar
CREATE TRIGGER contacts_ai AFTER INSERT ON contacts BEGIN
    INSERT INTO contacts_fts(rowid, contact_id, full_name, company, note)
    VALUES (new.rowid, new.id, new.full_name, new.company, new.note);
END;
CREATE TRIGGER contacts_ad AFTER DELETE ON contacts BEGIN
    INSERT INTO contacts_fts(contacts_fts, rowid, contact_id, full_name, company, note)
    VALUES ('delete', old.rowid, old.id, old.full_name, old.company, old.note);
END;
CREATE TRIGGER contacts_au AFTER UPDATE ON contacts BEGIN
    INSERT INTO contacts_fts(contacts_fts, rowid, contact_id, full_name, company, note)
    VALUES ('delete', old.rowid, old.id, old.full_name, old.company, old.note);
    INSERT INTO contacts_fts(rowid, contact_id, full_name, company, note)
    VALUES (new.rowid, new.id, new.full_name, new.company, new.note);
END;
```

---

### 2.4 Backend (Rust)

#### Dosya Yapısı
```
src/
├── models/
│   └── contact.rs              # Contact, ContactEmail, ContactPhone, ...
├── services/
│   ├── carddav_service.rs      # CardDAV sync motoru
│   └── contact_service.rs      # CRUD + conflict çözüm + autocomplete
├── routes/
│   └── contacts.rs             # REST API handler'ları
└── pim/
    ├── vcard.rs                # vCard 3.0 / 4.0 parser & serializer
    └── dav_client.rs           # Temel WebDAV HTTP client (PROPFIND, REPORT, PUT, DELETE)
```

#### API Endpoint'leri

| Method | URL | Query Params | Açıklama |
|---|---|---|---|
| GET | `/contacts` | `q`, `account_id`, `group_id`, `favorite`, `limit`, `offset`, `sort` | Kişi listesi |
| GET | `/contacts/:id` | — | Tekil kişi (tüm altlı alanlar dahil) |
| POST | `/contacts` | — | Yeni kişi (JSON body) |
| PUT | `/contacts/:id` | — | Kişi güncelle |
| DELETE | `/contacts/:id` | — | Kişi sil (sync_status → pending_delete) |
| GET | `/contacts/suggest` | `q` (min 1 char) | Autocomplete — e-posta ismi |
| GET | `/contacts/groups` | `account_id` | Grup listesi |
| POST | `/contacts/groups` | — | Grup oluştur |
| PUT | `/contacts/groups/:id` | — | Grup güncelle |
| DELETE | `/contacts/groups/:id` | — | Grup sil |
| POST | `/contacts/:id/groups/:gid` | — | Gruba ekle |
| DELETE | `/contacts/:id/groups/:gid` | — | Gruptan çıkar |
| POST | `/contacts/import` | — | `.vcf` dosyası yükle (multipart) |
| GET | `/contacts/export` | `account_id`, `group_id` | `.vcf` indir |
| POST | `/sync/carddav/:account_id` | — | Manuel sync tetikle |
| GET | `/contacts/conflicts` | `account_id` | Çözümsüz conflict listesi |
| POST | `/contacts/conflicts/:id/resolve` | — | Conflict çöz |

#### CardDAV Sync — Detaylı Algoritma

```
sync_carddav(account):
  1. PROPFIND /{principal}/ depth=1
     → adres defteri URL listesini al (displayname, resourcetype)
  
  2. Her adres defteri için:
     a. PROPFIND {addressbook_url} depth=1 prop=[etag, href]
        → mevcut {href → etag} haritasını al (remote_map)
     
     b. Yerel DB'den {href → etag} haritasını al (local_map)
     
     c. DIFF:
        - remote_map'te olup local_map'te yok → FETCH (yeni)
        - remote_map'te olup etag farklı → FETCH (güncellendi)
        - local_map'te olup remote_map'te yok → işaretle silinmiş
     
     d. Yeni/güncellenenler için:
        GET {href} → vCard al → parse → DB'ye upsert
        
        CONFLICT TESPİTİ:
        Eğer yerel sync_status == 'pending_update' VE remote etag değiştiyse:
          → conflict kaydı yarat (contact_conflicts tablosu)
          → sync_status = 'conflict'
          → kullanıcıya bildir
        Değilse:
          → normal güncelleme
     
     e. sync.status == 'pending_create' olanlar:
        PUT {new_href} (If-None-Match: *) → sunucuya gönder
     
     f. sync_status == 'pending_update' olanlar (conflict değilse):
        PUT {href} (If-Match: {local_etag}) → gönder
        201 → synced, etag güncelle
        412 Precondition Failed → conflict!
     
     g. sync_status == 'pending_delete' olanlar:
        DELETE {href} → sunucudan sil → DB'den sil
     
  3. sync_token / ctag güncelle
```

#### vCard Parser Gereksinimleri

- `BEGIN:VCARD` / `END:VCARD` blokları işle
- Line folding (CRLF + boşluk) düzelt
- Encoding: UTF-8, ISO-8859-1 → UTF-8 dönüşüm
- Parametre ayrıştırma: `TYPE=WORK,INTERNET`
- Property: `FN`, `N`, `EMAIL`, `TEL`, `ADR`, `ORG`, `TITLE`, `BDAY`, `NOTE`, `PHOTO`, `UID`, `REV`, `URL`, `CATEGORIES`
- vCard 3.0 ve 4.0 uyumlu (3.0 daha yaygın)
- `CATEGORIES` → `contact_groups` ile ilişkilendir

---

## 3. Takvim (Calendar / Events)

### 3.1 Kullanıcı Deneyimi (UX)

#### Ana Görünüm — calendar.html
```
┌──────────────────────────────────────────────────────────────┐
│ ◀ Mart 2026 ▶  [Gün] [Hafta] [Ay]    [+ Yeni Etkinlik]     │
├───────────────┬──────────────────────────────────────────────┤
│ MART 2026     │  HAFTALIK GÖRÜNÜM (09 - 15 Mart)            │
│ ─────────────  │  ──────────────────────────────────────────  │
│ H  S  Ç  P    │       PAZ   PZT   SAL   ÇAR   PER          │
│  1  2  3  4   │  09   [Takım Toplantısı 10:00-11:00]        │
│  8  9 10 11   │  10   [Proje Review 14:00-15:30]            │
│ [15]16 17 18   │  11                                         │
│ 22 23 24 25   │  12   [Öğle Yemeği 12:30-13:30]             │
│ 29 30 31      │  ...                                         │
│               │                                              │
│ TAKVİMLER     │  DETAY PANELİ                               │
│ ── ──────────  │  ─────────────────────────────────────────   │
│ 🔵 Kişisel    │  Takım Toplantısı                            │
│ 🔴 İş         │  Prş, 09 Mart 10:00 - 11:00                 │
│ 🟢 Aile       │  📍 Toplantı Odası 3                        │
│ [+ Takvim]    │  👥 Ahmet, Ayşe, Burak                      │
│               │  [ Düzenle ] [ Sil ] [ Yanıtla ]            │
└───────────────┴──────────────────────────────────────────────┘
```

#### Takvim Görünümleri

**Aylık Görünüm:**
- 7×6 grid, her gün hücresi
- Hücrede: renkli etkinlik başlığı (ilk 2-3), "+N daha" overflow
- Bugün vurgusu, seçili gün vurgulanır
- Tıkla → gün görünümüne git

**Haftalık Görünüm:**
- 7 sütun (Pzr-Cmt veya Pzt-Paz, ayarlanabilir)
- Saatlik satırlar (00:00 - 23:00)
- Çakışan etkinlikler yan yana gösterilir
- Sürükle-bırak ile taşıma (v1.7.1)

**Gün Görünümü:**
- Tek gün, saatlik detay
- "Tüm Gün Etkinlikleri" üstte bant
- Etkinlik yaratmak için boş alana tıkla

#### Etkinlik Oluşturma / Düzenleme Modal

```
Başlık:    [________________]
Takvim:    [Kişisel ▼]
Başlangıç: [2026-03-15] [10:00] [Saat Dilimi: Europe/Istanbul ▼]
Bitiş:     [2026-03-15] [11:00]
[ ] Tüm Gün
Konum:     [________________]
Açıklama:  [________________]
Katılımcılar: [ahmet@... + Ekle]  →  Kabul: ✅ Ayşe | ⏳ Burak
Tekrarla:  [Tekrarlamıyor ▼]   → Günlük | Haftalık | Aylık | Özel
Hatırlatma:[15 dk önce ▼]  [+ Ekle]
Görünürlük:[Herkese Açık ▼]
                          [ İptal ] [ Kaydet ] [ Kaydet & Davet Gönder ]
```

#### Toplantı Daveti Akışı (iMIP)

```
Sender (Organizer):
  1. Etkinlik oluştur → katılımcı e-postalarını gir
  2. "Kaydet & Davet Gönder" → outbox_service üzerinden e-posta gönder
     Content-Type: text/calendar; method=REQUEST
     Attachment: invite.ics

Recipient (Attendee):
  1. E-posta gelir → preview üstünde banner:
     "📅 Toplantı daveti: Takım Toplantısı — 15 Mart 10:00"
     [ ✅ Kabul Et ] [ ❌ Reddet ] [ ❓ Belki ]
  2. Yanıt → method=REPLY ile organizatöre dön
  3. Etkinlik takvime eklenir (status: ACCEPTED/DECLINED/TENTATIVE)
```

---

### 3.2 Tekrarlayan Etkinlikler (RRULE) — Detay

Bu alan en karmaşık kısımdır; doğru yapılması kritiktir.

#### Desteklenecek RRULE Kuralları

| Kural | Örnek | Açıklama |
|---|---|---|
| Günlük | `FREQ=DAILY;COUNT=5` | 5 gün boyunca her gün |
| Haftalık | `FREQ=WEEKLY;BYDAY=MO,WE,FR` | Pzt/Çar/Cum |
| Aylık (tarih) | `FREQ=MONTHLY;BYMONTHDAY=15` | Her ay 15'i |
| Aylık (gün) | `FREQ=MONTHLY;BYDAY=2MO` | Her ayın 2. Pazartesi'si |
| Yıllık | `FREQ=YEARLY;BYMONTH=3;BYMONTHDAY=15` | Her yıl 15 Mart |
| Bitiş tarihi | `UNTIL=20261231T000000Z` | Bu tarihe kadar |
| Tekrar sayısı | `COUNT=10` | 10 kez |
| Sonsuz | *(UNTIL ve COUNT yok)* | İptal edilene kadar |

#### RRULE Expansion Algoritması

Veritabanında tekrarlayan etkinlik **bir kez** kaydedilir (ana kayıt). UI için:
1. `/events?from=2026-03-01&to=2026-03-31` isteği geldiğinde
2. Backend, aralıktaki RRULE etkinliklerini **bellek içinde genişletir**
3. `EXDATE` (iptal edilmiş tekrarlar) çıkarılır
4. `event_exceptions` ile override edilmiş tekrarlar birleştirilir
5. Sonuç API'dan döndürülür — her tekrar ayrı nesne gibi

#### Exception Yönetimi

- **Tekil iptal:** O tarihte etkinlik iptal → `EXDATE` özelliği ekle
- **Bu ve sonrası iptal:** `RRULE:UNTIL` geri çek
- **Tek görünümü değiştir:** `event_exceptions` tablosuna yeni override yaz
- **Tüm seriyi değiştir:** Ana kaydı güncelle

---

### 3.3 Zaman Dilimi (Timezone) Yönetimi

Bu critical bir kısım — yanlış yapıldığında tüm etkinlikler bozulur.

#### Strateji

1. **Depolama:** Tüm zamanlar UTC olarak `TEXT` (ISO-8601: `2026-03-15T10:00:00Z`)
2. **Okuma:** Backend UTC döner, frontend kullanıcının yerel saat dilimine çevirir
3. **Yazma:** Frontend, kullanıcının girdiği saati UTC'ye çevirip gönderir
4. **Tüm Gün Etkinlik:** `DATE` formatında `2026-03-15` (saat yok, timezone yok)
5. **İCal Dışa Aktarma:** `DTSTART;TZID=Europe/Istanbul:20260315T100000`
6. **VTIMEZONE:** iCal'da timezone tanımı dahil edilir

#### Frontend Timezone Çözümü

```javascript
// Etkinlik gösterirken
const utc = new Date(event.start_at + 'Z'); // Backend'den UTC gelir
const local = utc.toLocaleString('tr-TR', { timeZone: userTimezone });

// Etkinlik kaydederken
const local = new Date(inputDatetime); // Kullanıcının girdiği
const utc = local.toISOString(); // UTC'ye çevir, backend'e gönder
```

---

### 3.4 Veritabanı Şeması (SQLite)

```sql
-- Takvimler
CREATE TABLE calendars (
    id              TEXT PRIMARY KEY,
    account_id      TEXT NOT NULL,
    name            TEXT NOT NULL,
    color           TEXT NOT NULL DEFAULT '#3b82f6',
    caldav_url      TEXT,
    sync_token      TEXT,
    ctag            TEXT,
    is_default      INTEGER NOT NULL DEFAULT 0,
    is_visible      INTEGER NOT NULL DEFAULT 1,
    description     TEXT,
    timezone        TEXT NOT NULL DEFAULT 'UTC',
    last_synced_at  TEXT,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);
CREATE INDEX idx_calendars_account ON calendars(account_id);

-- Etkinlikler
CREATE TABLE calendar_events (
    id                  TEXT PRIMARY KEY,
    calendar_id         TEXT NOT NULL,
    ical_uid            TEXT NOT NULL,
    etag                TEXT,
    href                TEXT,
    summary             TEXT NOT NULL,
    description         TEXT,
    location            TEXT,
    start_at            TEXT NOT NULL,      -- UTC ISO-8601 veya DATE (tüm gün için)
    end_at              TEXT NOT NULL,
    all_day             INTEGER NOT NULL DEFAULT 0,
    timezone            TEXT NOT NULL DEFAULT 'UTC',
    status              TEXT NOT NULL DEFAULT 'CONFIRMED',
    visibility          TEXT NOT NULL DEFAULT 'PUBLIC',
    recurrence_rule     TEXT,               -- Ham RRULE string
    recurrence_exdates  TEXT,               -- Virgülle ayrılmış EXDATE'ler
    organizer_email     TEXT,
    organizer_name      TEXT,
    color               TEXT,
    sequence            INTEGER DEFAULT 0,  -- iCal SEQUENCE (versiyon takibi)
    sync_status         TEXT NOT NULL DEFAULT 'local',
    raw_ical            TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (calendar_id) REFERENCES calendars(id) ON DELETE CASCADE,
    UNIQUE (calendar_id, ical_uid)
);
CREATE INDEX idx_events_calendar ON calendar_events(calendar_id);
CREATE INDEX idx_events_start ON calendar_events(start_at);
CREATE INDEX idx_events_uid ON calendar_events(ical_uid);

-- Katılımcılar
CREATE TABLE event_attendees (
    id          TEXT PRIMARY KEY,
    event_id    TEXT NOT NULL,
    email       TEXT NOT NULL,
    name        TEXT,
    role        TEXT NOT NULL DEFAULT 'REQ-PARTICIPANT',
    status      TEXT NOT NULL DEFAULT 'NEEDS-ACTION',
    rsvp        INTEGER NOT NULL DEFAULT 1, -- Yanıt bekleniyor mu
    FOREIGN KEY (event_id) REFERENCES calendar_events(id) ON DELETE CASCADE
);

-- Hatırlatmalar (VALARM)
CREATE TABLE event_alarms (
    id          TEXT PRIMARY KEY,
    event_id    TEXT NOT NULL,
    trigger_min INTEGER NOT NULL,           -- Negatif: önce, pozitif: sonra
    action      TEXT NOT NULL DEFAULT 'DISPLAY', -- 'DISPLAY' | 'EMAIL' | 'AUDIO'
    description TEXT,
    fired_at    TEXT,                       -- Bu alarm zaten çalındı mı?
    FOREIGN KEY (event_id) REFERENCES calendar_events(id) ON DELETE CASCADE
);

-- Tekrarlayan etkinlik override'ları
CREATE TABLE event_exceptions (
    id                  TEXT PRIMARY KEY,
    master_event_id     TEXT NOT NULL,
    recurrence_id       TEXT NOT NULL,      -- ISO-8601, hangi tekrarın override'ı
    exception_event_id  TEXT,               -- NULL ise iptal, değilse yeni etkinlik ID
    FOREIGN KEY (master_event_id) REFERENCES calendar_events(id) ON DELETE CASCADE
);

-- CalDAV conflict log
CREATE TABLE event_conflicts (
    id              TEXT PRIMARY KEY,
    event_id        TEXT NOT NULL,
    local_data      TEXT NOT NULL,
    remote_data     TEXT NOT NULL,
    detected_at     TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at     TEXT,
    resolution      TEXT
);

-- Bildirim kuyruk (gönderilmemiş alarmlar)
CREATE TABLE notification_queue (
    id          TEXT PRIMARY KEY,
    alarm_id    TEXT NOT NULL,
    event_id    TEXT NOT NULL,
    fire_at     TEXT NOT NULL,              -- UTC, ne zaman tetiklenecek
    type        TEXT NOT NULL DEFAULT 'DISPLAY',
    processed   INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (alarm_id) REFERENCES event_alarms(id) ON DELETE CASCADE
);
CREATE INDEX idx_notif_fire ON notification_queue(fire_at, processed);
```

---

### 3.5 Backend (Rust)

#### Dosya Yapısı
```
src/
├── models/
│   └── calendar.rs             # Calendar, CalendarEvent, Attendee, Alarm struct'ları
├── services/
│   ├── caldav_service.rs       # CalDAV sync motoru
│   ├── calendar_service.rs     # CRUD + RRULE expansion + alarm planlama
│   └── imip_service.rs         # iMIP davet gönderme / alma
├── routes/
│   └── calendar.rs             # REST API handler'ları
└── pim/
    ├── ical.rs                 # iCalendar parser & serializer
    └── rrule.rs                # RRULE expansion algoritması
```

#### API Endpoint'leri

| Method | URL | Query Params | Açıklama |
|---|---|---|---|
| GET | `/calendars` | `account_id` | Takvim listesi |
| POST | `/calendars` | — | Takvim oluştur |
| PUT | `/calendars/:id` | — | Takvim güncelle |
| DELETE | `/calendars/:id` | — | Takvim sil |
| GET | `/events` | `from`, `to`, `calendar_id`, `q`, `limit` | Etkinlik listesi (RRULE expand) |
| GET | `/events/:id` | — | Tekil etkinlik |
| POST | `/events` | — | Etkinlik oluştur |
| PUT | `/events/:id` | `scope` (this/following/all) | Etkinlik güncelle |
| DELETE | `/events/:id` | `scope` | Etkinlik sil |
| POST | `/events/:id/rsvp` | — | RSVP yanıtı (ACCEPT/DECLINE/TENTATIVE) |
| POST | `/events/import` | — | `.ics` dosyası yükle |
| GET | `/events/export` | `calendar_id`, `from`, `to` | `.ics` indir |
| POST | `/sync/caldav/:account_id` | — | Manuel sync |
| GET | `/events/conflicts` | — | Çözümsüz conflict'ler |

#### RRULE Expansion — Rust Implementasyonu

```rust
// pim/rrule.rs
pub fn expand_rrule(
    event: &CalendarEvent,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<EventOccurrence> {
    // 1. DTSTART'tan başla
    // 2. RRULE'u parse et (FREQ, INTERVAL, BYDAY, UNTIL, COUNT, ...)
    // 3. İterasyon: from..to arasına düşen tüm oluşumları üret
    // 4. EXDATE'leri filtrele
    // 5. event_exceptions ile override et
    // 6. Her oluşum için EventOccurrence döndür
}
```

**Kütüphane değerlendirmesi:** `rrule` crate'i (MIT) — doğruluğu test edilmiş RRULE implementasyonu.

#### CalDAV Sync — Detaylı Algoritma

```
sync_caldav(account):
  1. PROPFIND /{principal}/ depth=1
     → takvim koleksiyon URL'lerini al
  
  2. Her takvim için:
     a. GET sync-token veya ctag (değişti mi?)
     
     b. Değiştiyse → REPORT (sync-collection veya calendar-query)
        → {href → etag} haritası al (remote_map)
     
     c. Yerel DB haritasıyla diff:
        - Yeni: MULTIGET ile batch GET → parse → DB insert
        - Güncellendi: GET → parse → conflict kontrolü → upsert
        - Silindi: DB'den işaretle
     
     d. Yerel pending_create: PUT (If-None-Match: *)
     d. Yerel pending_update: PUT (If-Match: etag) — 412 conflict
     e. Yerel pending_delete: DELETE
     
  3. sync-token/ctag kaydet
  4. alarm_service.schedule_alarms() çağır → bildirim kuyruğu güncelle
```

---

## 4. Senkronizasyon Mimarisi

### 4.1 Desteklenecek Sağlayıcılar

| Sağlayıcı | CardDAV URL | CalDAV URL | Auth | Notlar |
|---|---|---|---|---|
| Google | `www.googleapis.com/carddav/v1/` | `calendar.google.com/dav/` | OAuth2 / App Password | App Password önerilir |
| Microsoft 365 | `people.googleapis.com` (hayır) | `outlook.office365.com/caldav/` | OAuth2 / App Password | CardDAV desteği sınırlı |
| Apple iCloud | `contacts.icloud.com` | `caldav.icloud.com` | App-specific password | 2FA zorunlu |
| Nextcloud | `{domain}/remote.php/dav/` | `{domain}/remote.php/dav/` | Username/Password | Self-hosted |
| Fastmail | `carddav.fastmail.com` | `caldav.fastmail.com` | App Password | RFC uyumlu |
| Yerel | — | — | — | DAV olmadan sadece DB |

### 4.2 Otomatik Discovery (RFC 6764)

```
1. /.well-known/carddav (HTTP redirect takip et)
2. /.well-known/caldav
3. DNS SRV: _carddav._tcp.{domain}, _caldav._tcp.{domain}
4. Hardcoded provider map:
   "gmail.com" → Google CardDAV/CalDAV URL
   "icloud.com" → Apple URL
   "outlook.com", "hotmail.com" → Microsoft URL
5. Başarısız → kullanıcıya manuel URL girişi sor
```

### 4.3 Sync Zamanlama

```
Periyodik (background service — her 15 dk):
  for each account:
    spawn_task(sync_carddav(account))
    spawn_task(sync_caldav(account))

Anlık (kullanıcı tetikli):
  POST /sync/carddav/:account_id
  POST /sync/caldav/:account_id

Akıllı sync:
  - Son sync < 5 dk → atla (rate limit koruma)
  - Sync hatası → exponential backoff (1dk, 2dk, 4dk, 8dk max)
  - Çevrimdışı → sync kuyruğa al, bağlantı gelince çalıştır
```

### 4.4 Conflict Çözüm Stratejisi

Conflict = Aynı kayıt hem yerel hem sunucuda değişti (ETag uyuşmuyor).

**Otomatik Çözüm (conflict yok):**
- Yerel değişmedi + sunucu değişti → sunucuyu al (remote wins)
- Yerel değişti + sunucu değişmedi → sunucuya gönder (local wins)

**Manuel Çözüm Gerekli:**
- İkisi de değişti → `contact_conflicts` / `event_conflicts` tablosuna yaz
- UI'da "⚠️ X çakışma çözümleniyor" banner'ı
- Conflict Resolution Modal:
  - Sol: Yerel versiyon | Sağ: Sunucu versiyonu
  - Alan bazında seçim (hangisi korunacak)
  - "Yerel Kazan" / "Sunucu Kazansın" / "Birleştir"

---

## 5. Bildirim Sistemi (Alarmlar)

### 5.1 Alarm Çeşitleri
- **DISPLAY:** Tarayıcı bildirimi (Web Notifications API)
- **EMAIL:** E-posta hatırlatması (Outbox Service üzerinden)

### 5.2 Alarm Pipeline

```
Etkinlik kaydedildi / güncellendi
  → calendar_service.schedule_alarms()
  → notification_queue tablosuna ekle (fire_at hesapla: start_at - trigger_min)

Background Alarm Servisi (her 1 dk polling):
  SELECT * FROM notification_queue WHERE fire_at <= datetime('now') AND processed = 0
  For each alarm:
    if type == 'DISPLAY':
      SSE event gönder → frontend bildirim gösterir
    if type == 'EMAIL':
      outbox_service.queue_email() → hatırlatma maili
    UPDATE notification_queue SET processed = 1 WHERE id = ...
```

### 5.3 Frontend Bildirim
- SSE `/events/stream` üzerinden bildirim gelir
- Web Notifications API (kullanıcı izin vermişse)
- Uygulama kapalıysa → Service Worker (v1.7.2)

---

## 6. Import / Export

### 6.1 Kişiler

| Format | Import | Export | Açıklama |
|---|---|---|---|
| `.vcf` (vCard 3.0) | ✅ | ✅ | Tek veya çoklu kişi |
| `.vcf` (vCard 4.0) | ✅ | ✅ | |
| `.csv` | ✅ (v1.6.1) | ✅ (v1.6.1) | Outlook/Gmail CSV format |

**Import Akışı:**
1. Dosya sürükle-bırak veya seç
2. Preview: "X kişi bulundu, Y zaten var → güncelleme, Z yeni"
3. Çakışan kayıtlar için duplicate karşılaştırma
4. Onayla → import

### 6.2 Takvim

| Format | Import | Export | Açıklama |
|---|---|---|---|
| `.ics` (iCal) | ✅ | ✅ | Tek veya çoklu etkinlik |
| Outlook CSV | ✅ (v1.7.1) | hayır | |

---

## 7. RBAC ve Gizlilik

### 7.1 Kişi Erişimi

| Rol | Ne görebilir |
|---|---|
| Admin | Tüm kişiler (tüm hesaplar) |
| Member | Sadece kendi hesabına ait kişiler |

### 7.2 Takvim Erişimi

| Görünürlük | Kimler görebilir |
|---|---|
| `PUBLIC` | Hesaba erişimi olan tüm kullanıcılar |
| `PRIVATE` | Sadece hesap sahibi |
| `CONFIDENTIAL` | Başlık görünür, detay gizli |

### 7.3 Kurumsal Senaryo
- Admin tüm takvimleri görebilir (sadece `PUBLIC` etkinlikler)
- Member yalnızca kendi takvimine etkinlik ekleyebilir
- `PRIVATE` etkinlikler API'dan filtre edilir (query'ye `AND visibility != 'PRIVATE'` eklenir rollere göre)

---

## 8. Hata İşleme (Error Handling)

### 8.1 Sync Hataları

| Hata | Davranış |
|---|---|
| HTTP 401 | Hesap kimlik bilgileri geçersiz → kullanıcıya uyarı, sync durdur |
| HTTP 409 (ETag mismatch) | Conflict → conflict tablosuna yaz |
| HTTP 503 / Timeout | Retry backoff (1, 2, 4, 8 dk) |
| Network yok | Offline mode, yerel çalışmaya devam |
| vCard parse hatası | Kaydı atla, `raw_vcard` olarak sakla, logla |
| iCal parse hatası | Kaydı atla, `raw_ical` olarak sakla, logla |

### 8.2 Kullanıcı Hataları

| Durum | Mesaj |
|---|---|
| Zorunlu alan boş | Satır kırmızı border, "Alan gereklidir" |
| Geçersiz e-posta | Anlık validasyon |
| Bitiş < Başlangıç | Kırmızı uyarı, kaydetme engellenir |
| Network hatası | Toast: "Bağlantı hatası, tekrar deneniyor..." |

---

## 9. Performans ve Ölçeklenebilirlik

### 9.1 Pagination
- Tüm liste endpoint'leri: `limit` (default: 100, max: 500), `offset`
- Kişi listesi için `cursor` bazlı pagination (isim alfabetik sırada, last_id)

### 9.2 Büyük Kişi Listeleri
- 10.000+ kişi → FTS5 arama zorunlu, IN-MEMORY filtre yok
- Frontend: Sanal liste (sadece görünen satırlar DOM'da)

### 9.3 RRULE Expansion Limiti
- Backend, en fazla 500 tekrar üretir (DoS koruması)
- Tarih aralığı olmadan `/events` çağrısı desteklenmez

### 9.4 Fotoğraf Optimizasyonu
- İmport sırasında 200x200 px'e thumbnail oluştur
- Büyük PHOTO alanları DB şişirmemesi için harici dosyaya yazılabilir (v1.6.1)

---

## 10. Test Stratejisi

### 10.1 Unit Testler
- `pim/vcard.rs` — vCard 3.0/4.0 parse ve serialize (test fixture'ları: Google, iCloud, Outlook vCard örnekleri)
- `pim/ical.rs` — iCal parse (VEVENT, RRULE, VALARM)
- `pim/rrule.rs` — RRULE expansion doğruluğu (haftalık, aylık, EXDATE, UNTIL)
- Timezone dönüşümleri — DST edge case'leri

### 10.2 Integration Testler
- CardDAV mock sunucusu (Rust `axum` ile basit mock) üzerinden full sync senaryosu
- Conflict detection ve resolution akışı
- Import/Export round-trip (vCard → DB → vCard)

### 10.3 Manuel Test Senaryoları
- Google Contacts sync (App Password ile)
- Apple iCloud Contacts sync
- Nextcloud CalDAV sync
- Tekrarlayan etkinlik oluştur + tek oluşumu değiştir
- iMIP davet → karşı tarafta Kabul → organizatörde güncelleme
- Çevrimdışı iken kişi ekle → çevrimiçi olunca sync

---

## 11. Bağımlılık Analizi (Rust Crates)

| Crate | Sürüm | Amaç | Lisans |
|---|---|---|---|
| `rrule` | ^0.11 | RRULE expansion | MIT |
| `icalendar` | ^0.16 | iCal parsing | MIT/Apache |
| `vcard` | değerlendiriliyor | vCard parsing | — |
| `chrono` | zaten var | Tarih/saat | MIT/Apache |
| `chrono-tz` | ^0.9 | IANA timezone | MIT/Apache |
| `reqwest` | zaten var | HTTP DAV calls | MIT/Apache |
| `uuid` | zaten var | UUID üretimi | MIT/Apache |

> **Not:** vCard için hazır bir crate bulunamazsa `nom` parser crate'i ile minimal custom parser daha güvenli.

---

## 12. Aşamalı Uygulama Planı

### Faz 1 — v1.6: Kişiler (Contacts) — ~4 hafta

**Sprint 1 (Backend):**
- [ ] DB migrasyon: tüm contact tabloları
- [ ] `models/contact.rs` struct'ları
- [ ] `pim/vcard.rs` parser + serializer (unit test dahil)
- [ ] `services/contact_service.rs` CRUD
- [ ] `routes/contacts.rs` API endpoint'ler
- [ ] FTS5 trigger'ları ve arama endpoint'i

**Sprint 2 (Sync + Frontend):**
- [ ] `pim/dav_client.rs` WebDAV HTTP client
- [ ] `services/carddav_service.rs` read-only sync
- [ ] Otomatik discovery
- [ ] CardDAV write (pending_create/update/delete)
- [ ] `contacts.html` sayfası — liste + detay paneli
- [ ] E-posta autocomplete entegrasyonu (app.html)

**Sprint 3 (Cilalama):**
- [ ] Conflict detection + UI
- [ ] Import/Export (.vcf)
- [ ] Duplicate detection
- [ ] RBAC filter uygulaması

### Faz 2 — v1.7: Takvim (Calendar) — ~6 hafta

**Sprint 4 (Backend):**
- [ ] DB migrasyon: tüm calendar tabloları
- [ ] `models/calendar.rs` struct'ları
- [ ] `pim/ical.rs` parser + serializer
- [ ] `pim/rrule.rs` RRULE expansion
- [ ] `services/calendar_service.rs` CRUD + alarm planlama
- [ ] `routes/calendar.rs` API endpoint'leri

**Sprint 5 (Sync + iMIP):**
- [ ] `services/caldav_service.rs` sync motoru
- [ ] `services/imip_service.rs` davet gönder/al
- [ ] E-postada `text/calendar` tespit + RSVP banner
- [ ] Bildirim kuyruğu (notification_queue)

**Sprint 6 (Frontend):**
- [ ] `calendar.html` sayfası
- [ ] Aylık görünüm
- [ ] Haftalık görünüm
- [ ] Gün görünümü
- [ ] Etkinlik oluştur/düzenle modal
- [ ] Takvim rengi / görünür/gizli toggle
- [ ] Frontend alarm bildirimi (SSE)

**Sprint 7 (Cilalama):**
- [ ] Import/Export (.ics)
- [ ] Conflict detection + UI
- [ ] Tekrarlayan etkinlik override UI

### Faz 3 — v1.8: Görevler (Tasks) — ~2 hafta
- [ ] CalDAV VTODO desteği (aynı altyapı, farklı bileşen tipi)
- [ ] Görev listesi UI (`tasks.html` veya takvim sidebar'ına entegre)
- [ ] E-postadan görev oluştur

---

## 13. Açık Sorular ve Kararlar

| Soru | Karar | Gerekçe |
|---|---|---|
| vCard crate seçimi? | `nom` ile minimal custom veya `vcard4` crate | Bağımlılık kontrolü |
| CardDAV: sync-token mu, ctag mi? | Her ikisini de dene, fallback REPORT | Sunucu uyumluluğu |
| RRULE kütüphanesi? | `rrule` crate (^0.11) | Doğruluğu kanıtlanmış |
| Timezone depolama? | UTC TEXT + frontend dönüşüm | Basit ve güvenilir |
| İlk sync: read-only mu? | Evet, önce read, sonra write aktif | Risk yönetimi |
| Fotoğraf depolama? | Base64 TEXT, 200px thumbnail | Kolay, tek sorunda |
| Conflict: auto mu manual mı? | Her ikisi (bak §4.4) | Kullanıcı kontrolü |
| Microsoft CardDAV? | Sınırlı test, Microsoft Graph API v2'de | EWS karmaşıklığı |
| Tekrarlayan UI — tek mi seri mi? | Modal ile sor: "Bu", "Bu ve sonrakiler", "Tümü" | Outlook gibi |
| Offline mode ne kadar derin? | Yerel CRUD her zaman çalışır, sync isteğe bağlı | Core prensip |
