# Yazılım Mühendisliği Bitirme Projesi Final Raporu

## 📄 Kapak ve Başlangıç Sayfaları
**Proje İsmi:** Mailora - Yapay Zeka Destekli, Yerel Çalışan E-posta ve İşbirliği Platformu
**Takım Üyeleri:** Cenk Orfa, Emirhan Yavuz
**Danışman:** Dr. Öğr. Üyesi Seda Kılıçer

**İÇİNDEKİLER**
(Bu bölüm kelime işlemci programında otomatik oluşturulacaktır)

**Şekiller Listesi**
(Bu bölüm kelime işlemci programında otomatik oluşturulacaktır)

**Resim Listesi**
1. Resim 1: Kullanıcı Giriş ve Hesap Ekleme Ekranı
2. Resim 2: Birleşik Gelen Kutusu (Unified Inbox) ve Temel E-posta Arayüzü
3. Resim 3: Yapay Zeka (AI) Özetleme ve Akıllı Cevap Analizi Ekranı
4. Resim 4: Performans, Duygu Analizi Grafikleri ve Yönetici (Admin) Paneli

**Kısaltma Listesi**
- **IMAP:** Internet Message Access Protocol
- **SMTP:** Simple Mail Transfer Protocol
- **NLP:** Natural Language Processing (Doğal Dil İşleme)
- **RBAC:** Role-Based Access Control (Rol Tabanlı Erişim Kontrolü)
- **API:** Application Programming Interface
- **MVP:** Minimum Viable Product (Minimum Uygulanabilir Ürün)
- **PIM:** Personal Information Management (Kişisel Bilgi Yönetimi)

---

## 📌 Özet
Mailora, modern dijital iletişimde kullanıcı gizliliğini ve veri güvenliğini merkeze alan, Rust programlama diliyle geliştirilmiş yüksek performanslı bir masaüstü e-posta istemcisidir. Projenin temel amacı, mevcut e-posta çözümlerinin hantallığını ve veri işleme açıklarını gidererek kullanıcılara bellek güvenliği (memory safety) standartlarına uygun, hızlı ve tamamen yerel bir iletişim ortamı sunmaktır. Bu motivasyonla yola çıkarak; düşük seviyeli MIME veri ayrıştırma, optimize edilmiş yerel veritabanı (SQLite) yönetimi ve ağ protokollerinin Rust’ın eşzamanlılık (concurrency) avantajlarından faydalanarak yapılandırılması hedeflenmiştir. 

Geliştirme sürecinin sonunda; yüksek performanslı, üçüncü parti bulut sunuculara veri sızdırmadan kendi donanımınızda e-posta özetleme ve dil çevirisi yapabilen yerel NLP (Doğal Dil İşleme) modellerine sahip kurumsal bir iletişim platformu elde edilmiştir. Elde edilen bu mimari, kurum verilerinin donanım sınırları içerisinde kalmasını garanti altına alarak siber veri sızıntılarını imkansız hale getirmekte ve gizliliğe önem veren işletmeler için sıfır maliyetli bir standart oluşturmaktadır.

---

## 1. GİRİŞ

### 1.1 Birleşmiş Milletler Sürdürülebilir Kalkınma Amaçları (SKA)
Mailora projesi, tasarım ve mimari yapısı gereği Birleşmiş Milletler Sürdürülebilir Kalkınma Amaçları'ndan iki temel hedefe doğrudan katkı sağlamaktadır:
- **Hedef 9 (Sanayi, Yenilikçilik ve Altyapı):** Kurumların kendi donanımları üzerinde (local-first) çalışabilen, bulut aboneliklerinden bağımsız, yenilikçi bir iletişim altyapısı sunması bakımından Hedef 9'u destekler.
- **Hedef 12 (Sorumlu Tüketim ve Üretim):** Geleneksel bulut tabanlı devasa veri merkezlerinin harcadığı enerji ve karbon ayak izini ortadan kaldırmak amacıyla, Rust programlama dilinin getirdiği düşük CPU/RAM tüketim avantajı kullanılarak son kullanıcı cihazlarında çevreci bir veri işleme modeli hedeflenmiştir.

### 1.2 Gerçekçi Koşullar ve Kısıtlar
Projenin gerçek hayattaki uygulanabilirliği temel koşullar altında değerlendirilmiştir:
- **Teknik ve Ekonomik Koşullar:** Tamamen açık kaynak teknolojiler kullanıldığı için yüksek kurumsal lisanslama ve sunucu maliyetleri sıfıra indirilmiştir. Ancak yapay zeka entegrasyonlarının cihazda sorunsuz çalışabilmesi için kurumsal makinelerde minimum donanım standartları gerekmektedir.
- **Operasyonel Koşullar:** Uygulamanın içerisinde barındırdığı entegre "Tablolar" ve "Takvim" modülleri sayesinde ofis çalışanlarının odak kaybı önlenmekte ve çalışma esnasında bağlam (context) değiştirme ihtiyacı ortadan kalkmaktadır.

### 1.3 Bitirme Çalışmasından Sağlanan Bilgi, Beceriler
Bu proje geliştirme süreci boyunca ekip üyeleri endüstriyel standartlarda kritik beceriler kazanmıştır. Rust programlama dilinde asenkron ağ programlama (Tokio, Axum) yetenekleri geliştirilmiş, veritabanı şema tasarımı (SQLite, SQLx) ve veri tutarlılığı sağlanmıştır. Ayrıca Python üzerinden lokal makine öğrenmesi (Machine Learning) modellerinin bir web servisine API olarak entegre edilmesi ve kurumsal uygulamalarda hayati öneme sahip RBAC (Rol Tabanlı Erişim Kontrolü) güvenlik mimarisi başarıyla uygulanmıştır.

### 1.4 Genel Bilgiler (Problemin Tanımı ve Amacı)
Günümüzde e-posta iletişimi en yoğun veri trafiğine sahip alanlardan biridir. Ancak mevcut e-posta istemcilerinin çoğu ya yüksek sistem kaynağı tüketen (RAM/CPU) hantal yapılara sahiptir ya da kullanıcı verilerini işlemek için bulut tabanlı sistemlere ihtiyaç duyarak gizlilik riskleri oluşturmaktadır. Özellikle yoğun ileti trafiği altında e-postaların manuel olarak sınıflandırılması ve önemli bilgilerin ayıklanması, kullanıcılar için ciddi bir zaman kaybına yol açmaktadır.

Mailora’nın temel amacı; Rust programlama dilinin sunduğu performans avantajlarını kullanarak, tamamen yerel çalışan, gizlilik odaklı ve yapay zeka destekli bir e-posta istemcisi geliştirmektir. Proje, kullanıcının e-postalarını harici bir sunucuya göndermeden makine öğrenimi modelleriyle özetlemesini ve kategorize etmesini hedeflemektedir. Veri egemenliğini (data sovereignty) doğrudan kullanıcıya teslim eden bu yaklaşım, uygulamanın en büyük inovasyonudur.

---

## 2. MEVCUT UYGULAMA VE ÇALIŞMALAR

### 2.1 Benzeri Çalışmalar, Literatür ve Karşılaştırma
Piyasadaki Outlook, Thunderbird, Spark gibi uygulamalar incelenmiş, e-posta iletişiminin temel standartları olan SMTP (RFC 821) ve IMAP4rev1 (RFC 3501) protokolleri literatürden araştırılmıştır. Outlook kapalı kaynak yapısı ve yüksek sistem gereksinimleriyle, Thunderbird ise nispeten eski teknoloji yığınıyla dikkat çekmektedir. Yeni nesil AI destekli e-posta istemcileri ise verileri bulut sunucularında (Örn. OpenAI API'leri üzerinden) işlemektedir. Mailora, yapay zeka işlemlerini (Helsinki-NLP, Naive Bayes vb.) cihaz içinde (on-premise) çözerek mevcut uygulamalara kıyasla hem gizlilik kalkanı oluşturmakta hem de veri işleme sürecini hızlandırmaktadır.

### 2.2 Ekonomik Yapılabilirlik
Mailora'nın bulut faturası veya kurumsal abonelik sistemi yoktur. İhtiyaç duyulan tüm sunucu altyapısı kullanıcının kendi yerel makinesinde (localhost) barındırılır. Veritabanı olarak SQLite tercih edilmesi lisanslama maliyetlerini sıfırlamıştır.

### 2.3 Çevresel Etki
Uygulama, ağ üzerinden sürekli olarak dış API'lere istek atmak yerine, yerel cihazın atıl (idle) kapasitesini kullanır. Rust tabanlı çekirdek sunucusunun sadece birkaç megabayt RAM tüketmesi, devasa bulut veri merkezlerinin aksine düşük karbon ayak izi üreten sürdürülebilir bir yazılım (Green IT) altyapısına sahiptir.

### 2.4 Kullanıcı Kitlesi
Mailora öncelikli olarak şu kullanıcı gruplarını hedefler:
- Veri mahremiyetine yasal zorunluluklarla önem vermek zorunda olan hukuk firmaları ve sağlık kuruluşları.
- Yoğun iletişim trafiğini hızlıca özetleyip kategorize etmek isteyen KOBİ'ler ve yöneticiler.
- Electron tabanlı hantal istemcilerden şikayetçi olan ve düşük kaynak tüketimi arayan geliştiriciler.

### 2.5 Ölçeklenebilirlik
Mailora'nın temel mimarisi olan Rust ve asenkron motoru (Tokio), yüksek eşzamanlı ağ bağlantılarını verimli yönetir. İlerleyen aşamalarda kurumsal talepler arttığında, SQLite veritabanı yapısı rahatlıkla PostgreSQL gibi daha büyük veritabanı sunucularına geçiş yapabilecek şekilde (SQLx kütüphanesi sayesinde) soyutlanmıştır.

### 2.6 Teknik ve Operasyonel Kısıtlar
Lokal Yapay Zeka çalıştırmanın getirdiği en büyük donanımsal kısıt, dil modellerini belleğe (RAM/VRAM) yüklerken ihtiyaç duyulan yüksek bellektir. Operasyonel anlamda ise kurum içi katı Güvenlik Duvarı (Firewall) engellemeleri, IMAP (993) ve SMTP (587) senkronizasyonlarında engel teşkil edebilmektedir.

---

## 3. GELİŞTİRME SÜRECİ VE MİMARİ

### 3.1 Yapay Zeka (YZ) Araçlarının Kullanımı
Projenin geliştirme sürecinde etik ve şeffaf bir yapay zeka politikası izlenmiştir:
- **Kod Geliştirme:** Rust mimarisindeki derleme (borrow-checker) hatalarının çözümü ve arayüz bileşenlerinin (Örn: Tablolar ızgara sistemi) prototiplenmesi süreçlerinde yapay zeka kod asistanlarından mentörlük alınmış ve kodlar projeye özelleştirilerek entegre edilmiştir.
- **Proje Altyapısı Olarak YZ:** Sistemin temel özelliklerinden biri olan yerel metin özetleme, çeviri ve duygu analizi işlemleri için Naive Bayes, SVM algoritmaları ve açık kaynaklı HuggingFace modelleri projeye manuel olarak yerleştirilmiştir.

### 3.2 Proje Ekibinin Takım Yapısı ve İş Bölümü
Proje, iki kişilik çevik (agile) bir takım yapısıyla yürütülmüştür:
- **Cenk Orfa (Takım Kaptanı):** Backend (Rust/Axum) sistem mimarisi, IMAP/SMTP asenkron ağ katmanı yönetimi, Python yapay zeka model entegrasyonu ve proje veri senkronizasyonu.
- **Emirhan Yavuz:** Frontend arayüz tasarımı, Vanilla JS etkileşimleri, "Tablolar" ve "Takvim" modüllerinin inşası, SQLite veritabanı şema kurgusu ve RBAC paneli entegrasyonu.

### 3.3 Sistem Tasarımı ve Kullanılan Teknolojiler
Mailora'nın teknik altyapısı, güvenliği ve hızı odağına alan "Yerel Öncelikli" (Local-First) bir mimariye sahiptir:
- **Backend (Rust):** Uygulamanın kalbini oluşturur. Asenkron I/O işlemleri için **Axum** ve **Tokio** framework'leri, IMAP/SMTP ağ işlemleri için **async-imap** ve **lettre** kütüphaneleri kullanılmıştır. Rust bellek yönetimi, ağdan gelen güvensiz verilerin (zararlı e-postalar) parse edilmesinde kritik güvenlik sağlar.
- **Veritabanı (SQLite):** SQLx kütüphanesi kullanılarak, e-postalar, başlıklar ve dosyalar hiyerarşik (Bütün-Parça / Composition) ilişkisi içerisinde tutulur. Kullanıcı hesabı silindiğinde tüm mailler "Cascade" kurallarıyla cihazdan temizlenir.
- **Yapay Zeka (Python/Flask):** Doğal dil modelleri yüksek RAM ihtiyacı sebebiyle, izole edilmiş bir yerel Python API (MailoraPro) sunucusu üzerinde çalışır.
- **Frontend (Vanilla JS):** Arayüz hiçbir ağır kütüphane (React, Vue) barındırmaz. Saf HTML/CSS/JS kullanılarak Chart.js tabanlı Admin Dashboard, Tablolar Modülü ve Magic Login özellikli giriş kısımları tasarlanmıştır.

### 3.4 Karşılaşılan Problemler ve Çözümler
Geliştirme sürecinde iletişim protokollerinin doğasından kaynaklı bazı darboğazlar yaşanmış ve teknik çözümler üretilmiştir:
- **Gmail UID Gecikmesi:** Gönderilen iletilerin Gmail tarafından dizine (Sent klasörüne) işlenmesi sırasında yaşanan UID eşleşme hatalarını çözmek için, arka planda 60 saniyelik periyotlarla çalışan bir **"Backoff-Retry" (Geri Çekilme ve Deneme)** kuyruk algoritması geliştirilmiştir.
- **Mimari Karmaşa:** Proje başlarında e-posta trafigini yönlendirmek için "Stalwart Proxy" mimarisi tasarlanmış, ancak bu durumun gereksiz RAM tüketimi yarattığı fark edilerek "Direct IMAP" mantığına geri dönülmüş ve verimlilik artırılmıştır.
- **Mojibake (Bozuk Karakter) Sorunları:** Eski Türkçe sunuculardan gelen RFC 2047 formatındaki sorunlu UTF-8 verilerini çözümlemek için özel bir tahmin (fallback) fonksiyonu yazılarak karakter bozulmaları engellenmiştir.

---

## 4. TEST SÜREÇLERİ, SONUÇLAR VE TARTIŞMA

### 4.1 Test Senaryoları
Sistemin kararlılığı farklı entegrasyon testleriyle doğrulanmıştır:
- **Auto-Discovery Senaryosu:** Sisteme sadece e-posta girildiğinde IMAP portlarının (993) ve SMTP portlarının (587) Mozilla ISPDB veri havuzuyla otomatik keşfedilip bağlanabildiği test edilmiş ve "Sıfır-Kurulum" felsefesi kanıtlanmıştır.
- **Ağ Kopması (Circuit Breaker):** İnternet aniden koptuğunda asenkron Tokio thread'lerinin donmasını engellemek için tasarlanan "Devre Kesici" sistemi test edilmiş; kilitlenmeden hata logu oluşturma başarısı gözlenmiştir.

### 4.2 Sonuçlar ve Tartışma
Elde edilen sonuçlar, "bulut şirketlerine veri sızdırmadan çalışan yapay zeka destekli yerel iletişim istemcisi" vizyonunun pratikte uygulanabilir olduğunu açıkça ortaya koymuştur. Mailora, IMAP sunucularından çektiği verileri internete ihtiyaç duymadan (sadece localhost üzerinde) özetleyebilmiş ve analiz edebilmiştir. Rust programlama dilinin seçimi, ağ yoğunluğu yaşandığı durumlarda düşük sistem tüketimi vaadini yerine getirmiştir.

Gelecek çalışmalar (vizyon) kapsamında; SQLite FTS5 modülü aracılığıyla binlerce veri içinde saliselerle arama (Full-Text Search) yapabilen motorun devreye alınması, ağ kesintilerine karşı mailleri bekletip tekrar gönderen "Kalıcı Kuyruk (Outbox)" sisteminin olgunlaştırılması ve AI modellerinin C/C++ tabanlı GGUF formatına küçültülerek çok daha düşük donanımlı eski ofis bilgisayarlarında da sorunsuz çalıştırılabilmesi planlanmaktadır.

---

## 5. KAYNAKLAR
1. Rust Foundation. (2025). *The Rust Programming Language Documentation*. https://doc.rust-lang.org/
2. Hugging Face. (2025). *Transformers and Local NLP Models*. https://huggingface.co/docs
3. Internet Engineering Task Force (IETF). (2003). *RFC 3501: INTERNET MESSAGE ACCESS PROTOCOL - VERSION 4rev1*. 
4. Internet Engineering Task Force (IETF). (1982). *RFC 821: SIMPLE MAIL TRANSFER PROTOCOL*.
5. Axum Web Framework. (2025). *Ergonomic and modular web framework built with Tokio*.
6. SQLite Consortium. (2025). *SQLite Database Engine Documentation*.
