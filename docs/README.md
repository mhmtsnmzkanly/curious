# Curious belgeleri

## Güncel giriş noktaları

- [Su öncesi bitişik öğün](WATER_MEAL_PRIORITY.md): V21 varsayılan kapalı karar ve öğrenme-atfı sınırı.
- [ROADMAP](ROADMAP.md): K1–K8 kategorileri, özgün 11 aşama, A–G izleri ve
  güncel kanıt sınırları.
- [TODO](../TODO.md): kısa, sıralı sonraki işler.
- [CONTROL](CONTROL.md): HTTP kontrolü ve servis işletimi.
- [Yem arama bütçesi](FOOD_SEARCH_RESERVE.md): V20 varsayılan kapalı karar sınırı.
- [LEARNING_NEXT](LEARNING_NEXT.md): bireysel öğrenmenin sözleşmesi ve
  ölçüm sınırları.
- [VISION](VISION.md): ürün amacı ve tasarım ilkeleri.
- [HISTORY_RETENTION](HISTORY_RETENTION.md): arşivleme ve saklama kuralları.
- [Dokümantasyon standardı](DOCUMENTATION_STANDARD.md): belge rolleri ve kanıt dili.

K1–K8'e doğrudan git: [K1](ROADMAP.md#dunya-cevre),
[K2](ROADMAP.md#canli-beden), [K3](ROADMAP.md#bireysel-ogrenme),
[K4](ROADMAP.md#grup-sosyal), [K5](ROADMAP.md#ekoloji-etkilesim),
[K6](ROADMAP.md#kalitim-evrim), [K7](ROADMAP.md#gozlem-arayuz),
[K8](ROADMAP.md#motor-isletim).

## Güncel kanıt

- [Su–öğün önceliği V1](reports/WATER_MEAL_PRIORITY_V1_20261026.md): dört kollu
  V21 kıyası; eski yem bütçesine bağlı dar etki ve habitat/seed ödünleşimleri.
- [Yem arama bütçesi V1](reports/FOOD_SEARCH_RESERVE_V1_20261026.md): V20 varsayılan kapalı
  adayın 36 koşuluk kıyası; tek fixture'da dar yarar, diğerlerinde tetiklenme yok.
- [26 Ekim birleşik sonuç özeti](reports/CONSOLIDATION_20261026.md): öğrenme,
  enerji, bakım paketi, habitat ve operasyon kanıtlarının tek kapsam özeti.
- [Su uyum V2](reports/WATER_ADAPTATION_V2_20261025.md): fixture çalıştı;
  üç learner kolunda ek öğrenme faydası kabul edilmedi.
- [Nitelik tanısı](reports/QUALIFICATION_DIAGNOSTIC_20261025.md): bakımın kısa
  fixture sonucu ve 1.000-tick V12/V19 gölge ödünleşimi; canlı kabulü yok.
- [Su uyum V1](reports/WATER_ADAPTATION_V1_20261025.md): başarısız kalibrasyon;
  formal deney çalıştırılmadı.
- [Tam kaçış refuge baseline'ı](reports/REFUGE_RISK_BASELINE_V1.md) ve
  [ilk resolver aralığı](reports/REFUGE_FIRST_RESOLUTION_V1.md): farklı hedef
  ufukları; tam-rota beklenen hasarı `Unknown`.
- [V12 arşiv bütçesi dağıtımı](reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md):
  canlıya alınmış, kapsamı belirtilmiş operasyonel düzeltme.
- [Coexistence gözlemi](reports/QUALIFIED_OBSERVATION.md): yalnız kayıtlı
  fixture ve protokol kapsamındaki sonuç.
- [Ekoloji transfer V1](reports/ECOLOGY_TRANSFER_V1_20261026.md): üç habitatlı
  sınırlı karşılaştırma; seyrek kaynakta tükenme ve karma habitatta etçil kaybı,
  genel fayda kabulü yok.
- [Operasyonel hazırlık](reports/OPERATIONS_READINESS_20261026.md): offline
  migration/storage smoke geçti; promotion kapısı kapalı. [Timing düzeltmesi](reports/OPERATIONS_READINESS_TIMING_CORRECTION_20261026.md)
  eski p50/p95 alanlarını geçersiz sayar.
- [Su sinyali tanısı](reports/WATER_SIGNAL_DIAGNOSIS_20261026.md) ve
  [kaynak güvenilirliği hedef taslağı](SOURCE_RELIABILITY_TARGET_20261026.md):
  tanı ile uygulanmamış K3 önerisi.
- [Enerji ve paket tanısı](reports/ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.md):
  enerji rezerv bekçisi formülü seçilmedi; bakım paketi kabul edilmedi.

Diğer tarihli raporlar [reports/](reports/) altında bulunur. Tarihli raporlar
ölçümün kendi fixture'ını, karşılaştırmasını ve sınırlarını taşır; güncel
garantiye dönüştürülmeden okunmalıdır.

## Protokoller ve artifaktlar

Protokoller ölçüm tasarımını, raporlar tarihli sonucu ve sınırlarını taşır.
Protokol, seed manifesti, hash veya başarısız ham sonuç geriye dönük
değiştirilmez. Canlı operasyon yönergeleri için [CONTROL](CONTROL.md) kullanılır.

[Su-yem önceliği V1 protokolü](WATER_MEAL_PRIORITY_V1_PROTOCOL_20261026.md)
62501–62506 tahsisli seedleri için dört kollu factorial tasarımı ve kaynak,
sağkalım, üreme, learner ve sansürlü episode ölçüm sözleşmesini tanımlar.
Sonuç raporu henüz yoktur.

`data/` deney verileri ve `bin/` derlenmiş ikililer yerel/ignore edilen
artifaktlardır. Bazı tarihli raporların ham JSONL dosyaları Zstandard arşivinde
saklanır; ham dosya gerektiğinde [EVIDENCE_ARCHIVES](EVIDENCE_ARCHIVES.md)
üzerinden indeks ve restore yönergesini izleyin. Arşiv burada yoksa rapordaki
hash ham veriyi geri getirmez.

Eski plan belgelerinin sadeleştirme geçmişi Git'te korunur. Bu belgelerin
kaldırılmış olması, içlerinde adı geçen her işin tamamlandığı anlamına gelmez.
