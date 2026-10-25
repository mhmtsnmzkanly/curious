# Bireysel öğrenme: güncel sözleşme ve açık kanıt kapıları

Bu belge V18 geliştirme kaynağındaki öğrenme sınırlarını ve eldeki sonuçları
özetler. **V18 canlıya dağıtılmış değildir; canlı 7882 dünyası V12'dir.**
Yaş ödülü, yavruya deneyim aktarımı, genel amaçlı sinir ağı veya kabul edilmiş
refuge-risk modeli yoktur.

## Değişmeyen fiziksel sözleşme

- Öğrenme bireyin kendi gözlem ve kendi kimlikli çözücü deneyimine dayanır.
  Gizli beden, gelecekteki RNG, başka bireyin deneyimi veya henüz görünmeyen
  sonuç girdi yapılamaz. Bireyin belleği yavruya kopyalanmaz.
- Öğrenici yalnız mevcut kurallı kararın izin verdiği adayları sıralayabilir;
  fiziksel hareket, saldırı, kaynak tüketimi ve yaşam sonuçları World ve
  resolver tarafından belirlenir. Her ölçüm maliyeti ayrı fiziksel birimde
  raporlar; tek ödül artışı fayda kanıtı sayılmaz.
- Yalnız resolver ile tamamlandığı ve aynı bireye güvenilir biçimde
  ilişkilendirildiği bilinen hedef güncellenebilir. Tam kaçış sözleşmesinde
  ölüm, kesinti veya eksik atıf bölümü sansürler. Ayrı ilk-aralık sözleşmesinde
  tamamlanmış fiziksel aralık, sonraki kaçış sansüründen etkilenmez; seçilen
  tehdidin bilinen hasarı diğer saldırganın hasarından ayrı tutulur. Eksik
  gözlem sıfır hasarlı başarıya dönüştürülmez; bilinmeyen risk “güvenli” demek
  değildir.
- Varsayılan kapalı yol, mevcut politika seçimini ve RNG tüketimini aynen
  korumalıdır. Yeni checkpoint/State göçü, kapalı yol ve State/RNG devamı
  ayrı test edilir. Online ve frozen dalların tüm State/RNG alanlarının eşit
  olması beklenmez: frozen yalnız öğrenilmiş weights/counters güncellemelerini
  dondurur, normal exploration sürebilir. Tam State/RNG eşitliği yalnız
  default-off kaynak uyumu ve checkpoint-resume sözleşmesinde aranır.
  Bireysel tablolar bounded olmalı; sınır aşımında güncelleme atomik
  reddedilmelidir.
- Yaş için öğrenme ödülü veya tercih bonusu yoktur. Yaş ve hayatta kalma
  seçilimi analizde deneyim etkisinden ayrılır.

## V17 ve av öğrenmesi: sonuçlar sınırlı

### Su artığı — V17, default-off

V12 canlı dünyasındaki temel su öğrenmesi ve mevcut genetik yaşam mekaniği,
V17 su artığı gibi ayrı deneysel seçeneklerle karıştırılmaz. Su artığı eski su
puanına aynı bireyin kendi resolver-artığını ekleyebilir;
öğrenme/karar seçenekleri varsayılan kapalıdır. `WATER_RESIDUAL_V1` raporunda
mekanizma fırsatı sekiz seedin üçünde görüldü, fakat kayıtlı bounded-utility
kapısı geçilmedi. Karşılaştırıcılarla fiziksel tradeoff'lar karışıktır; kabul
edilmiş net yarar yoktur. Özelliği live'da açmayın veya sonuçtan sonra eşik
ayarlamayın.

### Av deneyimi/risk — V13/V14, default-off

V13 deneyim tablosu ve V14 risk alanı bounded mekanizma olarak mevcuttur;
checkpoint/migration ve resolver attribution sözleşmesine tabidir. Hunt Lab
V3'te riskli kollar savunma takibindeki yaralanmaları azaltmadı. Deneyim/risk
durumu, fiziksel veya ekolojik üstünlük kabulü değildir; risk seçimini kapalı
tutun. Tekil mekanizma tanıkları politika yararı ya da gerçek hayvan öğrenmesi
iddiası olmaz.

Raporlar: [su artığı V1](reports/WATER_RESIDUAL_V1.md) ve
[hunt lab V3](reports/HUNT_LAB_V3.md).

## Refuge: beklenen hasar hâlâ Unknown

V16 bireysel exact-key refuge sonuç belleği, V18 ise opt-in temsil edilmiş
bağlam seçeneği sunar. Her ikisi de default-off ve yalnız geliştirme
kaynağındadır. Önemli kapsam farkı:

- **Opt-in V18 refuge learner** geçerli alternatifleri bellekte en çok 8
  adayla sınırlar.
- **Mevcut ordinary default-off davranış** tüm geçerli, boş olmayan ve
  işgal edilmemiş sığınak yollarını dikkate alır; bu yola V18'in 8 aday
  sınırını taşımayın.
- Saf residual prototype kendi başına en çok 8 endpoint satırı tutar. Bu ayrı
  prototype sınırıdır; engine veya ordinary policy'ye entegre değildir.

25 Ekim tarihli yeni tek-run engineering ölçümü aynı fixture ve seed 123 ile
1,000 tick tamamladı: 623 attempt, 67 owner, 182 attributable arrival ve 441
censor. Tamamlanan 182 arrival'ın threat damage değeri sıfırdı; **altı pozitif
threat-damage attempt'in altısı da sansürlüydü.** Bu başarıya koşullu support,
expected damage kalibrasyonu sağlamaz. Beş owner-ID fold'undaki zero MAE bu
seçilmiş sıfır hedeflerinin sonucu ve bilgilendirici doğruluk metriği değildir.
Hareket enerjisi ve arrival süresi nominal hesapla 182/182 eşleşti; bu yalnız
koşullu effort aritmetiğini doğrular.

İlk resolver aralığının ayrı extraction'ında 623 başlatılmış bölümün 617'si
sıfır, altısı 20 hasar olarak gözlendi; altı pozitif satır dört owner'a aitti.
Bu **ilk-aralık gözlenen hedefidir**; tam kaçış beklenen hasarı, seçilmeyen
yolun karşıolgusu veya route predictor değildir. Full-route outcome raporunda
182 arrival sıfır hasarlıydı; altı pozitif bölüm daha sonra sansürlendi.
Bu sonraki kapanış sansürü, ilk aralığı eksiksiz gözlenen altı pozitif hedefi
silmez; eksik bir ilk aralık Exact değildir ve Unknown kalır.
Dolayısıyla her route için beklenen threat damage **Unknown** kalır.

Raporlar: [full-route risk baseline V1](reports/REFUGE_RISK_BASELINE_V1.md),
[ilk resolver aralığı](reports/REFUGE_FIRST_RESOLUTION_V1.md),
[önceki teknik onarım doğrulaması](reports/REFUGE_REPAIR_VALIDATION.md) ve
[frozen refuge protokolü](REFUGE_FIRST_RESOLUTION_PROTOCOL.md).

## Pure residual adayının durumu

[`REFUGE_RESIDUAL_CANDIDATE`](REFUGE_RESIDUAL_CANDIDATE.md) dokümanındaki saf
mekanizma helper'ı default-off ve unintegrated kalır: World kararına, resolver'a,
learner belleğine, checkpoint'e veya RNG'ye bağlanmamıştır. Üç fiziksel
bileşeni ayrı tutmayı, kişinin kendi tamamlanmış deneyiminden residual
güncellemeyi ve Unknown/Provisional durumunu sınar. Bu testler transfer
güvenilirliğini ya da route seçimi yararını göstermez.

İlk gerçek entegrasyon önkoşulu her aday yol için güncel gözlemlerden üretilmiş,
kalibrasyon durumu açık ve fiziksel olarak savunulabilir bir damage baseline'dır.
Mevcut tek-run ölçümü bu önkoşulu karşılamadı. Species attack miktarı beklenen
hasar değildir; ilk interval hedefi de tam rota baseline'ı değildir. Herhangi
bir yeni fiziksel ölçüm, doğal tamamlanan outcome desteğini elde etmek için ayrı
review edilmiş, tek amaçlı ve sınırlı protokol ister. Mevcut run'ı tekrar
etmeyin, sonucu gördükten sonra fixture/threshold ayarlamayın.

Bu önkoşul ancak root review ve yeni protokolle karşılanırsa şu sıra izlenir:

1. Baseline ve sansür/attribution kurallarını pure testlerle sabitleyin.
2. Dünya, beden, algı, aksiyon veya resolver değişikliğini karıştırmadan
   bounded, default-off entegrasyon hazırlayın.
3. Checkpoint schema/migration, owner izolasyonu, yavru belleğinin boş
   başlaması, kapalı yol exact-output/RNG uyumu ve resolver muhasebesini
   doğrulayın.
4. Ayrı frozen World acceptance protokolünde mekanizma ve fiziksel sonuçları
   denetleyin. Teknik tamamlama utility/safety/fitness kabulü değildir.

Şimdilik hiçbir risk modeli, neural network, route faydası veya production
entegrasyonu kabul edilmemiştir. Aylar/yıllar süren reliability doğrulaması
kullanıcı isteğiyle ertelenmiştir; mevcut sağlık gözetimi sürer ve uzun soak
feature geliştirme için önkoşul değildir.
