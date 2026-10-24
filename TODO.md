# Sıradaki işler

Hedef [VISION](docs/VISION.md), kullanıcı sırası ve dürüst durum
[ROADMAP](docs/ROADMAP.md), ölçümler [sonuç raporu](docs/reports/ROADMAP_RESULTS.md).

## 24 Ekim — güncel A/B/C handoff

A V17 calibration 52099 1.400 tickte preregistered gates 8/8 geçti (20 prefix,
35 trained-online suffix fresh choices). Formal 52001–52008 toplam 11.200 tick
ve 164.554.619 byte raw ile tamamlandı; 32 focal branch'in tümü 500 tick yaşadı.
Root 32 physical vector'ı ve üç full normalized state/perception/body/RNG
greedy-score/resolver tanığını doğruladı (52002/247, 52003/415, 52006/241).

Bounded utility **FALSE**. Trained-online vs rules: acquired water −9 / paid
movement energy +32 / terminal energy +4 / terminal water −9 / health-survival
0. Vs trained-frozen: acquired water +5 / movement energy −13 / terminal energy
−3 / terminal water +5 / health-survival 0. Rules'a karşı 8/8 tradeoff;
frozen'a karşı 1 dominance, 2 tradeoff, 5 identical. V17 default-off kalır;
genel fayda, qualification, deployment, retune veya yeni batch yoktur. Tüm dokuz
registered seed rolü seen. [Rapor](docs/reports/WATER_RESIDUAL_V1.md) ve
[JSON](docs/reports/WATER_RESIDUAL_V1.json).

B refuge schema 13 synthetic check; C resource/lineage measurement 9 check ve
13 physical World tick ile doğrulandı. B/C production integration veya fitness
kanıtı değildir. C body energy unknown kalır; audit detail 2.000 satırla
sınırlıdır, identity/event state'i büyüyebilir. Seed123 engineering twin
2×1.400 tick / 1.400 matched frame formal evidence değildir. Root combined
checks: 211 Rust, 140 Python, release probe/app passed. Bu formal lab için aktif
process/pending wake yoktur; coexistence/replay services aktif ve değiştirilmedi.
Root target cleanup 967.6 MiB logical; raw/probe/protocol/frozen source korundu.
Yeni A işi context-stale residual/current-baseline redundancy için yeni
preregistration gerektirir; immediate retune yoktur. B/C integration root'un
ayrı kararıdır.

## 23 Ekim — paralel ekosistem planı ve sıradaki işler

Root, A/B/C ilk dalga teslimlerini inceledi; A'nın source integration adayı V17
olarak tamamlandı. `policy.water_residual_enabled` varsayılan kapalıdır; açıkken
güncel foraging puanına bireyin own-source resolver artığı eklenir. Choice-time
baseline episode boyunca sabit kalır. Exact bounded sum/count, own actor/source
attribution, persistent censoring, frozen update, V1–V16 migration, SQLite V2
unchanged, default-off RNG goldens ve yavruya boş learner doğrulandı.

Root final checks: Rust 191 passed / 0 failed (39 summaries), Python 132/0,
release build exit 0, HTTP/control/auth/SIGKILL-resume check exit 0, scoped
Rust formatting passed. A için formal deney veya seed seçilmedi; deployment,
live world ve yeni ecology batch yok. Sıradaki iş root preregistration ve seed
audit'tir: strong rule baseline, naive online, trained online, aynı eğitimli
tabloyla trained-frozen. Önceki 51001–51008 ve tarihsel su seedleri held-out
olarak tekrar kullanılmaz. B'nin compatibility calibration/schema tasarımı ve
C'nin fixture/resource muhasebesi ayrı açık işlerdir. Ayrıntı:
[PARALLEL_CAPABILITY_PLAN](docs/PARALLEL_CAPABILITY_PLAN.md).
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
