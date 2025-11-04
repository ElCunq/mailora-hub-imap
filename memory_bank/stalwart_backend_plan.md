# Stalwart Backend ile Doğrudan Entegrasyon Notları

- IMAP/SMTP protokolüyle uğraşmak yerine, doğrudan Stalwart'ın REST API veya yönetim API'si üzerinden tüm posta işlemleri yapılabilir.
- Kullanıcı ekleme, kimlik doğrulama, mesaj fetch/send, posta kutusu yönetimi gibi işlemler Stalwart'ın API'si ile merkezi olarak yönetilir.
- Programda IMAP/SMTP bağlantı mantığı ve hesap ekleme kodları es geçilecek; doğrudan Stalwart backendine HTTP/REST ile bağlanılacak.
- memory_bank/ klasöründe bu entegrasyon için mimari ve kod değişiklikleri planlanacak.
- Uygulama, Stalwart'a bağlı istemci olarak çalışacak ve tüm veriyi API üzerinden çekecek/gönderecek.

## Yapılacaklar
- Stalwart API dokümantasyonu incelenecek.
- Mevcut IMAP/SMTP kodları modülerleştirilecek ve opsiyonel hale getirilecek.
- API ile doğrudan veri çekme/gönderme fonksiyonları eklenecek.
- memory_bank/ klasöründe entegrasyon planı ve gereksinimler güncellenecek.

---
Bu not, Stalwart backend entegrasyonu için yol haritası olarak kullanılacaktır. Kod ve mimari güncellemeler memory_bank/ altında takip edilecektir.
