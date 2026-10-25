# Sonraki işler — 25 Ekim 2026

## Tamamlanan mekanizma doğrulamaları ve saf aday prototipleri (Mekanizma testi geçti)

- **K1 (Dünya ve çevre):**
  - **Gün-gece döngüsü ve çevresel izler:** Deterministik `DayNightCycle` ve
    sınırlı `SignalTrace` (`Footstep`, `Scent`, `Sound`) yayılımı ve bozulması
    uygulandı ve test edildi (`src/environment.rs`, `src/systems/perception.rs`,
    `tests/environment_signals.rs` — *Uygulandı / Mekanizma testi geçti, varsayılan kapalı*).
  - **Yeni Özellik 4 — Dinamik habitat:** Kazılan in ve bozulması (`DigBurrow`/`BurrowDecay`),
    aşırı otlama bozulması (`Overgrazing`) ve tohum yayılımı (`SeedDispersal`) kapalı
    stok-akış muhasebesiyle saf prototipte doğrulandı
    (`examples/support/dynamic_habitat_candidate.rs`, `tests/dynamic_habitat.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
  - **Yeni Özellik 5 — Mevsimsel iklim ve kış uykusu:** Deterministik 4 mevsim çevrimi
    (`SeasonCycle`), kış termoregülasyon BMR ek maliyeti ve inde kış uykusu (`Hibernation`)
    saf prototipte doğrulandı (`examples/support/seasonal_climate_candidate.rs`,
    `tests/seasonal_climate.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
  - **Yeni Özellik 6 — Ceset çürümesi ve leşçillik nişi:** Kademeli çürüme evreleri
    (`Fresh`/`Aging`/`Putrid`), leş kokusu yayılımı ve etçil vs hepçil bozulma direnci niş
    ayrımı saf prototipte doğrulandı (`examples/support/corpse_scavenging_candidate.rs`,
    `tests/corpse_scavenging.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
- **K2 (Canlı ve beden):**
  - **Ağır yaralanma kapasite kaybı:** `injury_capacity_penalty` opsiyonu ile ağır
    yaralanmada (`health <= max_health / 2`) etkin hızın yarıya inmesi ve hareket stamina
    maliyetinin 2 katına çıkması, pasif iyileşmeyle düzelmesi doğrulandı
    (`src/entity/lifestate.rs`, `src/policy.rs`, `tests/injury_physics.rs` — *Uygulandı / Mekanizma testi geçti, varsayılan kapalı*).
  - **Beden enerji ve su bilançosu:** Metabolizma, beslenme, hareket ve üremede beden
    rezervlerinin ve harita stok-akışının tam kapanışı deterministik fizikte doğrulandı
    (`tests/physiology.rs` — *Mekanizma testi geçti*).
  - **Yeni Özellik 2 — Ontogenetik yavru fizyolojisi ve ebeveyn bakımı:** Ölçekli yavru
    fizyolojisi (`age < maturity_age`) ve fiziksel ebeveyn bakımı (`FeedOffspring` enerji
    aktarımı, `GuardOffspring` hasar engelleme, sıfır öğrenilmiş bellek aktarımı) saf
    prototipte doğrulandı (`examples/support/juvenile_parental_care_candidate.rs`,
    `tests/juvenile_parental_care.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
- **K3 (Bireysel davranış ve öğrenme):**
  - **`Memory::toward` yerel baypas BFS:** Hedefe doğrudan adım engellendiğinde yerel BFS
    ile baypas rotası bulma uygulandı (`src/entity/memory.rs`, `tests/memory.rs` — *Uygulandı / Mekanizma testi geçti*).
  - **İlk-aralık hasar öğrenici saf prototipi (İz B):** İlk resolver aralığındaki per-owner
    hasar atfı ve sansür mantığı saf prototipte doğrulandı (`examples/support/refuge_first_interval_candidate.rs`,
    `tests/refuge_first_interval.rs`). Tamamlanan ilk aralığın sonraki tam-rota sansüründen
    bağımsız kaldığı deterministik testle doğrulandı (`tests/refuge_learning.rs`). Tam-rota
    hasarı `Unknown` kalır.
  - **Bağlam-uyumlu su artığı saf prototipi (İz A):** Güncel hız/maliyet bağlamı ve zaman
    penceresi/TTL mekanizması saf prototipte doğrulandı (`examples/support/water_context_residual_candidate.rs`,
    `tests/water_context_residual.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
  - **Yeni Özellik 1 — Mekânsal zihin haritası ve merak güdümlü keşif:** Bounded 16 sektörlü
    `4x4` ızgara, TTL eskime ve keşfedilmemiş/en eski sektörlere merak güdümlü yönelim saf
    prototipte doğrulandı (`examples/support/spatial_curiosity_candidate.rs`,
    `tests/spatial_curiosity.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
  - **Yeni Özellik 3 — Pusu / gizlenme vs. takip avcılığı:** Örtü tabanlı gizlenme
    (`effective_detectability_radius` azalması), ilk vuruş stamina bonusu ve takip vs. pusu
    allel ödünleşimi saf prototipte doğrulandı (`examples/support/ambush_predation_candidate.rs`,
    `tests/ambush_predation.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
- **K4 (Grup ve sosyal ilişkiler):**
  - **Yerel alarm ve akran koordinasyonu:** Yerel alarm çağrısı (`EmitAlarmCall`) ve görünür
    akran koordinasyonu (`AlignWithVisiblePeer`), ödenen bedel ve 3 kollu kontrol/solo/grup
    doğrulaması saf prototipte doğrulandı (`examples/support/local_coordination_candidate.rs`,
    `tests/local_coordination.rs` — *Saf mekanizma prototipi / Mekanizma testi geçti*).
- **K5 (Ekoloji ve çevreyle etkileşim) & K6 (Kalıtım ve evrim):**
  - **Kurucu/F1/F2 soy ve ekoloji audit doğrulaması (İz C):** Çok-nesilli deterministik ekoloji
    fixture'ında kurucu (`gen 0`), F1 (`gen 1`) ve F2 (`gen 2`) kohortlarının tam ayrımı,
    üreyen birey ve soyların tespiti ile sıfır açıklanamayan kaynak residüeli audit testinde
    doğrulandı (`examples/support/ecology_audit.rs`, `tests/ecology_candidate.rs` — *Mekanizma testi geçti*).
  - **Allel bedeli ve soy seçilimi ayrımı:** Allel ek maliyet ödemesi ile üreyen yavru soyu
    seçilim ayrımı deterministik testle doğrulandı (`tests/genetics.rs` — *Mekanizma testi geçti*).
- **K7 (Gözlem ve belgesel arayüz):**
  - **Nedensellik zinciri ve Auto-Director kamera (Yeni Özellik 7):** Salt-okunur nedensellik
    zinciri izi (`causalChain`: `evidence -> action -> resolverOutcome -> memoryUpdate`, eksik
    bağlar açık `Unknown`), sığınak deneyim özeti (`CuriousExperience.refuge`) ve en yüksek
    öncelikli belgesel olayını seçen `autoDirector` uygulandı ve doğrulandı (`web/documentary.js`,
    `web/experience.js` — *Uygulandı / Mekanizma testi geçti*).

## P2 — sınırlı entegrasyon

- **K1–K7:** Mekanizma testleri geçmiş saf prototipler ve opsiyonel modeller motor çekirdeğine
  ve World karar döngüsüne entegre edilmeden önce root onayına sunulmalıdır.
- Entegrasyon; default-off davranış, frozen update, birey izolasyonu, sansürün ilgili hedef
  ufkunda güncellenmemesi, checkpoint migration ve tek-writer sınırlarını geçmelidir. Disabled-path
  eski/yeni State+RNG eşitliği ile deterministik checkpoint-resume ayrıca doğrulanır; frozen model
  parametreleri sabit kalırken online keşif sürebilir.

## P3 — ecology ve bağımsız izler

- **K5/K6:** Yalnızca ölçüm ve entegrasyon kapılarından sonra yeni habitat/trait
  karşılaştırmalarını sabit protokol ve held-out seedlerle yürüt.
- Sonlu eylem/yerel koordinasyon kataloğu, allel seçilimi, deneyim arayüzü ve
  bounded throughput çalışmaları ROADMAP'teki ayrı izlerdir.

Uzun gerçek-zamanlı soak kullanıcı isteğiyle ertelendi; yukarıdaki sırayı
engellemez. Ayrıntı ve tamamlanmış sınırlar: [ROADMAP](docs/ROADMAP.md).
