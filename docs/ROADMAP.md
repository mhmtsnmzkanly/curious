# Curious yol haritası

**Durum tarihi: 25 Ekim 2026.** Aşağıdaki sekiz kategori güncel durum, açık
sınırlar, önerilen çalışma ve kabul ölçütünü ayırır. Öneriler uygulandıkları
anlamına gelmez. P0–P3 çalışma sırası kategori yapısından ayrıdır.

## Sistem durumu

- **Canlı ve ana kaynak ayrıdır:** 7882'deki dünya V12 checkpoint üzerinde,
  yalnız arşiv bütçesi backportu `curious-observation-budget-8dc23de` ile
  çalışıyor. V18 ana kaynak canlıya yükseltilmedi. V17 su-artığı, av-risk
  öğrenmesi ve V18 genelleme gibi yeni deneysel seçenekler varsayılan kapalı;
  bu, temel bellek/su davranışları veya canlı V12 genetiğinin tümünün kapalı
  olduğu anlamına gelmez. 7883 replay ikilisi dondurulmuştur.
- Kayıtlı habitat fixture'ında aynı 30 seed'in `fixed` ve `both` kolları ayrı
  ayrı 30/30 × 100.000 tick kapısını geçti. Bu yalnız o fixture için sınırlı
  coexistence kabulüdür; tüm habitatlar, şoklar veya kalıcılık için genelleme
  değildir.
- V17 su-artığı için kayıtlı bounded utility **FALSE**; habitat-vizyon sonucu
  karışık betimleyici kanıt ve tutarlı avantaj yok. Tam kaçış beklenen hasarı
  `Unknown`. Ayrı ilk-resolver aralığı çıkarımı 623 exact sonuç içerir: 617
  sıfır, altı × 20 hasar, dört pozitif birey. Sonraki tam-rota sansürü
  tamamlanmış ilk-aralık etiketini silmez; bu veri kabul edilmiş learner ya da
  full-route predictor değildir.
- Aylar/yıllar süren gerçek-zamanlı soak kullanıcı isteğiyle ertelendi; sonraki
  özelliğin önkoşulu değildir.

Kanıt: [coexistence](reports/QUALIFIED_OBSERVATION.md),
[su-artığı](reports/WATER_RESIDUAL_V1.md),
[habitat vision](reports/VISION_HABITAT_V1.md),
[tam kaçış baseline'ı](reports/REFUGE_RISK_BASELINE_V1.md),
[ilk aralık](reports/REFUGE_FIRST_RESOLUTION_V1.md),
[canlı depolama backportu](reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md).

## Sekiz sistem kategorisi

Her özellik için birincil sahip kategori belirtilmiştir. Çapraz bağımlılıklar
ilgili diğer kategoriye bağlanır; aynı iş için paralel, çatışan plan tutulmaz.

### <a id='dunya-cevre'></a>K1 — Dünya ve çevre

- **Mevcut:** Harita, arazi/duvarlar, çevre kuralları ve kaynak yenilenme
  mekanizmaları var.
- **Açık:** Gün-gece ritmi ve iz/koku/ses yayılımı yok; var olan zamanlama tüm
  yeni çevre koşullarını tanımlamıyor.
- **Önerilen:** Deterministik gün-gece çevrimini ve iz/ses/koku için
  sınırlı yayılma/bozulma modelini tasarla. Bu kategorinin sahibi çevresel
  yayılım ve zamanlamadır; canlı algısı K2, bireysel kullanımı K3'e bağlıdır.
- **Kabul:** Çevre sinyali kaynak, yayılma, mesafe, zaman ve kaybolma olaylarıyla
  açıklanır; seedli fizik testleri ve State+RNG kabulü geçer. Çevrenin gizli
  sonraki durumu bireysel algıya sızmaz.

### <a id='canli-beden'></a>K2 — Canlı ve beden

- **Mevcut:** Sağlık, enerji, metabolizma ve hareket maliyetleri vardır; temel
  dinlenme ve yeterli enerji/su koşulunda enerji ödeyen pasif iyileşme bulunur.
  Sağlık üreme uygunluğunu etkiler. İstenen maliyet ile ödenen maliyet ayrılır.
- **Açık:** Ayrı yaralanma durumunun hareket kapasitesi, avlanma ve dinlenmeye
  etkisi henüz doğrulanmış bir genişletme değildir. Yeni iz duyuları canlı algı
  sınırına bağlanmalıdır.
- **Önerilen:** Mevcut pasif iyileşmeyi temel alarak bedelli yaralanma ve
  kapasite kaybı modelini tanımla; çevre
  sinyallerini canlıya aktarma K1 ile bağımlıdır. Gündüz/gece dinlenme yanıtı
  K1 zamanlamasına bağlıdır.
- **Kabul:** Hasar, iyileşme, enerji ve hareket sonuçları resolver/metabolizma
  olaylarından yeniden hesaplanır; tam/ölü/kesilmiş sonuçlar karıştırılmaz.
  Deterministik küçük fizik testleri geçer.

### <a id='bireysel-ogrenme'></a>K3 — Bireysel davranış ve öğrenme

- **Mevcut:** Bireye ait sınırlı kaynak/tehlike belleği, unutma, kurallı seçim ve
  temel su davranışı var. V17 su artığı varsayılan kapalı; formal utility
  **FALSE**. İlk-resolver hedefi için 623 exact gözlem var, fakat model yok ve
  tam kaçış hasarı `Unknown`.
- **Açık:** Güncel bağlamla su artığının uyumu, per-owner ilk-aralık hasarı ve
  öğrenilmiş bilginin eskimesi kabul edilmedi; spatial known-map sistemi yok.
  Sekiz devam aralığında yeni karar-anı algısı yoktur; bunları taze sample
  saymak doğru değildir.
- **Önerilen:** P0'da per-owner ilk-aralık hasar ve su-artığı mekanizma
  sözleşmelerini tasarla. Öneri: bireysel mekânsal bellek/known-map aging ile
  eski öğrenilmiş bilgiyi unutma veya yeniden gözlemleme; iz/ses/koku karar
  girdisi K1/K2 algısına bağlıdır. Sonlu eylem bileşimleri D.1 bu kategorinin,
  sosyal koordinasyon D.2 ise K4'ün sorumluluğudur. Bu öneriler model eğitimi veya
  World/lab çalıştırma yetkisi değildir.
- **Kabul:** Yalnız bireyin gördüğü/hatırladığı kanıt kullanılır. Hedef ufku
  içinde atıfsız, eksik veya sansürlü sonuç `Unknown` kalır; tamamlanmış ilk
  aralığın exact sonucu sonraki tam-rota sansüründen etkilenmez. Bireysel
  öğrenilmiş tablolar başkasına veya yavruya kopyalanmaz; K4 iletişimleri yalnız
  alıcının doğrulanmış yerel gözlemi olarak işlenebilir. Disabled-path
  eski/yeni State+RNG ve deterministic resume doğrulanır.

### <a id='grup-sosyal'></a>K4 — Grup ve sosyal ilişkiler

- **Mevcut:** Birden çok canlı ve tür aynı dünyada hareket eder; resolver
  çakışma, paylaşım ve etkileşimleri sıraya koyar. Görülen yetişkin türdeş/eş
  belleği ve seçenekle açılan buluşma politikası vardır. Bunlar genel grup
  koordinasyonu veya iletişim yeteneği kabulü değildir.
- **Açık:** Yerel bilgiyle işbirliği, koordinasyon ve grup davranışı için kabul
  edilmiş katalog/ölçüt yoktur.
- **Önerilen:** D izinin sosyal parçası olarak görünür komşu/iletişim olaylarına
  dayalı tanıma, uyarı sinyali ve birlikte hareketi ayrı mekanizmalar olarak
  tanımla. Sonlu eylem kataloğu K3'e aittir;
  grup düzeyinde ortaya çıkan koordinasyon burada değerlendirilir.
- **Kabul:** Koordinasyon yalnızca doğrulanmış görünür yerel bilgi kullanır;
  resolver geçerli eylemi ve bedelini kaydeder. Tekil, grup ve kontrol
  kollarında etki ölçülür; global gizli bilgi veya otomatik koordinasyon
  varsayılmaz.

### <a id='ekoloji-etkilesim'></a>K5 — Ekoloji ve çevreyle etkileşim

- **Mevcut:** Yenilenen kaynaklar, birden çok tür ve habitat fixture'ı var.
  Kayıtlı 3 × 500 tick ecology fixture viability kapısını geçti; genel habitat
  veya fitness kabulü değildir. C izi bu fixture ve kaynak muhasebesidir.
- **Açık:** Kaynak akışı, tür başına tüketim, basal beden enerjisi ve trait
  maliyetlerinin bazıları tamamıyla doğrulanmadı. Coexistence sonucu yalnız
  kayıtlı fixture'a bağlıdır.
- **Önerilen:** C iziyle kaynak stok/yenilenme/tüketim ve beden enerji akışını
  denetle. Öneri: otlama sonrası bitki yenilenmesi, tohum yayılımı ve
  sığınakların canlılarca değişmesi; dünya kuralları K1'e bağlıdır. K6'nın
  habitat uzmanlaşması ölçümleri bu doğrulanmış akışa bağlıdır.
- **Kabul:** Başlangıç, yenilenme, tüketim, tür akışı ve ödenen beden maliyeti
  kapanan bir muhasebeyle izlenir; kurucu/F1/F2 ve yavru üretmiş soy ayrı
  raporlanır. Habitat ve trait etkileri karıştırılmaz; held-out roller önceden
  sabitlenir.

### <a id='kalitim-evrim'></a>K6 — Kalıtım ve evrim

- **Mevcut:** Sonlu allel/özellik aktarımı ve mutasyon mekanizmaları opt-in
  olarak vardır; öğrenilmiş bellek yavruya aktarılmaz. Seçilim/kalıcılık
  avantajı kabul edilmedi.
- **Açık:** Trait maliyeti, gerçek beden kazancı ve nesiller arası allel
  dağılımı ekolojik akıştan bağımsız tam doğrulanmış değil.
- **Önerilen:** E izini K5 kaynak ve soy muhasebesinden sonra test et. Öneri:
  besin/zemin/aktivite maliyetlerine göre niş uzmanlaşması; özellik K6'da
  kalıtılır/seçilir, K2 beden maliyeti ve K5 çevre-etkileşiminde ölçülür.
- **Kabul:** Sonlu kalıtım ve mutasyon, bedelin ödenmesi, üreyen yavru-soyu ve
  allel frekansı ayrı kaydedilir. Held-out seed/roller önkayıtlıdır; öğrenilmiş
  deneyim kalıtılmaz. Sağkalım tek başına evrimsel üstünlük kanıtı değildir.

### <a id='gozlem-arayuz'></a>K7 — Gözlem ve belgesel arayüz

- **Mevcut:** Salt-okunur birey, karar, geçmiş ve soy ekranı var; F izi ilk
  belgesel deneyim arayüzüdür.
- **Açık:** Bütün biliş/deneyim veya nedensellik görünürlüğü yoktur.
- **Önerilen:** Görülen/hatırlanan kanıttan seçilen eyleme, resolver maliyet ve
  sonucuna, oradan gözlenen bellek güncellemesine kadar iz sür. Eksik bağları
  `Unknown` göster.
- **Kabul:** Arayüz kayıtlı olguları gösterir; gizli durum veya model çıkarımını
  gözlem gibi sunmaz. UI/API okumaları simülasyon durumunu değiştirmez.

### <a id='motor-isletim'></a>K8 — Motor ve işletim

- **Mevcut:** Deterministik motor, checkpoint/SQLite/shard/retention ve servis
  kontrolü var. Canlı V12'de arşiv bütçesi backportu çalışıyor; kota soft guard
  olmaya devam eder. G izi depolama, throughput ve işletim ölçümüdür.
- **Açık:** Kısa yük sınırı ölçümleri genel performans/kapasite garantisi
  değildir. Aylar/yıllar soak ertelendi; bugün özellik çalışmasını bloke etmez.
- **Önerilen:** Sabit kısa yüklerde tick hızı, actor sayısı, RSS, SQLite/HTTP,
  disk büyümesi ve güvenli pause/retention sınırlarını ölç.
- **Kabul:** Sürüm/migration, eski SQLite okuma, checkpoint resume, writer lock,
  hata/pause ve arşiv bütçesi test edilir. Kapalı özelliğin disabled-path
  State+RNG'si eski sürümle eşleşir; deterministik checkpoint-resume doğrulanır.
  Frozen model parametreleri sabit tutulur, fakat online keşif sürebileceği için
  frozen ve online tüm sonuçların eşit olması beklenmez. Her entegrasyon root
  incelemesi ve bağımsız audit alır.

## A–G yetenek izleri

Bu izler kategoriler arası takibi korur; her özelliğin tek birincil sahibi vardır.
D izi, K3'teki eylem bileşimleri ve K4'teki sosyal koordinasyon olarak iki alt
işe ayrılır.

| İz | Birincil kategori | Güncel ölçülmüş durum | Önerilen sonraki iş |
|---|---|---|---|
| **A — Güncel bağlamlı su artığı** | K3 | V17 default-off; bounded utility **FALSE**. | P0: güncel bağlamla uyum hipotezi, deterministic test ve ayrı preregistration. |
| **B — Per-owner ilk-aralık hasar öğrenmesi** | K3 | 623 exact: 617 sıfır, altı × 20 hasar, dört birey; full-route `Unknown`, model yok. | P0: hedef ufku ve attribution/censor sözleşmesi; P1 fizik testinden önce training/World yok. |
| **C — Kaynak/habitat/bedelli-trait fixture'ı** | K5 | 3 × 500 tick viability geçti; genel fitness kabulü ve bazı enerji muhasebesi açık. | Kaynak akışı ve istenen/ödenen enerji denetimi; K1/K2/K6 bağımlılıklarını ölç. |
| **D — Eylem ve yerel koordinasyon** | K3 eylemler; K4 sosyal koordinasyon | Resolver'dan geçen temel eylemler var; katalog ve koordinasyon kabul edilmedi. | Sonlu bileşimleri K3'te, yalnız görünür yerel koordinasyonu K4'te tasarla/test et. |
| **E — Sonlu allel varyasyonu/seçilim** | K6 | Allel/bedelli trait mekanizmaları var; avantaj kabul edilmedi, bellek kalıtılmaz. | C muhasebesinden sonra held-out allel, bedel ve yavru-soyu testi. |
| **F — Bireysel deneyim arayüzü** | K7 | Salt-okunur birey/karar/geçmiş görünümü var. | Kanıt→eylem→resolver sonucu→öğrenme güncellemesi izi; eksik bağ `Unknown`. |
| **G — Kapasite/depolama/işletim** | K8 | Shard/checkpoint/retention ve canlı arşiv backportu var; kota soft guard. | Kısa sabit yük ölçümleri ve sınırlarda güvenli pause/retention doğrulaması. |

## P0–P3 yürütme sırası

Bu sıra bağımlılık ve yetkilendirme kapısıdır; K1–K8 sınıflandırması değildir.

### P0 — Önce destek ve mekanizma

1. K3 per-owner ilk-resolver hasarı hedefi ve su-artığı bağlam uyumunu
   tasarla. Mevcut veriden model eğitme veya yeni World/lab başlatma yetkisi
   çıkmaz. Tam rota hasarı bilinmeyen kalır.
2. Su artığı için güncel bağlam/beden tabanına uyum hipotezini tanımla; retune
   veya yeni seed için ayrı preregistration gerekir.

### P1 — Fizik ve muhasebe baseline'ları

K2 hasar/iyileşme ve enerji; K3 hasar atfı, hedef-ufku sansürü ve ilk-aralık
exposure; K5 kaynak akışı ve beden enerji bütçesi için küçük deterministik
testler. C fixture'ında kurucu/F1/F2 ve üreyen soy ayrı raporlanır.

İz yayılımı, gün-gece ritmi, mekânsal bellek/known-map aging ve yaralanma
önerileri kendi K1/K2/K3 baseline'larından sonra ayrıca gözden geçirilir; bu
yol haritası bunları otomatik başlatmaz.

### P2 — Sınırlı entegrasyon

P0/P1 root incelemesinden sonra, birincil kategori sahibinin sınırında default-off
entegrasyon. Bireysel veri izolasyonu, freeze/update, disabled-path eski/yeni
State+RNG, deterministik resume, migration/eski okuma, tek writer ve hata/pause
kapıları geçmeden ortak kaynakta etkinleştirme yoktur. Yeni özellikler modüler
kalır: K2 beden, K1/K2 perception, K3 memory/learning/decision, K3/K4
resolver eylem ve sosyal etkileri, K8 persistence/control.

### P3 — Ekoloji ve seçim kabulü

Fizik/entegrasyon kapılarından sonra K5 habitat/çevre değişimi ve K6 bedelli
trait/niş karşılaştırmaları. Her formal karşılaştırma için protokol, fixed seed
rolleri, held-out bölümler, doğal yok oluş kuralı ve fizik çıktıları önceden
sabitlenir. Sonuç görüldükten sonra seed/eşik/feature tuning yapılmaz.

## Özgün 11 aşamanın kısa durumu

| # | Aşama | 25 Ekim durumu |
|---:|---|---|
| 1 | Avcı yavrusu ve soyunun devamı | Üreme/olgunlaşma mekanizması ve sınırlı coexistence kapısı var; genel kalıcılık kanıtlanmadı. |
| 2 | Av seçimi, maliyet, bırakma ve dinlenme | İlk kurallı politika var; risk öğrenmesi fayda göstermedi. |
| 3 | Karma ekosistem | Kayıtlı fixture'da bounded coexistence kabulü var; genel habitat sonucu yok. |
| 4 | Hareket, duvar, sığınak, diyet, paylaşım ve çakışma | İlk resolver sözleşmeleri mevcut ve testli. |
| 5 | Sınırlı bireysel kaynak/tehlike belleği | Bounded bellek/unutma mekanizmaları var; genelleme kabul edilmedi. |
| 6 | Sonuçtan su öğrenmesi ve kontrol kolu | Mekanizma/formal ölçüm var; V17 bounded utility FALSE. |
| 7 | Bedelli kalıtsal hareket/trait ve mutasyon | İlk allel/bedel ölçümü var; seçilim avantajı kabul edilmedi. |
| 8 | Birey, karar, geçmiş, bellek ve soy arayüzü | İlk salt-okunur belgesel ekran var; bütün bilişin kaydı değildir. |
| 9 | Gerçek gün/hafta dayanıklılığı | Sınırlı tick testleri var; uzun gerçek-zamanlı soak kullanıcı isteğiyle ertelendi. |
| 10 | Kurucu ve harita ölçek ölçümü | 10–1.000 kurucu ve farklı haritalarda ilk ölçüm araçları var. |
| 11 | İzleme, yedekli dağıtım ve bildirim | İşletim/monitoring var; uzun soak veya kapasite garantisi değildir. |

Yeni seed, lab, model eğitimi, live deployment veya ortak çekirdek entegrasyonu
bu belgeyle otomatik olarak yetkilendirilmez; ayrı review ve protokol gerekir.
Kategori bağlantıları: [K1](#dunya-cevre), [K2](#canli-beden),
[K3](#bireysel-ogrenme), [K4](#grup-sosyal), [K5](#ekoloji-etkilesim),
[K6](#kalitim-evrim), [K7](#gozlem-arayuz), [K8](#motor-isletim).
