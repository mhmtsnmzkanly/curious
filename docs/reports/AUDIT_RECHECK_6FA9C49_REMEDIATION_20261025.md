# Wave 9: 6fa9c49 yeniden denetim bulgularının kapatılması ve tam hizalama raporu (25 Ekim 2026)

Birincil kategori: K8 Motor ve işletim; K1, K2, K3, K5 ile kesişir.

## 1. Kapsam ve amaç

Bu rapor, `6fa9c49` revizyonu üzerindeki bağımsız denetimde ([AUDIT_RECHECK_6FA9C49_20261025](AUDIT_RECHECK_6FA9C49_20261025.md)) tespit edilen açık bulguların (`P1 / F1`, `P2 / F8`) ve ek sınırların (`F5`, `F6`) tam teknik kapanışını ve kaynak kod ile proje dokümantasyonunun eksiksiz hizalanmasını belgeler.

[AGENTS.md](../../AGENTS.md) ve [dokümantasyon standardı](../DOCUMENTATION_STANDARD.md) gereğince önceki tüm tarihsel raporlar ([AUDIT_CA5971B_20261025](AUDIT_CA5971B_20261025.md), [AUDIT_CA5971B_REMEDIATION_20261025](AUDIT_CA5971B_REMEDIATION_20261025.md), [AUDIT_RECHECK_2002EE4_20261025](AUDIT_RECHECK_2002EE4_20261025.md), [AUDIT_RECHECK_REMEDIATION_20261025](AUDIT_RECHECK_REMEDIATION_20261025.md), [AUDIT_RECHECK_6FA9C49_20261025](AUDIT_RECHECK_6FA9C49_20261025.md), [P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025](P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)) dondurulmuş kanıt olarak **değiştirilmeden aynen korunmuştur**.

---

## 2. Denetim bulgularının kapanışı ve teknik ayrıntılar

### P1 / F1 — Su bağlam artığının `foraging::value` ölçeğiyle tam hizalanması

- **Tespit edilen açık:** Önceki `episode_physical_residual` formülü bölüm sonucunu net kaynak bazında (`water - movement_energy - incoming_damage - baseline`) hesaplarken, karar mekanizmasındaki `foraging::value` temel skoru `(2 * water - movement_energy - ticks) * 100 / ticks` ölçeğinde tutuluyordu. Farklı ölçeklerdeki değerlerin çıkarılması ve tekrar temel skora eklenmesi hatalı artıklar üretiyordu (ör. 10 su, 0 hareket, 1 tick, baseline 1900 için gerçekleşen skor 1900 iken artık 0 yerine -1000'e kırpılıyordu). Ayrıca baseline olmadığında 0 varsayılıyor ve gelen hasar nedensel su maliyeti gibi işlem görüyordu.
- **Kapanış çözümü (`src/entity/learning.rs`, `src/systems/water_outcomes.rs`):**
  1. `Learner::episode_physical_residual` doğrudan `foraging::value` ve legacy residual (`learning.rs:485-491`) ile birebir aynı ölçekte hesaplanacak şekilde güncellendi:
     $$\text{actual} = \frac{(2 \times \text{water} - \text{movement\_energy} - \text{ticks}) \times 100}{\max(1, \text{ticks})}$$
     $$\text{residual} = (\text{actual} - \text{baseline}).\text{clamp}(-1000, 1000)$$
  2. Denetim aritmetiği örneğinde: 10 su, 0 hareket, 1 tick ve `choice_baseline = Some(1900)` için `actual = 1900` ve net artık $\text{residual} = 0$ üretilir (tam beklentiyle örtüşen deneyimde sıfır kalıntı).
  3. `choice_baseline == None` durumunda `None` dönülerek kanıtsız/ilişkilendirilmemiş durumlar için yapay artık üretimi engellendi.
  4. `incoming_damage` (saldırı hasarı), su arama seçiminin nedensel maliyeti olarak net skordan düşülmez; `learning.rs` sözleşmesi gereği bağlamsal bilgi olarak korunur.
  5. `systems::water_outcomes::finish` içinde `WaterEnd::Failed` durumu için yalnız `missing_resource_target_matches == true` olduğunda bağlam gözlemi kaydedilir (doğrulanmamış başarısız denemeler elenir).

### P2 / F8 — Konfigürasyondan bağımsız dinamik durum, RNG ve 5 mekanizmalı çoklu-seed doğrulaması

- **Tespit edilen açık:** Önceki testte kontrol ve treatment dünyaları checkpoint JSON metni üzerinden karşılaştırılıyordu; bayraklar JSON metaverisinde yer aldığından simülasyon dinamiklerinde fark oluşmasa dahi `assert_ne!` geçebiliyordu. Ayrıca RNG karşılaştırılmıyor ve 5 V19 mekanizmasının treatment kolunda olay ürettiği zorunlu tutulmuyordu.
- **Kapanış çözümü (`tests/k8_throughput_audit.rs`):**
  1. `run_flag_on_vs_flag_off_comparison` karşılaştırması, yapılandırma bayraklarını dışarıda bırakan **konfigürasyondan bağımsız dinamik dünya özetini** (`DynamicWorldSnapshot`: canlıların konum, faz, enerji, su, sağlık, stamina, nesil durumları ve harita hücreleri) temel alacak şekilde yeniden yapılandırıldı.
  2. Simülasyon sonunda kontrol ve treatment dünyalarının RNG durumlarının farklılaştığı açıkça doğrulandı (`control_world.rng.state() != treatment_world.rng.state()`).
  3. Her iki bağımsız 5-seed grubunda (`[101..105]` ve `[201..205]`), her bir seed için 80 tick'lik gerçek `world.tick()` koşusunda 5 V19 mekanizmasının tamamı pozitif assert'lerle zorunlu tutuldu:
     - `treatment_known_sectors_total > 0` (mekânsal merak)
     - `treatment_context_entries_total > 0` (su bağlam artığı)
     - `treatment_first_interval_outcomes_total > 0` (ilk çözücü aralığı hasar öğrenimi)
     - `treatment_signals_emitted_total > 0` (yerel sesli alarm)
     - `treatment_burrows_total > 0` (otonom in kazımı)
  4. **Epistemik sınır:** Bu doğrulama, V19 mekanizmalarının gerçek `World::tick` yürütme döngüsünde nedensel olarak etkin olduğunu ve çoklu seed'de durum/RNG ayrışması ürettiğini kanıtlar; uzun dönemli çok-habitatlı ekolojik üstünlük iddiası taşımaz.

### F5 — Statik inlerin metadata kaydı, başarısız kazıda stamina koruması ve API ayrımı

- **Tespit edilen açık:** `Environment::from_map` statik harita inlerini `burrows` listesine almadığı için haritadaki inler kış termal koruması sağlamıyordu; otonom kazıda kapasite doluyken başarısız kazı stamina kesiyordu; harita doğrulayan API ile metadata-only API ayrımı belgelerde net değildi.
- **Kapanış çözümü (`src/environment.rs`, `src/world/mod.rs`):**
  1. `Environment::from_map` artık haritadaki mevcut `Cell::Burrow` hücrelerini `burrows` listesine (`ttl: 1_000_000`) kaydeder. Statik in üzerinde bulunan canlılar da kış mevsiminde termal korumadan (`winter_bmr_surcharge == 0`) yararlanır.
  2. Otonom kış kazısında 2 stamina yalnızca `dig_burrow_on_map` başarılı olduğunda (`true`) kesilir; `MAX_BURROWS` (32) kapasitesi doluyken başarısız girişimde canlı stamina kaybetmez.
  3. **Belgeleme ayrımı:** Harita koordinatı, yürünebilirlik, `ttl > 0` ve metadata atomik senkronizasyonunu sağlayan yetkili API `World::dig_burrow` / `Environment::dig_burrow_on_map` olarak belgelenmiştir. Harita referansı almayan alt-seviye `Environment::dig_burrow` fonksiyonu yalnızca metadata listesini yönetir.

### F6 — Ceset enerji çürümesi belge–davranış uyumu

- **Tespit edilen açık:** Belgelerdeki "her tick kademeli azalma" ifadesi kaynak kod davranışı ile birebir uyuşmuyordu (ilk tick'te azalma olmuyordu).
- **Kapanış çözümü (`src/systems/life_cycle.rs`):**
  - Kaynak kod davranışı ve belgeleme tam hizalandı:
    - Ölümü izleyen ilk tick'teki `CorpseStage::Fresh` (`remaining == 4`) evresinde, avcı/leşçil için başlangıç besin paritesini korumak amacıyla `corpse_energy` değeri tam tutulur.
    - İzleyen `CorpseStage::Aging` (`remaining == 2..=3`) ve `CorpseStage::Rotten` (`remaining == 1`) evrelerindeki 3 tick boyunca (`remaining` `1..=3` iken) `corpse_energy` her tick 1 birim azalarak `MIN_FOOD_FROM_CORPSE` (5) tabanına iner.
    - Toksisite muafiyeti korunmuştur: Cesetten sıfır besin alan (`amount == 0`) canlılar toksisite hasarı almaz.

---

## 3. Sistem durumu ve çalışma ayrımı

- **Geliştirme kaynağı:** Checkpoint `V19` / SQLite `V2`. Tüm yeni V19 özellikleri varsayılan kapalıdır (`default-off`).
- **Canlı dünya:** `7882` portundaki canlı sistem V12 checkpoint üzerinde stabil olarak çalışmaktadır; canlıya yükseltilmemiştir.
- **Replay ikilisi:** `7883` portundaki arşiv replay servisi dondurulmuştur.

## 4. İlgili belgeler ve bağlantılar

- [6fa9c49 yeniden denetim raporu (Açık bulgular)](AUDIT_RECHECK_6FA9C49_20261025.md)
- [2002ee4 yeniden denetim raporu](AUDIT_RECHECK_2002EE4_20261025.md)
- [Wave 8 tam kapanış raporu](AUDIT_RECHECK_REMEDIATION_20261025.md)
- [ca5971b ilk entegrasyon denetim raporu](AUDIT_CA5971B_20261025.md)
- [Wave 7 düzeltme ve V19 doğrulama raporu](AUDIT_CA5971B_REMEDIATION_20261025.md)
- [P2/P3 ampirik protokol ve K8 denetimi (Tarihsel dondurulmuş rapor)](P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)
- [Dokümantasyon standardı](../DOCUMENTATION_STANDARD.md)
- [Öğrenme sözleşmesi](../LEARNING_NEXT.md)
- [Yol haritası](../ROADMAP.md)
