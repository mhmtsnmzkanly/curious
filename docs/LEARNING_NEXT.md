# Bireysel öğrenme: sözleşme ve kanıt sınırları

Bu belge V20 geliştirme kaynağındaki bireysel öğrenme sınırlarını ve güncel
kanıt durumunu özetler. V19 canlıya dağıtılmamıştır; canlı dünya V12'dir.
Yaş ödülü, yavruya deneyim aktarımı veya kabul edilmiş refuge-risk modeli yoktur.

## Fiziksel ve epistemik sözleşme

- Öğrenme bireyin kendi gözlemi ve kendi kimlikli resolver deneyimine dayanır.
  Gizli beden, gelecek RNG, başka bireyin deneyimi veya görünmeyen sonuç girdi
  yapılamaz. Öğrenilmiş bellek yavruya kopyalanmaz.
- Öğrenici yalnız kurallı kararın sunduğu adayları sıralar. Fiziksel hareket,
  saldırı, kaynak tüketimi ve yaşam sonucu World/resolver tarafından belirlenir.
  Maliyetler fiziksel birimleriyle raporlanır; ödül toplamı tek başına fayda
  kanıtı değildir.
- Hedef ufku içinde eksik gözlem, dış hasar, atıf belirsizliği veya kapanmamış
  bölüm `Unknown`/sansürlü kalır; sıfır maliyet ya da başarıya çevrilmez.
  Tamamlanmış ilk resolver aralığı sonraki tam-rota sansüründen ayrı hedeftir.
- Default-off uyumluluğu, State/RNG göçü ve checkpoint resume ayrı doğrulanır.
  Frozen öğrenilmiş parametreleri sabit tutar; normal politika keşfi sürebilir.
  Online ve frozen dalların tüm State/RNG alanlarının eşit olması beklenmez.
- Bireysel tablolar bounded olmalı; sınır aşımında güncelleme atomik reddedilir.
  Yaş ve hayatta kalma seçilimi öğrenme etkisinden ayrı ölçülür.

## Güncel kanıt

- [Su uyum V2](reports/WATER_ADAPTATION_V2_20261025.md) geçerli gıda-zengin
  fixture'da yürütüldü. Deneyimli-online, frozen ve learner tabloları
  sıfırlanmış online kolların eylem/hedefleri ve fiziksel yörüngeleri aynıydı;
  ek deneyim faydası kabul edilmedi. Kurallı politika farkı öğrenme etkisi
  sayılmaz. Bir formal seed düşük prefix desteği nedeniyle öğrenme kabulüne
  uygun değildir. [V1 başarısız kalibrasyonu](reports/WATER_ADAPTATION_V1_20261025.md)
  korunur.
- [Su sinyali tanısı](reports/WATER_SIGNAL_DIAGNOSIS_20261026.md) bulduğu
  çoğunlukla sıfır residual ve değişmeyen seçim sıralaması nedeniyle ayrı,
  bireysel görünen-hizmet-tahmin hedefi önerir. Bu yalnızca
  [hedef sözleşmesidir](SOURCE_RELIABILITY_TARGET_20261026.md): uygulanmış kod,
  onaylı protokol, yeni World deneyi veya kabul edilmiş öğrenme faydası değildir.
- [Nitelik tanısı](reports/QUALIFICATION_DIAGNOSTIC_20261025.md) daha önce
  görülmüş 401–410 seedlerinde kısa fixture bakım/paket kolları için 2 iyileşme,
  8 nötr; refuge için 10 nötr sonuç verdi. Bu bağımsız held-out kabulü değildir.
  V12 arşiv başlangıcından offline V19 gölgesinde paket, V19 kapalı kontrole
  göre endpoint'te 13 fazla canlı ancak 1,428 daha az birey-tick üretti. Canlı
  fayda, Pareto kabulü veya V19 yükseltmesi yoktur.
- V17 su artığının kayıtlı bounded-utility ölçütü geçilmedi. V13/V14 av-risk
  deneyleri de fayda kabulü sağlamadı. Bunlar ayrı default-off seçeneklerdir;
  [ROADMAP](ROADMAP.md) kaynak kapsamını ve kanıt bağlantılarını taşır.
- Refuge tam-rota beklenen tehdit hasarı `Unknown`. [Full-route baseline](reports/REFUGE_RISK_BASELINE_V1.md)
  ve [ilk resolver aralığı](reports/REFUGE_FIRST_RESOLUTION_V1.md) farklı
  hedef ufuklarıdır; ilk aralık etiketi route predictor değildir. Saf residual
  prototipi World/karar/checkpoint yoluna bağlanmamıştır.

Yeni öğrenme iddiası ayrı hipotez, önceden sabitlenmiş protokol ve uygun
karşılaştırma gerektirir. Aynı kimlikle deneyi uzatmak, sonuçtan sonra eşik
ayarlamak veya politika farkını öğrenme kazancı saymak kabul kanıtı değildir.
Ekoloji aktarımı ve offline operasyon sonuçlarının kapsamı
[ROADMAP](ROADMAP.md)'de özetlenir; hiçbiri genel öğrenme faydasını veya
dağıtım uygunluğunu kabul etmez.
