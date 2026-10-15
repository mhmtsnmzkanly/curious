# İlk çok nesilli dünya — 15 Ekim 2026

## Sonuç ve sınırları

`scenarios/meadow.json` açık başlangıç koşullarında, sekiz seed'in yedisi 2.000
tick sonunda hâlâ yaşıyor. 42–45. nesle ulaşıldı. Seed 2, 270. tickte tükendi;
bu sonuç gizlenmedi veya yeniden canlı eklenerek düzeltilmedi.

Daha uzun doğrulama:

| Seed | Tick | En yüksek nesil | Doğum | Ölüm | Sonuç |
|---|---:|---:|---:|---:|---|
| 1 | 10.000 | 230 | 774 | 770 | Yaşam sürüyor |
| 2 | 10.000 | 2 | 12 | 20 | 270. tickte tükendi |
| 3 | 10.000 | 219 | 778 | 780 | Yaşam sürüyor |
| 42 | 10.000 | 225 | 764 | 764 | Yaşam sürüyor |

Bu **otçul çayırı** bir temel senaryodur. Başlangıçtaki dört kuruculu, 30×30
karışık tür senaryosuyla aynı koşullar değildir; aynı koşullarda tüm ekosistemi
kurtardık iddiası yoktur. Avcı–av dengesi, bireysel öğrenme ve kalıtsal evrim henüz
doğrulanmadı. Nesil sayısının artması tek başına evrim anlamına gelmez.

## Yapılanlar

- Kaynak konumları, miktarlardan ayrı çevre verisi oldu. Kaynak tamamen bitse de
  aynı yerde kapasite/sıklık/miktar kurallarıyla yenileniyor. Popülasyona bakarak
  kaynak veya canlı ekleyen bir mekanizma bulunmuyor.
- Ceset besini bitkiden ayrıldı; otçul et yiyemiyor. Et sonlu, tüketimle azalıyor.
  Ölüm otomatik olarak yenilebilir bitki üretmiyor.
- Canlılar aynı hücreye girmeden komşu kaynaktan yiyip içebiliyor.
- Kaynak seçimi yalnızca yakınlığa dayanmıyor; kullanılabilir miktar ve yol hesaba
  katılıyor. Başkalarının da aynı kaynağı tüketmesi hâlâ başarısız eylem doğurabilir.
- Çevre algısı her kaynak için ayrı BFS yerine tek sınırlı geçiş kullanıyor.
  Dolu hücrelerin içinden geçilmiyor; eş/av/kaynak hedeflerine yaklaşılabiliyor.
- Üreme beklemesi 25 tick. Ebeveynler en az yarım enerji/su rezervi ve sağlıkla
  üreyebiliyor. Her biri enerji/su kapasitesinin dörtte birini harcıyor; yavrunun
  başlangıç rezervi bu transferden geliyor. Kaynak yoktan üretilmiyor.
- Yavrunun nesli ve ebeveynleri checkpointte; doğum/çiftleşme olayları geçmişte.
- Aynı türlerin ortak karar politikası `entity/behavior.rs` içinde. Rastgele
  gezinme artık Idle niyetini motorun gizlice harekete çevirmesiyle yapılmıyor.
- Çatışma önceliği her tick dönüyor; küçük kimlik numarası sürekli ayrıcalık almıyor.
- Senaryolar JSON ile tanımlanabiliyor; dünya, kayıt ve HTTP katmanlarından ayrı.
- Snapshot/checkpoint kodu SQLite katmanından çıkarıldı. Motor snapshot'i kaynak
  türünü, çevreyi ve soyu da içeren aynı sürümlü temsili kullanıyor.

## Ekolojik ayarlar ve deneyler

Çayır 9×9 hücre, 8 otçul kurucu, 16 bitki ve 4 su kaynağı içeriyor.
Bitki kaynağı kapasite 20, tick başına 2; su kapasite 30, tick başına 4 yeniliyor.
Tamamen tüketilirlerse toplam üst üretim 32 enerji ve 16 su/tick. Doluyken fazlası
üretilmiyor. Bu ayarlar senaryo dosyasında açık; nüfusa göre değişmiyor.

Daha düşük üretimlerle yapılan ilk deneyler de arşivde tutuldu:

- `meadow-first`, `meadow-routing`: beslenme/yol bulma ve küçük öğün sorunları.
- `meadow-intake`: öğün kapasitesi artırılınca bir seed 39. nesle ulaştı, diğerleri tükendi.
- `meadow-fair`: dönüşümlü erişim, düşük üretimde popülasyonu kurtarmadı.
- `meadow-production`: bitki üretimi arttı ama su darboğazı sürdü.
- `meadow-water`: yukarıdaki 8 seedlik son senaryo.

Öğün sınırları yemek/su için 10 birim. Bunlar ilk ayarlar; başarı için değişmez
biyolojik doğrular olarak kabul edilmiyor. Maliyetler ve çevre üretimi sonraki
avcı–av deneylerinde de ölçülecek. Henüz farklı başlangıç nüfuslarında geniş
sağlamlık taraması yapılmadı.

## Doğrulama ve performans

19 Rust testi; HTTP kontrol, auth, SIGKILL devam ve kilit testleri geçti.
Yeni testler sonlu besin, komşudan ortak tüketim, dönüşümlü erişim, kaynak
kapasitesi, kaynak kapalıyken yok oluş ve torunların üremesini kapsıyor.

10.000 ticklik canlı koşular 4,5–5,3 saniyede tamamlandı. Bu yaklaşık 6–12 canlılı
küçük dünyadır; binlerce canlı kapasitesi veya aylarca dayanıklılık sonucu değildir.
Popülasyon ve işlem süresi örnekleri `meadow-population.csv` içinde.

Tam JSON snapshot ve olay kayıtları 10.000 tickte yaklaşık 145–148 MB tuttu.
3 tick/s ile bu, kabaca **3,8 GB/gün** demektir. Bu nedenle mevcut 10 GiB bütçe
aylarca kayıt tutmaya yetmez. Bir sonraki işletim işi delta/checkpoint veya kayıpsız
sıkıştırma stratejisini ölçmek; bu maliyet çözülmeden aylarca kayıt vaadi yok.

## Yeniden üretim

```bash
python3 scripts/measure_ecosystem.py --scenario scenarios/meadow.json \
  --output data/experiments/meadow-repeat --ticks 10000 --seeds 1,2,3,42
```

Ham SQLite deneyleri `data/experiments/` altında; Git'te küçük JSON özetleri ve CSV
örnekleri var. Deneyler ana `data/main` dünyasında çalıştırılmadı.
