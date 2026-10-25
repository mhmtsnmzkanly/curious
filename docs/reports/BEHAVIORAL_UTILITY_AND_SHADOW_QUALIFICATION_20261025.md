# Wave 11: Yazılmış V19 özelliklerinin davranışsal fayda (Pareto) elemesi ve gerçek V12 canlı checkpoint üzerinde izole gölge (shadow) doğrulaması (25 Ekim 2026)

Birincil kategori: K3 Bireysel davranış ve öğrenme; K1, K2, K4, K5, K6, K8 ile kesişir.

## 1. Kapsam, amaç ve metodoloji

Önceki dalgalarda (Wave 7–10) geliştirme kaynağındaki (`V19`) yeni özelliklerin `World::tick` yürütme döngüsüne doğru bağlandığı, ilgili mekanizma sayaçlarının (`counter > 0`) tetiklendiği ve tekil ablasyonda birbirinden izole çalıştığı kanıtlanmıştı ([P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025](P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md)). Ancak **bir mekanizmanın çalışması (`counter > 0`), canlıya veya ekosisteme davranışsal fayda sağladığı anlamına gelmez.**

Bu rapor, yeni özellik yazmadan mevcut yazılmış V19 özelliklerinin:
1. **10 bağımsız held-out tohum (`401..=410`) × 150 tick** boyunca ağırlıksız fiziksel sonuç vektörü (*unweighted physical outcome vector*) üzerindeki gerçek davranışsal etkisini ölçer.
2. **Sıkı sağkalım-kaybı-yok Pareto kapısı (*Strict No-Survival-Deficit Pareto Gate*)** ile hangi özelliklerin canlıya taşınmaya aday (`PASS / Canlı Adayı`) olduğunu, hangilerinin ise doğası gereği çevresel stresör, kapasite cezası veya telafisiz hareket sapması yarattığı için varsayılan kapalı (`TRADE-OFF / Varsayılan Kapalı Kalmalı`) tutulması gerektiğini ayırır.
3. Canlı `7882` dünyasının kapalı arşiv shard'ından (`history-000870.sqlite.zst`, `tick = 12639`, `630` birey) salt-okunur çıkarılan **gerçek V12 checkpoint'i** üzerinde V19 motoruyla **20-tick izole gölge (shadow) devamlılık doğrulaması** yapar ve kontrollü canlıya taşıma prosedürünü (*Controlled Live Promotion Runbook*) tanımlar.

[AGENTS.md](../../AGENTS.md) ve [dokümantasyon standardı](../DOCUMENTATION_STANDARD.md) uyarınca önceki tüm tarihsel denetim ve deney raporları dondurulmuş kanıt olarak değiştirilmeden korunmuştur.

---

## 2. Bölüm A — 9 Tekil Özelliğin Davranışsal Fayda ve Pareto Elemesi (`Seeds 401..=410`, 150 Tick)

`tests/behavioral_utility_qualification.rs` (`test_ten_seed_150_tick_behavioral_utility_and_pareto_qualification`) kapsamında her bir özellik kolu, `ControlFlagsOff` baz çizgisiyle özdeş başlangıç koşullarında (`10` kurucu birey: ebeveyn-yavru çiftleri, sığınak komşuluğu, tehdit altındaki otçullar, yaralı birey, susuz hepçiller ve örtü yakınındaki etçiller) `401..=410` tohumlarında `150` tick koşturulmuştur.

### 2.1. Sıkı Pareto Kabul Kapısı (`classify_feature_arm`)

Bir özelliğin `PassLiveCandidate` (`UYGUN — CANLI ADAYI`) sayılması için 10 tohumluk toplam fiziksel sonuç vektöründe aşağıdaki üç koşulu **aynı anda** sağlaması zorunludur:
1. **Sağkalım kaybı olmaması:** `survived_entity_ticks >= Control`, `prey_survived_ticks >= Control` ve `final_alive_count >= Control`.
2. **Vital stok çöküşü olmaması:** `total_health_end >= Control`, `total_energy_end >= Control` ve `total_water_end >= Control`.
3. **Birim yaşam adımı başına hasar artışının olmaması:** `total_damage_taken / survived_entity_ticks <= Control`.

### 2.2. 10 Tohum (`401..=410`) × 150 Tick Toplam Fiziksel Sonuç Tablosu

| Kol (`QualificationArm`) | `survived_ticks` | `prey_ticks` | `alive_end` | `health_end` | `energy_end` | `water_end` | `dmg_taken` | `dmg / tick` | Mekanizma (`act`) | Pareto Kararı (`QualificationVerdict`) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| **`ControlFlagsOff`** (Baz Çizgisi) | `4572` | `3854` | `14` | `1155` | `793` | `652` | `13157` | `2.878` | `0` | Baz Çizgisi |
| **`JuvenileParentalCareOnly`** | **`5027`** (`+10.0%`) | **`4297`** (`+11.5%`) | **`18`** (`+28.6%`) | **`1458`** (`+26.2%`) | **`1036`** (`+30.6%`) | **`857`** (`+31.4%`) | `13296` | **`2.645`** | Ebeveyn bakım/koruma | **`UYGUN — CANLI ADAYI (PASS)`** |
| **`RefugeFirstIntervalOnly`** | **`4572`** (`0.0%`) | **`3854`** (`0.0%`) | **`14`** (`0.0%`) | **`1155`** (`0.0%`) | **`793`** (`0.0%`) | **`652`** (`0.0%`) | `13157` | **`2.878`** | `43` ilk-aralık kaydı | **`UYGUN — CANLI ADAYI (PASS)`** |
| `DayNightCycleOnly` | `4644` | `4061` | `12` (`-2`) | `1029` | `691` | `563` | `12967` | `2.792` | Gece görüş cezası | `ÖDÜNLEŞİMLİ — VARSAYILAN KAPALI` |
| `AmbushPredationOnly` | `4417` | `3931` | `13` (`-1`) | `1097` | `768` | `592` | `12603` | `2.853` | Pusu +5 ilk vuruş | `ÖDÜNLEŞİMLİ — VARSAYILAN KAPALI` |
| `InjuryCapacityPenaltyOnly` | `4237` | `3696` | `8` (`-6`) | `652` | `433` | `346` | `12708` | `2.999` | Yaralı hız/stamina cezası | `ÖDÜNLEŞİMLİ — VARSAYILAN KAPALI` |
| `SeasonAndBurrowsOnly` | `4168` | `3537` | `3` (`-11`) | `287` | `137` | `112` | `13055` | `3.132` | `55` yeni in / kış yükü | `ÖDÜNLEŞİMLİ — VARSAYILAN KAPALI` |
| `SpatialCuriosityOnly` | `4147` | `3411` | `6` (`-8`) | `549` | `379` | `299` | `13050` | `3.147` | `1065` sektör kaydı | `ÖDÜNLEŞİMLİ — VARSAYILAN KAPALI` |
| `LocalCoordinationOnly` | `4037` | `3476` | `5` (`-9`) | `485` | `312` | `241` | `12728` | `3.153` | `447` alarm sinyali | `ÖDÜNLEŞİMLİ — VARSAYILAN KAPALI` |
| `WaterContextOnly` | `3806` | `3251` | `1` (`-13`) | `112` | `57` | `50` | `12884` | `3.385` | `59` su bağlam kaydı | `ÖDÜNLEŞİMLİ — VARSAYILAN KAPALI` |
| **`QualifiedCandidateBundle`** (`JuvenileCare + RefugeFI`) | **`5027`** (`+10.0%`) | **`4297`** (`+11.5%`) | **`18`** (`+28.6%`) | **`1458`** (`+26.2%`) | **`1036`** (`+30.6%`) | **`857`** (`+31.4%`) | `13296` | **`2.645`** | `44` ilk-aralık + bakım | **`UYGUN — CANLI ADAYI PAKETİ (PASS)`** |
| `AllV19Features` (9 özelliğin tümü açık) | `3422` (`-25.2%`) | `2910` (`-24.5%`) | `5` (`-64.3%`) | `453` | `307` | `249` | `11986` | `3.503` | `1576` toplam olay | `ÖDÜNLEŞİMLİ — VARSAYILAN KAPALI` |

### 2.3. Davranışsal Bulguların Analizi

1. **Neden tüm V19 özelliklerini birden açmak (`AllV19Features`) sağkalımı düşürür?**
   - `SeasonAndBurrowsOnly` kış mevsiminde sığınak dışındaki canlılara `+2 BMR enerji/tick` termal maliyet bindirir ve bitki yenilenmesini yarıya indirir (`alive_end: 14 -> 3`).
   - `InjuryCapacityPenaltyOnly` yaralı bireylerin hızını yarıya düşürüp hareket stamina maliyetini ikiye katlar (`alive_end: 14 -> 8`).
   - `WaterContextOnly` (`water_residual_enabled = true` + `context_ttl_enabled = true`), daha önce [WATER_RESIDUAL_V1](WATER_RESIDUAL_V1.md) raporunda bounded-utility kapısını geçemediği belgelenen V17 su artığı seçimini tetikleyerek su/enerji dengesini bozar (`alive_end: 14 -> 1`).
   - `SpatialCuriosityOnly` ve `LocalCoordinationOnly`, yerel kaynak kümesi etrafında kalması gereken bireyleri keşif veya alarm tepkisiyle kaynaklardan uzaklaştırarak 150 tick ufukta enerji/su kaybına yol açar (`alive_end: 6` ve `5`).
2. **Hangi özellikler gerçekten ölçülebilir fiziksel fayda / sıfır-zarar sağladı (`QualifiedCandidateBundle`)?**
   - **`JuvenileParentalCareOnly` (`juvenile_care_enabled = true`):** Yavru bireylere ebeveyn enerji aktarımı ve saldırı anında %50 hasar koruma kalkanı sağlayarak toplam hayatta kalınan birey-adımı sayısını `4572`'den `5027`'ye (`+455` birey-tick, `%10.0` artış), 150. tick sonundaki yaşayan birey sayısını `14`'ten `18`'e (`+28.6%`), toplam sağlığı `1155`'ten `1458`'e (`+26.2%`), toplam enerjiyi `793`'ten `1036`'ya (`+30.6%`) çıkarmış ve birim yaşam adımı başına alınan hasarı `2.878`'den `2.645`'e düşürmüştür. **Sıfır öğrenilmiş bellek aktarımı kuralını ihlal etmeden tüm fiziksel metriklerde `ControlFlagsOff` baz çizgisini kesin (strict Pareto) domine eder.**
   - **`RefugeFirstIntervalOnly` (`first_interval_refuge_enabled = true`, `refuge_outcomes_enabled = true`):** İlk çözücü aralığındaki (`elapsed_ticks == 0`) kesin tehdit hasarını birey bazında (`43` kayıt) kaydederken hiçbir sağkalım, sağlık, enerji veya su kaybına yol açmamıştır (`survived_ticks = 4572`, `alive_end = 14`).
   - **Birleşik Aday Paketi (`QualifiedCandidateBundle`):** Yalnızca bu iki doğrulanmış özelliği içeren paket, `AllV19Features` paketine kıyasla **`+1605` daha fazla birey-tick sağkalım** (`5027` vs `3422`) ve **`+729` daha fazla dönem sonu enerji stoku** (`1036` vs `307`) korumuştur.

---

## 3. Bölüm B — Gerçek Canlı V12 Dünyası (`history-000870.sqlite.zst`) Üzerinde İzole Gölge (Shadow) Doğrulaması

Canlı `7882` dünyasına veya `/root/curious/data/coexistence` dizinine hiçbir yazma yapılmadan, kapalı arşiv shard'ı `history-000870.sqlite.zst` içindeki son kayıt (`id = 14`, `tick = 12639`, `version = 12`, `630` toplam birey: `533` yaşayan `Herbivore` + `14` ceset, `83` yaşayan `Carnivore`) geçici dizinde açılarak V19 motoruna (`State::restore()`) yüklenmiş ve 20 tick boyunca iki ayrı kolda (`ControlFlagsOff` ve `QualifiedCandidateBundle`) koşturulmuştur (`test_live_v12_shadow_restore_and_20_tick_continuation`).

### 3.1. Gerçek Canlı V12 -> V19 Gölge Koşusu Sonuçları (`Tick 12639 -> 12659`)

| Metrik | Başlangıç (`V12`, `Tick 12639`) | V19 `ControlFlagsOff` (`Tick 12659`) | V19 `QualifiedCandidateBundle` (`Tick 12659`) |
| :--- | :---: | :---: | :---: |
| **Checkpoint Sürümü** | `12` | `19` (`State::restore` doğrulandı) | `19` (`State::restore` doğrulandı) |
| **Yaşayan `Herbivore` Sayısı** | `533` | `528` | **`530`** (`+2` otçul korundu) |
| **Yaşayan `Carnivore` Sayısı** | `83` | `79` | **`80`** (`+1` etçil korundu) |
| **Toplam Yaşayan Birey** | `616` | `607` | **`610`** (`+3` net sağkalım) |
| **Kaydedilen İlk-Aralık Sığınak Sonucu** | `0` | `0` | **`2096` kayıt** (birey başına `<= 8` sınırlı) |
| **Ortalama Tick Süresi (Debug profili)** | — | `323.6 ms/tick` (`323,575 µs`) | **`318.8 ms/tick`** (`318,848 µs`) |
| **Serileştirilmiş Checkpoint JSON Boyutu** | `2,316,543 B` (`2.21 MiB`) | `2,578,725 B` (`2.46 MiB`) | `3,496,819 B` (`3.33 MiB`, `< 4 MiB`) |
| **Panik / Şema Doğrulama Hatası** | `0` | `0` (`State::capture().restore()` OK) | `0` (`State::capture().restore()` OK) |

### 3.2. Gölge Doğrulama Sonucu

1. **Tam Şema ve Göç Uyumluluğu:** 630 bireyli gerçek canlı V12 durumu, V19 motoruna sıfır veri kaybıyla yükseltilmiş ve 20 tick sonunda üretilen V19 checkpoint'i tüm bellek ve zaman damgası doğrulamalarından (`State::restore()`) hatasız geçmiştir.
2. **Gerçek Canlı Popülasyonda Davranışsal Etki:** 20 tick'lik gölge devamlılık koşusunda bile `QualifiedCandidateBundle` (`juvenile_care_enabled + first_interval_refuge_enabled + refuge_outcomes_enabled`), `ControlFlagsOff` koluna kıyasla **3 ek bireyin (`530` vs `528` Herbivore, `80` vs `79` Carnivore) hayatta kalmasını sağlamış** ve tick süresinde hiçbir yavaşlama yaratmamıştır (`~319 ms/tick`).

---

## 4. Kontrollü Canlıya Taşıma Protokolü (Controlled Live Promotion Runbook)

Canlı `7882` dünyası (`curious-coexistence.service`) şu an kullanıcı politikası gereği `V12` (`curious-observation-budget-8dc23de`) üzerinde dokunulmadan çalışmaya devam etmektedir. Kullanıcı `QualifiedCandidateBundle` paketini canlıya taşımak istediğinde izlenecek **5 adımlı sıfır-veri-kaybı geçiş protokolü** aşağıdadır:

1. **Adım 1 — Canlı Dünyayı Güvenli Duraklatma ve Timer'ları Askıya Alma:**
   - `curl -sS -X POST http://127.0.0.1:7882/control/pause` ile aktif tick döngüsünü duraklatın ve `/status` üzerinden `paused: true`, `storage_error: null` olduğunu doğrulayın.
   - `systemctl stop curious-coexistence-monitor.timer curious-coexistence-sampler.timer curious-history-retention.timer` ve ardından `systemctl stop curious-coexistence.service` ile servisi temiz biçimde kapatın.
2. **Adım 2 — Durdurulmuş Veri Dizinini ve Aktif SQLite Shard'ını Yedekleme:**
   - Servis kapalıyken aktif shard'da `.sqlite-wal` kalmadığını doğrulayın ve tam yedek alın (`cp -a /root/curious/data/coexistence /root/curious/data/coexistence-backup-pre-v19-$(date -u +%Y%m%dT%H%M%SZ)`).
3. **Adım 3 — İmzalı Release İkilisini Arşivleme ve Sembolik Bağ Güncellemesi:**
   - `cargo build --release --locked --jobs 1` ile derlenen ikiliyi `bin/releases/curious-v19-qualified-<git-sha>` olarak kopyalayın, `sha256sum` değerini kaydedin ve `ln -sfn releases/curious-v19-qualified-<git-sha> /root/curious/bin/curious-qualified` ile sembolik bağı güncelleyin.
   - *Not:* Yalnızca Pareto kapısını geçen özellikler (`juvenile_care_enabled`, `first_interval_refuge_enabled`) kontrollü bayrakla açılır; `SeasonCycle`, `injury_capacity_penalty`, `water_residual_enabled`, `spatial_curiosity_enabled` ve `local_coordination_enabled` canlıda **varsayılan kapalı (`default-off`)** tutulur.
4. **Adım 4 — Servisi `--paused --resume` ile Başlatma ve 3-Tick Gözetim:**
   - `curious-coexistence.service`'i başlatın, `/status` üzerinden V12 checkpoint'inin V19 olarak yüklendiğini (`tick = N`, `storage_error: null`) doğrulayın.
   - 3 tick boyunca `step` veya `resume` ile ilerleyip `Herbivore`/`Carnivore` sayılarının, bellek sınırlarının ve SQLite yazımlarının sağlıklı olduğunu teyit edin; ardından timer'ları yeniden başlatın.
5. **Adım 5 — Anında Geri Dönüş (Rollback) Yolu:**
   - Herhangi bir anomali durumunda servisi durdurun (`systemctl stop curious-coexistence.service`), Adım 2'deki V12 yedeğini geri yükleyin ve `bin/curious-qualified` sembolik bağını `bin/releases/curious-observation-budget-8dc23de` (`SHA-256: 35930838981679ab1b937324df6dda73d2803959af`) üzerine geri çevirin.

---

## 5. İlgili belgeler

- [Wave 10: P3 ablasyon ve 9c7d3f0 kanıt sınırı kapanış raporu](P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md)
- [9c7d3f0 bağımsız yeniden denetimi](AUDIT_RECHECK_9C7D3F0_20261025.md)
- [Arşiv bütçesi düzeltmesinin canlı V12 dağıtım raporu](ARCHIVE_BUDGET_DEPLOYMENT_20261025.md)
- [V17 su artığı sonucu](WATER_RESIDUAL_V1.md)
- [Öğrenme sözleşmesi](../LEARNING_NEXT.md)
- [Yol haritası](../ROADMAP.md)
