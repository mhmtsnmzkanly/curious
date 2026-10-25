# Curious yol haritası

**Durum tarihi: 25 Ekim 2026.** Yol haritası ölçülmüş durumla önerilen işleri
ayırır. Mekanizma testi biyolojik fayda; tek fixture kabulü bütün habitatlarda
denge; hızlandırılmış tick gerçek zamanlı dayanıklılık kanıtı değildir.

## Mevcut durum

- **Kaynak ve canlı sistemler ayrıdır.** V17 su-artığı, av-risk öğrenmesi ve
  V18 genelleme gibi yeni deneysel seçenekler varsayılan kapalıdır; bu, temel
  bellek/su davranışlarının veya canlı V12 genetiğinin tümünün kapalı olduğu
  anlamına gelmez. 7882'deki canlı dünya V12 checkpoint üzerinde,
  yalnız arşiv bütçesi düzeltmesini içeren
  `curious-observation-budget-8dc23de` backportuyla çalışır. Bu dağıtım canlı
  dünyayı V18'e yükseltmedi. 7883 replay ikilisi dondurulmuştur.
- **Coexistence:** Aynı 30 seed, kayıtlı fixture'da `fixed` ve `both`
  kollarında ayrı ayrı 30/30 × 100.000 tick kapısını geçti. Bu yalnız kayıtlı
  koşula bağlı kabul; bütün habitatlarda, her şokta veya kalıcı olarak
  coexistence iddiası değildir.
- **Su öğrenmesi:** V17 artığı varsayılan kapalı. Ön kayıtlı bounded utility
  sonucu **FALSE**; genelleştirilmiş fayda veya dağıtım kabulü yoktur.
- **Görüş/vision:** Habitat karşılaştırması karışık betimleyici sonuç verdi;
  tutarlı avantaj göstermedi.
- **Sığınak hasarı:** Tam kaçış için beklenen hasar `Unknown`. İlk resolver
  aralığı çıkarımı 623 exact gözlem (617 sıfır, altı × 20 hasar, dört birey)
  buldu; bir learner kabul edilmedi. Sonraki tam-rota sansürü tamamlanmış ilk
  aralığın exact etiketini geçersiz kılmaz.
- **İzleyici:** Birey/karar/geçmiş için ilk salt-okunur belgesel arayüzü var;
  bütün biliş veya fayda kanıtı değildir.
- **Uzun soak:** Aylar/yıllar süren gözlem kullanıcı isteğiyle ertelendi; yeni
  özellik geliştirmeyi bloke etmez.

Bağlı kanıtlar: [coexistence kabulü](reports/QUALIFIED_OBSERVATION.md),
[su sonucu](reports/WATER_RESIDUAL_V1.md),
[habitat-vision sonucu](reports/VISION_HABITAT_V1.md),
[tam kaçış baseline'ı](reports/REFUGE_RISK_BASELINE_V1.md),
[ilk-aralık ölçümü](reports/REFUGE_FIRST_RESOLUTION_V1.md) ve
[canlı depolama backportu](reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md).

## Öncelik sırası

Bağımlı işler bu sırayla ilerler; A/G gibi bağımsız ölçüm işleri kendi izinde
paralel hazırlanabilir.

### P0 — destek ve mekanizma

1. İlk-aralık hasarı için per-owner öğrenme sözleşmesini tanımla. Hedef yalnız
   seçilmiş tehdidin bir sonraki tam resolver aralığındaki doğrulanmış hasarıdır;
   tam kaçış hasarı veya seçilmemiş yol riski değildir. Bu ufukta atıfsız,
   eksik ya da sansürlü sonuç `Unknown` kalır; sonradan sansürlenmiş tam kaçış,
   zaten tamamlanmış ilk aralığı geriye dönük bilinmeyen yapmaz. 623 satırlı
   mevcut çıkarım model değildir;
   model eğitimi veya yeni lab bu yol haritası notuyla yetkilendirilmez.
2. Su artığı için güncel miktar/rota/beden tabanı ile eski bireysel artığın
   bağlam uyumunu sınayacak hipotezi tasarla. Mevcut bounded utility başarısız
   olduğundan retune ve yeni seed çalışması ayrı preregistration gerektirir.

### P1 — fiziksel ve muhasebe testleri

3. Resolver testleri: seçilen tehdidin ilk-aralık hasarı, diğer saldırgan,
   açıklanamayan sağlık kaybı, eksik atıf, ölüm/kesinti ve güvenilir sıfır
   birbirinden ayrılır. Gerçek ödenen enerji ve olay tanıkları karşılaştırılır.
4. Sürdürülebilir ecology fixture'ı: kaynak başlangıç/yenilenme/tüketim,
   tür başına akış, beden enerji bütçesi, kurucu/F1/F2 ve kendi yavrusunu
   üretmiş soy ayrı denetlenir. Mevcut 3 × 500 tick fixture sonucu sınırlıdır;
   basal enerji ve trait bedelleri henüz tam doğrulanmış sayılmaz.

### P2 — varsayılan kapalı entegrasyon

5. Yalnız P0/P1 root incelemesinden sonra learning/action kodunu entegre et.
   Her bireyin verisi ayrı kalır; global fallback veya yavruya öğrenilmiş bellek
   aktarımı yoktur. Feature flag, update/freeze kontrolü ve sınırlar açıktır.
6. Checkpoint migration, eski SQLite okuma, resume, writer lock ve hata
   davranışını doğrula. Kapalı özellikte başlangıçtan itibaren tam State+RNG
   eşitliği ve eski kontrol akışı korunmalıdır. Canlı V12 ayrı sahipliktedir;
   yeni ana dal ikilisiyle kendiliğinden yükseltilmez.

### P3 — ecology, trait ve davranış kabulü

7. Fizik ve entegrasyon kapılarından sonra sabitlenmiş fixture/seedlerle habitat,
   kaynak baskısı, bedelli trait ve öğrenme karşılaştırmaları yap. Her deneyin
   kendi protokolü, fiziksel çıktıları, held-out rolleri ve doğal yok oluş
   kuralı olur. Seed seçimi, eşik veya feature tuning formal sonuçtan sonra
   değiştirilmez.

## Yedi yetenek ve sistem izi

| İz | Sınırlandırılmış mevcut durum | Önerilen sonraki iş |
|---|---|---|
| **A — Güncel bağlamlı su artığı** | V17'de mevcut ve varsayılan kapalı. Formal utility **FALSE**; kabul veya canlı dağıtım iddiası yok. | Bağlam eskimesi ve taban skorla artık etkileşimini tanımla; önce saf/deterministik test, sonra ayrı preregistration. |
| **B — Per-owner ilk-aralık hasar öğrenmesi** | 623 exact ilk-aralık kaydı; altı pozitif hasar dört bireyde. Tam kaçış `Unknown`; model yok. | P0'da per-owner hedef/sansür sözleşmesi. Yalnız hedef ufkunda Unknown update etmez; sonraki rota sansürü exact ilk-aralığı silmez. P1'den önce training veya World yok. |
| **C — Kaynak, habitat ve bedelli trait fixture'ı** | Üç 500-tick formal seed kayıtlı fixture viability kapısını geçti. Bu genel habitat/fitness kabulü değil; bazı kaynak ve beden enerji muhasebesi açık. | Kaynak akışı ve istenen/ödenen enerji denetimini tamamla; habitat ve trait sorularını ayrı faktörlere böl. |
| **D — Sonlu birleştirilebilir eylem ve yerel koordinasyon** | Mevcut niyetler/eylemler resolver'dan geçiyor; genişletilebilir katalog ve sosyal koordinasyon kabul edilmedi. | Sonlu primitifler, izinli bileşimler ve yalnız görünür komşu/iletişim olaylarına dayalı yerel koordinasyon tanımla. |
| **E — Sonlu allel varyasyonu ve seçilim** | Bedelli allel/mutasyon mekanizmaları opt-in olarak var. Fitness/kalıcılık avantajı kabul edilmedi; öğrenilmiş bellek kalıtılmaz. | C'nin maliyet/soy muhasebesinden sonra allel frekansı, ödenen bedel ve üreyen yavru-soyu için ayrı held-out test. |
| **F — Bireysel deneyim belgesel arayüzü** | Salt-okunur birey/karar/geçmiş görünümü var; tüm deneyim veya nedensellik iddiası yok. | Görülen/hatırlanan kanıt → seçilen eylem → resolver maliyeti/sonucu → gözlenen öğrenme güncellemesini göster; eksikler `Unknown`. |
| **G — Bounded kapasite, depolama ve işletim** | Shard, checkpoint/arşiv ve retention mevcut. Canlı V12 storage backportu cached arşiv sayımını düzeltir; kota hâlâ soft guard. | Sabit kısa yüklerde actor/tick, RSS, SQLite/HTTP ve disk büyümesini ölç; sınırlarda güvenli pause/retention davranışını doğrula. Aylar/yıllar soak bu izin kabul kapısı değildir. |

## Ortak modül ve kabul kapıları

- **Beden:** sağlık, enerji, dayanıklılık ve fiziksel bedel. Yeni trait maliyeti
  istek ile gerçekten ödeneni ayıran resolver/metabolizma olayında görünür.
- **Algı:** yalnız karar anında görülen durum. Sonraki dünya bilgisi veya gizli
  fizyoloji feature olamaz.
- **Bellek:** bireye ait, sınırlı ve checkpoint'lenmiş. Yavruya öğrenilmiş tablo
  aktarılmaz; bireyler arası update izolasyonu test edilir.
- **Öğrenme:** yalnız resolver doğrulamalı kendi sonuçlarından ve tanımlı hedef
  ufkunda. Atıfsız, ufku içinde kesilmiş veya bilinmeyen sonuç başarı ya da sıfır
  hasar sayılmaz. Daha sonraki rota sansürü tamamlanmış kısa ufuk etiketini silmez.
  Frozen model parametreleri değişmez; online keşif sürebileceğinden frozen ve
  online kolların tüm sonuçları eşit olma şartı yoktur.
- **Resolver:** eylemi/çakışmayı doğrular, bedeli uygular ve kaynak olayı üretir.
  Model dünya durumuna doğrudan yazmaz.
- **Saklama ve kontrol:** yeni alanlar sürümlü migration alır; default-off
  migration, checkpoint resume, tek yazıcı, eski okuma ve hata/pause sınırları
  denetlenir.
- **State+RNG:** disabled-path eski/yeni karşılaştırması aynı fixture'da tam
  durum ve RNG eşitliği; checkpoint-resume ise deterministik devam göstermelidir.
  Şema göçünde yalnız açıkça belgelenmiş sürüm/yeni alan farkları ayrı
  doğrulanabilir; biyolojik durum veya RNG farkı normalizasyonla gizlenmez.
  Frozen parametre değişmezliği ayrıca sınanır, ancak online keşif sürebilir.
  Her değişiklik için root incelemesi ve bağımsız audit gerekir.

## Özgün 11 aşamanın kısa durumu

| # | Aşama | 25 Ekim durumu |
|---:|---|---|
| 1 | Avcı yavrusu ve soyunun devamı | Üreme/olgunlaşma mekanizması ve sınırlı coexistence kapısı var; bütün koşullarda kalıcılık kanıtlanmadı. |
| 2 | Av seçimi, maliyet, bırakma ve dinlenme | İlk kurallı politika mevcut; risk öğrenmesi fayda göstermedi. |
| 3 | Karma ekosistem | Kayıtlı fixture'da bounded coexistence kabulü var; genel habitat sonucu yok. |
| 4 | Hareket, duvar, sığınak, diyet, paylaşım ve çakışma | İlk resolver sözleşmeleri mevcut ve testli. |
| 5 | Sınırlı bireysel kaynak/tehlike belleği | İlk bounded bellek ve unutma mekanizmaları mevcut; güvenli aktarım genellemesi kanıtlanmadı. |
| 6 | Sonuçtan su öğrenmesi ve kontrol kolu | Mekanizma/formal ölçüm var; V17 bounded utility FALSE. |
| 7 | Bedelli kalıtsal hareket/trait ve mutasyon | İlk allel/bedel ölçümü var; seçilim avantajı kabul edilmedi. |
| 8 | Birey, karar, geçmiş, bellek ve soy arayüzü | İlk salt-okunur belgesel ekran mevcut; bilişin tam kaydı değildir. |
| 9 | Gerçek gün/hafta dayanıklılığı | Sınırlı kayıtlı tick testleri var; uzun gerçek-zamanlı soak kullanıcı isteğiyle ertelendi. |
| 10 | Kurucu ve harita ölçek ölçümü | 10–1.000 kurucu ve farklı haritalar için ilk ölçüm araçları mevcut. |
| 11 | İzleme, yedekli dağıtım ve bildirim | Ayrı işletim/monitoring mevcut; uzun soak ve genel kapasite garantisi değildir. |

Yeni seed, lab, model eğitimi, live deployment veya ortak çekirdek entegrasyonu
bu yol haritasıyla otomatik olarak yetkilendirilmez; ayrı root review ve
protokol gerekir. Belgelerin güncel giriş noktaları [docs/README](README.md)
içindedir.
