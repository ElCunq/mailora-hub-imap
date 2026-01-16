# Stalwart Backend Entegrasyon Notları (Gelecek Vizyonu - v2.0)

**Durum:** Bu plan, projenin **v2.0 (Uzun Vadeli)** hedefleri arasındadır. Mevcut sürüm (v1.x), "Universal IMAP Client" olarak tüm sağlayıcılara (Gmail, Outlook vb.) bağlanacak şekilde geliştirilmektedir.

## Vizyon: Personal Cloud Mode
Kullanıcı kendi sunucusunu barındırmak istediğinde (örneğin "Personal Mode"), gömülü bir Stalwart sunucusu devreye girecektir.

- **Hibrit Yapı:** Uygulama hem dış IMAP hesaplarını (mevcut yapı) hem de yerel Stalwart sunucusunu (gelecek yapı) aynı anda yönetebilecek.
- **Performans:** Yerel Stalwart hesabı için REST/JMAP API kullanılarak IMAP darboğazları aşılacak.
- **Veri Sahipliği:** Tüm postalar yerel diskte, Stalwart'ın güvenli depolama alanında tutulacak.

## Yapılacaklar (v2.0 Hazırlığı)
- [ ] Stalwart REST API dokümantasyonu incelenecek.
- [ ] Mevcut `EmailProvider` yapısına `StalwartLocal` tipi eklenecek.
- [ ] IMAP istemcisi yerine doğrudan API çağrısı yapan bir `MessageSource` trait'i (arayüzü) tasarlanacak.
- [ ] Uygulama içine Stalwart binary'sini indiren/başlatan bir süreç yöneticisi eklenecek.
