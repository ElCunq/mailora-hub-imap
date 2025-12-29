
# Project Brief

Purpose:
- Summarize core requirements and goals for Mailora Hub IMAP component.

Goals:
- Provide reliable IMAP-based sync (multi-folder, UID-based) and HTTP API (/diff, /body, /folders, etc.).
- Ensure new mail detection is robust across servers (Gmail focus), with proper cursor semantics.
- Serve message bodies and metadata with correct mailbox context.

Non-Goals (for now):
- Full MIME rendering and attachment extraction beyond basic metadata.
- Complete push (IDLE) lifecycle across all servers.

Stakeholders:
- Owner: ElCunq
- Users: Mailora client/UI consumers, integration tests

Success Criteria:
- /diff returns new messages consistently when IMAP shows newer UIDs.
- /body returns content given correct folder+uid; auto-scan fallback works.
- Trace logs enable quick diagnosis of IMAP anomalies.

---


# Mailora — Project Brief (v0.1)

Tarih: 2025-10-10

Özet
- Tek masaüstü uygulama (Rust + Tauri) iki modda çalışır:
  - Personal (Local-Only): Gömülü Stalwart (tek binary) loopback üzerinden; SQLite (WAL); unified inbox; temel spam süzgeci.
  - Enterprise (Remote): Kurumsal IMAP/SMTP/JMAP uçlarına bağlanır; RBAC görünürlüğü; event log senkronu.
- Hedef: Outlook/Thunderbird benzeri deneyimi düşük maliyet ve güçlü denetimle birleştirmek.

Amaçlar
- Anlık alma (IMAP IDLE) ve lazy body/attachment fetch ile verimli senkronizasyon.
- Tüm hesaplarda birleşik indeks ve “Unified Inbox” görünümü.
- Enterprise modda RBAC’e uygun görünürlük ve olay denetimi.
- Personal modda tamamen lokal ve 0-maliyetli çalışma.

Kapsam (MVP)
- Kurulum sihirbazı (mod seçimi, uç nokta ve kimlik bilgisi).
- **Auto-Discovery:** ISPDB ve DNS SRV ile otomatik sunucu tespiti (Magic Login).
- IMAP motoru: UID/UIDVALIDITY delta, IDLE, BODYSTRUCTURE temelli parça seçimi.
- SMTP submission (587/465, PIPELINING destekliyse kullan).
- SQLite şeması: messages (unified index), events (IN/OUT logları).
- RBAC görünümü: admin vs üye maskesi.
- Temel telemetri ve loglama.

Kapsam Dışı (Non-goals)
- Sunucu tarafı MTA/antispam yönetimi (Enterprise’da mevcut altyapı kullanılır).
- İleri düzey ML tabanlı sınıflandırma (ileriki aşama).
- Tam kapsamlı admin konsolu (istemci tarafı görünürlükle sınırlı).

Başarı Metrikleri
- Performans: 100 ileti listeleme ≤ 3.5 sn (Gmail) / ≤ 4.5 sn (genel) ~50ms RTT’de.
- İstek sayısı: 100 ileti ≤ 12 istek; indirilen byte/ileti ≤ 80KB (ekler hariç).
- Güvenilirlik: FETCH-empty durumları yapılandırılmış hata kodlarıyla yönetilir ve otomatik yeniden denemeyle toparlanır.


Varsayımlar
- Stalwart binary paketle birlikte gelir (Personal), yalnız 127.0.0.1’e bağlanır.
- Desktop OS hedefleri: Linux/macOS/Windows (Tauri).
- Kimlik bilgileri OS keychain/secure storage’da saklanır.

---

# Yol Haritası & Milestone Özeti (v0.1)

0. Hazırlık (Gün 0)
Amaç: Lokal Stalwart örneği + temel erişim.
Kabul Kriterleri: IMAP/SMTP bağlantısı, domain+user eklenebiliyor, openssl ve nc ile temel testler geçiyor.

1. İstemci Temel Entegrasyon (Hafta 1)
Amaç: IMAP delta + IDLE, SMTP gönderim, tek hesap. DB şeması: messages, events.
Kabul Kriterleri: Yeni mail DB'de, gönderim sonrası events(OUT) kaydı.

2. Unified Inbox & Çok Hesap (Hafta 2)
Amaç: Çoklu hesap, unified görünüm, flag senkronizasyonu.
Kabul Kriterleri: 2+ hesapla INBOX tek listede, flag değişimi kaynağa yansıyor.

3. Kişisel Mod: Gömülü Stalwart (Hafta 3)
Amaç: Gömülü Stalwart ile tam lokal kullanım, port keşfi, health-check.
Kabul Kriterleri: Tek tıkla Stalwart başlat, dışa açık port yok.

4. Kurumsal Mod: Uzak Sunucu + RBAC (Hafta 4–5)
Amaç: Uzak IMAP/SMTP/JMAP, RBAC görünürlüğü, event log senkronu.
Kabul Kriterleri: Admin actor'ı görür, üye maskeli, anlamlı hata ve retry/backoff.

5. Performans & Güvenilirlik (Hafta 6)
Amaç: Stabil senkron, düşük gecikme, hata toparlama, lazy fetch, paralel fetch.
Kabul Kriterleri: Ağ kesilip gelince toparlanır, büyük mailbox’ta UI donmaz.

6. Spam & Kurallar (Hafta 7)
Amaç: Basit spam süzme, kullanıcı kuralı, Rspamd opsiyonel.
Kabul Kriterleri: Spam klasör akışı, kullanıcı kuralı uygulanıyor.

7. JMAP (Opsiyonel) (Hafta 8)
Amaç: Büyük posta kutularında verimli listeleme/arama.
Kabul Kriterleri: 50k+ mailde hızlı listeleme, fallback sorunsuz.

8. Güvenlik & Sertifikalar (Hafta 9)
Amaç: TLS, kimlik bilgisi güvenliği, secrets dosyada değil.
Kabul Kriterleri: MITM'e açık akış kalmaz, secrets güvenli.

9. Paketleme & QA (Hafta 10)
Amaç: Tauri paketleri, temel testler, CI/CD.
Kabul Kriterleri: "İndir → kur → çalış" akışı, testler yeşil.

10. Telemetri & Log Görselleme (Hafta 11–12)
Amaç: Admin event viewer, quota/hata metrikleri, log rotasyonu.
Kabul Kriterleri: Admin olayları filtreler, büyük log dosyaları şişmez.

Milestone “Definition of Done”
- Local run: IMAP/SMTP çalışıyor, yeni mail DB’de
- Multi-account: Unified Inbox listeliyor, flag senkron
- Personal mode: Gömülü Stalwart tek tıkla
- Enterprise mode: Uzak sunucu + RBAC
- Stability & Perf: backoff, lazy body, pipeline
- Security: TLS, secrets güvenli
- Release: Tauri paket + testler yeşil
