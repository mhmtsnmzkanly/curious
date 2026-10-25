# Sonraki işler — 25 Ekim 2026

## Tamamlanan P0/P1 mekanizma ve fizik doğrulamaları (Mekanizma testi geçti)

- **K3 (İz B) — İlk-aralık hasar öğrenici saf prototipi:** İlk resolver
  aralığındaki per-owner hasar atfı ve sansür mantığı saf prototipte uygulandı
  ve test edildi (`examples/support/refuge_first_interval_candidate.rs`,
  `tests/refuge_first_interval.rs`). Tamamlanan ilk aralığın sonraki tam-rota
  sansüründen bağımsız kaldığı deterministik testle doğrulandı
  (`tests/refuge_learning.rs`). World/motordan izoledir, tam-rota hasarı
  `Unknown` kalır.
- **K3 (İz A) — Bağlam-uyumlu su artığı saf prototipi:** Güncel bağlam ve
  zaman penceresi/TTL mekanizması saf prototipte uygulandı ve test edildi
  (`examples/support/water_context_residual_candidate.rs`,
  `tests/water_context_residual.rs`). Bounded utility ampirik kabulü değildir;
  World/motordan izoledir.
- **K3 — `Memory::toward` yerel baypas BFS:** Hedefe doğrudan adım engellendiğinde
  en çok 8 düğümlük BFS ile yerel baypas rotası üreten mekanizma uygulandı
  (`src/entity/memory.rs`, `tests/memory.rs`).
- **K2/K5 (İz C) — Beden ve kaynak stok-akış bilançosu:** Metabolizma, beslenme,
  hareket ve üremede beden rezervlerinin ve harita stok-akışının tam kapanışı
  deterministik fizikte doğrulandı (`tests/physiology.rs`).
- **K5 (İz C) — Kurucu/F1/F2 soy ve ekoloji audit doğrulaması:** Çok-nesilli
  deterministik ekoloji fixture'ında kurucu (`gen 0`), F1 (`gen 1`) ve F2
  (`gen 2`) kohortlarının tam ayrımı, üreyen birey ve soyların tespiti ile sıfır
  açıklanamayan kaynak residüeli audit testinde doğrulandı
  (`examples/support/ecology_audit.rs`, `tests/ecology_candidate.rs`).

## P2 — sınırlı entegrasyon

- **K3/K4:** P0/P1 mekanizma testleri geçmiş prototipleri (İz A, İz B) ortak
  kaynakta ve World karar döngüsünde etkinleştirmeden önce root onayına sun.
- Entegrasyon; default-off davranış, frozen update, birey izolasyonu, sansürün
  ilgili hedef ufkunda güncellenmemesi, checkpoint migration ve tek-writer
  sınırlarını geçmelidir. Disabled-path eski/yeni State+RNG eşitliği ile
  deterministik checkpoint-resume ayrıca doğrulanır; frozen model parametreleri
  sabit kalırken online keşif sürebilir.

## P3 — ecology ve bağımsız izler

- **K5/K6:** Yalnızca ölçüm ve entegrasyon kapılarından sonra yeni habitat/trait
  karşılaştırmalarını sabit protokol ve held-out seedlerle yürüt.
- Sonlu eylem/yerel koordinasyon kataloğu, allel seçilimi, deneyim arayüzü ve
  bounded throughput çalışmaları ROADMAP'teki ayrı izlerdir.

Uzun gerçek-zamanlı soak kullanıcı isteğiyle ertelendi; yukarıdaki sırayı
engellemez. Ayrıntı ve tamamlanmış sınırlar: [ROADMAP](docs/ROADMAP.md).
