# Wave 7: ca5971b entegrasyon denetimi (F1–F8) düzeltme ve V19 doğrulama raporu (25 Ekim 2026)

Birincil kategori: K8 Motor ve işletim; K1, K2, K3, K4, K5, K6 ve K7 ile kesişir.

## 1. Kapsam ve amaç

Bu rapor, `ca5971b` revizyonu üzerinde gerçekleştirilen bağımsız entegrasyon denetiminde tespit edilen sekiz bulgunun ([AUDIT_CA5971B_20261025](AUDIT_CA5971B_20261025.md)) düzeltmelerini, `World::tick` etkin yürütme yolu entegrasyonlarını, `V19` checkpoint sürüm yükseltmesini ve epistemik kapsam düzeltmesini ([F8]) belgeler.

[AGENTS.md](../../AGENTS.md) ve [dokümantasyon standardı](../DOCUMENTATION_STANDARD.md) kuralları gereğince:
- Önceki denetim raporu ([AUDIT_CA5971B_20261025](AUDIT_CA5971B_20261025.md)) ve önceki sentetik karşılaştırma raporu ([P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025](P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)) dondurulmuş tarihsel kanıt olarak **değiştirilmeden aynen korunmuştur**.
- Bu ek düzeltme raporu, önceki raporlarda yer alan sentetik fixture iddialarını düzeltir, normal `World::tick` çağrı yollarının bağlandığını kanıtlar ve ekolojik üstünlük iddialarının kapsam dışı olduğunu netleştirir.

---

## 2. Düzeltilen denetim bulguları (F1–F8)

### F1 (P1) — Öğrenme ve merak özelliklerinin `World::tick` yürütme yoluna bağlanması

- **Sorun:** `context_ttl_enabled`, `spatial_curiosity_enabled` ve `first_interval_refuge_enabled` yardımcıları mevcuttu ancak normal `World::tick` veya learner karar döngüsünden çağrılmıyordu.
- **Düzeltme:**
  - `WaterLearner::context_ttl_enabled`: `systems::water_outcomes::finish` içinde tamamlanan su bölümlerinin sonuçları `record_context_observation` ile kaydedilir; `Learner::choose_with_residual_and_context` ve `choose_inner` içinde `predict_context_residual` çağrılarak güncel hız ve maliyet bağlamıyla hedef seçimi yapılır.
  - `Memory::spatial_curiosity_enabled`: `systems::intent_collection::IntentCollectionSystem::collect` içinde `Reason::Explore` kararı sırasında `curiosity_direction` çağrılarak en az ziyaret edilen veya en eski sektörlere merak yönelimi uygulanır.
  - `Memory::first_interval_refuge_enabled`: `systems::refuge_outcomes::finish` içinde ilk resolver tick'inde (`elapsed_ticks == 0`) `record_first_interval_outcome` ile per-owner hasar atfı kaydedilir; aday sıralamasında `choose_refuge_for_species` ile ilk-aralık deneyimi dikkate alınır.
- **Doğrulama:** `tests/p2_learning_integration.rs` içinde flag-on vs. flag-off `World::tick` diferansiyel testleriyle doğrulandı.

### F2 (P1) — Ebeveyn bakımı, pusu vuruşu ve yerel alarmın `World::tick` çözümleyicisine aktarılması

- **Sorun:** `World::tick` saldırıları politikasız çözücüyle çözüyor, ebeveyn koruması ve pusu hasarı normal dünyada devreye girmiyordu. `resolve_parental_care` ve `emit_local_alarm` çağrılmıyordu.
- **Düzeltme:**
  - `World::tick`, saldırı çözümlemesinde `InteractionSystem::resolve_attacks_with_policy(..., &self.policy)` çağırır. `juvenile_care_enabled` açıkken bitişik ebeveyn yavruya gelen hasarı %50 azaltır (hasar `20 -> 10`) ve ebeveyn 4 stamina öder (`16 -> 12`). `ambush_predation_enabled` açıkken sığınaktaki avcı ilk vuruşta +5 hasar bonusu uygular.
  - `World::tick` döngüsüne `InteractionSystem::resolve_parental_care` eklendi; ebeveyn aç yavruya enerji aktarır (sıfır bellek aktarımı kuralı korunur).
  - `policy.local_coordination_enabled` açıkken tehdit altındaki veya avcı gören bireyler `InteractionSystem::emit_local_alarm` ile `SignalKind::Sound` yayar.
- **Doğrulama:** Denetimdeki repro senaryosu (`seed 123`) normal `World::tick` üzerinde çalıştırılarak yavru hasarının 10'a indiği ve parent staminasının 12'ye düştüğü exact değerlerle doğrulandı.

### F3 (P1) — Checkpoint sürümünün V19'a yükseltilmesi ve uyumluluk denetimi

- **Sorun:** Yeni alanlar V18 sürümünde yazıldığından, eski V18 okuyucusu bu alanları sessizce atarak veri kaybına yol açıyordu.
- **Düzeltme:**
  - `State::capture` artık `version: 19` yazar (`src/simulation/checkpoint.rs`).
  - `State::restore`, `1..=19` arası sürümleri kabul eder; `< 19` sürümleri tüm yeni V19 bayrakları varsayılan kapalı (`default-off`) olacak şekilde güvenle göç ettirir.
  - V19 kontrolünde gelecek zaman damgalı sinyal ve inler (`emitted_tick > self.tick`, `created_tick > self.tick`) reddedilir.
  - Eski V18 ikilileri (`1..=18`) V19 kayıtlarını sessizce bozmak yerine açıkça hata vererek reddeder.
- **Doğrulama:** V19 yakalama, V18 göçü, sınır dışı sürüm reddi ve deterministik resume testleriyle doğrulandı.

### F4 (P2) — Kış termal ek maliyetinin `World::tick` yaşam döngüsüne eklenmesi

- **Sorun:** `winter_bmr_surcharge` hesaplanıyor ancak `World::tick` döngüsünde uygulanmıyordu.
- **Düzeltme:**
  - `World::tick`, `environment.season_cycle.is_some()` durumunda `MetabolismSystem::apply_seasonal_thermal_costs` ve ardından `LifeCycleSystem::finalize_tick` çağırır.
  - İnde sığınan bireyler kış termal koruması sayesinde ek maliyet ödemez.
- **Doğrulama:** Denetimdeki 4 dönemli mevsim reprosunda, sığınaksız bireyin enerjisi 3 `World::tick` adımında `[79, 78, 60]` oldu (3. tick kış, surcharge = 17); indeki birey ise `[79, 78, 77]` olarak korundu.

### F5 (P2) — Dinamik in (burrow) kayıtlarının fiziksel harita hücresiyle atomik senkronizasyonu

- **Sorun:** `Environment::dig_burrow` yalnızca metaveri tutuyor, haritadaki fiziksel `Cell::Burrow` hücresini açıp kapatmıyordu.
- **Düzeltme:**
  - `Environment::dig_burrow_on_map` ve `Environment::update` mekanizmaları eklendi. Aktif inler haritada `Cell::Burrow` olarak açılır; TTL dolduğunda in hücresi güvenli şekilde `Cell::Empty` durumuna geri döner.
  - Haritada önceden var olan inler başlangıçta `Environment::from_map` ile `burrows` listesine kaydedilir.
- **Doğrulama:** İn kazma, fiziksel sığınak kullanımı ve TTL bitiminde harita hücresinin boşalması fizik testleriyle doğrulandı.

### F6 (P2) — Ceset ömrü eşikleri ve tüketimde leş toksisitesi

- **Sorun:** Normal ceset ömrü 5 tick iken eşikler `> 20` ve `> 8` olarak tanımlandığı için tüm cesetler doğrudan Putrid görünüyordu; leş toksisitesi tüketimde uygulanmıyordu.
- **Düzeltme:**
  - `EntityPhase::corpse_stage` eşikleri `CORPSE_DURATION_TICKS = 5` ile uyumlu hale getirildi: `remaining >= 4 => Fresh`, `remaining >= 2 => Aging`, aksi halde `Putrid`.
  - `InteractionSystem::resolve_eats` içine `scavenging_toxicity` bağlandı: `Putrid` leş yiyen `Carnivore` 4 hasar alır; `Omnivore` dirençli olup hasar almaz (0 hasar). Çürüyen cesedin enerjisi her tick azalır.
- **Doğrulama:** Fresh, Aging ve Putrid evre geçişleri ile etçil hasarı ve hepçil direnci test edildi.

### F7 (P2) — Bağlam artığı i64 toplamı ve sınır doğrulaması

- **Sorun:** `Learner::validate` bağlam girdilerinin sayısal değerlerine sınır koymuyor, `predict_context_residual` `i32` toplamı debug modda overflow panik üretebiliyordu.
- **Düzeltme:**
  - `Learner::validate`, `context_entries` gözlem değerlerinin `-1000..=1000` aralığında olduğunu doğrular.
  - `record_context_observation` tüm girişleri `-1000..=1000` aralığına clamp eder.
  - `predict_context_residual`, toplamı `i64` genişliğinde alır; böylece uç girdilerde dahi panik oluşması matematiksel olarak imkânsız kılınır.
- **Doğrulama:** `i32::MAX` ve aşırı değerlerle yapılan testlerde sıfır panik ve güvenli kırpma doğrulandı.

### F8 (P2) — Epistemik kapsam düzeltmesi ve gerçek çoklu seed `World::tick` protokolü

- **Sorun:** `P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md` raporundaki tablolar, motor entegrasyonu yerine izole saf prototipleri ölçmüş; su ve koordinasyon döngülerinde aynı durum tekrarlanarak çoklu seed gibi sunulmuştu.
- **Düzeltme ve Epistemik Sınırlar:**
  - `tests/k8_throughput_audit.rs` içindeki testler, her seed (`[101, 102, 103, 104, 105]`) için bağımsız `World::with_seed` ve bağımsız `World::tick` yörüngesi yürütecek şekilde güncellendi.
  - Sentetik fikstür sonuçları ile tam ekosistem sonuçları kesin olarak ayrıldı:
    - **Doğrulanan:** Mekanizmaların kurallı mantığı, `World::tick` yürütme yolu entegrasyonu, V19 checkpoint serileştirme/göçü ve kısa yük throughput sınırları (`SignalTrace <= 64`, `sites <= 8`, `dangers <= 8`, `known_sectors <= 16`, `burrows <= 32`).
    - **Kapsam Dışı / Kanıtlanmayan:** Hiçbir mekanizma için evrensel çok-habitatlı ekolojik üstünlük, genel rota seçimi optimizasyonu veya uzun vadeli popülasyon kalıcılığı iddia edilmemiştir. Tam rota sığınak hasarı `Unknown` kalmaya devam eder.

---

## 3. Sistem sürümleri ve canlı ayrımı

- **Geliştirme kaynağı:** Checkpoint `V19` / SQLite `V2`. Tüm yeni özellikler varsayılan kapalıdır (`default-off`).
- **Canlı dünya:** `7882` portunda çalışan dünya `V12` checkpoint üzerinde, `curious-observation-budget-8dc23de` ikilisiyle bağımsız çalışmayı sürdürür. Canlıya yükseltilmemiştir.
- **Replay ikilisi:** `7883` portundaki salt-okunur replay servisi dondurulmuştur.

## 4. İlgili belgeler ve bağlantılar

- [25 Ekim ca5971b denetim raporu (F1–F8 bulguları)](AUDIT_CA5971B_20261025.md)
- [P2/P3 ampirik protokol ve K8 denetimi (Tarihsel dondurulmuş rapor)](P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)
- [Dokümantasyon standardı](../DOCUMENTATION_STANDARD.md)
- [Öğrenme sözleşmesi](../LEARNING_NEXT.md)
- [Yol haritası](../ROADMAP.md)
