# İlk yaşam döngüsü ölçümü — 15 Ekim 2026

Kaynak revision: `ecd4ec6`. Sekiz ayrı süreçte varsayılan dört kurucu,
30×30 harita, seed dışında aynı ayarlar; her koşuda 500 tick. Canlı/kaynak
müdahalesi yapılmadı. Ana servis değiştirilmedi.

| Seed | Tam yok oluş tick'i | Doğum | En yüksek nesil (kurucular=0) | İçme olayı |
|---|---:|---:|---:|---:|
| 1 | 121 | 0 | 0 | 0 |
| 2 | 121 | 1 | 1 | 2 |
| 3 | 128 | 1 | 1 | 2 |
| 4 | 121 | 0 | 0 | 0 |
| 5 | 126 | 1 | 1 | 5 |
| 42 | 121 | 0 | 0 | 1 |
| 12345 | 127 | 1 | 1 | 1 |
| 2026 | 126 | 1 | 1 | 2 |

8/8 koşu yok oldu. 5/8 koşuda bir yavru doğdu; hiçbir koşuda torun doğmadı.
Bazı kayıtlarda ölüm olayı sayısı birey sayısından fazladır: ceset tüketimi ikinci
bir ölüm olayı üretiyor. Ölüm nedenleri eski şemada bulunmadığından bu ölçümden
kesin ölüm nedeni dağılımı çıkarılamaz. Başlangıçta yalnızca birer etçil/hepçil
olduğundan bu türler için eş yokluğu senaryonun kendisinden kaynaklanır.

## Kodla doğrulanan sorunlar

- Yavru, üreme sonrasında ebeveynden kopyalanan 100 ticklik cooldown'u taşıyor.
  Otçulun max_age değeri 105; olgunlaşma yaşı 20 olmasına rağmen üreme fırsatı
  yaşamının sonuna erteleniyor.
- Sağlık düşükse Survival, Thirst'ten önce seçiliyor. Sağlık kaybıyla başlayan
  susuzluk sonrasında canlı su aramak yerine yiyeceğe yönlenebiliyor.
- Açlık/susuzluk hasarı sağlık dörtte bire düşünce kesiliyor. Bu durumda yaşayan
  canlılar kayıtlarda bulunuyor; bu bir ekolojik denge değil, hasar kuralı hatası.
- Algılanan varlıkların canlı/ceset ayrımı yok. Karar sistemi cesetleri tehdit,
  av veya eş sanabiliyor; uygulama katmanı reddetse de niyetler tekrar ediyor.
- Uyku metabolizma ve yaşlanmayı tamamen durduruyor.
- Kaynak yenilenmesi henüz yok. Bunun eksikliği tamamlanmamış kapsamdır.

## Bir sonraki aşama

Önce bu sözleşmeleri ve ölüm/başarısız eylem teşhisini düzeltmek; aynı seed'lerle
sonucu tekrar ölçmek. Yalnızca yaşam süresini artırmak veya kaynak ekleyerek
sorunları maskelemek kabul ölçütü değil.

Yeniden üretim:

```bash
python3 scripts/measure_ecosystem.py --output data/experiments/baseline-new
```

Ham SQLite kayıtları `data/experiments/baseline-ecd4ec6/` altında tutuldu;
Git'e yalnızca bu rapor ve küçük JSON özeti eklendi. Duvar saati ölçümleri bu
küçük senaryoya aittir; büyük nüfus performans iddiası değildir.
