# Curious yol haritası

**Durum tarihi: 25 Ekim 2026.** Aşağıdaki sekiz kategori güncel durum, açık
sınırlar, önerilen çalışma ve kabul ölçütünü ayırır. Öneriler uygulandıkları
anlamına gelmez. P0–P3 çalışma sırası kategori yapısından ayrıdır.

## Sistem durumu

- **Canlı ve ana kaynak ayrıdır:** 7882'deki dünya V12 checkpoint üzerinde,
  yalnız arşiv bütçesi backportu `curious-observation-budget-8dc23de` ile
  çalışıyor. V19 ana kaynak canlıya yükseltilmedi. V17 su-artığı, av-risk
  öğrenmesi ve V19 deneysel seçenekleri varsayılan kapalı; bu, temel
  bellek/su davranışları veya canlı V12 genetiğinin tümünün kapalı olduğu
  anlamına gelmez. 7883 replay ikilisi dondurulmuştur.
- **P2 çekirdek motor entegrasyonları, Wave 7–8 düzeltmeleri ve Wave 9 tam kapanışları tamamlandı (default-off):**
  K3 öğrenme/bellek (su bağlamı net artık ölçeği `foraging::value` ile hizalandı, ilk-aralık sığınak hasarı, mekânsal merak),
  K1/K2/K5 çevre/faz (mevsim çevrimi, statik ve kazılan inler, otonom kış kazısı ve stamina koruması, ilk tick korumalı kademeli ceset çürümesi),
  K2/K3/K4/K6 politika/etkileşim (ebeveyn bakımı, erişilebilir bitki/duvar pusu örtüsü, sıfır tüketim toksisite muafiyeti,
  yerel alarm yayılımı) modüler ve varsayılan kapalı olarak çekirdeğe entegre edildi.
  `ca5971b`, `2002ee4` ve `6fa9c49` bağımsız denetimlerinde tespit edilen tüm eksiklikler giderildi:
  tüm özellikler aktif `World::tick` yürütme yoluna bağlandı, su bağlam ölçeği eşitlendi,
  pusu örtüsü sıradan avcılara genişletildi, statik harita inleri ve dinamik kazı metadata ile fiziksel haritaya
  senkronize edildi, ceset enerjisi çürüme evreleri ve toksisite kuralları doğrulandı, checkpoint sürümü `V19`'a
  yükseltilerek ham V18 JSON göçü doğrulandı; disabled-path State+RNG denkliği ve deterministik checkpoint-resume korundu.
- **K7 web replay ve experience UI entegrasyonu tamamlandı:** Sığınak deneyimi
  (`CuriousExperience.refuge`), nedensellik zinciri (`causalChain`: `Kanıt -> Eylem -> Resolver -> Bellek`)
  ve `#auto-director` odak paneli arayüze bağlandı.
- **P3 held-out ampirik protokolü ve K8 throughput denetimi yürütüldü:** 5 bağımsız seed'lik iki ayrı grup
  (`[101..105]` ve `[201..205]`) üzerinde konfigürasyondan bağımsız `DynamicWorldSnapshot` ve RNG ayrışmasıyla
  `Control (Flags Off)` vs `Treatment (Flags On)` gerçek `World::tick` karşılaştırması ve kısa yük throughput denetimi
  (`SignalTrace <= 64`, `sites <= 8`, `dangers <= 8`, `known_sectors <= 16`, `burrows <= 32`) tamamlandı; 5 V19 mekanizmasının
  tümü pozitif asserts (`> 0`) ile kanıtlandı.
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
  özelliğin önkoşulu değildir. Evrensel çok-habitatlı ekolojik üstünlük iddia edilmez.

Kanıt: [coexistence](reports/QUALIFIED_OBSERVATION.md),
[su-artığı](reports/WATER_RESIDUAL_V1.md),
[habitat vision](reports/VISION_HABITAT_V1.md),
[tam kaçış baseline'ı](reports/REFUGE_RISK_BASELINE_V1.md),
[ilk aralık](reports/REFUGE_FIRST_RESOLUTION_V1.md),
[canlı depolama backportu](reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md),
[ca5971b denetim raporu](reports/AUDIT_CA5971B_20261025.md),
[Wave 7 düzeltme ve V19 doğrulama](reports/AUDIT_CA5971B_REMEDIATION_20261025.md),
[2002ee4 yeniden denetimi](reports/AUDIT_RECHECK_2002EE4_20261025.md),
[Wave 8 tam kapanış](reports/AUDIT_RECHECK_REMEDIATION_20261025.md),
[6fa9c49 yeniden denetimi](reports/AUDIT_RECHECK_6FA9C49_20261025.md),
[Wave 9 tam kapanış ve hizalama](reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md),
[P2/P3 ampirik protokol ve K8 denetimi (Tarihsel dondurulmuş rapor)](reports/P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md).

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
  P2 çekirdek motor entegrasyonuyla: mevsimsel iklim çevrimi (`Environment::season_cycle`,
  `SeasonCycle`: 4 mevsim, kış bitki rejenerasyon katsayısı, kış termal BMR ek maliyeti),
  kazılan inler (`Environment::burrows`, `BurrowSite` max 32, TTL bozulması, kış termal koruması)
  ve kademeli ceset çürümesi/leşçillik nişi (`EntityPhase::corpse_stage`, `CorpseStage`:
  `Fresh`/`Aging`/`Putrid`, leş kokusu yayılımı ve hepçil bozulma direnci) çekirdek motora
  default-off olarak entegre edildi (`src/environment.rs`, `src/entity/lifestate.rs`).
  Wave 8 ve Wave 9 düzeltmeleriyle: haritadaki statik inler (`Cell::Burrow`) `Environment::from_map`
  ile metadata listesine kaydedilerek kış termal koruması sağlandı, otonom kış kazısında stamina
  harcaması kazının başarısına bağlandı ve `World::dig_burrow` / `Environment::dig_burrow_on_map` ile
  doğrudan metadata ekleyen `dig_burrow` ayrıldı; ceset enerjisi ilk tick'te (`remaining == 4`)
  korunup sonraki 3 tick boyunca (`remaining == 1..=3`) her tick 1 azalarak tabana indi.
  Disabled-path State+RNG denkliği ve deterministik checkpoint-resume doğrulandı.
- **Açık:** P2 çevre dinamikleri varsayılan kapalıdır (`default-off`). Çoklu habitatlarda
  uzun dönemli ekolojik etkileri ve kalıcılık dinamikleri ampirik olarak test edilmemiştir
  (P3 5-seed held-out protokolü kısa yükle sınırlıdır).
- **Önerilen:** Çoklu harita topolojilerinde mevsim, in ve ceset dinamiklerinin ekolojik
  etkilerini ölçmek için önkayıtlı P3 deney protokollerini genişlet.
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
  P2 çekirdek motor entegrasyonuyla: ontogenetik yavru fizyolojisi ve ebeveyn bakımı
  (`PolicySettings::juvenile_care_enabled`, `InteractionSystem::resolve_parental_care`,
  `parent_guards_juvenile`: besleme enerji aktarımı, koruma hasar engelleme, sıfır
  öğrenilmiş bellek aktarımı) çekirdek motora default-off olarak entegre edildi
  (`src/policy.rs`, `src/systems/interaction.rs`).
  Wave 8 ve Wave 9 doğrulamalarıyla: kazma yeteneği olmayan canlılar için bitki örtüsü
  veya duvar/in bitişiğinde bekleme pusu örtüsü olarak kabul edildi ve pusu ilk vuruş
  bonusu (+5) bağlandı; sıfır besin tüketen canlılar (`amount == 0`) leş toksisitesinden
  muaf tutuldu.
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
- **Açık:** P2 öğrenme ve bellek modelleri varsayılan kapalıdır (`default-off`). Tam-rota
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
  alarm varsayılan kapalıdır (`default-off`). Grup düzeyinde ampirik sağkalım
  avantajı henüz kanıtlanmamıştır.
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
  kabul edilmedi; P3 sabit held-out deneyi kısa yüklü karşılaştırmadır.
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
  Wave 9 denetimiyle: `tests/k8_throughput_audit.rs` içinde konfigürasyondan bağımsız
  `DynamicWorldSnapshot` ve RNG ayrışması karşılaştırması eklendi; `[101..105]` ve `[201..205]`
  tohum gruplarında Control vs Treatment kollarının 5 V19 mekanizmasında pozitif assert'lerle
  (`> 0`) çalıştığı doğrulandı (`reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md`).
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
| **A — Güncel bağlamlı su artığı** | K3 | P2 çekirdek motora opt-in (`context_ttl_enabled`, `W=4`, `TTL=48`) entegre edildi (`tests/water_context_residual.rs`); disabled-path State+RNG denkliği korundu. | Farklı su kıtlığı koşullarında karar kalitesi denetimi; varsayılan kapalı kalır. |
| **B — Per-owner ilk-aralık hasar öğrenmesi** | K3 | P2 çekirdek motora opt-in (`first_interval_refuge_enabled`) entegre edildi (`tests/refuge_first_interval.rs`, `tests/refuge_learning.rs`); ilk aralık tam-rota sansüründen bağımsız; full-route `Unknown`. | Farklı avcı yoğunluklarında ilk-aralık kararlarının analizi; full-route hasar modeli açık. |
| **C — Kaynak/habitat/bedelli-trait fixture'ı** | K5 | Mevsimler (`SeasonCycle`), inler (`burrows`) ve kademeli ceset evreleri (`CorpseStage`) P2 kapalı stok-akış muhasebesine entegre edildi; P3 5-seed held-out protokolü yürütüldü (`reports/P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md`). | Çoklu habitat ve değişken çevre topolojilerinde uzun dönemli popülasyon dengesi denetimleri. |
| **D — Eylem ve yerel koordinasyon** | K3 eylemler; K4 sosyal koordinasyon | Mekânsal zihin haritası (`known_sectors <= 16`), pusu örtüsü/stamina bonusu ve yerel sesli alarm (`emit_local_alarm`) P2 çekirdeğe entegre edildi; sıfır bellek aktarımıyla ebeveyn bakımı (`resolve_parental_care`) bağlandı. | Avcı-av karşılaşmalarında pusu ve alarm dinamiklerinin grup düzeyinde incelenmesi. |
| **E — Sonlu allel varyasyonu/seçilim** | K6 | Allel maliyet ödemesi ve soy seçilimi ayrımı doğrulandı (`tests/genetics.rs`); pusu/takip ve çürüme/leşçillik niş mekanizmaları çekirdeğe bağlandı; bellek kalıtılmaz. | Farklı mevsim ve kaynak dağılımlarında allel dengesi ve soy kalıcılığı ölçümü. |
| **F — Bireysel deneyim arayüzü** | K7 | Sığınak deneyimi (`CuriousExperience.refuge`), nedensellik zinciri (`causalChain`) ve Auto-Director kamera (`autoDirector`) web arayüzlerine bağlandı (`web/documentary.js`, `web/experience.js`, `web/replay.html`, `web/replay.js`, `web/replay.css`). | Yeni K1–K6 olaylarının ve mevsim/in durumlarının belgesel arayüzüne eklenmesi. |
| **G — Kapasite/depolama/işletim** | K8 | K8 throughput denetimi ve sınırlı bellek tabloları (`SignalTrace <= 64`, `sites <= 8`, `dangers <= 8`, `known_sectors <= 16`, `burrows <= 32`) doğrulandı (`tests/k8_throughput_audit.rs`). Disabled-path State+RNG ve resume determinizmi korundu. | Farklı donanım ve yük profillerinde checkpoint boyutu ve bellek kullanımının izlenmesi. |

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

### P2 — Sınırlı çekirdek entegrasyonu (Mekanizma testi geçti / Doğrulandı)

P0/P1 prototipleri root incelemesinden sonra modüler ve varsayılan kapalı (`default-off`)
olarak motor çekirdeğine entegre edildi:
- K3 su bağlamı TTL (`context_entries <= 4`), ilk-aralık sığınak hasarı ve mekânsal merak (`known_sectors <= 16`).
- K1/K2/K5 mevsimsel iklim çevrimi (`SeasonCycle`), kazılan inler (`burrows <= 32`) ve kademeli ceset evreleri (`CorpseStage`).
- K2/K3/K4/K6 ebeveyn bakımı (`resolve_parental_care`, sıfır bellek aktarımı), sığınak pusu gizlenmesi ve yerel sesli alarm (`emit_local_alarm`).
- Wave 7, Wave 8 ve Wave 9 denetim düzeltmeleri: `ca5971b`, `2002ee4` ve `6fa9c49` denetim bulguları (F1–F8) kapatıldı; su bağlamı `foraging::value` ölçeğine tam eşitlendi, baseline yokluğunda yapay artık üretilmedi ve başarısız bölümlerde hedef kaynak türü aranan suyla eşleştiğinde artık kaydı bağlandı, pusu örtüsü sıradan avcılara genişletildi, haritadaki statik inler metadata listesine kaydedilip kış koruması sağlandı, otonom kazıda stamina korundu, ceset enerjisi çürümesi ilk tick (`remaining == 4`) korunup sonraki 3 tick boyunca kademeli azaltıldı ve sıfır tüketim toksisite muafiyeti sağlandı, checkpoint sürümü `V19`'a yükseltildi ve ham V18 JSON şeması göçü doğrulandı.
- Disabled-path eski/yeni State+RNG denkliği, deterministik checkpoint-resume, bounded bellek yapıları ve tek-writer kuralı doğrulandı.
- K7 web deneyim ve replay arayüzü (`CuriousExperience.refuge`, `causalChain`, `autoDirector`) bağlandı.

### P3 — Ekoloji ve held-out protokolü (Yürütüldü / Denetlendi)

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
