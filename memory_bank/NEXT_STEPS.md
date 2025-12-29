# NEXT STEPS – Faz 2.5 (v1.1.0+)

Tarih: 2025-12-25
Durum: Auto-Discovery ve UI iyileştirmeleri tamamlandı. Stabilite sağlandı.

## Öncelikler (Sıra ile)

1) PIM Entegrasyonu (CalDAV/CardDAV)
- İletişim kişileri ve takvim senkronizasyonu.
- `contacts` ve `events` tablolarının tasarlanması.
- `dav-server` veya benzeri crate entegrasyonu.

2) Tam Metin Arama (FTS)
- SQLite FTS5 sanal tablolarının (`messages_fts`) oluşturulması.
- Mesaj gövdesinin (body) indekslenmesi.
- `GET /search` endpoint'inin FTS kullanacak şekilde güncellenmesi.

3) Unified Search ve Gelişmiş Filtreleme
- Birden fazla hesapta aynı anda arama.
- Filtreler: Kimden, Konu, Tarih, Ek var mı?, Okunmadı mı?

4) Kalıcı Kuyruklar
- `pending_outbox` tablosu: Gönderilemeyen mesajların saklanması ve retry mekanizması.
- `sent_finalize_queue`: Gönderilen mesajların senkronizasyonunun garanti altına alınması.

5) JMAP (Opsiyonel / İleri Seviye)
- Çok büyük posta kutuları için performans artışı gerekirse JMAP proxy katmanının tekrar değerlendirilmesi.

## Kabul Kriterleri
- PIM: Kişiler ve Takvim etkinlikleri çift yönlü senkronize edilmeli.
- FTS: "merhaba" araması başlıkta ve içerikte anlık sonuç dönmeli.
- Kuyruk: Sunucu yeniden başlatıldığında bekleyen e-postalar kaybolmamalı, tekrar denenmeli.

## Sprint Planı
- **Sprint 3:** (1) PIM Entegrasyonu, (2) FTS Temel Kurulum
- **Sprint 4:** (3) Unified Search, (4) Kalıcı Kuyruklar
