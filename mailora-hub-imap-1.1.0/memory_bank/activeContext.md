# Active Context

Tarih: 2025-12-25
Branch: v1.1.0-dev

## Mevcut Durum
Proje **Auto-Discovery (Magic Login)** ve **UI Unifikasyonu (Dark Theme)** ile güçlendirildi. Stabilite sorunları çözüldü. Şimdi ise **Kurumsal (Enterprise) Mod** özelliklerine başlanmıştır; temel **RBAC** altyapısı kuruldu.

### Son Tamamlananlar (v1.1.0 -> v1.2.0-dev)
- **Kurumsal Mod Hazırlığı:**
  - `users` tablosu ile **RBAC** (Role-Based Access Control) temeli atıldı (Admin / Member rolleri).
  - `auth_service` ve API uç noktaları (`/auth/register`, `/auth/login`) oluşturuldu.
  - Basit bir `static/login.html` ve `static/register.html` arayüzü eklendi.
  - İlk kayıt olan kullanıcı otomatik olarak **Admin** yetkisine sahip oluyor.
  - Ana uygulama (`app.html`), token yoksa giriş sayfasına yönlendiriyor.
- **Auto-Discovery (Magic Login):** (Tamamlandı)
  - Mozilla ISPDB entegrasyonu.
  - DNS SRV (`_imap._tcp`, `_submission._tcp`) sorgulama.
- **UI Unifikasyonu:** (Tamamlandı)
  - `add_account.html` ana uygulama ile uyumlu hale getirildi.

## Odak
- Kurumsal mod için "Olay Günlüğü" (Event Logging) ve Admin Paneli.
- PIM Entegrasyonu (CalDAV/CardDAV) hazırlığı.

## Sıradaki İşler (Faz 3 - Enterprise & PIM)
- **Admin Paneli:** Kullanıcıları listeleme, silme ve sistem loglarını (`event_logs`) görüntüleme.
- **Olay Günlüğü:** `LOGIN`, `FETCH`, `SEND` gibi önemli eylemleri veritabanına kaydetme.
- **Tam Metin Arama (FTS):** SQLite FTS5 entegrasyonu.
- **Kalıcı Kuyruklar:** Giden e-postalar için `Outbox` mekanizması.

## Kararlar
- **RBAC:** Basit bir JWT benzeri token ("id:role") yapısı şimdilik yeterli görüldü (MVP için). İleride gerçek JWT'ye geçilecek.
- **Kayıt Politikası:** "Herkese açık" kayıt politikası benimsendi (ilk üye Admin, diğerleri Member).
