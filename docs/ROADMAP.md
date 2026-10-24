# Kullanıcının onayladığı sıra ve gerçek durum

Her aşamada deney, test, not, commit, derleme temizliği ve önemli sonuçlarda
Notify uygulanır. Bir özelliğin ilk sürümü, bütün projenin tamamlandığı anlamına
gelmez. Hızlandırılmış tick gerçek gün/hafta gözleminin yerine geçmez.

1. [ ] Etçil yavru ve avcı soyunun devamı: enerji sorunu düzeldi, yavru olgunlaşıp üreyebiliyor; kalıcı avcı soyu açık.
2. [x] Av seçimi, tahmini maliyet, bırakma ve dinlenme için ilk kurallı politika.
3. [ ] Karma ekosistem: küçük/geniş/yoğun/sığınaklı karşılaştırmalar yapıldı; kalıcı birlikte yaşam sağlanmadı.
4. [x] Hareket, duvar, sığınak, diyet, paylaşım ve çakışma sözleşmeleri.
5. [x] Sınırlı bireysel kaynak/tehlike hafızası ve unutma.
6. [x] Su seçimi için sonuçtan öğrenme ve öğrenmeyen bireyle kontrollü deney.
7. [x] İlk hareket özelliğinde bedelli kalıtım, mutasyon ve nesil dağılımı ölçümü.
8. [x] Birey, karar, geçmiş, hafıza ve soy bilgisi için ilk görsel gözlem ekranı.
9. [ ] Gerçek gün/hafta dayanıklılığı: 10.000 tam kayıtlı tick geçti; gerçek süreli gözlem sürüyor.
10. [x] 10–1.000 kurucu ve farklı haritalarda ölçüm; algı indeksiyle hızlandırma.
11. [x] Ayrı izleyici gözetimi, yedekli dağıtım ve gerçek süreli bildirimler.

## Süren işler

### 24 Ekim — wave-2 final sonucu

V17 calibration 52099 1.400 tickte 8/8 opportunity gate geçti; prefixte 20,
trained-online suffixte 35 fresh choices vardı. Formal 52001–52008 toplam
11.200 tick ve 164.554.619 byte raw ile tamamlandı; tüm 32 focal branch 500 tick
yaşadı. Root 32 physical vector'ı ve üç matched full-state/perception/body/RNG
greedy-score/resolver witness'ı doğruladı (52002/247, 52003/415, 52006/241).

Preregistered bounded utility **FALSE**. Online − rules: acquired water −9,
movement energy +32, terminal energy +4, terminal water −9, health/survival 0.
Online − frozen: acquired water +5, movement energy −13, terminal energy −3,
terminal water +5, health/survival 0. Rule karşısında 8/8 tradeoff; frozen'a
karşı bir dominance, iki tradeoff ve beş identical. V17 default-off; genel
yarar, qualification, deployment, retune ya da yeni batch yoktur. Dokuz seed
rolü seen. [Rapor](reports/WATER_RESIDUAL_V1.md) / [JSON](reports/WATER_RESIDUAL_V1.json).

B 13 synthetic schema check; C 9 measurement check/13 physical World tick geçti.
B/C production entegrasyonu veya fitness kabulü değildir. C body energy unknown
bırakır ve kimlik/event koleksiyonları büyüyebilir. Seed123 engineering çiftinde
2×1.400 tick ve 1.400 matched frame vardır; formal evidence değildir. Root
combined checks: 211 Rust, 140 Python, release probe/app passed. Bu formal lab
process/pending wake yok; coexistence/replay services aktif kaldı, değiştirilmedi.
Root üç inactive target cleanup 967.6 MiB logical; kayıtlı experiment artifacts
korundu. Yeni A hipotezi context-stale residual/current-baseline redundancy'yi
yeni preregistration ile ele alabilir; immediate retune yoktur. B/C integration
ayrı root kararıdır.

### 23 Ekim — paralel capability planı

Altı capability izi A–F ile bounded performans/storage izi G'nin kapsamı ve
kanıt kapıları [PARALLEL_CAPABILITY_PLAN.md](PARALLEL_CAPABILITY_PLAN.md) içinde.
Root A/B/C izole teslimlerini inceledi; A su residual learner'ı V17 source
integration adayı olarak tamamlandı. Varsayılan kapalıdır; açıkken mevcut
`foraging::value` tabanına bireyin kendi resolver doğrulamalı su/movement/time
artığı eklenir. Choice-time baseline episode boyunca tutulur; exact bounded
sum/count kullanılır. V1–V16 göçü policy/table/baseline'ı kapalı/boş yapar,
SQLite V2 değişmez, yavru öğrenilmiş residual almaz. Default-off RNG goldens,
resolver+resume, census/frozen/migration ve sınır testleri geçti.

Root final doğrulaması: Rust 191 passed / 0 failed (39 summaries), Python 132/0,
incremental release build exit 0, HTTP/control/auth/SIGKILL-resume script exit 0,
scoped Rust format check geçti. Bu mekanizma/entegrasyon kontrolüdür; ekolojik
yarar kabulü değildir. A'nın formal online-vs-frozen/naive/rule experiment,
seed seçimi ve preregistration açık; hiçbir formal batch, deployment veya yeni
world başlatılmadı. B compatibility calibration/schema tasarımı ve C fixture
resource accounting/runtime audit açık. Ortak source/checkpoint değişiklikleri
root'un seri review'undan geçer. Formal çalışma için protocol/seed audit gerekir.
Aylar/yıllar reliability soak'u kapsam dışıdır.
- Asıl açık biyolojik konu: avcı nesillerinin kalıcı olması. Sığınaklı seed
  12345'te 20. nesle ulaştı, 907'den sonra avcı kalmadı. Başarı diye etiketlenmez.
  Sıradaki teşhis bu soyda çift bulma, rezerv ve ölüm zamanlarını ayırmak.
- Gerçek süreli 9. aşama otomatik örnekleniyor. 1/7/30 gerçek gün ve yeterli
  örnek birikince Notify raporu gönderilecek; henüz gün/hafta geçti denemez.
- Öğrenme tek karar alanında; evrim tek özellikte. Genel öğrenme, kalıtsal yeni
  yeteneklerin kendiliğinden oluşması ve açık uçlu karmaşıklık henüz yok.

Ayrıntılı sonuçlar: [toplu rapor](reports/ROADMAP_RESULTS.md), aşama raporları
`reports/ROADMAP_01.md`–`ROADMAP_11.md`, son habitat deneyi `reports/REFUGE.md`.

Av–avcı için onaylanan yeni11 aşamalı çalışma `COEXISTENCE_PLAN.md` içindedir.
V8 teşhisi ve V9 düzeltmeleri uygulandı. Kalibrasyon/duyarlılık/uzun testler ve
koşullu ayrı dünya için çalıştırıcı hazırlandı; biyolojik kabul kapıları henüz
geçilmiş değildir. Ayrıntı `reports/COEXISTENCE_IMPLEMENTATION.md`.

## 20 Ekim 2026 — karma kabul

32ddfb2 rezerv/buluşma koşulu, sabitlenen yeni30seedde fixed ve both için
ayrı30/30×100000 adım ve son bölüm üreyen yavru soyu kapısını geçti.
Duyarlılık ve şok/öğrenme/mutasyon kapıları da tamamlandı. Bu, 1 ve3 için
sınanmış koşulda güçlü süreklilik kanıtıdır; sonsuz denge veya genel evrim
iddiası değildir. Gerçek gün/hafta9 hâlâ açık. Ayrı dünya ve işletim kanıtları
[QUALIFIED_OBSERVATION](reports/QUALIFIED_OBSERVATION.md) raporunda tutulur.

## 23 Ekim 2026 — ekoloji takvimi ve bedelli fizik adayları

İsteğe bağlı coğrafi kaynak takvimi ile kalıtsal görüş/dayanıklılık allelleri
uygulama ve mekanizma test aşamasında. Yeni fizik maliyeti enerjiyle ödenir;
varsayılan opt-in kapalıdır. Sabit karşılaştırma tasarımı
[ECOLOGY_TRAITS_PROTOCOL](ECOLOGY_TRAITS_PROTOCOL.md) içinde önkayıtlıdır;
henüz calibration veya held-out fitness değerlendirmesi çalıştırılmadı.
Kayıtlı çalışma temiz kaynak commiti ve incelemesinden sonra yürütülür.
