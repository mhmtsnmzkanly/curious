# Wave 10: P3 ablasyon ve 9c7d3f0 kanıt sınırı kapanış raporu (25 Ekim 2026)

Birincil kategori: K8 Motor ve işletim; K1, K2, K3, K5, K7 ile kesişir.

## 1. Kapsam ve amaç

Bu rapor, `95a8629` revizyonu üzerindeki bağımsız denetimde ([AUDIT_RECHECK_9C7D3F0_20261025](AUDIT_RECHECK_9C7D3F0_20261025.md)) tespit edilen dar kanıt sınırlarının kapatılmasını, `CorpseStage::Putrid` adlandırma hizalamasını, P3 çoklu-habitat tekil-özellik ablasyon matrisini ve K7 web replay/experience V19 çevre ve beden telemetrisi entegrasyonunu belgeler.

[AGENTS.md](../../AGENTS.md) ve [dokümantasyon standardı](../DOCUMENTATION_STANDARD.md) gereğince önceki tüm tarihsel raporlar ([AUDIT_CA5971B_20261025](AUDIT_CA5971B_20261025.md), [AUDIT_CA5971B_REMEDIATION_20261025](AUDIT_CA5971B_REMEDIATION_20261025.md), [AUDIT_RECHECK_2002EE4_20261025](AUDIT_RECHECK_2002EE4_20261025.md), [AUDIT_RECHECK_REMEDIATION_20261025](AUDIT_RECHECK_REMEDIATION_20261025.md), [AUDIT_RECHECK_6FA9C49_20261025](AUDIT_RECHECK_6FA9C49_20261025.md), [AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025](AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md), [AUDIT_RECHECK_9C7D3F0_20261025](AUDIT_RECHECK_9C7D3F0_20261025.md), [P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025](P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)) dondurulmuş kanıt olarak **değiştirilmeden aynen korunmuştur**.

---

## 2. Bölüm 1 — 9c7d3f0 denetimindeki dar kanıt sınırlarının kapatılması

`AUDIT_RECHECK_9C7D3F0_20261025.md` raporunda `tests/k8_throughput_audit.rs` için işaret edilen iki dar kanıt sınırı ve bir adlandırma kusuru eksiksiz giderilmiştir:

### 2.1. Otonom olarak yeni kazılan inlerin kesin ayrımı (`treatment_newly_excavated_burrows_total`)

- **Önceki sınır:** `treatment_burrows_total`, simülasyon boyunca `treatment_world.environment.burrows.len()` değerinin maksimumunu alıyordu. Fikstür başlangıçta iki adet statik in hücresi (`Cell::Burrow`) yerleştirdiği için ve `World` inde duran canlılar için metadata kaydı açabildiği için, pozitif in sayısı canlıların kış mevsiminde otonom olarak *yeni bir in kazdığını* tek başına kanıtlamaya yetmiyordu.
- **Kapanış çözümü (`tests/k8_throughput_audit.rs`):**
  1. Başlangıçta haritaya statik olarak yerleştirilen 2 in koordinatı (`(-7+ox, -6+oy)` ve `(-6+ox, -7+oy)`) başlangıç filtresine alınmıştır.
  2. Yalnızca `World::tick` esnasında kış mevsiminde `Herbivore` bireyleri tarafından otonom olarak yeni kazılan in koordinatları (`treatment_newly_excavated_burrows_total`) takip edilir.
  3. `[101..105]` ve `[201..205]` tohum gruplarının tamamında `treatment_newly_excavated_burrows_total > 0` assert'ü pozitif olarak doğrulanmıştır. Böylece statik inlerin varlığı veya pasif metadata kaydı sayacı artıramaz; canlıların kış koşullarında otonom in kazma eylemini fiilen yürüttüğü kanıtlanmıştır.

### 2.2. Benzersiz yayımlanan sinyallerin mükerrer sayım olmaksızın ölçülmesi (`treatment_unique_signals_emitted_total`)

- **Önceki sınır:** Sinyal sayacı her tick mevcut `environment.signals.len()` değerini topluyordu. TTL süresince havada kalan sinyaller birden çok tick boyunca toplanabildiği için toplam yayım adedini değil, varlık süresini yansıtıyordu.
- **Kapanış çözümü (`tests/k8_throughput_audit.rs`):**
  1. Her tick sonunda yalnızca yayım zamanı mevcut tick'e eşit olan (`signal.emitted_tick == world.tick_counter`) yeni üretilmiş sinyaller toplanır (`treatment_unique_signals_emitted_total`).
  2. Sinyallerin mükerrer sayımı engellenmiş ve her iki 5-seed grubunda (`[101..105]` ve `[201..205]`) benzersiz üretilen sinyal sayısının `> 0` olduğu doğrulanmıştır.

### 2.3. `CorpseStage::Putrid` Rust enum varyantı hizalaması

- Rust kaynak kodunda (`src/entity/phase.rs`) tanımlı gerçek enum varyantları:
  ```rust
  pub enum CorpseStage {
      Fresh,
      Aging,
      Putrid,
  }
  ```
- Dokümantasyondaki bazı tarihsel geçişlerde yer alan `Rotten` yazımı kaynak kodla uyumlu hale getirilerek tüm güncel belgelerde `CorpseStage::Putrid` olarak hizalanmıştır.

---

## 3. Bölüm 2 — P3 tekil-özellik ablasyon ve çoklu-habitat kıyaslaması

`tests/p3_multi_habitat_ablation.rs` kapsamında yürütülen P3 deney protokolü, V19 mekanizmalarının tekil nedensel izolasyonunu ve farklı arazi topolojilerindeki seçiciliğini doğrular.

### 3.1. Deney matrisi mimarisi

- **3 Habitat Topolojisi:**
  1. `OpenPlains`: Düşük duvar yoğunluğu, geniş hareket alanı, homojen kaynak dağılımı.
  2. `CorridorMaze`: Yüksek duvar yoğunluğu, dar koridorlar, pusu ve yerel kaçış odaklı arazi.
  3. `SeasonalScarcity`: Kıt kaynaklar, kış mevsiminde sıfıra yakın besin yenilenmesi, in kazımı ve kış uykusu baskısı.
- **5 Bağımsız Tohum:** `301..=305` (daha önce audit'te kullanılmamış held-out tohumlar).
- **7 Ablasyon Kolu (80 Tick / Deney):**
  1. `ControlFlagsOff`: Tüm V19 bayrakları kapalı (disabled-path baseline).
  2. `WaterContextOnly`: Yalnızca `WaterLearner::context_ttl_enabled = true`.
  3. `RefugeFirstIntervalOnly`: Yalnızca `Memory::first_interval_refuge_enabled = true`.
  4. `SpatialCuriosityOnly`: Yalnızca `Memory::spatial_curiosity_enabled = true`.
  5. `ClimateAndBurrowsOnly`: Yalnızca mevsimsel döngü ve in kazımı aktif.
  6. `SocialAndNicheOnly`: Yalnızca ebeveyn bakımı, pusu gizlenmesi, alarm ve leşçillik aktif.
  7. `AllV19Features`: Tüm V19 mekanizmaları birlikte aktif.

### 3.2. Seçicilik ve izolasyon doğrulaması

- Her tekil ablasyon kolunda **yalnızca o kola ait hedef sayaç** pozitif değer üretmiştir; koldaki diğer tüm bağımsız V19 mekanizma sayaçları kesin olarak `0` kalmıştır.
- `AllV19Features` kolunda ise tüm mekanizmalar eşzamanlı olarak pozitif olay üretmiş ve `DynamicWorldSnapshot` ile RNG ayrışması kontrol koluna göre doğrulanmıştır.
- **Epistemik sınır:** Bu deney matrisi 80-tick kısa ufuklu seçicilik, ortogonalite ve çoklu-habitat nedensel etki denetimidir; 100.000-tick uzun dönemli popülasyon dengesi veya ekolojik üstünlük iddiası taşımaz.

---

## 4. Bölüm 3 — K7 Web Replay & Experience V19 çevre/beden telemetrisi

K7 gözlem ve arayüz katmanı (`web/experience.js`, `web/replay.js`, `web/replay.html`), V19 çekirdek mekanizmalarının durumunu görselleştiren çevre ve beden telemetrisiyle zenginleştirilmiştir:

1. **`CuriousExperience.environment(state, individual)` Metodu:**
   - Simülasyon durumundan (`state`) mevsim (`Spring`, `Summer`, `Autumn`, `Winter`), gün/gece fazı (`Dawn`, `Day`, `Dusk`, `Night`), aktif sinyal sayısı (`signals.len()`) ve aktif in sayısı (`burrows.len()`) çıkarılır.
   - Bireysel durumdan (`individual`) keşfedilen sektör sayısı (`known_sectors.len()`), su bağlam kayıtları (`context_entries.len()`) ve ceset tazelik evresi (`Fresh`, `Aging`, `Putrid`) telemetriye dahil edilir.
2. **`#env-telemetry` Arayüz Paneli (`web/replay.html`):**
   - Replay arayüzünde seçilen bireyin çevre bağlamını ve beden durumunu anlık olarak gösteren telemetri paneli eklenmiştir.
   - Salt-okunur sözleşme korunmuştur: Telemetri paneli simülasyon durumunu değiştirmez, yalnızca motorun fiziksel durumunu yansıtır.

---

## 5. Sistem durumu ve epistemik sınırlar

- **Geliştirme kaynağı:** Checkpoint `V19` / SQLite `V2`. Tüm yeni V19 özellikleri modüler ve varsayılan kapalıdır (`default-off`).
- **Canlı dünya:** `7882` portundaki canlı sistem V12 checkpoint üzerinde stabil olarak çalışmaktadır; canlıya yükseltilmemiştir.
- **Replay servisi:** `7883` portundaki arşiv replay servisi dondurulmuştur.
- **Epistemik sınır:** Mekanizmaların `World::tick` yürütme döngüsündeki nedensel etkinliği, tekil ablasyon seçiciliği ve telemetri şeffaflığı doğrulanmıştır; evrensel çok-habitatlı ekolojik üstünlük veya fitness avantajı kanıtlanmamıştır / kapsam dışıdır.

---

## 6. İlgili belgeler ve bağlantılar

- [9c7d3f0 bağımsız yeniden denetimi](AUDIT_RECHECK_9C7D3F0_20261025.md)
- [6fa9c49 bağımsız yeniden denetimi](AUDIT_RECHECK_6FA9C49_20261025.md)
- [Wave 9 tam kapanış ve hizalama raporu](AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md)
- [Wave 8 tam kapanış raporu](AUDIT_RECHECK_REMEDIATION_20261025.md)
- [Wave 7 düzeltme ve V19 doğrulama raporu](AUDIT_CA5971B_REMEDIATION_20261025.md)
- [P2/P3 ampirik protokol ve K8 denetimi (Tarihsel dondurulmuş rapor)](P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)
- [Dokümantasyon standardı](../DOCUMENTATION_STANDARD.md)
- [Öğrenme sözleşmesi](../LEARNING_NEXT.md)
- [Yol haritası](../ROADMAP.md)
