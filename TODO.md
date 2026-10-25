# Sonraki işler — 25 Ekim 2026

## P0 — destek ve mekanizma tasarımı

- **K3:** İlk resolver aralığındaki kendi tehdidine ait hasar için per-owner, açıkça
  tanımlanmış bir öğrenme mekanizması tasarla. Mevcut 623 gözlemde altı pozitif
  sonuç dört bireyde; bu veri kabul edilmiş model veya tam kaçış tahmini
  üretmez. Bu tasarım işi model eğitimi ya da World/lab çalışması izni değildir.
- **K3:** Su artığı deneyinin başarısız bounded utility sonucunu ve güncel bağlamla
  eski deneyimin uyuşmama riskini ayrı hipotez olarak ele al. Retune veya yeni
  batch için ayrıca preregistration gerekir.

## P1 — fiziksel doğrulama

- **K2/K3:** Hasar atfı, belirsiz/sansürlü sonuç, ilk-aralık exposure ve
  resolver'da ödenen maliyet için küçük, deterministik fizik/ölçüm testleri tasarla.
  Daha sonraki tam-rota sansürü, zaten tamamlanmış ilk-aralık hedefini sansürlemez.
- **K5 (K1/K2/K6 bağımlı):** Sürdürülebilir ecology fixture'ında kaynak
  stok-akışı, beden enerji bilançosu, kurucu/F1/F2 ve gerçekten üreyen soy
  metriklerini ayrı doğrula.

## P2 — sınırlı entegrasyon

- **K3/K4:** P0/P1 kapıları root tarafından kabul edilmeden learner veya yeni eylem
  davranışını ortak kaynakta etkinleştirme.
- Entegrasyon; default-off davranış, frozen update, birey izolasyonu, sansürün
  ilgili hedef ufkunda güncellenmemesi, checkpoint migration ve writer
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
