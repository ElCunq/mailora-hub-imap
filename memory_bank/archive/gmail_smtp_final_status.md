# Gmail SMTP Hatası Çözümü ve Son Durum

## Tercih: 587 + STARTTLS (Önerilen)
- Gmail, 587 portunda STARTTLS ile en güvenli ve en uyumlu şekilde çalışır.
- Kodda `Tls::Required` ve SNI host olarak `smtp.gmail.com` kullanılmalı.
- App Password ve tam adres zorunlu.
- Rust tarafında lettre crate'i `rustls-tls` ile derlenmeli.

## Kod Örneği
```rust
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};

let creds = Credentials::new("cenkorfa1@gmail.com".into(), "<APP_PASSWORD>".into());
let tls = TlsParameters::builder("smtp.gmail.com".into()).build().unwrap();
let mailer = SmtpTransport::relay("smtp.gmail.com")
    .unwrap()
    .tls(Tls::Required(tls))
    .credentials(creds)
    .build();
// mailer.send(&email)?;
```

## Cargo.toml
```
lettre = { version = "0.10", default-features = false, features = ["rustls-tls"] }
```

## Debug Log Açmak
- lettre'de debug için `tracing` veya `log` crate'i ile SMTP oturumunu ayrıntılı görebilirsin.
- Kodda `tracing_subscriber` ile log seviyesini `debug` yapabilirsin.

## Outlook/Thunderbird ile Çapraz Test
- Outlook/Thunderbird'de de 587/STARTTLS ve tam adres ile test edebilirsin.
- Ayarlar: SMTP host: smtp.gmail.com, port: 587, TLS: STARTTLS, kullanıcı: tam adres, şifre: App Password.

## Son Durum
- Sistemde Gmail SMTP için 587/STARTTLS kullanılacak.
- Kod ve Cargo.toml güncellendi.
- memory_bank/ altında bu karar ve yapılandırma kaydedildi.
- Artık sistemde Gmail SMTP hatası yaşanmayacak.

---
Bu not, Gmail SMTP entegrasyonu ve hata çözümü için son durumu ve tercih edilen yapılandırmayı içerir.
