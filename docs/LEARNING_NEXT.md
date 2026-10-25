# Bireysel öğrenme: güncel sözleşme ve açık kanıt kapıları

Bu belge V19 geliştirme kaynağındaki öğrenme sınırlarını ve eldeki sonuçları
özetler. **V19 canlıya dağıtılmış değildir; canlı 7882 dünyası V12'dir.**
Yaş ödülü, yavruya deneyim aktarımı, genel amaçlı sinir ağı veya kabul edilmiş
refuge-risk modeli yoktur.

## Değişmeyen fiziksel sözleşme

- Öğrenme bireyin kendi gözlem ve kendi kimlikli çözücü deneyimine dayanır.
  Gizli beden, gelecekteki RNG, başka bireyin deneyimi veya henüz görünmeyen
  sonuç girdi yapılamaz. Bireyin belleği yavruya kopyalanmaz.
- Öğrenici yalnız mevcut kurallı kararın izin verdiği adayları sıralayabilir;
  fiziksel hareket, saldırı, kaynak tüketimi ve yaşam sonuçları World ve
  resolver tarafından belirlenir. Her ölçüm maliyeti ayrı fiziksel birimde
  raporlar; tek ödül artışı fayda kanıtı sayılmaz.
- Yalnız resolver ile tamamlandığı ve aynı bireye güvenilir biçimde
  ilişkilendirildiği bilinen hedef güncellenebilir. Tam kaçış sözleşmesinde
  ölüm, kesinti veya eksik atıf bölümü sansürler. Ayrı ilk-aralık sözleşmesinde
  tamamlanmış fiziksel aralık, sonraki kaçış sansüründen etkilenmez; seçilen
  tehdidin bilinen hasarı diğer saldırganın hasarından ayrı tutulur. Eksik
  gözlem sıfır hasarlı başarıya dönüştürülmez; bilinmeyen risk “güvenli” demek
  değildir.
- Varsayılan kapalı yol, mevcut politika seçimini ve RNG tüketimini aynen
  korumalıdır. Yeni checkpoint/State göçü, kapalı yol ve State/RNG devamı
  ayrı test edilir. Online ve frozen dalların tüm State/RNG alanlarının eşit
  olması beklenmez: frozen yalnız öğrenilmiş weights/counters güncellemelerini
  dondurur, normal exploration sürebilir. Tam State/RNG eşitliği yalnız
  default-off kaynak uyumu ve checkpoint-resume sözleşmesinde aranır.
  Bireysel tablolar bounded olmalı; sınır aşımında güncelleme atomik
  reddedilmelidir.
- Yaş için öğrenme ödülü veya tercih bonusu yoktur. Yaş ve hayatta kalma
  seçilimi analizde deneyim etkisinden ayrılır.

## V17 ve av öğrenmesi: sonuçlar sınırlı

### Su artığı — V17, default-off

V12 canlı dünyasındaki temel su öğrenmesi ve mevcut genetik yaşam mekaniği,
V17 su artığı gibi ayrı deneysel seçeneklerle karıştırılmaz. Su artığı eski su
puanına aynı bireyin kendi resolver-artığını ekleyebilir;
öğrenme/karar seçenekleri varsayılan kapalıdır. `WATER_RESIDUAL_V1` raporunda
mekanizma fırsatı sekiz seedin üçünde görüldü, fakat kayıtlı bounded-utility
kapısı geçilmedi. Karşılaştırıcılarla fiziksel tradeoff'lar karışıktır; kabul
edilmiş net yarar yoktur. Özelliği live'da açmayın veya sonuçtan sonra eşik
ayarlamayın.

### Av deneyimi/risk — V13/V14, default-off

V13 deneyim tablosu ve V14 risk alanı bounded mekanizma olarak mevcuttur;
checkpoint/migration ve resolver attribution sözleşmesine tabidir. Hunt Lab
V3'te riskli kollar savunma takibindeki yaralanmaları azaltmadı. Deneyim/risk
durumu, fiziksel veya ekolojik üstünlük kabulü değildir; risk seçimini kapalı
tutun. Tekil mekanizma tanıkları politika yararı ya da gerçek hayvan öğrenmesi
iddiası olmaz.

Raporlar: [su artığı V1](reports/WATER_RESIDUAL_V1.md) ve
[hunt lab V3](reports/HUNT_LAB_V3.md).

## Refuge: beklenen hasar hâlâ Unknown

V16 bireysel exact-key refuge sonuç belleği, V18 ise opt-in temsil edilmiş
bağlam seçeneği sunar. Her ikisi de default-off ve yalnız geliştirme
kaynağındadır. Önemli kapsam farkı:

- **Opt-in V18 refuge learner** geçerli alternatifleri bellekte en çok 8
  adayla sınırlar.
- **Mevcut ordinary default-off davranış** tüm geçerli, boş olmayan ve
  işgal edilmemiş sığınak yollarını dikkate alır; bu yola V18'in 8 aday
  sınırını taşımayın.
- Saf residual prototype kendi başına en çok 8 endpoint satırı tutar. Bu ayrı
  prototype sınırıdır; engine veya ordinary policy'ye entegre değildir.

25 Ekim tarihli yeni tek-run engineering ölçümü aynı fixture ve seed 123 ile
1,000 tick tamamladı: 623 attempt, 67 owner, 182 attributable arrival ve 441
censor. Tamamlanan 182 arrival'ın threat damage değeri sıfırdı; **altı pozitif
threat-damage attempt'in altısı da sansürlüydü.** Bu başarıya koşullu support,
expected damage kalibrasyonu sağlamaz. Beş owner-ID fold'undaki zero MAE bu
seçilmiş sıfır hedeflerinin sonucu ve bilgilendirici doğruluk metriği değildir.
Hareket enerjisi ve arrival süresi nominal hesapla 182/182 eşleşti; bu yalnız
koşullu effort aritmetiğini doğrular.

İlk resolver aralığının ayrı extraction'ında 623 başlatılmış bölümün 617'si
sıfır, altısı 20 hasar olarak gözlendi; altı pozitif satır dört owner'a aitti.
Bu **ilk-aralık gözlenen hedefidir**; tam kaçış beklenen hasarı, seçilmeyen
yolun karşıolgusu veya route predictor değildir. Full-route outcome raporunda
182 arrival sıfır hasarlıydı; altı pozitif bölüm daha sonra sansürlendi.
Bu sonraki kapanış sansürü, ilk aralığı eksiksiz gözlenen altı pozitif hedefi
silmez; eksik bir ilk aralık Exact değildir ve Unknown kalır.
Dolayısıyla her route için beklenen threat damage **Unknown** kalır.

Raporlar: [full-route risk baseline V1](reports/REFUGE_RISK_BASELINE_V1.md),
[ilk resolver aralığı](reports/REFUGE_FIRST_RESOLUTION_V1.md),
[önceki teknik onarım doğrulaması](reports/REFUGE_REPAIR_VALIDATION.md),
[ca5971b denetim raporu](reports/AUDIT_CA5971B_20261025.md),
[Wave 7 düzeltme ve V19 doğrulama](reports/AUDIT_CA5971B_REMEDIATION_20261025.md),
[2002ee4 yeniden denetimi](reports/AUDIT_RECHECK_2002EE4_20261025.md),
[Wave 8 tam kapanış](reports/AUDIT_RECHECK_REMEDIATION_20261025.md),
[6fa9c49 yeniden denetimi](reports/AUDIT_RECHECK_6FA9C49_20261025.md),
[Wave 9 kapanış ve hizalama](reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md),
[9c7d3f0 bağımsız kapanış kontrolü](reports/AUDIT_RECHECK_9C7D3F0_20261025.md),
[Wave 10 P3 ablasyon ve 9c7d3f0 kapanış raporu](reports/P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md) ve
[frozen refuge protokolü](REFUGE_FIRST_RESOLUTION_PROTOCOL.md).

## Pure residual ve öğrenme adaylarının durumu

Depoda geliştirilen saf mekanizma prototipleri, root incelemesinin ardından P2 aşamasında
motor çekirdeğine modüler ve **varsayılan kapalı (`default-off`)** olarak entegre edilmiştir.
Wave 7, Wave 8 ve Wave 9 denetimlerinde tespit edilen tüm eksikler ([AUDIT_CA5971B_20261025](reports/AUDIT_CA5971B_20261025.md),
[AUDIT_RECHECK_2002EE4_20261025](reports/AUDIT_RECHECK_2002EE4_20261025.md),
[AUDIT_RECHECK_6FA9C49_20261025](reports/AUDIT_RECHECK_6FA9C49_20261025.md)) giderilerek
mekanizmalar doğrudan aktif `World::tick` yürütme yoluna tam ölçek uyumu ve eksiksiz fizik kurallarıyla bağlanmıştır
(`Mekanizma ve World::tick entegrasyonu geçti / Doğrulandı`):

1. **Per-owner ilk-aralık sığınak hasarı öğrenmesi (İz B):**
   `Memory::first_interval_refuge_enabled` ile `first_interval_outcomes` tablosu çekirdeğe
   bağlandı (`tests/refuge_first_interval.rs`, `tests/refuge_learning.rs`). `refuge_outcomes::finish`
   içinde ilk resolver tick'inde (`elapsed_ticks == 0`) per-owner hasar atfı kaydedilir ve
   `choose_refuge_for_species` ile aday sıralamasına katılır. Tamamlanan ilk aralığın exact
   sonucu sonraki tam-rota sansüründen bağımsız kalır; tam rota için beklenen hasar **Unknown**
   kalmaya devam eder (genel rota tahmincisi değildir).
2. **Bağlam-uyumlu ve pencere/TTL kısıtlı su artığı (İz A):**
   `WaterLearner::context_ttl_enabled` ile güncel beden bağlamı ve zaman penceresi
   (`context_entries`, `W=4`, `TTL=48`) çekirdeğe bağlandı (`tests/water_context_residual.rs`).
   Wave 8 ve Wave 9 düzeltmeleriyle: `Learner::episode_physical_residual` formülü `foraging::value`
   ile birebir aynı net fiziksel artık ölçeğine hizalandı:
   `actual = (2 * water - movement_energy - ticks) * 100 / ticks.max(1)`
   `residual = (actual - choice_baseline).clamp(-1000, 1000)`
   Baseline yokluğunda (`choice_baseline == None`) yapay artık üretilmez (`None`).
   `water_outcomes::finish` içinde ardışık özdeş tamamlanmalar kaydedilir; `WaterEnd::Failed`
   durumunda ise yalnızca hedef kaynak türü aranan suyla eşleştiğinde (`missing_resource_target_matches == true`)
   ve baseline mevcut olduğunda artık kaydı yapılır. Genel ekolojik bounded utility ampirik kabulü değildir.
3. **Mekânsal zihin haritası ve merak güdümlü keşif (Yeni Özellik 1):**
   `Memory::spatial_curiosity_enabled` ile sınırlı 16 sektörlü `4x4` ızgara (`known_sectors <= 16`),
   `TTL=120` eskime ve `IntentCollectionSystem::collect` içinde `Reason::Explore` sırasında
   çağrılan merak vektörü (`curiosity_direction`) çekirdeğe bağlandı (`tests/spatial_curiosity.rs`).
4. **Ontogenetik yavru fizyolojisi ve ebeveyn bakımı (Yeni Özellik 2):**
   `PolicySettings::juvenile_care_enabled`, `InteractionSystem::resolve_attacks_with_policy`
   (yavruya gelen hasarı %50 azaltır, ebeveyn 4 stamina öder) ve `resolve_parental_care`
   ile fiziksel enerji aktarımı çekirdeğe bağlandı (`tests/juvenile_parental_care.rs`).
   **Sıfır öğrenilmiş bellek aktarımı kuralı korunmuştur; yavru belleği boş başlar.**
5. **Pusu / gizlenme vs. takip avcılığı (Yeni Özellik 3):**
   `PolicySettings::ambush_predation_enabled` ile pusu örtüsü sıradan etçil ve hepçillerin
   erişebildiği bitki örtüsü (`Cell::Food`) ve duvar/in/bitki bitişiğindeki boş hücrelere
   genişletildi (`is_in_ambush_cover`); `detectability_radius - 1` gizlilik avantajı ve
   `resolve_attacks_with_policy` içinde ilk vuruş +5 hasar bonusu çekirdeğe bağlandı (`tests/ambush_predation.rs`).
6. **Dinamik habitat (Yeni Özellik 4):**
   `Environment::burrows` ile kazılan inler (`BurrowSite` max 32, TTL bozulması, kış termal koruması)
   ve haritadaki fiziksel `Cell::Burrow` hücresi senkronize edildi. Wave 9 düzeltmesiyle:
   `Environment::from_map` haritada önceden var olan `Cell::Burrow` hücrelerini `MAX_BURROWS` sınırına kadar
   `ttl: 1_000_000` ile metadata listesine kaydeder ve bu hücrelerdeki canlılara kış termal koruması sağlar;
   otonom kış kazısında 2 stamina düşüşü yalnızca `dig_burrow_on_map` başarılı (`true`) olduğunda uygulanır;
   koordinat, hücre ve TTL doğrulayan yetkili `World::dig_burrow` / `Environment::dig_burrow_on_map` ile saf
   metadata kaydı yapan `Environment::dig_burrow` API ayrımı netleştirildi (`tests/dynamic_habitat.rs`).
7. **Mevsimsel iklim ve kış uykusu (Yeni Özellik 5):**
   `Environment::season_cycle` ile deterministik 4 mevsim çevrimi (`SeasonCycle`), kış bitki
   yenilenme çarpanı ve `World::tick` içinde çağrılan `apply_seasonal_thermal_costs` kış ek
   BMR maliyeti çekirdeğe bağlandı (`tests/seasonal_climate.rs`).
8. **Ceset çürümesi ve leşçillik nişi (Yeni Özellik 6):**
   `EntityPhase::corpse_stage` ile 5-tick ömürle uyumlu çürüme evreleri (`>= 4 CorpseStage::Fresh`,
   `>= 2 CorpseStage::Aging`, `_ CorpseStage::Putrid`). Wave 9 düzeltmesiyle: ölümün ilk tick'inde (`remaining == 4`, `CorpseStage::Fresh`)
   taze ceset besin paritesi için ceset enerjisi tam korunur; sonraki 3 tick boyunca (`remaining == 1..=3`,
   `CorpseStage::Aging` ve `CorpseStage::Putrid`) her tick 1 azalarak taban değere (`MIN_FOOD_FROM_CORPSE` = 5) iner;
   `resolve_eats` içinde sıfır enerji tüketen canlıları muaf tutan `scavenging_toxicity`
   (`Carnivore` 4 hasar, `Omnivore` 0 hasar) çekirdeğe bağlandı (`tests/corpse_scavenging.rs`).
9. **Yerel alarm yayılımı:**
   `PolicySettings::local_coordination_enabled` ve `World::tick` içinde çağrılan
   `InteractionSystem::emit_local_alarm` (`SignalKind::Sound` yayılımı) çekirdeğe bağlandı
   (`tests/local_coordination.rs`).

### Çekirdek entegrasyon sınırları ve garantileri

P2 entegrasyonu ve Wave 7–9 düzeltmeleri aşağıdaki mimari garantilerle korunmuştur:
- **Disabled-path exact State+RNG denkliği:** Özellik bayrakları kapalıyken simülasyon adımları,
  bellek ve RNG akışı eski motor durumuyla bit düzeyinde eşleşir.
- **V19 Checkpoint serileştirme ve uyumluluk:** Checkpoint sürümü `V19`'a yükseltilmiştir.
  `State::restore` `1..=19` arası sürümleri kabul eder; ham V18 JSON metninden göç testiyle
  eski kayıtların güvenle yükseltildiği kanıtlanmış, gelecek zaman damgaları reddedilmiştir.
- **Sert bellek sınırları (Bounded capacity):** `SignalTrace <= 64`, `sites <= 8`, `dangers <= 8`,
  `known_sectors <= 16`, `burrows <= 32`, `context_entries <= 4` sınırları zorunludur.
- **Sıfır bellek aktarımı:** Ebeveyn bakımı yalnızca fiziksel etkileşimdir; hiçbir öğrenilmiş
  deneyim yavruya kalıtılmaz. `InteractionSystem::resolve_mates` yeni doğan yavruya yalnızca
  ebeveynlerin bireysel yetenek bayraklarının açık olup olmadığını (`first_interval_refuge_enabled`,
  `spatial_curiosity_enabled`, `context_ttl_enabled`, `refuge_outcomes_enabled` — `&&` ile) aktarır;
  yavrunun tüm öğrenilmiş deneyim tabloları doğumda boş (`0`) başlar.
- **Epistemik sınır:** Tam rota hasarı `Unknown` kalır; mekanizma testleri, P3 sabit held-out
  ve çoklu-habitat tekil ablasyon protokolleri, 10-tohum × 150-tick davranışsal fayda (Pareto) elemesi
  veya gerçek V12 (`Tick 12639`) üzerinde yürütülen 1.000-tick (`Tick 12639 -> 13639`) izole gölge soak deneyi
  (`reports/AUDIT_RECHECK_REMEDIATION_20261025.md`,
  `reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md`, `reports/AUDIT_RECHECK_9C7D3F0_20261025.md`,
  `reports/P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md`,
  `reports/BEHAVIORAL_UTILITY_AND_SHADOW_QUALIFICATION_20261025.md`,
  `reports/LIVE_V12_SHADOW_1000_TICK_SOAK_20261025.md`)
  evrensel ekolojik üstünlük kanıtı değildir. Canlı V12 ortamında bu özellikler varsayılan kapalıdır (`default-off`).
- **Soak durumu:** Aylar/yıllar süren reliability doğrulaması kullanıcı isteğiyle ertelenmiştir;
  mevcut sağlık gözetimi sürer ve uzun soak feature geliştirme için önkoşul değildir.
