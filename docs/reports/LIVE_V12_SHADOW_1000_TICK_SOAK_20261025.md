# Wave 12: Çok-nesilli V19 yetenek bayrağı kalıtımı (`resolve_mates`) ve gerçek V12 canlı checkpoint (`Tick 12639`) üzerinde 1.000-tick (`Tick 12639 -> 13639`) izole gölge (shadow) soak deneyi (25 Ekim 2026)

Birincil kategori: K8 Motor ve işletim; K2, K3, K5, K6 ile kesişir.

## 1. Kapsam ve amaç

[Wave 11 raporunda](BEHAVIORAL_UTILITY_AND_SHADOW_QUALIFICATION_20261025.md) 9 tekil V19 özelliği 10 bağımsız held-out tohum (`401..=410`) × 150 tick üzerinde sıkı sağkalım-kaybı-yok Pareto kapısından geçirilmiş; `JuvenileParentalCareOnly` ve `RefugeFirstIntervalOnly` (`QualifiedCandidateBundle`) canlı adayı olarak ayrılmış ve gerçek canlı V12 dünyasının kapalı arşiv shard'ı (`history-000870.sqlite.zst`, `Tick 12639`, `630` birey) üzerinde 20-tick kısa gölge devamlılık testiyle doğrulanmıştı.

Bu rapor, kullanıcı kararı doğrultusunda yürütülen **1.000-tick (`Tick 12639 -> Tick 13639`, 34+ yeni nesil ve ~5.150 yeni doğum)** uzun ufuklu izole gölge (shadow) soak deneyini ve bu deney öncesinde tespit edilip kapatılan **çok-nesilli V19 yetenek bayrağı kalıtımı (`InteractionSystem::resolve_mates`)** düzeltmesini belgeler.

[AGENTS.md](../../AGENTS.md) ve [dokümantasyon standardı](../DOCUMENTATION_STANDARD.md) gereğince önceki tüm tarihsel raporlar ([BEHAVIORAL_UTILITY_AND_SHADOW_QUALIFICATION_20261025](BEHAVIORAL_UTILITY_AND_SHADOW_QUALIFICATION_20261025.md) dahil) dondurulmuş kanıt olarak değiştirilmeden korunmuştur.

---

## 2. Bölüm 1 — Çok-Nesilli V19 Yetenek Bayrağı Kalıtımı ve Sıfır Öğrenilmiş Bellek Aktarımı (`src/systems/interaction.rs`)

### 2.1. Tespit Edilen Çok-Nesilli Sınır
`InteractionSystem::resolve_mates` (`src/systems/interaction.rs`), iki ebeveyn (`entities[s]` ve `entities[t]`) çiftleştiğinde doğan yavruya (`child`) mevcut V13–V18 bireysel yetenek bayraklarını (`hunt_learner.enabled`, `memory.refuge_outcomes_enabled`, `memory.refuge_generalization_enabled`) `&&` operatörüyle aktarıyordu. Ancak V19 ile eklenen üç bireysel yetenek bayrağı:
- `child.memory.first_interval_refuge_enabled`
- `child.memory.spatial_curiosity_enabled`
- `child.learner.context_ttl_enabled`

yeni doğan yavrulara aktarılmıyordu (`false` kalıyordu). 20 tick gibi kısa koşularda kurucu nesil hayatta kaldığı için bu eksiklik gizli kalırken, 200+ tick süren çok-nesilli koşularda kurucu bireylerin yaşlanıp ölmesiyle popülasyondaki V19 bireysel öğrenme bayrakları sönümleniyordu.

### 2.2. Uygulanan Çözüm ve Mimari Garantiler
`src/systems/interaction.rs` (`resolve_mates`) içine aşağıdaki bayrak aktarımı eklendi:
```rust
child.memory.first_interval_refuge_enabled =
    entities[s].memory.first_interval_refuge_enabled
        && entities[t].memory.first_interval_refuge_enabled;
child.memory.spatial_curiosity_enabled =
    entities[s].memory.spatial_curiosity_enabled
        && entities[t].memory.spatial_curiosity_enabled;
child.learner.context_ttl_enabled =
    entities[s].learner.context_ttl_enabled
        && entities[t].learner.context_ttl_enabled;
```
- **Sıfır Öğrenilmiş Bellek Aktarımı (`Zero Learned Memory Inheritance`):** Yavru yalnızca yetenek bayrağının açık olup olmadığını (`bool`) devralır; yavrunun deneyim tabloları (`first_interval_outcomes`, `known_sectors`, `context_entries`, `refuge_outcomes`) doğum anında **kesinlikle boş (`len() == 0`)** başlar (`test_offspring_v19_capability_flag_propagation_with_zero_memory_inheritance`).
- **Disabled-Path Denkliği:** Varsayılan kapalı (`false`) durumda `false && false == false` olduğundan eski `State + RNG` akışı bit düzeyinde değişmeden korunur.

---

## 3. Bölüm 2 — Gerçek Canlı V12 Checkpoint (`Tick 12639`) Üzerinde 1.000-Tick (`Tick 12639 -> 13639`) Gölge Soak Deneyi

Canlı `7882` servisine ve `/root/curious/data/coexistence` dizinine dokunulmadan, `history-000870.sqlite.zst` içindeki son kayıt (`id = 14`, `tick = 12639`, `version = 12`, `next_entity_id = 68273`, `630` birey: `533` yaşayan `Herbivore` + `14` ceset, `83` yaşayan `Carnivore`, `max_generation = 455`) V19 motoruna yüklenmiş ve `--release` profilinde 4 ayrı kolda `1.000` tick (`Tick 12639 -> 13639`) koşturulmuştur (`run_live_v12_shadow_1000_tick_soak_four_arms`).

### 3.1. Kol 1 — `ControlFlagsOff` (Tüm V19 Bayrakları Kapalı Baz Çizgisi)

| Tick | Yaşayan `Herbivore` | Yaşayan `Carnivore` | Toplam Yaşayan | Kümülatif Doğum | Maks. Nesil | Aktif `FI_Rec` | Yeni Nesil `Offspring_FI` | Ort. Sağlık | Ort. Enerji | Ort. Su | Checkpoint JSON (`B`) | Hız (`ms/tick`) |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `12639` | `533` | `83` | `616` | `0` | `455` | `0` | `0` | `71.7` | `61.2` | `33.4` | `2,547,648` | `—` |
| `12739` | `513` | `82` | `595` | `507` | `458` | `0` | `0` | `71.8` | `63.2` | `34.5` | `2,506,062` | `41.66` |
| `12839` | `527` | `81` | `608` | `1,029` | `462` | `0` | `0` | `72.7` | `62.6` | `34.7` | `2,557,051` | `39.77` |
| `12939` | `525` | `81` | `606` | `1,563` | `465` | `0` | `0` | `70.6` | `60.8` | `33.3` | `2,559,084` | `39.52` |
| `13039` | `508` | `82` | `590` | `2,073` | `469` | `0` | `0` | `72.0` | `64.2` | `35.2` | `2,479,310` | `39.61` |
| `13139` | `524` | `84` | `608` | `2,597` | `473` | `0` | `0` | `72.2` | `62.3` | `34.3` | `2,551,054` | `39.69` |
| `13239` | `519` | `89` | `608` | `3,125` | `475` | `0` | `0` | `72.7` | `62.8` | `33.5` | `2,594,917` | `39.65` |
| `13339` | `495` | `88` | `583` | `3,625` | `479` | `0` | `0` | `73.9` | `65.8` | `35.8` | `2,466,507` | `39.70` |
| `13439` | `490` | `92` | `582` | `4,143` | `482` | `0` | `0` | `74.0` | `66.8` | `35.4` | `2,437,349` | `39.87` |
| `13539` | `495` | `92` | `587` | `4,656` | `486` | `0` | `0` | `73.3` | `65.3` | `35.0` | `2,472,265` | `39.79` |
| **`13639`** | **`482`** | **`98`** | **`580`** | **`5,161`** | **`489`** | **`0`** | **`0`** | **`74.3`** | **`67.1`** | **`35.4`** | **`2,434,701`** | **`39.65`** |

### 3.2. Kol 2 — `JuvenileParentalCareOnly` (`juvenile_care_enabled = true`)

| Tick | Yaşayan `Herbivore` | Yaşayan `Carnivore` | Toplam Yaşayan | Kümülatif Doğum | Maks. Nesil | Aktif `FI_Rec` | Yeni Nesil `Offspring_FI` | Ort. Sağlık | Ort. Enerji | Ort. Su | Checkpoint JSON (`B`) | Hız (`ms/tick`) |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `12639` | `533` | `83` | `616` | `0` | `455` | `0` | `0` | `71.7` | `61.2` | `33.4` | `2,547,677` | `—` |
| `12739` | `516` | `83` | `599` | `514` | `457` | `0` | `0` | `71.5` | `63.7` | `34.5` | `2,499,517` | `40.11` |
| `12839` | `494` | `83` | `577` | `1,013` | `461` | `0` | `0` | `72.7` | `65.0` | `35.2` | `2,443,941` | `40.79` |
| `12939` | `518` | `81` | `599` | `1,540` | `465` | `0` | `0` | `72.9` | `63.1` | `34.9` | `2,536,316` | `39.57` |
| `13039` | `505` | `81` | `586` | `2,035` | `467` | `0` | `0` | `72.2` | `65.3` | `35.1` | `2,478,079` | `39.60` |
| `13139` | `518` | `83` | `601` | `2,560` | `471` | `0` | `0` | `72.3` | `62.9` | `34.4` | `2,533,238` | `39.25` |
| `13239` | `518` | `90` | `608` | `3,084` | `475` | `0` | `0` | `72.3` | `63.6` | `34.3` | `2,522,005` | `39.46` |
| `13339` | `493` | `92` | `585` | `3,593` | `479` | `0` | `0` | `73.4` | `65.0` | `34.8` | `2,473,554` | `39.52` |
| `13439` | `479` | `92` | `571` | `4,084` | `482` | `0` | `0` | `73.8` | `63.9` | `35.0` | `2,399,650` | `39.44` |
| `13539` | `513` | `83` | `596` | `4,604` | `486` | `0` | `0` | `72.1` | `62.0` | `34.1` | `2,530,323` | `39.44` |
| **`13639`** | **`488`** (`+6`) | **`87`** | **`575`** | **`5,101`** | **`489`** | **`0`** | **`0`** | **`72.9`** | **`64.7`** | **`34.9`** | **`2,475,286`** | **`39.41`** |

### 3.3. Kol 3 — `RefugeFirstIntervalOnly` (`first_interval_refuge_enabled = true`, `refuge_outcomes_enabled = true`)

| Tick | Yaşayan `Herbivore` | Yaşayan `Carnivore` | Toplam Yaşayan | Kümülatif Doğum | Maks. Nesil | Aktif `FI_Rec` | Yeni Nesil `Offspring_FI` | Ort. Sağlık | Ort. Enerji | Ort. Su | Checkpoint JSON (`B`) | Hız (`ms/tick`) |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `12639` | `533` | `83` | `616` | `0` | `455` | `0` | `0` | `71.7` | `61.2` | `33.4` | `2,570,328` | `—` |
| `12739` | `511` | `84` | `595` | `510` | `458` | `4,653` | `4,135` (`88.9%`) | `72.8` | `63.9` | `35.1` | `4,602,854` | `41.16` |
| `12839` | `538` | `82` | `620` | `1,040` | `463` | `5,482` | `5,482` (`100%`) | `71.7` | `61.6` | `33.3` | `4,986,821` | `42.33` |
| `12939` | `501` | `81` | `582` | `1,552` | `466` | `4,481` | `4,481` (`100%`) | `72.6` | `64.4` | `35.4` | `4,474,963` | `42.51` |
| `13039` | `502` | `84` | `586` | `2,068` | `469` | `4,794` | `4,794` (`100%`) | `71.5` | `63.4` | `34.9` | `4,626,615` | `43.07` |
| `13139` | `540` | `84` | `624` | `2,608` | `472` | `5,055` | `5,055` (`100%`) | `72.0` | `61.5` | `33.9` | `4,816,302` | `42.96` |
| `13239` | `508` | `90` | `598` | `3,120` | `476` | `5,357` | `5,357` (`100%`) | `72.3` | `66.3` | `34.8` | `4,844,344` | `43.15` |
| `13339` | `494` | `95` | `589` | `3,631` | `480` | `5,082` | `5,082` (`100%`) | `72.7` | `65.2` | `34.7` | `4,744,005` | `43.09` |
| `13439` | `476` | `93` | `569` | `4,121` | `484` | `4,877` | `4,877` (`100%`) | `74.5` | `67.4` | `35.5` | `4,562,605` | `42.81` |
| `13539` | `476` | `93` | `569` | `4,623` | `488` | `4,869` | `4,869` (`100%`) | `73.5` | `65.5` | `35.0` | `4,459,216` | `42.60` |
| **`13639`** | **`488`** (`+6`) | **`96`** | **`584`** (`+4`) | **`5,121`** | **`492`** (`+3`) | **`4,917`** | **`4,917`** (`100%`) | **`74.6`** | **`66.1`** | **`35.9`** | **`4,567,186`** | **`42.35`** |

### 3.4. Kol 4 — `QualifiedCandidateBundle` (`JuvenileParentalCare + RefugeFirstInterval`)

| Tick | Yaşayan `Herbivore` | Yaşayan `Carnivore` | Toplam Yaşayan | Kümülatif Doğum | Maks. Nesil | Aktif `FI_Rec` | Yeni Nesil `Offspring_FI` | Ort. Sağlık | Ort. Enerji | Ort. Su | Checkpoint JSON (`B`) | Hız (`ms/tick`) |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `12639` | `533` | `83` | `616` | `0` | `455` | `0` | `0` | `71.7` | `61.2` | `33.4` | `2,570,357` | `—` |
| `12739` | `513` | `82` | `595` | `511` | `457` | `5,201` | `4,670` (`89.8%`) | `71.4` | `63.9` | `34.4` | `4,813,960` | `42.85` |
| `12839` | `483` | `87` | `570` | `1,014` | `461` | `4,820` | `4,820` (`100%`) | `72.3` | `66.0` | `35.3` | `4,676,886` | `42.54` |
| `12939` | `508` | `81` | `589` | `1,530` | `464` | `5,100` | `5,100` (`100%`) | `71.7` | `63.6` | `35.0` | `4,743,334` | `42.83` |
| `13039` | `539` | `76` | `615` | `2,070` | `468` | `5,265` | `5,265` (`100%`) | `71.2` | `60.0` | `33.1` | `4,961,696` | `44.27` |
| `13139` | `507` | `83` | `590` | `2,588` | `472` | `4,697` | `4,697` (`100%`) | `72.7` | `63.1` | `34.7` | `4,617,254` | `44.56` |
| `13239` | `505` | `89` | `594` | `3,103` | `476` | `5,146` | `5,146` (`100%`) | `73.1` | `65.1` | `35.1` | `4,738,096` | `44.85` |
| `13339` | `506` | `86` | `592` | `3,607` | `479` | `5,156` | `5,156` (`100%`) | `73.5` | `64.1` | `34.9` | `4,743,400` | `44.86` |
| `13439` | `529` | `89` | `618` | `4,138` | `483` | `5,544` | `5,544` (`100%`) | `72.5` | `64.0` | `34.0` | `5,020,307` | `44.59` |
| `13539` | `479` | `91` | `570` | `4,620` | `485` | `4,657` | `4,657` (`100%`) | `74.2` | `67.6` | `36.2` | `4,462,787` | `44.41` |
| **`13639`** | **`499`** (`+17`) | **`94`** | **`593`** (`+13`) | **`5,146`** | **`489`** | **`5,052`** | **`5,052`** (`100%`) | **`73.9`** | **`65.2`** | **`34.9`** | **`4,658,798`** | **`44.39`** |

---

## 4. 1.000-Tick Uzun Ufuklu Gölge Soak Bulgularının Sentezi

1. **Çok-Nesilli Öğrenme Devamlılığı (`Offspring_FI == FI_Rec`):**
   - `Tick 12839` (`+200` tick, `~1.014–1.040` yeni doğum) itibarıyla kurucu `Tick 12639` bireylerinin tamamı yerini yeni doğan nesillere (`Gen 456..492`) bırakmıştır.
   - `resolve_mates` düzeltmesi sayesinde yeni doğan yavrular `first_interval_refuge_enabled` ve `refuge_outcomes_enabled` bayraklarını boş bellekle devralmış ve kendi yaşamları boyunca `Tick 13639`'da aktif `5,052` (`QualifiedCandidateBundle`) ve `4,917` (`RefugeFirstIntervalOnly`) ilk-aralık sığınak deneyimi kaydı oluşturmuştur.
2. **Tür Dengesi (Coexistence) ve Otçul/Toplam Sağkalım Üstünlüğü:**
   - 1.000 tick sonunda `QualifiedCandidateBundle` kolu, `ControlFlagsOff` baz çizgisine (`482 Herbivore + 98 Carnivore = 580 yaşayan`) kıyasla **`499 Herbivore (+17 otçul) + 94 Carnivore = 593 toplam yaşayan birey (+13 net yaşayan birey)`** ile en yüksek dönem sonu popülasyonu ve en dengeli av-avcı oranını korumuştur.
   - `RefugeFirstIntervalOnly` tek başına da `488 Herbivore (+6) + 96 Carnivore = 584 toplam yaşayan (+4)` ve `MaxGen = 492 (+3 nesil)` ile baz çizgisinin üzerinde kalmıştır.
3. **Sert Bellek ve Checkpoint Boyutu Platosu (`Zero Unbounded Growth`):**
   - `QualifiedCandidateBundle` kolunda serileştirilmiş V19 checkpoint boyutu ilk 100 tick'te `2.57 MB`'tan `4.81 MB`'a çıktıktan sonra, kalan 900 tick ve `5,146` yeni doğum boyunca **hiçbir sınırsız büyüme göstermemiş** ve `13639`. tick'te `4.66 MB` (`4,658,798 B`) düzeyinde sabitlenmiştir.
   - `13639`. tick sonunda yakalanan V19 checkpoint'lerinin tamamı `State::capture().restore()` doğrulamasından `0` hatayla geçmiştir.
4. **Release-Mode İşletim Hızı (`~39–44 ms/tick`):**
   - 600+ bireyli gerçek ekosistemde `--release` profiliyle ortalama tick süresi `ControlFlagsOff` için `39.65 ms/tick`, `QualifiedCandidateBundle` için `44.39 ms/tick` (~`22.5 tick/s` kapasite) olarak ölçülmüştür; canlı `7882` dünyasının hedef hızı (`0.03–1.0 tick/s`) için gereken bütçenin çok altındadır.

---

## 5. İlgili belgeler

- [Wave 11: Yazılmış V19 özelliklerinin davranışsal fayda (Pareto) elemesi ve 20-tick V12 gölge doğrulaması](BEHAVIORAL_UTILITY_AND_SHADOW_QUALIFICATION_20261025.md)
- [Wave 10: P3 ablasyon ve 9c7d3f0 kanıt sınırı kapanış raporu](P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md)
- [Öğrenme sözleşmesi](../LEARNING_NEXT.md)
- [Yol haritası](../ROADMAP.md)
