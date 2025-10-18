# Product Context

Why:
- Enable a minimal IMAP hub that powers a simple mail UI with reliable new mail visibility.

Problems solved:
- Inconsistent IMAP FETCH behavior across servers by adding robust fallbacks and diagnostics.
- Confusion around UID vs sequence numbers and folder contexts (Gmail labels).

How it should work:
- Users login; server persists creds; UI queries /diff and /body with cursors.
- Changes include folder to request body from correct mailbox.

UX goals:
- Fast initial feedback (visible new mail even when fetch meta fails).
- Clear errors when UID exists but fetch is empty (actionable messages/logs).

---

# Ürün Vizyonu ve Bağlam

Neden
- Kişisel kullanıcılar için tamamen yerel, hızlı ve düşük maliyetli bir e-posta deneyimi.
- Kurumsal kullanıcılar için denetlenebilirlik (event log), rol tabanlı görünürlük (RBAC) ve mevcut altyapıyla entegrasyon.

Çözülen Problemler
- Çoklu hesaplarda parçalı deneyim → Unified Inbox ile tek görünüm.
- Yavaş ve pahalı tam gövde indirme → Lazy fetch ve BODYSTRUCTURE ile hedefli indirme.
- Kurumlarda kim hangi kimlikle gönderdi sorusu → Event log + actor görünürlüğü.

Personalar
- Bireysel Kullanıcı: Hızlı kurulum, unified inbox, temel spam filtresi.
- Kurumsal Üye: Yetkisi dahilindeki klasörleri görür, actor maskeli.
- Kurumsal Admin: RBAC görünürlüğü, audit/event metriklerine erişim.

Kilit Kullanım Senaryoları
- İlk kurulum sihirbazı ile Personal/Enterprise seçimi ve hesap ekleme.
- Çoklu hesap/klasörden gelen iletilerin birleşik listelenmesi ve okunma/flag eşitleme.
- Gönderim sonrası OUT olayının kayıtlanması ve (Enterprise) sunucuya raporlama.

UX Hedefleri
- “Tek sorguda” unified liste; gecikmesiz IDLE güncellemeleri.
- İçerik görüntüleme: text/plain öncelikli, yoksa güvenli HTML fallback.
- Hatalarda net ve eyleme dönük mesajlar (ör. IMAP_EMPTY_FETCH).
