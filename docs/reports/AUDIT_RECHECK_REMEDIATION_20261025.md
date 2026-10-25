# Wave 8: 2002ee4 yeniden denetim bulgularının (F1, F2, F5, F6, F8) tam kapanış raporu (25 Ekim 2026)

Birincil kategori: K8 Motor ve işletim; K1, K2, K3, K4, K5, K6 ve K7 ile kesişir.

## 1. Kapsam ve amaç

Bu rapor, `2002ee4` revizyonu üzerindeki bağımsız yeniden denetimde ([AUDIT_RECHECK_2002EE4_20261025](AUDIT_RECHECK_2002EE4_20261025.md)) tespit edilen kısmi bulguların (`F1`, `F2`, `F5`, `F6`, `F8`) ve ek `F3` doğrulama notunun tam kapanışını belgeler.

Önceki denetimde `F3` (V19 sürüm kapısı ve göç), `F4` (kış termal BMR ek maliyeti) ve `F7` (bağlam artığı i64 toplamı ve -1000..=1000 sınır doğrulaması) bulgularının başarıyla çözüldüğü doğrulanmıştı. Bu ek rapor, kalan bulguların kod ve test düzeyindeki tam çözümünü ve epistemik sınırlarını kayıt altına alır.

[AGENTS.md](../../AGENTS.md) gereğince önceki tüm tarihsel raporlar ([AUDIT_CA5971B_20261025](AUDIT_CA5971B_20261025.md), [AUDIT_CA5971B_REMEDIATION_20261025](AUDIT_CA5971B_REMEDIATION_20261025.md), [AUDIT_RECHECK_2002EE4_20261025](AUDIT_RECHECK_2002EE4_20261025.md), [P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025](P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)) dondurulmuş kanıt olarak **değiştirilmeden aynen korunmuştur**.

---

## 2. Kapanan denetim bulguları (F1, F2, F3, F5, F6, F8)

### F1 (Tamamlandı) — Su bağlam ölçek uyumu ve ardışık özdeş deneyim kaydı

- **Önceki durum:** Bağlam tablosuna yazılan bölüm ödülü (`water*10 - movement_energy*2 - ticks`), karar anındaki foraging temel skoruyla (`choice_baseline`) farklı fiziksel ölçekteydi; ayrıca `last_episode != prev_completed` filtresi nedeniyle aynı kaynakta üst üste tekrarlanan özdeş bölümler bağlam penceresine kaydedilmiyordu.
- **Kapanış çözümü:**
  - `Learner::episode_physical_residual` (`src/entity/learning.rs`) eklendi: Bağlam tablosuna yapay bölüm ödülü yerine, `foraging::water_score` ile birebir aynı fiziksel ölçekteki net artık (`(water - movement_energy - incoming_damage) - choice_baseline`, `[-1000, 1000]` aralığına sınırlandırılmış) kaydedilir. Karar anında `predict_context_residual` bu fiziksel artık değerini doğrudan baz skorla toplar.
  - `systems::water_outcomes::finish` (`src/systems/water_outcomes.rs`) içindeki `last_episode != prev_completed` engeli kaldırıldı. Canlının tamamladığı her su bölümü (`episode.is_none()` geçişi), aynı kaynakta ardışık özdeş sonuç verse dahi `context_entries` penceresine eksiksiz kaydedilir.
- **Doğrulama:** `tests/p2_learning_integration.rs` ve birim testleriyle ölçek uyumu ve ardışık kayıt doğrulandı.

### F2 (Tamamlandı) — Sıradan etçil ve hepçiller için erişilebilir pusu örtüsü

- **Önceki durum:** Pusu gizliliği ve +5 ilk vuruş hasar bonusu yalnızca `Cell::Burrow` hücresinde beklemeye bağlıydı; ancak `can_burrow() == false` olan sıradan `Carnivore` veya `Omnivore` türleri inleri kullanamadığından olağan kullanımda pusuya geçemiyordu.
- **Kapanış çözümü:**
  - `is_in_ambush_cover` (`src/systems/perception.rs` ve `src/systems/interaction.rs`) mantığı genişletildi: Yalnızca in değil, sıradan etçil ve hepçillerin doğal olarak bulunabildiği **bitki örtüsü (`Cell::Food`) üstünde veya duvar/in/bitki örtüsü (`Cell::Wall | Cell::Burrow | Cell::Food`) bitişiğindeki `Cell::Empty` hücresinde** tam stamina ile bekleme durumu pusu örtüsü sayıldı.
  - Pusu örtüsündeki birey `detectability_radius - 1` gizlilik avantajı kazanır ve ilk saldırısında +5 stamina/hasar bonusu uygular.
- **Doğrulama:** Normal `World::tick` üzerinde sıradan etçil bireyin bitki veya duvar bitişiğinde pusuya yatması ve ilk vuruş bonusunu uygulaması test edildi.

### F3 (Ek doğrulama notu tamamlandı) — Ham V18 JSON şeması göç testi

- **Önceki durum:** Sürüm kapısı kod düzeyinde `1..=19` ile korunmuştu, ancak testler gerçek V18 JSON metni yerine V19 State'in sürüm alanını indirgeyerek yapılmıştı.
- **Kapanış çözümü:**
  - `tests/persistence.rs` içine V19 alanlarını (`context_ttl_enabled`, `spatial_curiosity_enabled`, `first_interval_refuge_enabled`, `season_cycle`, `burrows`, `corpse_stage`) hiç içermeyen ham V18 JSON metninden (`"version": 18`) deserialization ve göçü sınayan `raw_v18_checkpoint_json_without_v19_keys_deserializes_and_migrates_to_v19` entegrasyon testi eklendi.
  - Test, eksik alanların hatasız şekilde `default-off` olarak başlatıldığını ve State'in başarıyla V19'a yükseltildiğini kanıtlar.

### F5 (Tamamlandı) — Dinamik in girdi doğrulaması ve otomatik kış kazı eylemi

- **Önceki durum:** `dig_burrow` geçersiz koordinat, dolu hücre veya `ttl=0` durumlarında da metadata ekleyebiliyordu; ayrıca dünya içinde otomatik in kazma eylemi bulunmuyordu.
- **Kapanış çözümü:**
  - `Environment::dig_burrow_on_map` ve `dig_burrow` (`src/environment.rs`): `ttl == 0`, harita sınırları dışı koordinat veya boş olmayan hücre (`Cell::Wall`, `Cell::Food`, `Cell::Water`) verildiğinde işlem atomik olarak reddedilir (`false` döner) ve metadata listesine kayıt eklenmez.
  - `World::tick` (`src/world/mod.rs`): Kış mevsiminde (`Season::Winter`), görüş alanında aktif in bulunmayan ve `can_burrow()` yeteneğine sahip bireyler (`Herbivore`), yeterli enerji ve stamina durumunda bulundukları `Cell::Empty` hücresinde 2 stamina harcayarak otomatik olarak fiziksel in (`Cell::Burrow` + `BurrowSite`) kazar.
- **Doğrulama:** Geçersiz girdi sınır testleri ve kış mevsiminde otonom kazı davranışının harita hücresiyle senkronizasyonu test edildi.

### F6 (Tamamlandı) — Her tick kademeli ceset enerji çürümesi ve sıfır tüketimde toksisite koruması

- **Önceki durum:** Ceset enerjisi sadece son tick'te (`remaining == 1`) azalıyordu ve sıfır enerji alınan ceset etkileşimlerinde de toksisite hasarı uygulanabiliyordu.
- **Kapanış çözümü:**
  - `LifeCycleSystem::begin_tick` (`src/systems/life_cycle.rs`): Ceset enerjisi (`corpse_energy`), `MIN_FOOD_FROM_CORPSE` tabanına kadar **her tick** düzenli olarak 1 birim azalacak şekilde kademeli çürümeye bağlandı.
  - `InteractionSystem::resolve_eats` (`src/systems/interaction.rs`): `scavenging_toxicity` hasarı `if amount > 0` şartına bağlandı. Enerjisi tam dolu olduğu için cesetten sıfır besin alan (`amount == 0`) birey toksisite hasarı almaz.
- **Doğrulama:** Kademeli enerji kaybı ve sıfır tüketimde hasar almama durumları fizik testleriyle doğrulandı.

### F8 (Tamamlandı) — 5 bağımsız seed üzerinde Control vs Treatment gerçek `World::tick` karşılaştırması

- **Önceki durum:** Çoklu seed testleri World yerine sentetik fixture üzerinde yürütülmüş, gerçek `World::tick` üzerinde bayrak açık/kapalı nedensel karşılaştırma yapılmamıştı.
- **Kapanış çözümü:**
  - `tests/k8_throughput_audit.rs` güncellendi: `[101..105]` ve `[201..205]` tohumlarının her biri için gerçek `World::with_seed` başlatılarak:
    - **Kol A (Control):** Tüm V19 deneysel bayrakları kapalı (`default-off`).
    - **Kol B (Treatment):** İlgili V19 bayrakları açık (`flags-on`).
  - 80 tick boyunca `world.tick()` çalıştırılarak karşılaştırıldı; her seed'de Treatment kolunun `known_sectors`, `context_entries`, `first_interval_outcomes`, `burrows` ve `signals` ürettiği ve Control kolundan nedensel olarak farklı `State + RNG` yörüngesi çizdiği exact assert'lerle doğrulandı.
- **Epistemik sınır:** Bu çalışma, mekanizmaların gerçek `World::tick` yürütme döngüsünde nedensel olarak etkin olduğunu ve çoklu seed'de deterministik çalıştığını kanıtlar; uzun süreli çok-habitatlı ekolojik üstünlük veya evrimsel adaptasyon iddiası taşımaz.

---

## 3. Sistem ve çalışma durumu

- **Geliştirme kaynağı:** Checkpoint `V19` / SQLite `V2`. Tüm yeni mekanizmalar varsayılan kapalıdır (`default-off`).
- **Canlı dünya:** `7882` portundaki canlı sistem V12 checkpoint üzerinde stabil olarak çalışmaya devam etmektedir; canlıya yükseltilmemiştir.
- **Replay ikilisi:** `7883` portundaki arşiv replay servisi dondurulmuştur.

## 4. İlgili belgeler ve bağlantılar

- [2002ee4 yeniden denetim raporu (Kısmi bulgular)](AUDIT_RECHECK_2002EE4_20261025.md)
- [ca5971b ilk entegrasyon denetim raporu](AUDIT_CA5971B_20261025.md)
- [Wave 7 düzeltme ve V19 doğrulama raporu](AUDIT_CA5971B_REMEDIATION_20261025.md)
- [P2/P3 ampirik protokol ve K8 denetimi (Tarihsel dondurulmuş rapor)](P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)
- [Dokümantasyon standardı](../DOCUMENTATION_STANDARD.md)
- [Öğrenme sözleşmesi](../LEARNING_NEXT.md)
- [Yol haritası](../ROADMAP.md)
