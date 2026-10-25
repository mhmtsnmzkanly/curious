# P2/P3 Ampirik Karşılaştırma ve K8 İşletim Bütçesi Denetim Raporu (25 Ekim 2026)

Bu rapor, Curious simülatöründe **K8 Motor ve İşletim Bütçesi** denetimini ve **P3 Sabit Seed Ampirik Karşılaştırma** protokollerini belgelemektedir. Dokümantasyon kuralları gereği ([docs/DOCUMENTATION_STANDARD.md](../docs/DOCUMENTATION_STANDARD.md)), mekanizma doğrulaması ile ekosistem genellemesi arasındaki sınırlar açıkça ayrılmıştır.

---

## 1. Kapsam ve Protokol Kimliği

- **Tarih:** 25 Ekim 2026
- **Test ve Denetim Dosyası:** [`tests/k8_throughput_audit.rs`](file:///root/curious-research/wave6-ui-p3-k8/tests/k8_throughput_audit.rs)
- **Dizin & Çalışma Dalı:** `wave6-ui-p3-k8`
- **İlgili Yol Haritası ve Görevler:**
  - **K8 (Motor ve İşletim):** Çok tür popülasyonu altında kısa yük tick verimi, sinyal FIFO tampon tavanı (`<= 64`), bireysel bellek tehlike (`<= 8`) ve kaynak (`<= 32`) tavanları, JSON checkpoint serileştirme boyutu (`< 1 MB`).
  - **P3 (Sabit Seed Karşılaştırmaları):**
    - Su bağlam artığı öğrenicisi (`water_context_residual_candidate`) vs. açgözlü (greedy) unlearning baseline (Seedler: `101, 102, 103, 104, 105`).
    - İlk resolver aralığı sığınak hasar öğrenicisi (`refuge_first_interval_candidate`) vs. bilgisiz baseline (Seedler: `101, 102, 103, 104, 105`).
    - Yerel sosyal koordinasyon alarm çağrısı (`local_coordination_candidate`) 3 kollu deney: `ControlSolo`, `UnverifiedIgnored`, `VerifiedLocalCoordination` (Seedler: `201, 202, 203, 204, 205`).
    - Pusu (`AmbushSpecialist`) ve Takip (`PursuitSpecialist`) niş ayrımı ile Çürümüş leş (`FreshnessStage::Putrid`) tüketiminde Hepçil (`Omnivore`) ve Etçil (`Carnivore`) fizyolojik ayrımı ve tam stok-akış enerji denkliği (`corpse_scavenging_candidate`).

---

## 2. K8 Kısa Yük İşletim ve Bellek Bütçesi Denetimi

### Deney Kurulumu
- **Dünya Boyutları:** `x: [-15, 15]`, `y: [-15, 15]`
- **Başlangıç Nüfusu:** 30 kurucu (10 Herbivore, 10 Carnivore, 10 Omnivore) – `EntityController::AI`, `EntityPhase::Active`.
- **Çevre Özellikleri:**
  - `DayNightCycle`: periyot 50 tick, gündüz 30 tick, gece görüş cezası 2.
  - `signals_enabled`: `true` (Her 2 tick'te bir düzenli `SignalTrace` yayımlandı).
  - `injury_capacity_penalty`: `true` (Yaralanma hız kısıtlaması aktif).
- **Simülasyon Süresi:** 250 tick kesintisiz çalışma.

### Ölçüm Sonuçları

| Metrik | Ölçülen Değer | İzin Verilen Tavan / Eşik | Durum |
| :--- | :--- | :--- | :--- |
| **250 Tick Çalışma Süresi (Wall-clock)** | 1.90 – 2.13 saniye | < 10.0 saniye | **GEÇTİ** (Mekanizma Doğrulandı) |
| **Ortalama Tick Verimi** | ~117 – 131 tick/saniye | > 25 tick/saniye | **GEÇTİ** |
| **Değerlendirilen Aktif Birey-Tick Sayısı** | 3,414 birey-tick | > 0 | **GEÇTİ** |
| **Maksimum Çevre Sinyal Sayısı (`signals.len()`)** | 64 | `<= MAX_ENVIRONMENT_SIGNALS` (64) | **GEÇTİ** (FIFO tahliyesi doğrulandı) |
| **Maksimum Bireysel Tehlike Belleği (`dangers.len()`)** | 8 | `<= 8` | **GEÇTİ** (Tavan aşılamaz) |
| **Maksimum Bireysel Kaynak Belleği (`sites.len()`)** | 32 | `<= 32` | **GEÇTİ** (Tavan aşılamaz) |
| **Serileştirilmiş Checkpoint Boyutu (`serde_json`)** | 9,389 byte (~9.2 KB) | `< 1,000,000 byte` (1 MB) | **GEÇTİ** (Bütçenin <%1'i) |

---

## 3. P3 Sabit Seed Karşılaştırma Sonuçları

Tüm testler belirlenen 5'er sabit seed ile çalıştırılmış ve deterministik olarak doğrulanmıştır.

### 3.1. Su Bağlam Artığı (Water Context Residual) Protokolü
- **Protokol:** 10 epizotluk su arama planı. İlk 3 epizotta Kaynak A ve B doludur. 4. epizottan itibaren Kaynak A kurur (`FailedMissingResource`), Kaynak B açık kalır.
- **Açgözlü (Greedy) Baseline:** Statik başlangıç skoru yüksek olan Kaynak A'yı (450 > 340) seçmeye devam eder ve kuruma durumunda her epizotta başarısız olur.
- **Bağlam Uyumlu Aday (`WaterContextResidualTable`):** Kaynak A'daki başarısızlıkları negatif artık (`residual = -750`) olarak kaydeder, pencere ortalaması düşer ve 5. epizottan itibaren güvenilir Kaynak B'ye yönelir.

| Seed | Baseline Kazanılan Su | Aday Kazanılan Su | Baseline Başarısız Deneme | Aday Başarısız Deneme | Alternatif Kaynağa Geçiş |
| :---: | :---: | :---: | :---: | :---: | :---: |
| **101** | 60 | **160** | 7 | **2** | 5 kez |
| **102** | 60 | **160** | 7 | **2** | 5 kez |
| **103** | 60 | **160** | 7 | **2** | 5 kez |
| **104** | 60 | **160** | 7 | **2** | 5 kez |
| **105** | 60 | **160** | 7 | **2** | 5 kez |

> **Bulgu:** Kuruyan kaynak fixture'ında bağlam uyumlu öğrenici baseline'a kıyasla su verimini %166 artırmış, boşa harcanan hareket enerjisini ve başarısız denemeleri 7'den 2'ye indirmiştir.

---

### 3.2. Sığınak İlk Resolver Aralığı (Refuge First-Interval Damage) Protokolü
- **Protokol:** Birey iki kaçış rotası arasında tercih yapar:
  - **Rota 1 (Tehlike Yönü):** Doğrudan avcı sektörüne doğru ilk adım; ilk aralıkta yüksek avcı hasarı alır (`20 + seed % 4`).
  - **Rota 2 (Dikey Kaçış):** Dikey sektörde ilk adım; ilk aralıkta yalnızca 4 hasar alır, ancak rota 3. tick'te dış etkenler sebebiyle tam-rota sansürlenir (`later_full_route_censored: true`).
- **Baseline Davranış:** Bilgisiz baseline veya tam-rota sansürünü gören klasik öğrenici Rota 2'yi sansür gerekçesiyle atar ve Rota 1'e yönelebilir.
- **İlk Aralık Öğrenicisi (`FirstIntervalLearner`):** İlk aralıkta doğrulanmış hasarı (`4.0`), sonraki rota sansüründen bağımsız olarak adil değerlendirir ve Rota 2'yi ilk sıraya yerleştirir.

| Seed | Baseline Seçimi | Öğrenici Seçimi | Rota 1 Beklenen Hasar | Rota 2 Beklenen Hasar | Karar Kalitesi |
| :---: | :---: | :---: | :---: | :---: | :---: |
| **101** | Rota 1 (İndeks 0) | **Rota 2 (İndeks 1)** | 21.0 | **4.0** | Düşük hasarlı rota önceliklendirildi |
| **102** | Rota 1 (İndeks 0) | **Rota 2 (İndeks 1)** | 22.0 | **4.0** | Düşük hasarlı rota önceliklendirildi |
| **103** | Rota 1 (İndeks 0) | **Rota 2 (İndeks 1)** | 23.0 | **4.0** | Düşük hasarlı rota önceliklendirildi |
| **104** | Rota 1 (İndeks 0) | **Rota 2 (İndeks 1)** | 20.0 | **4.0** | Düşük hasarlı rota önceliklendirildi |
| **105** | Rota 1 (İndeks 0) | **Rota 2 (İndeks 1)** | 21.0 | **4.0** | Düşük hasarlı rota önceliklendirildi |

> **Bulgu:** Sansür ayrımı sayesinde ilk aralıkta gözlemlenen net 4.0 hasar korunmuş, Rota 1'deki 20–23 arası yıkıcı hasardan deterministik olarak kaçınılmıştır.

---

### 3.3. Yerel Sosyal Koordinasyon (Local Coordination 3-Arm) Protokolü
- **Protokol:** 8 tick'lik avcı tehdidi senaryosu.
  - **Kol 1 (`ControlSolo`):** Yalnız odak birey; alarm gönderici yok.
  - **Kol 2 (`UnverifiedIgnored`):** Gönderici var ancak alarm resolver tarafından ödenmemiş/doğrulanmamış (`is_paid = false`).
  - **Kol 3 (`VerifiedLocalCoordination`):** Gönderici yerel görüş alanı içinde alarm maliyetini (10 enerji, 5 dayanıklılık) öder; odak ajan sadece sektör bilgisiyle tehditten uzaklaşır.

| Seed | Kol 1 (Solo) Yaşam | Kol 2 (Unverified) Yaşam | Kol 2 Reddedilen Alarm | Kol 3 (Verified) Yaşam | Kol 3 Ödenen Alarm | Kol 3 Gönderici Harcanan Enerji |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **201** | Öldü (`false`) | Öldü (`false`) | 4 | **Yaşadı (`true`)** | 8 | 80 |
| **202** | Öldü (`false`) | Öldü (`false`) | 4 | **Yaşadı (`true`)** | 8 | 80 |
| **203** | Öldü (`false`) | Öldü (`false`) | 4 | **Yaşadı (`true`)** | 8 | 80 |
| **204** | Öldü (`false`) | Öldü (`false`) | 4 | **Yaşadı (`true`)** | 8 | 80 |
| **205** | Öldü (`false`) | Öldü (`false`) | 4 | **Yaşadı (`true`)** | 8 | 80 |

> **Bulgu:** Doğrulanmamış veya maliyeti ödenmemiş alarmlar odak birey tarafından güvenle reddedilmiş; doğrulanmış yerel alarmlar ise global koordinat sızdırmaksızın odak bireyin hayatta kalmasını sağlamıştır.

---

### 3.4. Ekolojik Niş Ayrışması ve Leş Enerji Korunumu
- **Pusu vs. Takip:**
  - `AmbushSpecialist` (`mobility: 0, stamina: 2`): Siper yanında pusuya yatarak bekleme maliyetini 1 enerjiye düşürür, gizlilik yarıçapını 3 birim küçültür ve ilk vuruşta **14 hasar** (10 temel + 4 bonus) verir.
  - `PursuitSpecialist` (`mobility: 2, stamina: 1`): Açık arazide yüksek takip verimine sahiptir ancak pusuda yalnızca **11 hasar** (10 temel + 1 bonus) verir ve gizlilik avantajı 1 birimle sınırlıdır.
- **Çürümüş Leş Tüketimi:**
  - `Omnivore` (Hepçil çöpçü direnci): Çürümüş leş (`FreshnessStage::Putrid`) yerken **0 sağlık cezası** ve **0 stamina cezası** alır.
  - `Carnivore` (Etçil taze avcı): Çürümüş leş yediğinde **6 sağlık cezası** ve **8 stamina cezası** alır.
- **Stok-Akış Korunumu:** `initial_energy == eaten_carnivore + eaten_omnivore + decayed + remaining` eşitliği 5 seed'in tamamında mutlak olarak doğrulanmıştır.

| Seed | Pusu İlk Vuruş Hasarı | Takip İlk Vuruş Hasarı | Hepçil Çürük Leş Hasarı | Etçil Çürük Leş Hasarı | Leş Stok-Akış Dengesi |
| :---: | :---: | :---: | :---: | :---: | :---: |
| **201** | **14** | 11 | **0** | 6 | **Dengede (`true`)** |
| **202** | **14** | 11 | **0** | 6 | **Dengede (`true`)** |
| **203** | **14** | 11 | **0** | 6 | **Dengede (`true`)** |
| **204** | **14** | 11 | **0** | 6 | **Dengede (`true`)** |
| **205** | **14** | 11 | **0** | 6 | **Dengede (`true`)** |

---

## 4. Sınırlar ve Açık Alanlar (Dokümantasyon Standardı Ayrımı)

Dokümantasyon standartlarımızın gerektirdiği katı ayrım aşağıda özetlenmiştir:

1. **Mekanizma Doğrulandı / Entegre Edildi:**
   - K8 işletim bütçesi tavanları (sinyal <= 64, tehlike <= 8, kaynak <= 32, checkpoint < 1MB) çok tür popülasyonu altında tam test edilmiş ve doğrulanmıştır.
   - Sosyal alarm çağrılarının ödeme mekanizması, sahte/ödenmemiş sinyallerin yoksayılması ve yerel yönelme matematiği doğrulanmıştır.
   - İlk resolver aralığı hasar ayrıştırması ve leş çürüme stok-akış muhasebesi tam mekanik denklik içindedir.
   - Web arayüzünde (`web/`) Otonom Yönetmen (`autoDirector`), sığınak deneyim paneli ve nedensellik zinciri (`causalChain`) bileşenleri bağlanmış ve UI testleriyle tescillenmiştir.

2. **Ampirik Ekosistem Genellemesi (Kapsam Dışı / Yalnızca Sabit Seed Protokolü):**
   - Bu rapordaki su adaptasyonu, sığınak önceliklendirmesi ve avcı-çöpçü niş ayrımı sonuçları **yalnızca bu raporda adlandırılan sentetik fikstürler ve sabit seedler (`101..105`, `201..205`) için ampirik olarak kabul edilmiştir**.
   - Bu ampirik sonuçlar, Curious deposundaki kontrolsüz tüm açık habitatlara veya rastgele dağıtılmış heterojen haritalara otomatik olarak genellenemez.
   - Canlı motor çekirdeği varsayılan olarak kapalı (default-off) kalmaya devam etmekte olup, üretim entegrasyonu aşamalı kabul kapılarına tabidir.
