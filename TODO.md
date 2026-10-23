# Sıradaki işler

Hedef [VISION](docs/VISION.md), kullanıcı sırası ve dürüst durum
[ROADMAP](docs/ROADMAP.md), ölçümler [sonuç raporu](docs/reports/ROADMAP_RESULTS.md).

## 23 Ekim — paralel ekosistem planı ve sıradaki işler

Root A/B/C ilk dalga tasarım/prototiplerini inceledi; A water example `769f683`
ile ana dala alındı. A/B/C henüz üretim öğrenme politikası veya ekoloji kabulü
değildir. Ajan 7 A testi ve 11 B targeted kontrolü bildirdi; root `cargo test --all-targets`
181 passed / 0 failed, Python 132 passed ve `cargo build --release` exit 0
bildirdi. Scoped `rustfmt --edition 2024 --check` geçti. C yalnız `cargo check`
aldı; runtime muhasebe doğrulaması yok. Yeni ecology batch'i veya seed koşusu yapılmadı.

Sıradaki işler: A'nın default-off production entegrasyonu, baseline snapshot,
residual persistence ve checkpoint değişikliği; B compatibility calibration/schema
tasarımı; C kaynak accounting ve single-world runtime audit. Ayrıntı ve ilk dalga
dosyaları [PARALLEL_CAPABILITY_PLAN](docs/PARALLEL_CAPABILITY_PLAN.md) içindedir.
Kullanıcı onayı beklenmiyor; root'un teknik incelemesi ve her formal işin
preregistrasyonu ayrı mühendislik kapılarıdır.
- [ ] Kaynak şokundan toparlanmayı çöz: V9 sabit kurallı uzun doğrulama 30/30
  geçti; kaynak şoku 4/8 ile araştırmayı durdurdu. Teşhis ve sonraki sınırlı
  deney [RESOURCE_SHOCK_ANALYSIS](docs/reports/RESOURCE_SHOCK_ANALYSIS.md).
  202 beslenme hedefi / 238 üreme eşiği uyumsuzluğunu ve daralmada yerel eş
  bulmayı ayrı ölç. Otçul toparlanması avcı soyunun toparlanması anlamına gelmez.
  Öğrenme/mutasyon kapıları çalışmadı, yeni dünya açılmadı. Gizli takviye ekleme.
  Dad97e9 adayı normalde 7/8, şokta 6/8 ile reddedildi. V11 varsayılanı eski
  kurala döndürür; tek adımlık rezerv adayı açık tercih gerektirir.
  D36a10a normal 8/8, şok 7/8: seed 2 yaşlılıkla tükendi. Karşılıklı hazır
  olma şartını kaldıran yeni deneysel aday önce seed 2/3 hedefli şok elemesinden
  geçmeli; sonra normal/şok sekizli kontroller. Varsayılan kapalı kalır.
  C4d2b35 hedefli şokta 2/2 geçti, normal kontrolde 7/8 ile reddedildi:
  seed 2 otçulları 17.660 adımda tükendi; kalan şok sekizlisi başlatılmadı.
  Teşhis: av baskısı altında yerel gruplar dağılıyor; son iki birey hazır ama
  görüş dışındaki eşi bulamadan yaşlanıyor. Aynı 16.000 kaydında hazırlık
  kapalıysa 17.660'ta 535 otçul/44 avcı var. Ayrıntı/karşı-olgular:
  [PREY_COLLAPSE_SEED2](docs/reports/PREY_COLLAPSE_SEED2.md). Sıradaki dar
  mekanizma testi küçük grupta güvenli eş arama. V12 opt-in ilk buluşma adayı
  erken kopyada20.000'e449 otçul/51 avcıyla ulaştı,17.400 geç kopyasını
  kurtaramadı; kabul ve uzun tarama yok. [MATE_RENDEZVOUS](docs/reports/MATE_RENDEZVOUS.md).
  Bu incelemede bilinen dolu sığınağa kaçma ve yeniden denemede güvenli hazır
  eşi kaçırma düzeltildi. Birleşik erken597 otçul/45 avcı; geç646/0.
  [LATE_RENDEZVOUS_FIXES](docs/reports/LATE_RENDEZVOUS_FIXES.md).
  Erişilebilir leş algısı düzeltildi; geç24.000 kopyasında606 otçul/28 avcı.
  [PREDATOR_CORPSE_ACCESS](docs/reports/PREDATOR_CORPSE_ACCESS.md).
  Yeni kaynak normal8/8 geçti; shock6/8 reddedildi. Seed3/2026 avcıları
  şok sonrası az sayıda doğumla yaşlanarak tükendi. Aynı durum+RNG auditinde
  27/23 yerinde rezerv öğünü yalnız eş mesafesi yüzünden engellenmiş.
  Deneysel avcı hazırlığında görünür yetişkin yeterli; yerinde öğün şartı
  korunur. Otçul ve fizik değişmedi.
  [SHOCK_PREDATOR_RECOVERY](docs/reports/SHOCK_PREDATOR_RECOVERY.md).
  32ddfb2 hedefli2/2, normal8/8, shock8/8 geçti.
  [Uzun protokol](docs/RESERVE_QUALIFICATION.md):±%10 duyarlılık, yeni
  20001–20030 fixed100k, öğrenme/mutasyon kontrolleri, both100k.
  20 Ekim: bütün11kapı geçti; fixed30/30 ve both30/30×100k.
  Ayrı karma dünyanın kayıtlı başlangıcı ve gerçek süreli gözlemi:
  [QUALIFIED_OBSERVATION](docs/reports/QUALIFIED_OBSERVATION.md).
  [RESERVE_POLICY_ISOLATION](docs/reports/RESERVE_POLICY_ISOLATION.md) ve
  [önceki aday](docs/reports/RESERVE_OPPORTUNITY.md).
- [ ] Su öğrenmesinin ortam değişimi deneyini değerlendir (`learning_trial` reversal);
  V10 su denemeleri gecikmiş sonucu ve gerçek hareket maliyetini izliyor.
  [Öğrenme sözleşmesine](docs/LEARNING_NEXT.md) göre sıradaki iş sınırlı av
  tercihi; yeni öğreniciyi ayrıca karma ekosistemde doğrula.
- [ ] Gerçek 1/7/30 gün gözlemini değerlendirme; raporlayıcı otomatik Notify gönderir.
- [ ] Kalıtım deneyini av baskısında tekrarlama; dağılım değişimini sürüklenmeden ayırma.
- [ ] Ön kayıtlı kaynak döngüsü ve bedelli görüş/dayanıklılık alleli adayını
  mekanizma testleri, temiz kaynak incelemesi ve sınırlı deneyden geçir;
  held-out sonuçları görülmeden protokolü veya parametreleri değiştirme.
- [x] İlk soy görünümü: ebeveyn–birey–yavru bağlantıları, numarayla ölmüş birey
  seçimi ve sayfalı karar arşivi. Kullanım/sınırlar CONTROL.md soy bölümü.
- [ ] Daha büyük nüfusta SQLite/HTTP maliyetini ölçme; motor-only hızını kayıt hızı sanma.
- [ ] Disk dolmadan uzun vadeli arşiv kapasitesi planı; geçmişi sessizce silme.
- [ ] Host dışından gözetim: aynı sunucudaki bütün izleyiciler birlikte durursa içeride
  çalışan bir timer bunu dışarı bildiremez.

Tamamlanan ilk uygulamalar: kaynak yenilenmesi/nesiller, deterministik RNG,
SQLite+arşiv+checkpoint, kontrol/gözlem, hareket sözleşmeleri, bireysel hafıza,
sınırlı öğrenme, bedelli kalıtım/mutasyon, responsive gözlem ekranı, nüfus ölçümü,
izleyici gözetimi, sürümlü ikili/yedekli laboratuvar dağıtımı. Test/commit/temizlik/
önemli Notify döngüsü devam eder.

21 Ekim öğrenme geliştirmesi: varsayılan kapalı av seçimi adayı+fiziksel
muhasebe+V13 kurtarma ve deneyim görünümü eklendi. Av öğrenmesinin gerçek
karşılaştırmalı eğitim/frozen/naive deneyleri ve ayrı ekosistem kabulü açık;
mevcut7882dünya bu adayla yükseltilmez. Su ölçümü EXPERIENCE_PROTOCOL.md
sözleşmesiyle çalışır; sonuçlar deneyim raporunda tutulur.
