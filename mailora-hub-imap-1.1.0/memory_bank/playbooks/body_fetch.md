# Playbook: Fetching Message Body Reliably

1) SELECT <mailbox>
2) NOOP (flush pipeline)
3) UID FETCH <uid> (UID ENVELOPE FLAGS BODYSTRUCTURE)
   - If empty, UID SEARCH UID <uid>, then re-SELECT and retry UID FETCH.
   - If still empty, try SEQ FETCH as last resort.
4) Decide section via BODYSTRUCTURE; fallback path:
   - BODY.PEEK[TEXT] → BODY.PEEK[1.TEXT] → BODY.PEEK[1.1.TEXT] → BODY.PEEK[1] → BODY.PEEK[1.1]
5) Decode and truncate body safely; log chosen section and len.

---

# BODYSTRUCTURE Tabanlı Gövde Getirme

Hedef
- Gereksiz byte indirmeden, doğru parça(lar)ı seçerek ileti gövdesini hızlı sunmak.

Adımlar
1) SEARCH/UID ile hedef UID setini belirle.
2) FETCH UID BODYSTRUCTURE (ve gerekirse ENVELOPE) al.
3) Seçim algoritması:
   - multipart/alternative: text/plain > text/html (HTML güvenli render fallback)
   - multipart/mixed: ana gövde + ekler (COUNT/Top-N boyut)
   - message/rfc822 içeren durumlarda iç iletinin aynı kuralla işlenmesi
4) Gerekli parçalar için FETCH UID BODY.PEEK[<section>]; ekler istekle indirilir.
5) Charset ve transfer-encoding decode; hatalar metriklenir.

Örnek İstek Dizisi (özet)
- UID SEARCH 123:456
- UID FETCH 123:456 (BODYSTRUCTURE)
- UID FETCH 200 (BODY.PEEK[1])

# FETCH-empty Triage

Belirti
- UID var ama FETCH boş dönüyor.

Triage Adımları
- UIDVALIDITY ve HIGHESTMODSEQ kontrolü; oturumu NOOP ile tazele.
- UID SEARCH ile varlığı doğrula; ESEARCH varsa kullan.
- Hata sınıflandır:
  - IMAP_EMPTY_FETCH: geçici boş yanıt → 3 kez yeniden dene (250ms artan bekleme), sonra NOOP sonrası 1 deneme daha.
  - IMAP_UID_STALE: UID/validity uyuşmazlığı → klasörü yeniden aç.
  - IMAP_PERM: yetki/kalıcı hata → UI’ya eylem önerisiyle aktar.
- Tüm vakaları: istek/yanıt tag’ı, kullanılan UID listesi ve SEARCH ham çıktısını logla.
