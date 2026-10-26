# Curious yol haritası

**Durum tarihi: 26 Ekim 2026.** Aşağıdaki sekiz kategori güncel durum, açık
sınırlar, önerilen çalışma ve kabul ölçütünü ayırır. Öneriler uygulandıkları
anlamına gelmez. P0–P3 çalışma sırası kategori yapısından ayrıdır.

## Sistem durumu

- Geliştirme kaynağı V21 / SQLite V2; canlı dünya V12 checkpoint'tir. V21
  canlıya yükseltilmemiştir. Nitelikli binary kimliği ve güncel operasyon
  kuralları [AGENTS.md](../AGENTS.md)'dedir.
- Kaynakta mekanizmalar ve deneysel seçenekler bulunur; bunlar tek başına
  davranışsal veya ekolojik fayda kabulü değildir. Varsayılan durum seçenek
  bazında değişir: örneğin V17 su artığı default-off'tur; ceset evreleri
  fiziksel yaşam döngüsünün parçasıdır. Tam özellik listesi yerine kategori
  bölümlerine bakın.
- [V2 su uyum labı](reports/WATER_ADAPTATION_V2_20261025.md) üç learner
  kolunda aynı fiziksel eylem/yörüngeyi buldu; ek öğrenme faydası kabul edilmedi.
  [Su sinyali tanısı](reports/WATER_SIGNAL_DIAGNOSIS_20261026.md) yeni K3
  hedefini yalnızca öneri olarak belgeliyor; hedef uygulanmadı ve yeni hedef için
  kayıtlı bir deney yürütülmedi.
  [Nitelik tanısı](reports/QUALIFICATION_DIAGNOSTIC_20261025.md) önceki 401–410
  seed tekrarında bakım ve bakım+sığınak için 2 iyileşme, 8 nötr; refuge için
  10 nötr buldu. Bunlar yeni held-out seedler değildir.
- V12 arşiv başlangıcından yürütülen 1.000-tick **offline V19 gölge** koşusunda
  paket, V19 kapalı kontrole göre endpoint'te 13 fazla canlı fakat 1,428 daha
  az toplam birey-tick verdi. Bu bir canlı V19 yükseltmesi veya kabul edilmiş
  Pareto/fayda sonucu değildir. V12 canlı sisteminde yükseltme yapılmadı.
- Refuge tam-rota beklenen tehdit hasarı `Unknown`; tamamlanmış ilk resolver
  aralığı ayrı hedef ufkudur. Kayıtlı coexistence sonucu yalnız kendi fixture
  ve protokol kapsamındadır. Uzun gerçek-zamanlı soak ertelenmiştir.
- [Ekoloji transfer V1](reports/ECOLOGY_TRANSFER_V1_20261026.md) 45 World ve
  27,000 ana tick tamamladı. Seyrek kaynak habitatında her kolun üç koşusu da
  söndü; karma avcı habitatında 15 koşunun tümünde etçiller kayboldu, bir alarm
  koşusunda tüm popülasyon söndü. Çayırdaki kalıcılık özellik üstünlüğü değildir;
  genel ekolojik fayda kabul edilmedi.
- [Operasyonel hazırlık probu](reports/OPERATIONS_READINESS_20261026.md) offline
  V12→V19 migration, State/RNG round-trip ve izole SQLite/shard kontrollerini
  geçti. Bu smoke deployment kapısını açmaz. Dünya-tick ortalama/maksimum ve
  toplam süre raporlanabilir; eski p50/p95 hesapları
  [düzeltme notunda](reports/OPERATIONS_READINESS_TIMING_CORRECTION_20261026.md)
  açıklanan sıralama hatasıyla geçersizdir.
- [Yem arama bütçesi](FOOD_SEARCH_RESERVE.md) V20 kaynağında varsayılan kapalı
  uygulandı; yalnız SearchFood hareketinin harcamasını sınırlar. [36 koşuluk lab](reports/FOOD_SEARCH_RESERVE_V1_20261026.md), tek yemli
  ortamda 4/6 seedde yem ve sağkalım artışı; 2/6 nötr sonuç verdi. Bütün tek
  yemli bireyler yine öldü, diğer iki fixture'da seçenek hiç tetiklenmedi.
  Genel ekolojik fayda veya canlı kabulü yoktur.
  Bakım+sığınak paketi davranışsal fayda olarak kabul edilmedi. Gelecek öneriler
  ayrı protokol ve review gerektirir.

Kanıt ayrıntısı: [öğrenme sözleşmesi](LEARNING_NEXT.md),
[nitelik tanısı](reports/QUALIFICATION_DIAGNOSTIC_20261025.md),
[V2 su labı](reports/WATER_ADAPTATION_V2_20261025.md),
[coexistence gözlemi](reports/QUALIFIED_OBSERVATION.md) ve ilgili tarihli
[denetim/ölçüm raporları](reports/).

Yeni kanıt: [ekoloji aktarım taraması](reports/ECOLOGY_TRANSFER_V1_20261026.md),
[operasyonel hazırlık](reports/OPERATIONS_READINESS_20261026.md),
[su sinyali tanısı](reports/WATER_SIGNAL_DIAGNOSIS_20261026.md) ve
[kaynak güvenilirliği hedef taslağı](SOURCE_RELIABILITY_TARGET_20261026.md).

## Sekiz sistem kategorisi

Her özellik için birincil sahip kategori belirtilmiştir. Çapraz bağımlılıklar
ilgili diğer kategoriye bağlanır; aynı iş için paralel, çatışan plan tutulmaz.

### <a id='dunya-cevre'></a>K1 — Dünya ve çevre

- **Mevcut:** Harita, arazi/duvarlar, çevre kuralları ve kaynak yenilenme
  mekanizmaları var. Gün/gece çevrimi (`DayNightCycle`) fiziksel olarak gündüz/gece olmak üzere iki
  görüş durumunu hesaplar; arayüzdeki Şafak/Gündüz/Alacakaranlık/Gece etiketleri
  dört ayrı fiziksel faz değildir. Sınırlı çevresel iz/sinyal yayılımı
  (`SignalTrace`: `Footstep`, `Scent`, `Sound` yayılım ve sönümlenmesi) opt-in
  olarak uygulandı ve deterministik fizik testleriyle doğrulandı
  (`src/environment.rs`, `src/systems/perception.rs`,
  `tests/environment_signals.rs` — *Uygulandı / Mekanizma testi geçti, varsayılan kapalı*).
  P2 çekirdek motor entegrasyonuyla: mevsimsel iklim çevrimi (`Environment::season_cycle`,
  `SeasonCycle`: 4 mevsim, kış bitki rejenerasyon katsayısı, kış termal BMR ek maliyeti),
  kazılan inler (`Environment::burrows`, `BurrowSite` max 32, TTL bozulması, kış termal koruması)
  ve kademeli ceset çürümesi/leşçillik nişi (`EntityPhase::corpse_stage`, `CorpseStage`:
  `Fresh`/`Aging`/`Putrid`, leş kokusu yayılımı ve hepçil bozulma direnci) çekirdek motora
  uygulandı (`src/environment.rs`, `src/entity/lifestate.rs`). Ceset evreleri
  temel fiziksel yaşam döngüsüdür; hepsi için ortak bir default-off bayrağı yoktur.
  Wave 8 ve Wave 9 düzeltmeleriyle: haritadaki statik inler (`Cell::Burrow`) `Environment::from_map`
  ile metadata listesine kaydedilerek kış termal koruması sağlandı, otonom kış kazısında stamina
  harcaması kazının başarısına bağlandı ve `World::dig_burrow` / `Environment::dig_burrow_on_map` ile
  doğrudan metadata ekleyen `dig_burrow` ayrıldı; ceset enerjisi ilk tick'te (`remaining == 4`)
  korunup sonraki 3 tick boyunca (`remaining == 1..=3`) her tick 1 azalarak tabana indi.
  Disabled-path State+RNG denkliği ve deterministik checkpoint-resume doğrulandı.
- **Açık:** Mevsim ve kazılmış-in seçeneklerinin opt-in/default durumu cesetlerin
  temel fiziksel yaşam döngüsünden ayrıdır. Çoklu habitatlarda uzun dönemli ekolojik etkileri ve kalıcılık dinamikleri ampirik olarak test edilmemiştir
  (P3 5-seed held-out protokolü kısa yükle sınırlıdır).
- **Önerilen:** Çoklu harita topolojilerinde mevsim, in ve ceset dinamiklerinin ekolojik
  etkilerini ölçmek için önkayıtlı P3 deney protokollerini genişlet.
- **Kabul:** Çevre sinyali kaynak, yayılma, mesafe, zaman ve kaybolma olaylarıyla
  açıklanır; seedli fizik testleri ve State+RNG kabulü geçer. Çevrenin gizli
  sonraki durumu bireysel algıya sızmaz.

### <a id='canli-beden'></a>K2 — Canlı ve beden

- [V21 su öncesi bitişik öğün](WATER_MEAL_PRIORITY.md) varsayılan kapalı adaydır;
  yalnız ödenemeyen ilk su adımı yerine ücretsiz bitki öğününü seçer. Öğrenme
  atfı kesilir; dört kollu önkayıtlı kıyas, eski yem arama bütçesinden ayrı
  ve birlikte etkisini ölçer. Mekanizma testi genel ekolojik kabul değildir.

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
  Yavru fizyolojisi yardımcı kodu vardır; tek başına yavruya özel beden
  mekaniğinin World akışına entegre edildiğini göstermez. Ayrı default-off
  ebeveyn bakım seçeneği (`PolicySettings::juvenile_care_enabled`) fiziksel
  besleme/koruma yollarını `World` akışına bağlar; bellek aktarımı yapmaz
  (`src/policy.rs`, `src/world/mod.rs`, `src/systems/interaction.rs`).
  Wave 8 ve Wave 9 doğrulamalarıyla: kazma yeteneği olmayan canlılar için bitki örtüsü
  veya duvar/in bitişiğinde bekleme pusu örtüsü olarak kabul edildi ve pusu ilk vuruş
  bonusu (+5) bağlandı; sıfır besin tüketen canlılar (`amount == 0`) leş toksisitesinden
  muaf tutuldu.
- [Yem arama bütçesi](FOOD_SEARCH_RESERVE.md) V20'de varsayılan kapalı karar
  kısıtı olarak uygulandı; mekanizma testi geçti. [Dar lab sonucu](reports/FOOD_SEARCH_RESERVE_V1_20261026.md)
  kalıcı yaşam veya habitat genellemesi değildir.
- **Açık:** Ebeveyn bakımı ve yaralanma cezası varsayılan kapalıdır (`default-off`).
  Popülasyon düzeyinde uzun vadeli üreme ve sağkalım ödünleşimleri açık ampirik
  önkayıtlı deney bekler.
- **Önerilen:** Beden yaralanma ve ebeveyn bakımı dinamiklerini K1 çevre döngüsü
  ve K5 kaynak akışıyla bağlantılı olarak çok-nesilli P3 protokollerine bağla.
- **Kabul:** Hasar, iyileşme, enerji ve hareket sonuçları resolver/metabolizma
  olaylarından yeniden hesaplanır; tam/ölü/kesilmiş sonuçlar karıştırılmaz.
  Deterministik küçük fizik testleri geçer.

### <a id='bireysel-ogrenme'></a>K3 — Bireysel davranış ve öğrenme

- **Mevcut:** Bireye ait sınırlı kaynak/tehlike belleği, unutma, kurallı seçim ve
  temel su davranışı var. V17 su artığı varsayılan kapalı; formal utility
  **FALSE**. İlk-resolver hedefi için 623 exact gözlem var, fakat model yok ve
  tam kaçış hasarı `Unknown`. Hedefe doğrudan adım engellendiğinde yerel BFS
  baypası `Memory::toward` ile uygulandı (`tests/memory.rs` — *Uygulandı / Mekanizma testi geçti*).
  P2 çekirdek motor entegrasyonuyla: bağlam-uyumlu su artığı (`WaterLearner::context_ttl_enabled`,
  `context_entries`, `W=4`, `TTL=48`), ilk-aralık sığınak hasarı öğrenmesi
  (`Memory::first_interval_refuge_enabled`, `first_interval_outcomes`), mekânsal zihin haritası
  ve merak güdümlü keşif (`Memory::spatial_curiosity_enabled`, `known_sectors`, `4x4` ızgara,
  max 16, `TTL=120`, `curiosity_direction`) ve örtü tabanlı sığınak pusu avcılığı
  (`PolicySettings::ambush_predation_enabled`, `effective_detectability_radius` düşüşü & ilk
  vuruş bonusu) çekirdek motora default-off olarak entegre edildi (`src/entity/memory.rs`,
  `src/entity/lifestate.rs`, `src/policy.rs`).
  Wave 8 ve Wave 9 düzeltmeleriyle: `Learner::episode_physical_residual` formülü `foraging::value`
  ile tam hizalanarak `actual = (2*water - movement_energy - ticks) * 100 / ticks.max(1)` ve
  `residual = (actual - choice_baseline).clamp(-1000, 1000)` olarak güncellendi; baseline
  yokluğunda yapay artık üretilmez ve başarısız su bölümlerinde hedef kaynak türü aranan suyla
  eşleştiğinde (`missing_resource_target_matches == true`) artık kaydı yapılır.
  Tam-rota sığınak hasarı `Unknown` kalmaya devam eder; tamamlanan ilk-aralık sonucu sonraki
  tam-rota sansüründen bağımsızdır.
- [Su sinyali tanısı](reports/WATER_SIGNAL_DIAGNOSIS_20261026.md) mevcut V2
  fikstüründe artığın seçim sırasını değiştirmediğini buldu. [Kaynak güvenilirliği
  hedefi](SOURCE_RELIABILITY_TARGET_20261026.md) yalnızca öneridir: uygulanmış
  kod, çalıştırılmış protokol veya kabul edilmiş fayda değildir.
- **Açık:** Deneysel P2 öğrenme seçenekleri varsayılan kapalıdır (`default-off`); temel
  kurallı seçim ve bellek bundan ayrı tutulur. Tam-rota
  sığınak hasarı `Unknown` kalır. Bounded utility genel ampirik kabulü henüz yoktur.
- **Önerilen:** İlk-aralık hasar öğrenmesi ve su bağlamının ampirik karar kalitesini
  farklı habitatlarda sabit P3 protokolleriyle ölç.
- **Kabul:** Yalnız bireyin gördüğü/hatırladığı kanıt kullanılır. Hedef ufku
  içinde atıfsız, eksik veya sansürlü sonuç `Unknown` kalır; tamamlanmış ilk
  aralığın exact sonucu sonraki tam-rota sansüründen etkilenmez. Bireysel
  öğrenilmiş tablolar başkasına veya yavruya kopyalanmaz; K4 iletişimleri yalnız
  alıcının doğrulanmış yerel gözlemi olarak işlenebilir. Disabled-path
  eski/yeni State+RNG ve deterministic resume doğrulanır.

### <a id='grup-sosyal'></a>K4 — Grup ve sosyal ilişkiler

- **Mevcut:** Birden çok canlı ve tür aynı dünyada hareket eder; resolver
  çakışma, paylaşım ve etkileşimleri sıraya koyar. Görülen yetişkin türdeş/eş
  belleği ve seçenekle açılan buluşma politikası vardır. P2 çekirdek motor
  entegrasyonuyla: yerel alarm yayılımı (`PolicySettings::local_coordination_enabled`,
  `InteractionSystem::emit_local_alarm`, `SignalKind::Sound`) çekirdek motora
  default-off olarak entegre edildi (`src/policy.rs`, `src/systems/interaction.rs`).
- **Açık:** Genel sürü zekâsı veya telepatik grup optimizasyonu yoktur; yerel
  alarm varsayılan kapalıdır (`default-off`). Ekoloji aktarım taramasında alarm
  koşullarından birinde tüm popülasyon söndü; grup sağkalım avantajı kanıtlanmadı.
- **Önerilen:** Yalnız görünür yerel bilgiyle sınırlı yerel uyarı yayılımını
  K5/K6 avcı-av ekoloji protokollerinde test et.
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
  P2 entegrasyonuyla mevsimsel döngü, inler ve kademeli ceset çürümesi kapalı
  stok-akış muhasebesine bağlandı. P3 sabit 5-seed held-out protokolü (`control`,
  `learning`, `climate`, `social_care`, `all_features`) yürütüldü
  (`reports/P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md`).
- 26 Ekim ekoloji aktarım taraması üç habitat, üç seed ve beş kolu kapsadı
  (`reports/ECOLOGY_TRANSFER_V1_20261026.md`). Seyrek kaynak fikstüründeki tüm
  koşular söndü; karma habitatta etçiller tüm koşularda kayboldu ve alarm kolunda
  bir koşuda tüm popülasyon söndü. Çayırdaki kalıcılık genel özellik üstünlüğü
  göstermedi.
- **Açık:** Farklı habitatlarda kaynak akışı genellemesi, uzun vadeli popülasyon
  kalıcılığı ve şoklara dayanıklılık henüz test edilmemiştir; P3 protokolü
  yalnızca 5 seed'lik sabit fikstür ölçümüdür ve evrensel ekolojik üstünlük
  iddia etmez.
- **Önerilen:** C iziyle kaynak stok/yenilenme/tüketim ve beden enerji akışını
  farklı harita topolojilerinde test et.
- **Kabul:** Başlangıç, yenilenme, tüketim, tür akışı ve ödenen beden maliyeti
  kapanan bir muhasebeyle izlenir; kurucu/F1/F2 ve yavru üretmiş soy ayrı
  raporlanır. Habitat ve trait etkileri karıştırılmaz; held-out roller önceden
  sabitlenir.

### <a id='kalitim-evrim'></a>K6 — Kalıtım ve evrim

- **Mevcut:** Sonlu allel/özellik aktarımı ve mutasyon mekanizmaları opt-in
  olarak vardır; öğrenilmiş bellek yavruya aktarılmaz. Trait maliyeti ödemesi
  ile üreyen yavru soyu seçilim ayrımı deterministik testle doğrulandı
  (`tests/genetics.rs` — *Mekanizma testi geçti*). P2 entegrasyonunda pusu/takip
  ve leşçillik niş mekanizmaları ile ebeveyn bakımında sıfır bellek aktarımı
  kuralı korundu.
- **Açık:** Seçilim avantajı veya niş uzmanlaşması uzun dönemli ampirik olarak
  kabul edilmedi. Ekoloji aktarımında vision/stamina allelleri tüm kollarda
  kalıtıldı, yalnızca bir kolda ifade edildi; RNG hizalaması da korunmadı. Bu
  kol, trait seçilimi veya fitness üstünlüğü kanıtı değildir.
- **Önerilen:** Allel maliyetleri ve soy devamlılığı ayrımını uzun dönemli
  niş uzmanlaşması testlerine bağla.
- **Kabul:** Sonlu kalıtım ve mutasyon, bedelin ödenmesi, üreyen yavru-soyu ve
  allel frekansı ayrı kaydedilir. Held-out seed/roller önkayıtlıdır; öğrenilmiş
  deneyim kalıtılmaz. Sağkalım tek başına evrimsel üstünlük kanıtı değildir.

### <a id='gozlem-arayuz'></a>K7 — Gözlem ve belgesel arayüz

- **Mevcut:** Salt-okunur birey, karar, geçmiş ve soy ekranı var; F izi ilk
  belgesel deneyim arayüzüdür. Nedensellik zinciri (`causalChain`:
  `evidence -> action -> resolverOutcome -> memoryUpdate`, eksik halkalar açık
  `Unknown`), sığınak deneyim özeti (`CuriousExperience.refuge`) ve Auto-Director
  kamera (`autoDirector`: en yüksek öncelikli belgesel olayını otomatik odaklama)
  web arayüzlerine bağlandı (`web/documentary.js`, `web/experience.js`, `web/replay.html`,
  `web/replay.js`, `web/replay.css` — *Uygulandı / Mekanizma testi geçti*).
  Wave 10 geliştirmesiyle: `CuriousExperience.environment(state, individual)` ve `#env-telemetry`
  arayüz paneli bağlanarak mevsim döngüsü (`Spring/Summer/Autumn/Winter`), gün/gece fazı (`Dawn/Day/Dusk/Night`),
  aktif sinyal ve in sayıları, keşfedilen merak sektörleri, su bağlam kayıtları ve ceset tazelik evresi
  (`Fresh/Aging/Putrid`) arayüzde görselleştirildi.
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
  P2 çekirdek motor entegrasyonlarında disabled-path State+RNG denkliği ve
  deterministik checkpoint-resume doğrulandı. `ca5971b` denetimi sonrasında
  checkpoint sürümü `V19`'a yükseltildi; `State::restore` `1..=19` arası sürümleri
  kabul eder, eski V18 okuyucuların yeni alanları sessizce düşürmesi engellendi ve
  gelecek zaman damgalı girdiler reddedilir. Bağlam artığında `i64` toplamı ve
  `-1000..=1000` sınır kontrolü ile debug panikleri giderildi. P3/K8 throughput
  denetiminde kısa yük tick hızları ve sınırlandırılmış bellek yapıları
  (`SignalTrace <= 64`, `sites <= 8`, `dangers <= 8`, `known_sectors <= 16`,
  `burrows <= 32`) bağımsız `World::with_seed` yörüngeleriyle doğrulandı
  (`tests/k8_throughput_audit.rs`, `reports/AUDIT_CA5971B_REMEDIATION_20261025.md`).
  Wave 9 ve Wave 10 denetimleriyle: `tests/k8_throughput_audit.rs` içinde konfigürasyondan bağımsız
  `DynamicWorldSnapshot` ve RNG ayrışması karşılaştırması eklendi; `[101..105]` ve `[201..205]`
  tohum gruplarında Control vs Treatment kollarının 5 V19 mekanizmasında pozitif assert'lerle
  (`> 0`) çalıştığı doğrulandı. Fikstür başlangıç inleri filtrelenerek kış mevsiminde otonom
  yeni kazılan inler (`treatment_newly_excavated_burrows_total`) ve mükerrer sayım olmaksızın
  her tick yayımlanan benzersiz sinyaller (`treatment_unique_signals_emitted_total`) pozitif
  assert'lerle kanıtlandı (`reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md`,
  `reports/P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md`).
- 26 Ekim çevrimdışı operasyon probu, arşivlenmiş V12 checkpoint'ini V19 State'e
  çevirdi; State/RNG round-trip, iki SQLite shard bütünlük denetimi ve kapalı
  zstd arşiv okuması geçti (`reports/OPERATIONS_READINESS_20261026.md`). Bu
  sınırlı smoke canlı dağıtım, writer-lock veya davranışsal kabul değildir.
- **Açık:** Kısa yük sınırı ölçümleri genel performans/kapasite garantisi
  değildir. Aylar/yıllar soak ertelendi; bugün özellik çalışmasını bloke etmez.
- **Önerilen:** Farklı donanım ve yük profillerinde sınırlandırılmış tabloların
  ve checkpoint boyutlarının izlenmesi.
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
| **A — Güncel bağlamlı su artığı** | K3 | P2 kaynağında opt-in (`context_ttl_enabled`, `W=4`, `TTL=48`); V2 su labında ek öğrenme faydası kabul edilmedi. Kaynak güvenilirliği hedefi öneri aşamasındadır, uygulanmadı. | Ancak ayrı hipotez/protokol ve review ile karar kalitesi ölçümü; seçenek default-off kalır. |
| **B — Per-owner ilk-aralık hasar öğrenmesi** | K3 | P2 çekirdek motora opt-in (`first_interval_refuge_enabled`) entegre edildi (`tests/refuge_first_interval.rs`, `tests/refuge_learning.rs`); ilk aralık tam-rota sansüründen bağımsız; full-route `Unknown`. | Farklı avcı yoğunluklarında ilk-aralık kararlarının analizi; full-route hasar modeli açık. |
| **C — Kaynak/habitat/bedelli-trait fixture'ı** | K5 | Mevsim/in seçenekleri ve temel ceset evreleri stok-akış muhasebesinde yer alır; 26 Ekim aktarım taraması bazı koşullarda tam tükenme buldu, genelleme kabulü yoktur. | Çoklu habitat ve değişken çevre topolojilerinde uzun dönemli popülasyon dengesi denetimleri. |
| **D — Eylem ve yerel koordinasyon** | K3 eylemler; K4 sosyal koordinasyon | Merak, pusu ve alarm seçenekleri P2 kaynağında; `resolve_parental_care` default-off bakım yoludur. Yavru fiziolojisi yardımcıları tek başına entegre juvenile physiology değildir. | Avcı-av karşılaşmalarında pusu ve alarm dinamiklerinin grup düzeyinde incelenmesi. |
| **E — Sonlu allel varyasyonu/seçilim** | K6 | Allel maliyet ödemesi ve soy seçilimi ayrımı doğrulandı (`tests/genetics.rs`); pusu/takip ve çürüme/leşçillik niş mekanizmaları çekirdeğe bağlandı; bellek kalıtılmaz. | Farklı mevsim ve kaynak dağılımlarında allel dengesi ve soy kalıcılığı ölçümü. |
| **F — Bireysel deneyim arayüzü** | K7 | Sığınak deneyimi (`CuriousExperience.refuge`), nedensellik zinciri (`causalChain`) ve Auto-Director kamera (`autoDirector`) web arayüzlerine bağlandı (`web/documentary.js`, `web/experience.js`, `web/replay.html`, `web/replay.js`, `web/replay.css`). | Yeni K1–K6 olaylarının ve mevsim/in durumlarının belgesel arayüzüne eklenmesi. |
| **G — Kapasite/depolama/işletim** | K8 | K8 throughput denetimi ve sınırlı bellek tabloları doğrulandı; offline storage smoke teknik kontrolleri geçti, fakat p50/p95 özeti hatalı hesaplandığından bu yüzdelikler kullanılamaz. | Farklı donanım ve yük profillerinde checkpoint boyutu ve bellek kullanımının izlenmesi. |

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
   `tests/corpse_scavenging.rs`); K2 yavru fizyolojisi yardımcıları ve ebeveyn bakımı mekanizması (`tests/juvenile_parental_care.rs`)
   ile K4 yerel koordinasyon (`tests/local_coordination.rs`) dar mekanizma testlerinden geçti.
   Yavru yardımcılarının bulunması tek başına World entegrasyonu değildir.

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

### P2 — Sınırlı çekirdek entegrasyonu (Mekanizma testi geçti / Doğrulandı)

P0/P1 prototiplerinin bir bölümü modüler seçenekler olarak çekirdeğe bağlandı.
Default durumu özellik bazında değişir; ceset evreleri temel fiziksel yaşam döngüsüdür:
- K3 su bağlamı TTL (`context_entries <= 4`), ilk-aralık sığınak hasarı ve mekânsal merak (`known_sectors <= 16`).
- K1/K2/K5 mevsimsel iklim çevrimi (`SeasonCycle`), kazılan inler (`burrows <= 32`) ve kademeli ceset evreleri (`CorpseStage`).
- K2/K3/K4/K6 ebeveyn bakımı (`resolve_parental_care`, sıfır bellek aktarımı), sığınak pusu gizlenmesi ve yerel sesli alarm (`emit_local_alarm`).
- Bağımsız Wave 7–10 raporları belirli entegrasyon ve ölçüm bulgularını
  düzeltip yeniden denetledi; bunlar evrensel ekolojik fayda kabulü değildir.
  Ayrıntı [denetim raporlarında](reports/) yer alır.
- Disabled-path eski/yeni State+RNG denkliği, deterministik checkpoint-resume, bounded bellek yapıları ve tek-writer kuralı doğrulandı.
- K7 web deneyim ve replay arayüzü (`CuriousExperience.refuge`, `causalChain`, `autoDirector`) bağlandı.

### P3 — Ekoloji ve sınırlı karşılaştırma protokolleri (yürütüldü)

- Sabit 5-seed held-out protokolü (`control`, `learning`, `climate`, `social_care`, `all_features`)
  ve kısa yük K8 throughput denetimi her seed için bağımsız `World::with_seed` üzerinde
  `Control (Flags Off)` vs `Treatment (Flags On)` 80 tick yörüngeleriyle yürütüldü; Wave 9 ile
  konfigürasyondan bağımsız `DynamicWorldSnapshot` ve RNG ayrışması karşılaştırması ve iki grupta
  (`[101..105]` ve `[201..205]`) 5 V19 mekanizmasında pozitif assert'ler kanıtlandı
  (`tests/k8_throughput_audit.rs`, `reports/AUDIT_RECHECK_REMEDIATION_20261025.md`,
  `reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md`).
- Epistemik dürüstlük: Çekirdek entegrasyonu, State+RNG denkliği, checkpoint determinizmi ve
  sabit protokol tekrarlanabilirliği doğrulanmış; evrensel çok-habitatlı ekolojik üstünlük veya
  uzun süreli soak iddia edilmemiş / ertelenmiştir.
- Çoklu habitatlarda uzun vadeli popülasyon kalıcılığı ve şok dinamikleri için önkayıtlı
  protokoller sonraki aşamalar için tasarlanabilir.
- **Ekoloji transfer V1:** sabit protokolle 45 koşu ve 27,000 ana World tick
  tamamlandı; sonuçlar ayrı [tarihli rapordadır](reports/ECOLOGY_TRANSFER_V1_20261026.md).
  Seyrek kaynak koşularında tüm kollar çöktü; karma avcı fixture'ında etçiller
  hiçbir koşuda dönem sonuna kalmadı. Alarm kolundaki bir koşuda tüm türler
  söndü. Bu veri evrensel ekolojik üstünlük veya tercih edilen özellik kolu
  belirlemez.
- **Operasyonel hazırlık:** 120 ana + 1 storage-smoke World tick tamamlandı.
  Offline State/RNG ve SQLite/archive kontrolleri geçti; ham ölçümün p50/p95
  alanları düzeltilmemiş örnek sırasıyla hesaplandığı için geçersizdir. Ayrıntı
  [operasyonel raporda](reports/OPERATIONS_READINESS_20261026.md); promotion
  kapalıdır.

## Özgün 11 aşamanın kısa durumu

| # | Aşama | 26 Ekim durumu |
|---:|---|---|
| 1 | Avcı yavrusu ve soyunun devamı | Üreme/olgunlaşma mekanizması ve sınırlı coexistence kapısı var; genel kalıcılık kanıtlanmadı. |
| 2 | Av seçimi, maliyet, bırakma ve dinlenme | İlk kurallı politika var; risk öğrenmesi fayda göstermedi. |
| 3 | Karma ekosistem | Eski bounded coexistence gözlemi kendi fixture'ıyla sınırlı; yeni aktarım fikstüründe seyrek kaynakta çöküş ve avcı cohort kaybı görüldü. Genel habitat kabulü yok. |
| 4 | Hareket, duvar, sığınak, diyet, paylaşım ve çakışma | İlk resolver sözleşmeleri mevcut ve testli. |
| 5 | Sınırlı bireysel kaynak/tehlike belleği | Bounded bellek/unutma mekanizmaları var; genelleme kabul edilmedi. |
| 6 | Sonuçtan su öğrenmesi ve kontrol kolu | Mekanizma/formal ölçüm var; V17 bounded utility FALSE. |
| 7 | Bedelli kalıtsal hareket/trait ve mutasyon | İlk allel/bedel ölçümü var; aktarım taraması trait/fitness üstünlüğünü ayırmıyor; seçilim avantajı kabul edilmedi. |
| 8 | Birey, karar, geçmiş, bellek ve soy arayüzü | İlk salt-okunur belgesel ekran var; bütün bilişin kaydı değildir. |
| 9 | Gerçek gün/hafta dayanıklılığı | Sınırlı tick testleri var; uzun gerçek-zamanlı soak kullanıcı isteğiyle ertelendi. |
| 10 | Kurucu ve harita ölçek ölçümü | 10–1.000 kurucu ve farklı haritalarda ilk ölçüm araçları var. |
| 11 | İzleme, yedekli dağıtım ve bildirim | İşletim/monitoring var; uzun soak veya kapasite garantisi değildir. |

Yeni seed, lab, model eğitimi, live deployment veya ortak çekirdek entegrasyonu
bu belgeyle otomatik olarak yetkilendirilmez; ayrı review ve protokol gerekir.
Kategori bağlantıları: [K1](#dunya-cevre), [K2](#canli-beden),
[K3](#bireysel-ogrenme), [K4](#grup-sosyal), [K5](#ekoloji-etkilesim),
[K6](#kalitim-evrim), [K7](#gozlem-arayuz), [K8](#motor-isletim).
