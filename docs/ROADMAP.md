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
  mekanizmaları var. Deterministik gün-gece çevrimi (`DayNightCycle`: 4 fazlı
  Şafak/Gündüz/Alacakaranlık/Gece) ve sınırlı çevresel iz/sinyal yayılımı
  (`SignalTrace`: `Footstep`, `Scent`, `Sound` yayılım ve sönümlenmesi) opt-in
  olarak uygulandı ve deterministik fizik testleriyle doğrulandı
  (`src/environment.rs`, `src/systems/perception.rs`,
  `tests/environment_signals.rs` — *Uygulandı / Mekanizma testi geçti, varsayılan kapalı*).
  Ayrıca dinamik habitat (Yeni Özellik 4: `DigBurrow`/`BurrowDecay`, `Overgrazing`,
  `SeedDispersal`), mevsimsel iklim ve kış uykusu (Yeni Özellik 5: `SeasonCycle`,
  kış termoregülasyon BMR ek maliyeti, `Hibernation`) ve ceset çürümesi/leşçillik nişi
  (Yeni Özellik 6: `Fresh`/`Aging`/`Putrid`, leş kokusu, etçil vs hepçil bozulma direnci)
  saf mekanizma prototipleriyle test edildi (`tests/dynamic_habitat.rs`,
  `tests/seasonal_climate.rs`, `tests/corpse_scavenging.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
- **Açık:** Saf prototipler (dinamik habitat, mevsimler, ceset çürümesi) motor
  çekirdeğine entegre değildir; uzun dönemli ekolojik etkileri ampirik olarak test
  edilmemiştir.
- **Önerilen:** P2 entegrasyon sınırlarında çevre sinyalleri ve dinamik habitat
  modellerini root incelemesine sun. Canlı algısı K2, bireysel kullanımı K3'e bağlıdır.
- **Kabul:** Çevre sinyali kaynak, yayılma, mesafe, zaman ve kaybolma olaylarıyla
  açıklanır; seedli fizik testleri ve State+RNG kabulü geçer. Çevrenin gizli
  sonraki durumu bireysel algıya sızmaz.

### <a id='canli-beden'></a>K2 — Canlı ve beden

- **Mevcut:** Sağlık, enerji, metabolizma ve hareket maliyetleri vardır; temel
  dinlenme ve yeterli enerji/su koşulunda enerji ödeyen pasif iyileşme bulunur.
  Sağlık üreme uygunluğunu etkiler. İstenen maliyet ile ödenen maliyet ayrılır.
  Metabolizma, hareket, beslenme ve üremede beden enerji ve su bilançosunun
  deterministik kapanışı küçük fizik testleriyle doğrulandı
  (`tests/physiology.rs` — *Mekanizma testi geçti*).
  Ağır yaralanma kapasite kaybı modeli (`injury_capacity_penalty`: `health <= max_health / 2`
  durumunda `effective_speed` yarıya iner ve hareket staminası 2 katına çıkar,
  pasif iyileşmeyle düzelir) uygulandı ve deterministik fizik testleriyle doğrulandı
  (`src/entity/lifestate.rs`, `src/policy.rs`, `tests/injury_physics.rs` — *Uygulandı / Mekanizma testi geçti, varsayılan kapalı*).
  Ontogenetik yavru fizyolojisi ve fiziksel ebeveyn bakımı (Yeni Özellik 2: `age < maturity_age`
  ölçekli fizyoloji, `FeedOffspring` enerji aktarımı, `GuardOffspring` hasar engelleme,
  sıfır öğrenilmiş bellek aktarımı) saf prototipte doğrulandı
  (`examples/support/juvenile_parental_care_candidate.rs`,
  `tests/juvenile_parental_care.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
- **Açık:** Ebeveyn bakımı ve yavru fizyolojisi motor çekirdeğine entegre değildir;
  yaralanma cezası canlıda default-off'tur.
- **Önerilen:** Beden yaralanma ve ebeveyn bakımı dinamiklerini K1 çevre döngüsü
  ve K5 kaynak akışıyla bağlantılı olarak P2 kapısına hazırla.
- **Kabul:** Hasar, iyileşme, enerji ve hareket sonuçları resolver/metabolizma
  olaylarından yeniden hesaplanır; tam/ölü/kesilmiş sonuçlar karıştırılmaz.
  Deterministik küçük fizik testleri geçer.

### <a id='bireysel-ogrenme'></a>K3 — Bireysel davranış ve öğrenme

- **Mevcut:** Bireye ait sınırlı kaynak/tehlike belleği, unutma, kurallı seçim ve
  temel su davranışı var. V17 su artığı varsayılan kapalı; formal utility
  **FALSE**. İlk-resolver hedefi için 623 exact gözlem var, fakat model yok ve
  tam kaçış hasarı `Unknown`. Hedefe doğrudan adım engellendiğinde yerel BFS
  baypası `Memory::toward` ile uygulandı (`tests/memory.rs` — *Uygulandı / Mekanizma testi geçti*).
  Per-owner ilk-aralık hasar öğrenici saf prototipi (`tests/refuge_first_interval.rs`)
  ile ilk-aralık ve tam-rota sansür ayrımı (`tests/refuge_learning.rs`) mekanizma
  testini geçti. Güncel bağlam ve TTL/pencere kısıtlı su artığı saf prototipi
  mekanizma testini geçti (`tests/water_context_residual.rs`). Mekânsal zihin haritası
  ve merak güdümlü keşif (Yeni Özellik 1: sınırlı 16 sektörlü `4x4` ızgara, TTL eskime,
  keşfedilmemiş/en eski sektörlere merak yönelimi; `tests/spatial_curiosity.rs`) ile
  pusu/gizlenme vs. takip avcılığı (Yeni Özellik 3: örtü tabanlı gizlenme, ilk vuruş
  stamina bonusu, takip vs. pusu ödünleşimi; `tests/ambush_predation.rs`) saf
  prototip mekanizma testlerini geçti (*Saf mekanizma prototipi / Mekanizma testi geçti*).
  Prototipler World ve motordan izoledir; genel rota tahmincisi veya ampirik fayda
  iddia edilmez.
- **Açık:** Saf prototipler motor çekirdeğine bağlı değildir; tam-rota sığınak hasarı
  `Unknown` kalır. Bounded utility ampirik kabulü henüz yoktur.
- **Önerilen:** P2 entegrasyon sınırları için per-owner ilk-aralık hasarı, su artığı
  ve zihin haritası sözleşmelerini dondur.
- **Kabul:** Yalnız bireyin gördüğü/hatırladığı kanıt kullanılır. Hedef ufku
  içinde atıfsız, eksik veya sansürlü sonuç `Unknown` kalır; tamamlanmış ilk
  aralığın exact sonucu sonraki tam-rota sansüründen etkilenmez. Bireysel
  öğrenilmiş tablolar başkasına veya yavruya kopyalanmaz; K4 iletişimleri yalnız
  alıcının doğrulanmış yerel gözlemi olarak işlenebilir. Disabled-path
  eski/yeni State+RNG ve deterministic resume doğrulanır.

### <a id='grup-sosyal'></a>K4 — Grup ve sosyal ilişkiler

- **Mevcut:** Birden çok canlı ve tür aynı dünyada hareket eder; resolver
  çakışma, paylaşım ve etkileşimleri sıraya koyar. Görülen yetişkin türdeş/eş
  belleği ve seçenekle açılan buluşma politikası vardır. Yerel alarm çağrısı
  (`EmitAlarmCall`) ve görünür akran koordinasyonu (`AlignWithVisiblePeer`),
  ödenen bedel ve 3 kollu kontrol/solo/grup deneyi saf prototipte doğrulandı
  (`examples/support/local_coordination_candidate.rs`,
  `tests/local_coordination.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
- **Açık:** Genel sürü zekâsı veya telepatik grup optimizasyonu yoktur;
  koordinasyon motor çekirdeğine entegre değildir.
- **Önerilen:** Yalnız görünür yerel bilgiyle sınırlı uyarı ve akran koordinasyonunu
  P2 kapısına hazırla.
- **Kabul:** Koordinasyon yalnızca doğrulanmış görünür yerel bilgi kullanır;
  resolver geçerli eylemi ve bedelini kaydeder. Tekil, grup ve kontrol
  kollarında etki ölçülür; global gizli bilgi veya otomatik koordinasyon
  varsayılmaz.

### <a id='ekoloji-etkilesim'></a>K5 — Ekoloji ve çevreyle etkileşim

- **Mevcut:** Yenilenen kaynaklar, birden çok tür ve habitat fixture'ı var.
  Kayıtlı 3 × 500 tick ecology fixture viability kapısını geçti; genel habitat
  veya fitness kabulü değildir. C izi bu fixture ve kaynak muhasebesidir.
  Çok-nesilli ekoloji audit'inde Kurucu (`gen 0`), F1 (`gen 1`) ve F2 (`gen 2`)
  kohortlarının kesin ayrımı, üreyen birey ve soylar ile üremeyen hayatta
  kalanların ayrımı ve tüm tick'lerde sıfır açıklanamayan besin ve su residüeli
  deterministik audit testiyle doğrulandı (`examples/support/ecology_audit.rs`,
  `tests/ecology_candidate.rs` — *Mekanizma testi geçti*).
- **Açık:** Farklı habitatlarda kaynak akışı genellemesi, tür başına tüketim ve
  dinamik habitat değişimlerinin ekolojik etkisi tamamıyla doğrulanmadı. Coexistence
  sonucu yalnız kayıtlı fixture'a bağlıdır.
- **Önerilen:** C iziyle kaynak stok/yenilenme/tüketim ve beden enerji akışını
  denetle. Dinamik habitat (Yeni Özellik 4) ve mevsimsel iklim (Yeni Özellik 5)
  etkilerini bu muhasebeye bağla.
- **Kabul:** Başlangıç, yenilenme, tüketim, tür akışı ve ödenen beden maliyeti
  kapanan bir muhasebeyle izlenir; kurucu/F1/F2 ve yavru üretmiş soy ayrı
  raporlanır. Habitat ve trait etkileri karıştırılmaz; held-out roller önceden
  sabitlenir.

### <a id='kalitim-evrim'></a>K6 — Kalıtım ve evrim

- **Mevcut:** Sonlu allel/özellik aktarımı ve mutasyon mekanizmaları opt-in
  olarak vardır; öğrenilmiş bellek yavruya aktarılmaz. Trait maliyeti ödemesi
  ile üreyen yavru soyu seçilim ayrımı deterministik testle doğrulandı
  (`tests/genetics.rs` — *Mekanizma testi geçti*).
- **Açık:** Seçilim avantajı veya niş uzmanlaşması (Yeni Özellik 3 pusu vs takip,
  Yeni Özellik 6 leşçillik) ampirik olarak kabul edilmedi.
- **Önerilen:** Allel maliyetleri ve soy devamlılığı ayrımını niş uzmanlaşması
  testlerine bağla.
- **Kabul:** Sonlu kalıtım ve mutasyon, bedelin ödenmesi, üreyen yavru-soyu ve
  allel frekansı ayrı kaydedilir. Held-out seed/roller önkayıtlıdır; öğrenilmiş
  deneyim kalıtılmaz. Sağkalım tek başına evrimsel üstünlük kanıtı değildir.

### <a id='gozlem-arayuz'></a>K7 — Gözlem ve belgesel arayüz

- **Mevcut:** Salt-okunur birey, karar, geçmiş ve soy ekranı var; F izi ilk
  belgesel deneyim arayüzüdür. Nedensellik zinciri izi (`causalChain`:
  `evidence -> action -> resolverOutcome -> memoryUpdate`, eksik halkalar açık
  `Unknown`), sığınak deneyim özeti (`CuriousExperience.refuge`) ve Yeni Özellik 7
  (Auto-Director Camera — `autoDirector`: en yüksek öncelikli belgesel olayını
  otomatik odaklama) uygulandı ve doğrulandı (`web/documentary.js`,
  `web/experience.js` — *Uygulandı / Mekanizma testi geçti*).
- **Açık:** Bütün biliş/deneyim veya nedensellik görünürlüğü yoktur; model çıkarımı
  veya gizli durum gözlem gibi sunulamaz, eksik bağlar `Unknown` kalır.
- **Önerilen:** Arayüzde nedensellik zinciri ve auto-director olay önceliklendirmesini
  yeni K1–K6 sinyalleriyle genişlet.
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
| **A — Güncel bağlamlı su artığı** | K3 | V17 default-off; bounded utility **FALSE**. Saf bağlam/TTL mekanizma prototipi testi geçti (`tests/water_context_residual.rs`); World/motordan izole. | P2 entegrasyon sınırları ve ampirik fayda için ayrı preregistration. |
| **B — Per-owner ilk-aralık hasar öğrenmesi** | K3 | 623 exact: 617 sıfır, altı × 20 hasar, dört birey; full-route `Unknown`. Saf learner prototipi ve ilk-aralık sansür ayrımı testi geçti (`tests/refuge_first_interval.rs`, `tests/refuge_learning.rs`). | P2 entegrasyon sınırları; full-route hasar modeli ve ampirik kabul açık. |
| **C — Kaynak/habitat/bedelli-trait fixture'ı** | K5 | 3 × 500 tick viability geçti; beden bilançosu (`tests/physiology.rs`) ve Kurucu/F1/F2 soy ve stok audit'i (`tests/ecology_candidate.rs`) doğrulandı. Dinamik habitat (Yeni Özellik 4) ve mevsimsel iklim (Yeni Özellik 5) saf prototipleri geçti. | Habitat/trait genellemesi ve K1/K2/K6 bağımlılıklarını ölç. |
| **D — Eylem ve yerel koordinasyon** | K3 eylemler; K4 sosyal koordinasyon | Resolver'dan geçen temel eylemler var. Yerel alarm ve akran koordinasyonu (`tests/local_coordination.rs`), merak güdümlü keşif (Yeni Özellik 1, `tests/spatial_curiosity.rs`) ve pusu avcılığı (Yeni Özellik 3, `tests/ambush_predation.rs`) saf prototip testleri geçti. | P2 entegrasyon sınırları; grup düzeyinde ampirik koordinasyon kabulü açık. |
| **E — Sonlu allel varyasyonu/seçilim** | K6 | Allel/bedelli trait mekanizmaları var; allel bedeli ödemesi ve soy seçilimi ayrımı doğrulandı (`tests/genetics.rs`). Pusu/takip ve leşçillik allel dengesi prototipleri geçti; ampirik avantaj kabul edilmedi, bellek kalıtılmaz. | C muhasebesinden sonra held-out allel, bedel ve yavru-soyu testi. |
| **F — Bireysel deneyim arayüzü** | K7 | Salt-okunur birey/karar/geçmiş görünümü var. Nedensellik zinciri (`causalChain`), sığınak deneyim özeti ve Auto-Director kamera (Yeni Özellik 7) uygulandı (`web/documentary.js`, `web/experience.js`). | Eksik bağlar açık `Unknown`; yeni çevre ve sosyal olayları arayüze ekle. |
| **G — Kapasite/depolama/işletim** | K8 | Shard/checkpoint/retention ve canlı arşiv backportu var; kota soft guard. | Kısa sabit yük ölçümleri ve sınırlarda güvenli pause/retention doğrulaması. |

## P0–P3 yürütme sırası

Bu sıra bağımlılık ve yetkilendirme kapısıdır; K1–K8 sınıflandırması değildir.

### P0 — Destek ve mekanizma prototipleri (Mekanizma testi geçti)

1. K3 per-owner ilk-resolver hasarı saf prototipi (`tests/refuge_first_interval.rs`)
   ve ilk-aralık/tam-rota sansür ayrımı (`tests/refuge_learning.rs`) mekanizma
   testini geçti. Model eğitimi veya World/lab yetkisi değildir; tam-rota hasarı
   `Unknown` kalır.
2. Su artığı bağlam/TTL saf prototipi (`tests/water_context_residual.rs`), mekânsal
   zihin haritası/merak (Yeni Özellik 1, `tests/spatial_curiosity.rs`) ve pusu/gizlenme
   (Yeni Özellik 3, `tests/ambush_predation.rs`) saf prototip mekanizma testlerini geçti.
3. K1 dinamik habitat (Yeni Özellik 4, `tests/dynamic_habitat.rs`), mevsimsel iklim
   (Yeni Özellik 5, `tests/seasonal_climate.rs`), ceset çürümesi (Yeni Özellik 6,
   `tests/corpse_scavenging.rs`); K2 yavru fizyolojisi/ebeveyn bakımı (Yeni Özellik 2,
   `tests/juvenile_parental_care.rs`) ve K4 yerel koordinasyon (`tests/local_coordination.rs`)
   saf mekanizma prototipleri tamamlandı ve doğrulandı.

### P1 — Fizik ve muhasebe baseline'ları (Mekanizma testi geçti)

K1 gün-gece çevrimi ve çevresel iz yayılımı (`tests/environment_signals.rs`);
K2 ağır yaralanma kapasite kaybı (`tests/injury_physics.rs`) ve beden enerji/su
stok-akış bilançosu (`tests/physiology.rs`); K3 ilk-aralık sansür ayrımı
(`tests/refuge_learning.rs`); K5 kurucu/F1/F2 kohort ayrımı, üreyen soy ve sıfır
açıklanamayan kaynak residüeli audit testleri (`tests/ecology_candidate.rs`);
K6 allel maliyet ödemesi ile soy seçilimi ayrımı (`tests/genetics.rs`); K7
nedensellik zinciri ve Auto-Director kamera (`web/documentary.js`) doğrulandı.

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
