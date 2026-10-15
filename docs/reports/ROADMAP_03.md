# 3 — Karma ekosistem karşılaştırması

Aynı motorla küçük çayır (8+2), geniş çayır (24+2) ve yoğun çayır (36+2) ölçüldü.
`examples/ecology_probe.rs` SQLite yükü olmadan tür, soy, yavru üremesi ve son
hayatta olma tickini raporlar. Probe canlı servise bağlanmaz; kayıtlı deneyler de
ayrı dizindedir. Farklı kayıt stratejileri süre karşılaştırması için kullanılmaz.

- Küçük harita: 4 seed'in tamamı 175–196. tickte tükendi.
- Geniş harita: 4 seed'in ikisinde otçullar 2.000'e ulaştı; avcılar devam etmedi.
- Yoğun harita: 8 seed'in dördünde otçullar sürdü. Diğer dört seed'de avcı soyları
  4–12. nesle ulaştı, fakat aşırı avlanma ardından tüm dünya tükendi.
- Ayrı deney: etçil üreme aralığını 25'ten 60'a çıkarma, yoğun haritada sekiz
  başlangıçta otçulları 4.000'e taşıdı ama avcıları kalıcı yapmadı. **Bu deneme
  kaynak koda alınmadı.** Salt sayısal üreme ayarı temel davranış sorununu çözmüyor.

**Kalıcı karma denge henüz yok.** Bu aşama koşul karşılaştırmasını tamamlar;
ekosistemin tamamlandığı anlamına gelmez. Sonraki sözleşme, hafıza ve öğrenme
aşamaları sonrasında aynı matris tekrar çalıştırılmalı. Yaşam kendiliğinden
karmaşıklık üretmek zorunda değildir; başarısız koşular da korunur.

Kaynak senaryolar: `predator-meadow.json`, `mixed-wide.json`, `mixed-dense.json`.
Özet `ROADMAP_03.json`, ham kayıtlar `data/experiments/roadmap-*` altında.
