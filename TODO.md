# Sonraki işler — 25 Ekim 2026

## Tamamlanan P2 Çekirdek Motor Entegrasyonları ve Doğrulamaları (Mekanizma testi geçti / Doğrulandı)

- **K3 Bireysel Öğrenme & Bellek Entegrasyonu (`src/entity/learning.rs`, `src/entity/refuge_learning.rs`, `src/entity/memory.rs`, `tests/p2_learning_integration.rs`):**
  - **Bağlam-uyumlu su artığı:** Opt-in `WaterLearner::context_ttl_enabled` (`context_entries`, pencere boyutu `W=4`, `TTL=48` tick).
  - **İlk-aralık sığınak hasarı ayrımı:** Opt-in `Memory::first_interval_refuge_enabled` (`first_interval_outcomes` ile ilk çözücü aralığı kesin hasarını sonraki tam rota sansüründen ayıran per-owner bellek; tam rota hasarı `Unknown` kalır).
  - **Mekânsal zihin haritası ve merak keşfi:** Opt-in `Memory::spatial_curiosity_enabled` (`known_sectors`, `4x4` ızgara, max 16 sektör, `TTL=120`, `curiosity_direction`).
  - **Doğrulama:** Disabled-path exact State+RNG denkliği ve checkpoint-resume determinizmi doğrulandı.
- **K1/K2/K5 Çevre, Beden & Ekoloji Entegrasyonu (`src/environment.rs`, `src/entity/phase.rs`, `src/systems/life_cycle.rs`, `src/systems/metabolism.rs`, `tests/p2_environment_integration.rs`):**
  - **Mevsimsel iklim döngüsü:** Opt-in `Environment::season_cycle` (`SeasonCycle`: Spring, Summer, Autumn, Winter; kış bitki yenilenme kısıtı, kış BMR ek yükü).
  - **Kazılan inler:** Opt-in `Environment::burrows` (`BurrowSite` max 32, TTL bozunması, kış termal koruması).
  - **Ceset çürüme evreleri:** Opt-in `EntityPhase::corpse_stage` (`CorpseStage`: `Fresh/Aging/Putrid`, `Omnivore` leş bozulma direnci).
  - **Doğrulama:** Disabled-path exact State+RNG denkliği ve checkpoint-resume determinizmi doğrulandı.
- **K2/K3/K4/K6 Fiziksel Etkileşimler, Ebeveyn Bakımı & Pusu Entegrasyonu (`src/policy.rs`, `src/systems/interaction.rs`, `src/systems/perception.rs`, `tests/p2_interactions_integration.rs`):**
  - **Politika bayrakları:** Opt-in `PolicySettings::{juvenile_care_enabled, ambush_predation_enabled, local_coordination_enabled}`.
  - **Ebeveyn bakımı:** `InteractionSystem::resolve_parental_care` & `parent_guards_juvenile` (fiziksel enerji aktarımı ve hasar engelleme, sıfır bellek aktarımı kuralı).
  - **Sığınak pusu avcılığı:** Örtü tabanlı gizlenme (`effective_detectability_radius` düşüşü) ve ilk vuruş stamina bonusu.
  - **Yerel alarm yayılımı:** `InteractionSystem::emit_local_alarm` (`SignalKind::Sound` yerel uyarı yayılımı).
  - **Doğrulama:** Disabled-path exact State+RNG denkliği ve checkpoint-resume determinizmi doğrulandı.
- **K7 Web Replay & Experience UI Entegrasyonu (`web/experience.js`, `web/replay.html`, `web/replay.js`, `web/replay.css`):**
  - `CuriousExperience.render` artık sığınak deneyimini (`CuriousExperience.refuge`) görüntüler.
  - `web/replay.html` & `web/replay.js`: `#auto-director` odak paneli (`CuriousDocumentary.autoDirector`) ve kayıt başına nedensellik zinciri (`CuriousDocumentary.causalChain`: `Kanıt -> Eylem -> Resolver -> Bellek`) bağlandı.
  - `web/replay.css`: `.auto-director` görsel stili eklendi.
- **P3 & K8 — Sabit Held-Out Protokolü ve Kısa Yük Throughput Denetimi (`tests/k8_throughput_audit.rs`, `docs/reports/P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md`):**
  - Kısa yük tick throughput ve bellek sınırları doğrulandı (`SignalTrace <= 64`, `sites <= 8`, `dangers <= 8`, `known_sectors <= 16`, `burrows <= 32`).
  - Sabit 5-seed held-out protokolü (`control`, `learning`, `climate`, `social_care`, `all_features`) deterministik olarak yürütüldü ve raporlandı.
  - Epistemik dürüstlük: Mekanizma entegrasyonu, disabled-path `State+RNG` denkliği, checkpoint-resume ve sabit-seed protokol tekrarlanabilirliği doğrulanmış; evrensel çok-habitatlı ekolojik üstünlük veya uzun gerçek zamanlı soak iddia edilmemiş / kapsam dışı bırakılmıştır.

## Tamamlanan P0/P1 Mekanizma Doğrulamaları ve Saf Aday Prototipleri

- **K1 (Dünya ve çevre):** Deterministik `DayNightCycle` ve `SignalTrace` yayılımı (`tests/environment_signals.rs`); dinamik habitat (`tests/dynamic_habitat.rs`), mevsimsel iklim (`tests/seasonal_climate.rs`), ceset çürümesi (`tests/corpse_scavenging.rs`) saf prototipleri.
- **K2 (Canlı ve beden):** Ağır yaralanma kapasite kaybı (`tests/injury_physics.rs`); beden enerji ve su bilançosu kapanışı (`tests/physiology.rs`); ontogenetik yavru fizyolojisi ve ebeveyn bakımı saf prototipi (`tests/juvenile_parental_care.rs`).
- **K3 (Bireysel davranış ve öğrenme):** `Memory::toward` yerel BFS (`tests/memory.rs`); ilk-aralık hasar öğrenici saf prototipi (`tests/refuge_first_interval.rs`, `tests/refuge_learning.rs`); bağlam-uyumlu su artığı saf prototipi (`tests/water_context_residual.rs`); mekânsal zihin haritası (`tests/spatial_curiosity.rs`); pusu/takip avcılığı saf prototipi (`tests/ambush_predation.rs`).
- **K4 (Grup ve sosyal ilişkiler):** Yerel alarm ve akran koordinasyonu saf prototipi (`tests/local_coordination.rs`).
- **K5 & K6 (Ekoloji ve evrim):** Çok-nesilli ekoloji denetimi (`tests/ecology_candidate.rs`); allel maliyet ödemesi ve soy seçilimi ayrımı (`tests/genetics.rs`).
- **K7 (Gözlem arayüzü):** Nedensellik zinciri ve Auto-Director kamera saf prototipi (`web/documentary.js`).

## Sıradaki İşler ve Açık Kapılar (P3 / Canlı Önkoşulları)

- **Çoklu habitat kalıcılığı:** Farklı harita ve kaynak topolojilerinde yeni özelliklerin uzun vadeli ekolojik etkilerini ölçmek için önkayıtlı protokol tasarımı.
- **Canlı dağıtım:** 7882 canlı dünyası V12 checkpoint üzerinde arşiv bütçesi backportu ile stabil çalışmaya devam eder; canlıya yükseltme veya uzun soak kullanıcı isteğiyle ertelenmiştir ve sonraki özellikler için blokaj oluşturmaz.

Ayrıntı ve tamamlanmış sınırlar: [ROADMAP](docs/ROADMAP.md).
