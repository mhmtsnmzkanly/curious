# Curious belgeleri

## Güncel giriş noktaları

- [ROADMAP](ROADMAP.md): K1–K8 kategorileri, özgün 11 aşama, A–G izleri ve
  güncel kanıt sınırları.
- [TODO](../TODO.md): kısa, sıralı sonraki işler.
- [CONTROL](CONTROL.md): HTTP kontrolü ve servis işletimi.
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

Diğer tarihli raporlar [reports/](reports/) altında bulunur. Tarihli raporlar
ölçümün kendi fixture'ını, karşılaştırmasını ve sınırlarını taşır; güncel
garantiye dönüştürülmeden okunmalıdır.

## Protokoller ve artifaktlar

Protokoller ölçüm tasarımını, raporlar tarihli sonucu ve sınırlarını taşır.
Protokol, seed manifesti, hash veya başarısız ham sonuç geriye dönük
değiştirilmez. Canlı operasyon yönergeleri için [CONTROL](CONTROL.md) kullanılır.

`data/` deney verileri ve `bin/` derlenmiş ikililer yerel/ignore edilen
artifaktlardır. Bazı tarihli raporların ham JSONL dosyaları Zstandard arşivinde
saklanır; ham dosya gerektiğinde [EVIDENCE_ARCHIVES](EVIDENCE_ARCHIVES.md)
üzerinden indeks ve restore yönergesini izleyin. Arşiv burada yoksa rapordaki
hash ham veriyi geri getirmez.

Eski plan belgelerinin sadeleştirme geçmişi Git'te korunur. Bu belgelerin
kaldırılmış olması, içlerinde adı geçen her işin tamamlandığı anlamına gelmez.
