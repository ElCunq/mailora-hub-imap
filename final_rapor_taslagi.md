# Yazılım Mühendisliği Bitirme Projesi Final Raporu

## 📄 Kapak ve Başlangıç Sayfaları
**Proje İsmi:** Mailora - Yapay Zeka Destekli, Yerel Çalışan E-posta ve İşbirliği Platformu
**Takım Üyeleri:** Cenk Orfa, Emirhan Yavuz

**İÇİNDEKİLER**
(Bu bölüm kelime işlemci programında otomatik oluşturulacaktır)

**Şekiller Listesi**
(Bu bölüm kelime işlemci programında otomatik oluşturulacaktır)

**Resim Listesi**
1. Resim 1: Birleşik Gelen Kutusu (Inbox) ve Temel E-posta Arayüzü
2. Resim 2: Yapay Zeka Destekli E-posta Özetleme ve Duygu Analizi Ekranı
3. Resim 3: Entegre Tablolar (Sheets) Modülü
4. Resim 4: Admin Paneli, Sistem Performansı ve RBAC Yetkilendirme

**Kısaltma Listesi**
- **IMAP:** Internet Message Access Protocol
- **SMTP:** Simple Mail Transfer Protocol
- **NLP:** Natural Language Processing (Doğal Dil İşleme)
- **RBAC:** Role-Based Access Control (Rol Tabanlı Erişim Kontrolü)
- **API:** Application Programming Interface

---

## 📌 Özet
Bu projede, veri gizliliğini temel alan, bulut bağımlılığını ortadan kaldıran ve entegre ofis araçlarına (Tablolar, Takvim) sahip olan yeni nesil, yapay zeka destekli bir yerel e-posta istemcisi (Mailora) geliştirilmiştir. Günümüzde standart hale gelen Elektron tabanlı hantal uygulamaların yüksek bellek tüketimi ve bulut tabanlı e-posta servislerinin (Gmail, Outlook vb.) kullanıcı verilerini analiz etmesi, kurumsal gizlilik açısından ciddi güvenlik riskleri doğurmaktadır. Mailora, bu motivasyonla yola çıkarak; arka planda sistem kaynaklarını minimum seviyede kullanan **Rust (Axum)** dilinde geliştirilmiş, veritabanı olarak **SQLite** kullanan ve e-posta içeriklerini tamamen kullanıcının kendi donanımında analiz edebilmek için **yerel NLP modelleri** (MT5-Small, BERT) ile entegre edilmiştir. 

Geliştirme sürecinin sonunda; yüksek performanslı, üçüncü parti sunuculara veri sızdırmadan kendi cihazınızda e-posta özetleme, yabancı dil çevirisi ve konu analizi yapabilen, aynı zamanda rol tabanlı erişim kontrolüne (RBAC) sahip kurumsal bir iletişim platformu elde edilmiştir. Elde edilen bu sistem mimarisi, kurumsal verilerin donanım sınırları içerisinde kalmasını sağlayarak veri sızıntılarını imkansız hale getirmekte ve KOBİ'ler başta olmak üzere gizliliğe önem veren tüm yapılar için sıfır maliyetli bir haberleşme standardı sunabileceğini kanıtlamaktadır.

---

## 1. GİRİŞ

### 1.1 Birleşmiş Milletler Sürdürülebilir Kalkınma Amaçları (SKA)
Mailora projesi, tasarım ve mimari yapısı gereği Birleşmiş Milletler Sürdürülebilir Kalkınma Amaçları'ndan iki temel hedefe doğrudan katkı sağlamaktadır:
- **Hedef 9 (Sanayi, Yenilikçilik ve Altyapı):** Kurumların kendi donanımları üzerinde (local-first) çalışabilen, bulut aboneliklerinden bağımsız, esnek ve yenilikçi bir iletişim altyapısı sunmaktadır.
- **Hedef 12 (Sorumlu Tüketim ve Üretim):** Geleneksel bulut tabanlı devasa veri merkezlerinin harcadığı devasa enerji ve karbon ayak izini ortadan kaldırmak amacıyla, Rust programlama dilinin getirdiği bellek güvenliği ve düşük CPU/RAM tüketim avantajı kullanılarak son kullanıcı cihazlarında "çevreci" bir veri işleme modeli (Green Computing) hedeflenmiştir.

### 1.2 Gerçekçi Koşullar ve Kısıtlar
Projenin gerçek hayattaki uygulanabilirliği üç temel koşulda değerlendirilmiştir:
- **Teknik ve Ekonomik:** Tamamen açık kaynak teknolojiler (Rust, SQLite, HuggingFace modelleri) kullanıldığı için, kurumsal firmaların karşılaştığı yüksek "kullanıcı başı aylık e-posta lisanslama" ve sunucu maliyetleri sıfıra indirilmiştir.
- **Operasyonel:** E-posta okurken uygulamadan hiç ayrılmadan entegre "Tablolar (Sheets)" ve "Takvim" modüllerine geçiş yapılabilmesi, ofis çalışanlarının odak kaybını önlemekte ve operasyonel verimliliği artırmaktadır.

### 1.3 Bitirme Çalışmasından Sağlanan Bilgi, Beceriler
Bu proje geliştirme süreci boyunca ekip üyeleri akademik ve endüstriyel standartlarda kritik beceriler kazanmıştır. Rust programlama dilinde bellek güvenliği (memory-safety) ve eşzamanlı (concurrent) ağ programlama (Axum Framework) konularında uzmanlaşılmıştır. Bunun yanı sıra Python üzerinden lokal yapay zeka modellerinin (NLP) bir web servisine API olarak entegre edilmesi, Vanilla JavaScript ile reaktif (framework-less) yüksek performanslı arayüz tasarımı ve kurumsal uygulamalarda hayati öneme sahip RBAC (Rol Tabanlı Erişim Kontrolü) mimarisi başarıyla öğrenilip uygulanmıştır.

### 1.4 Genel Bilgiler
Günümüz dijital iş dünyasında haberleşmenin kalbi e-postalardır. Ancak mevcut sistemlerin çoğunluğu hantal, yüksek RAM tüketen (Electron tabanlı) veya verileri reklam/analiz amacıyla işleyen bulut tabanlı platformlardan oluşmaktadır. Mailora projesinin temel problemi, "Performans ve Gizlilik" ikilemini ortadan kaldırmaktır. Projenin amacı; kullanıcıların tüm ofis süreçlerini tek bir sekmeden yönetebileceği, hızlı, güvenli ve yapay zeka ile metin analizi (özetleme, duygu analizi, otomatik yanıt) yapabilen ama bu işlemleri yaparken veriyi asla internete göndermeyen yerel bir istemci yaratmaktır. Kurumsal şirket verilerinin ve kişisel yazışmaların siber casusluktan ve izinsiz veri işleme algoritmalarından korunması, projenin endüstriyel önemini vurgulamaktadır.

---

## 2. MEVCUT UYGULAMA VE ÇALIŞMALAR

### 2.1 Benzeri Çalışmalar ve Karşılaştırma
Piyasadaki Outlook, Thunderbird, Spark gibi mevcut uygulamalar analiz edildiğinde; Outlook kapalı kaynak yapısı ve yüksek sistem gereksinimleriyle, Thunderbird ise eski teknoloji yığını ve entegre yapay zeka eksikliğiyle dikkat çekmektedir. Yeni nesil AI destekli e-posta istemcileri ise (örn. Spark AI) verileri kendi bulut sunucularına göndererek işlemektedir. Mailora, yapay zeka işlemlerini (Helsinki-NLP vb.) cihaz içinde (on-premise) çözerek literatürdeki diğer uygulamalara göre benzersiz bir gizlilik kalkanı ve Rust diliyle sağlanan hız avantajı sunmaktadır.

### 2.2 Ekonomik Yapılabilirlik
Mailora'nın herhangi bir bulut faturası veya abonelik sistemi yoktur. İhtiyaç duyulan tüm sunucu altyapısı (Rust ve Python API), kullanıcının/kurumun kendi yerel makinesinde barındırılabilir. Veritabanı olarak SQLite tercih edilmesi, harici bir veritabanı lisanslama veya bakım maliyetini tamamen sıfırlamıştır.

### 2.3 Çevresel Etki
Uygulama, ağ üzerinden sürekli olarak dış API'lere (OpenAI, DeepL vb.) istek atmak yerine, yerel cihazın atıl kapasitesini kullanarak işlem yapar. Rust tabanlı arka plan (backend) sunucusunun sadece birkaç megabayt RAM tüketmesi ve CPU'yu neredeyse hiç yormaması, sunucu tarafındaki enerji tüketimini dramatik bir şekilde düşürerek çevreci (green IT) prensiplerine uymaktadır.

### 2.4 Kullanıcı Kitlesi
Uygulamanın hedef kitlesi; öncelikli olarak veri mahremiyetine yasal zorunluluklarla veya kurumsal politikalarla önem vermek zorunda olan hukuk firmaları, sağlık kuruluşları, siber güvenlik geliştiricileri ve bağımsız ofis çalışanlarıdır. Sistemin ticari entegrasyonu adına başlangıçta 50 pilot KOBİ kullanıcısı üzerinden test ve adaptasyon süreçlerinin yürütülmesi hedeflenmektedir.

### 2.5 Ölçeklenebilirlik
Mailora'nın temel arka plan mimarisi olan Rust ve Axum, yüksek eşzamanlı (highly-concurrent) bağlantıları son derece verimli bir şekilde yönetir. İlerleyen aşamalarda SQLite yerine PostgreSQL geçişi yapıldığında, mevcut Rust kod mimarisi hiçbir değişikliğe uğramadan binlerce kullanıcının eşzamanlı e-posta trafiğini darboğaz oluşturmadan sırtlayabilecek şekilde (asenkron thread havuzları ile) tasarlanmıştır.

### 2.6 Teknik ve Operasyonel Kısıtlar
Lokal Yapay Zeka çalıştırmanın getirdiği en büyük donanımsal kısıt, Python AI sunucusunun (MailoraPro) dil modellerini belleğe (RAM/VRAM) yüklerken ihtiyaç duyduğu yüksek kapasitedir (minimum 4-8GB boş RAM). Operasyonel anlamda ise kurumsal IMAP/SMTP sunucularındaki katı Güvenlik Duvarı (Firewall) engellemeleri, e-posta senkronizasyonu sırasında ekstra ağ yapılandırması gerektirebilmektedir.

---

## 3. GELİŞTİRME SÜRECİ: Kullanılan Teknolojiler, Araçlar ve Teknikler

### 3.1 Yapay Zeka (YZ) Araçlarının Kullanımı
Bu projenin geliştirme sürecinde etik ve şeffaf bir yapay zeka kullanım politikası benimsenmiştir:
- **Kod Geliştirme Süreci:** Rust backend mimarisinde karşılaşılan zorlu derleme hatalarının (borrow checker errors) çözümü, karmaşık asenkron fonksiyonların yazımı ve JavaScript tarafındaki arayüz bileşenlerinin (Örn: Tablolar grid sistemi) hızlıca prototiplenmesi süreçlerinde ChatGPT ve GitHub Copilot gibi AI kod asistanlarından destek alınmış; bu kodlar proje mimarisine özelleştirilerek entegre edilmiştir.
- **Proje Altyapısı Olarak YZ:** Projenin sunduğu temel özellik olan yerel metin analizi ve çeviri için, açık kaynaklı HuggingFace modelleri (MT5-Small, Helsinki-NLP, BERT) altyapı olarak projeye manuel entegre edilmiştir.

### 3.2 Proje Ekibinin Takım Yapısı ve İş Bölümü
Proje, iki kişilik çevik (agile) bir takım yapısıyla paralel geliştirme modeli üzerinden yürütülmüştür:
| Takım Üyesi | Üstlendiği Roller ve Modüller |
| :--- | :--- |
| **Cenk Orfa** | Backend (Rust/Axum) sistem mimarisi, IMAP/SMTP asenkron bağlantı protokolleri, Python AI Model entegrasyonu, Takım Kaptanlığı ve proje koordinasyonu. |
| **Emirhan Yavuz** | Frontend arayüz tasarımı ve Vanilla JS etkileşimleri, "Tablolar" ve "Takvim" modüllerinin geliştirilmesi, SQLite veritabanı şema yönetimi ve RBAC paneli. |

### 3.3 Çalışmanın Akışı ve Mimari
Mailora, istemci-sunucu (client-server) tabanlı hibrit bir masaüstü/web uygulamasıdır. 
1. **Veri Katmanı:** Tüm e-posta başlıkları, kullanıcı yetkileri ve tablo/takvim verileri yerel SQLite üzerinde şifrelenmiş olarak tutulur.
2. **Backend Katmanı (Rust):** Uygulamanın kalbini oluşturan bu katman, kullanıcının uzak e-posta sunucusuna IMAP üzerinden bağlanarak verileri çeker, asenkron olarak veritabanına yazar ve istemciye (frontend) RESTful API ile sunar. 
3. **Yapay Zeka Katmanı (Python):** Modellerin yüksek boyutları ve çalışma kısıtları nedeniyle, yapay zeka işlemleri ayrı bir lokal Python Flask API'sine izole edilmiştir.
4. **İstemci Katmanı (Frontend):** Hiçbir framework (React, Vue vb.) kullanılmadan, tamamen Saf (Vanilla) CSS ve JS ile ultra hafif bir Gelen Kutusu (Inbox), Excel benzeri çalışan Tablolar ve detaylı Admin yetkilendirme paneli tasarlanmıştır. Proje raporunda yer alan ekran görüntüleri (Gelen Kutusu, AI Özetleme, Tablolar, Admin RBAC Paneli) bu hibrit yapının eksiksiz çalıştığını göstermektedir.

---

## 4. SONUÇLAR VE TARTIŞMA

**Nasıl bir sonuca ulaşıldı?**
Projenin temel amacı olan "bulut şirketlerine veri sızdırmadan çalışan yapay zeka destekli yerel iletişim istemcisi" hedefine başarıyla ulaşılmıştır. Mailora, IMAP sunucularından çektiği verileri, internet bağlantısına ihtiyaç duymadan (sadece localhost üzerinden) kendi içerisinde özetleyebilmekte ve çevirebilmektedir. Ayrıca entegre ofis araçlarıyla birleşik bir deneyim sunmaktadır.

**Bilimsel ve pratik anlamı nedir?**
Yapay zekanın son kullanıcı cihazlarında makul süreler içerisinde ve verimli şekilde (Rust aracılığıyla koordine edilerek) çalıştırılabileceğinin kanıtlanması, ticari ve kurumsal veri gizliliği açısından devrim niteliğindedir. Özellikle veri sızıntı cezalarının (KVKK, GDPR) çok ağır olduğu günümüzde, kurumların şirket içi iletişim verilerini üçüncü parti sunuculara göndermeden analiz edebilmesi büyük bir pratiklik sağlamaktadır.

**Eksiklikler ve Gelecek Vizyonu**
Geliştirilen sistemin en belirgin eksikliği, yeni gelen e-postaları anında ekrana düşürmek için gereken "IMAP IDLE" ve WebSocket (push notification) yapısının henüz tam senkronize olmamasıdır. Gelecek çalışmalarda, anlık bildirim altyapısının iyileştirilmesi ve Python tabanlı yerel AI modellerinin yerine "GGML/GGUF" formatında, doğrudan C/Rust üzerinden bellekte çok daha az yer kaplayarak çalışan kuantize edilmiş (quantized) mikro modellerin sisteme entegre edilmesi planlanmaktadır.

---

## 5. KAYNAKLAR
1. Rust Foundation. (2025). *The Rust Programming Language Documentation*. https://doc.rust-lang.org/
2. Hugging Face. (2025). *Transformers and Local NLP Models: MT5 & Helsinki-NLP*. https://huggingface.co/docs
3. Internet Engineering Task Force (IETF). (2003). *RFC 3501: INTERNET MESSAGE ACCESS PROTOCOL - VERSION 4rev1*. 
4. Axum Web Framework. (2025). *Ergonomic and modular web framework built with Tokio, Tower, and Hyper*.
5. SQLite Consortium. (2025). *SQLite Database Engine Documentation*.
