# Curious üzerinde çalışma

## 23 Ekim 2026 — yeni sürekli-yaşam su öğrenmesi devri

Kullanıcı, bireyin gerçek yaşam geçmişinin karar ve fiziksel sonuca etki
etmesini görmek istiyor. İlk yaşam-sürekliliği raporu
[LIFETIME_CONTINUITY.md](docs/reports/LIFETIME_CONTINUITY.md)
kimlik/beden/yaş/checkpoint devamlılığını doğruladı; sekiz kısa yaşamda online
ve frozen kararlar aynıydı. Salt-okunur neden incelemesi
[LEARNING_EXPOSURE_DIAGNOSIS.md](docs/reports/LEARNING_EXPOSURE_DIAGNOSIS.md)
içindedir. Su öğrenicisi
yalnız kural kararı SeekWater ve en az iki pozitif algılanan su bulunduğunda
seçim yapar; mevcut tarihsel sonuçta yeterli tekrar/yenilenme kontrastı yoktu.
Bu bir motor hata bulgusu değildir.

Yeni dar protokol [LIFELONG_WATER_PROTOCOL.md](docs/LIFELONG_WATER_PROTOCOL.md); kök preregistrasyonu
`02b3d89` ile sabitledi. Probe ve ham şema `8f58d14` ile sabitlendi; Rust
çekirdeği/politika/öğrenici değiştirilmedi. Yedi Rust örnek testi yalnız kayıt
dışı seed 123 fixture'ıyla geçti. Kalibrasyon seed 51099 için 1.400 fiziksel
tick yakalandı; ilk Python doğrulaması başlangıç RNG'sinin doğrudan seed'e eşit
olmasını yanlış biçimde şart koştuğu için başarısız oldu. Dünya kurucusu seeded
RNG ile kaynak yerleşimini çalıştırıp ardından haritayı fixture ile değiştirir;
beklenen başlangıç RNG'si bu deterministik çağrı dizisiyle hesaplanmalıdır.
Orijinal raw ve başarısız manifest değişmeden korunur; sıfır-tick validator
reanalysis'i [`LIFELONG_WATER_REANALYSIS.md`](docs/LIFELONG_WATER_REANALYSIS.md)
içindedir. 51099 yalnız kalibrasyonda kullanıldı; formal 51001–51008 koşuları
başlamadı ve kalibrasyon kapısı yeniden doğrulanana kadar başlamaz. Gate,
biyoloji, fixture ve bütçe değiştirilmez.

Kök dış runtime'a iki 60 saniyelik completion timer kurdu:
`curious-lifetime-learning-calibration-completion` ve
`curious-lifetime-learning-completion`. Dondurulmuş helper/config ve systemd
root sorumluluğundadır. Bunlar sırasıyla
`data/experiments/lifetime-lab-v2-calibration-20261023/` ve
`data/experiments/lifetime-lab-v2-20261023/` V2 şemalarını izler. Formal için
önce ayrı kalibrasyon kanıtını ve kök yetkilendirmesini doğrula. İlerleme
bildirim sayfası `p_20c1696770ea5942`.

Tarihsel ham deneyler, arşivler, canlı dünyalar, frozen release'ler ve eski
lifetime-lab timerları değiştirilmez. Yeni iş/servis durumunda gerçek manifests
ve raw kanıtı oku; eski roadmap notlarını güncel çalışma kanıtı sayma.

## Ajan bağımsız beceriler

Göreve uygun talimatı [.agents/skills/README.md](.agents/skills/README.md)
kataloğundan seçip ilgili `SKILL.md` dosyasını oku: geliştirme, deney, işletim veya
denetim. Dört beceri de düz Markdown kullanır; özel ajan/model/araç gerektirmez.
Otomatik keşif yoksa dosyayı doğrudan okuyarak uygula. Katalog görev kapsamını veya
mevcut kullanıcı yetkisini genişletmez; tarihsel durum yerine güncel kanıtı doğrula.

## 20 Ekim — kabul tamam, ayrı karma gözleme geçiş

Bütün11kapı ham sonuçlardan yeniden doğrulandı; fixed30/30 ve both30/30×100k,
duyarlılık4×4/4, shock8/8 ve üç kısa öğrenme/genetik8/8 geçti. Kaynak32ddfb2
biyolojisi korunur. reports/QUALIFIED_OBSERVATION.md ve JSON nihai kanıttır.
prepare_observation.py + Rust örneği aynı açık seçeneklerle yeni tick0 oluşturur;
servis --resume kullanır, eksik veriden yeniden kurucu üretmez. Port7882,
bin/curious-qualified yeni sürüm; eski curious-coexistence linki kullanılmaz.
Yeni kapalı shardlar zstd; gzip okunur, her bayt doğrulanmadan ham dosya silinmez.
0,03tick/s,64MiBshard,32GiB bütçe: kısa örnekten~0,70GiB/gün tahmini, garanti
değil. Gerçek1/7/30gün ve uzun vadeli disk kapasitesi hâlâ açık. Yeni tarama
başlatma; ayrı gözlem servisinin gerçek durumunu ve deployment.json kaydını oku.
87Rust/52Python+HTTP/SIGKILL+Chromium/kompreseli soy geçti. Kaynaklar frozen
worktree'den çalıştırılır; 7878–7881, eski araştırma/veriler/ikili korunur.


Başlamadan `docs/VISION.md`, `docs/CONTROL.md`, `docs/ROADMAP.md` ve `TODO.md`
okunur. Kullanıcının sonraki talimatları bu notların önündedir.

Yeni altı aşamalı ekosistem öğrenme görevinin sahipliği, kabul sınırları ve
durumu [`docs/ECOSYSTEM_LEARNING_PLAN.md`](docs/ECOSYSTEM_LEARNING_PLAN.md)
dosyasındadır. Her faz için protokol ve bounded lab kanıtı gerekir; root
protokolü/commitleri/deneyi denetler, Luna6 kaynak kodu ve ölçüm araçlarını
hazırlar. Completion helper v2 hunt/lifetime/herbivore/ecology-traits schema
adlarını izinli kılar. Üç yeni bounded-lab timerı root tarafından kuruldu;
terminal kanıtlar alındıktan sonra devre dışı bırakıldı. Runtime config, frozen
helper SHA ve systemd denetimi root sorumluluğundadır.

Hunt-risk V3 ve aynı bireyin yaşam sürekliliği laboratuvarları tamamlandı;
sonuç ve sınırlamalar `docs/ECOSYSTEM_LEARNING_PLAN.md` ile bağlı raporlardadır.
Risk seçimi canlı ayarda varsayılan kapalıdır. Completion timer V3 ve lifetime
çalışmalarını doğal terminal poll'unda bir kez `codex queue` ile iletti; gerçek
queue kanıtı plan belgesindedir. V2 helper ile lifetime/herbivore/ecology-traits
timerları ops tarafından kuruldu; dış config, frozen helper ve systemd denetimi
root sorumluluğundadır.

Faz 3 V16 core `970b974` içinde varsayılan kapalıdır; 153 Rust testi,
HTTP/SIGKILL kontrolü ve 100/200 tick frozen-source/State/RNG eşitliği root
testlerinden geçti; eski checkpoint göçü V15 alanlarını korur. Prereg ve
bounded lab source/protocol `60ab151` ile sabitlendi. Calibration seed45999
134 tickte, formal seed45001–45008 toplam1.148 tickte tamamlandı. İlk validator
denemesi enum isimlerini normalize etmediği için başarısız oldu; düzeltilmiş
analiz yalnız korunmuş ham çıktıyı kullandı ve sıfır yeni tick çalıştırdı.
574 eşleşmiş tickte fiziksel state/RNG ve fiziksel resolver olayları aynıydı;
tek tekrarlanan benzer bağlam seed45002'de tick66 ve82'ydi. Kapatışlar
`threat_lost` sonra `died` olarak sansürlendi; güvenli oldukları doğrulanmadı,
ancak `safe_arrival=false` da güvensiz olduklarını kanıtlamaz. Bu nedenle
iyileşme karşılaştırması tanımlanabilir değildir. Gözlenen davranış/fiziksel
yarar yok; özellik varsayılan kapalı kalır. Kanıt ve sınırlamalar
`docs/ECOSYSTEM_LEARNING_PLAN.md` ile bağlı raporlardadır. Canlı dünya,
dondurulmuş ikili ve tarihsel ham deney arşivleri değiştirilmez.

Faz 5 salt okunur bireysel geçmiş görünümü `4c2cef7` içinde uygulanıp
`5736e6d` içinde sınırlandırılmış panellerle iyileştirildi. Güncel salt okunur
dağıtım frozen `ba71b7f` kullanır; üç bireysel geçmiş kaydı gözlendi, dünya
yeniden başlatılmadı. Bu görünüm tam yaşam boyu biliş iddiası değildir.

## Rezerv adayının uzun kabul kampanyası — 18 Ekim

32ddfb2 hedefli şok2/2, normal8/8, şok8/8 geçti; önceki unit bitti.
Kanıt `docs/reports/RESERVE_TRAINING_ACCEPTANCE.json`. Sıradaki protokol
`docs/RESERVE_QUALIFICATION.md`: dört±%10 duyarlılık×4seed→yeni20001–20030
fixed100k→geçmişshock8 salt okunur doğrulama→learning/mutation/both eğitim8
→aynı sabit30seed both100k. Yeni liste mevcut manifestlerde kullanılmamıştı;
sonuç sonrası liste/eşik değiştirilmez,27/30 ve son%10 soy/doğum şartı korunur.
Eski10001–10030 artık görülmüş olduğundan yeni kabul diye sunulmaz.

Kök `data/experiments/reserve-qualification`; gerçek `config/protocol/state.json`
ve aşama `progress.json`/log'larından okunur. Unit `curious-reserve-qualification`;
Notify timer aynı adın `-notify.timer` varyantı. Saf Python çalıştırıcı güncellendi;
Rust/ikili32ddfb2 korunur. Derleme kimliği ile runner commit'i farklıdır;
src/Cargo.toml/Cargo.lock aynı olmalı. Bayraklar her aşamada geçer ve çıktıda
teyit edilir. Geçmiş normal/şok ham sonuçları SHA+ayar eşitliğinde yeniden
hesaplanır; eski deney dosyaları değişmez.49 Python+gerçek probe/resume+Node geçti.

Protokol değişiminde resume reddedilir; biyolojik ret sonraki kapıyı durdurur.
İlerleme sayfası timer ile güncellenir, önemli sonuçlar kalıcı Notify outbox'tan
iletilir. Global watchdog yeni protocol.json kampanyalarının ana heartbeat ve
bildirim kuyruğunu ayrı izler. Final qualified bildirimi timerda teklenir;
bildirim worktree/revision kimliği source.json/configte ayrıca kaydedilir. İki işçi,%180CPU/512MiB; tüm kapılar geçerse
~2gün sürebilir. Ajan arka planda yeni kod üretmez. Otomatik dağıtım kapalı:
eski live_deploy açık deneysel bayrakları taşımadığı için artık o durum reddedilir.
Kabulden sonra ayrı karma dünyaya aynı ayarlarla kayıtlı başlangıç işi kalır.
7878–7881 veya eski frozen worktree/deneyleri değiştirme.

## Şok sonrası avcı rezervi — 18 Ekim

C2d9da8 normal8/8, shock6/8 ile reddedildi. Seed3/2026 avcı tükenmeleri
13584/14751. Eski biyolojik çıktılar tekrarlandı (süre/istenen son pencere
sayaçları hariç);11000→13000 audit bütün durum+RNG ile eşit.27/23 yerinde
hazırlık fırsatı yalnız türdeş bitişik değil diye kaçmış. Yeni dar aday avcıya
aynı görünür yetişkin için bitişik öğünle hazırlanma izni verir; yemek yolculuğu,
fizik, otçul ve varsayılan kapalı bayrak değişmez. V12/SQLiteV2 korunur.
Kopyadan15000'de578/47 ve572/60 (otçul/avcı), son1000'de38/51 avcı doğumu;
11000 avcıları tamamen değişmiş, yaşayan yavru ebeveynler var. Kalıcı kabul değil.
Rapor `docs/reports/SHOCK_PREDATOR_RECOVERY.md`/JSON;83 Rust/37 Python,
Node ve HTTP kurtarma geçti. Eski ve kapalı kontrol çıktıları eşit.

Yeni adayın başlangıçtan kapıları hedefli3/2026 şok→normal8→shock8.
`data/experiments/visible-adult-reserve-validation/{study,state}.json` ve
`curious-visible-adult-reserve-validation` gerçek başlangıç/durum kaynağıdır.
Biyolojik ret sonraki aşamayı durdurur; program kod değiştirmez/dağıtmaz.
Eski `curious-corpse-access-validation` inactive/rejected beklenir. Observer
tekrarları ve kısa kollar bitti; ham veriler
`data/experiments/{shock-predator-diagnosis,visible-adult-reserve}` korunur.
Canlı dünyalar yükseltilmedi. Uzun görülmemiş seed kabulü hâlâ bekliyor.

## Avcı leşi algı düzeltmesi ve doğrulama

Geç avcı tükenmesinde erişilebilir sığınak leşi yürünebilirlik filtresiyle
gizleniyordu; algı artık mevcut Eat fiziğine uygun bitişik yaklaşımı kurar.
Canlı sığınak avı korunur. Rapor `docs/reports/PREDATOR_CORPSE_ACCESS.md`/JSON.
Geç20.000:628/5; devam24.000:606/28 (otçul/avcı), devamda53 avcı doğumu.
Erken20.000:576/69. 81 Rust/35 Python+Node+HTTP kurtarma geçti.
Bu tek tohum deneyi kalıcı denge kabulü değildir. Sonraki bounded normal8,
yalnız geçerse shock8 çalışmasının kimliği/durumu
`data/experiments/corpse-access-validation/{study,state}.json` ve
`curious-corpse-access-validation.service` üzerinden kontrol edilir. Eski
canlılar yükseltilmez; V12/SQLiteV2 ve kapalı varsayılanlar korunur.
Aşağıdaki eski raporlar tarihsel sonuçtur; yeni algı için kabul sayılmaz.

## Geç kopya: iki somut karar düzeltmesi

Rapor `docs/reports/LATE_RENDEZVOUS_FIXES.md` ve JSON. Tam algılı eski geç tekrar
bütün çıktı+RNG ile eşit.1.109 sığınma kararının486'sı bilinen dolu hedef;
476'sında boş alternatif var. Kaynak artık bunları eler (buluşma bayrağından
BAĞIMSIZ temel politika düzeltmesi). Deneysel buluşmada57 güvenli görünür
hazır eş fırsatı yeniden deneme arasıyla engellenmiş; artık süreyi sıfırlamadan
tek adım yapılabilir. Gerçek bekleme WaitForMate, ertelenme PauseMateSearch.
V12/SQLiteV2 değişmez; rezerv ve buluşma varsayılan false, eski canlılar korunur.

Aynı geç kayıtta yalnız eş düzeltmesi yine0/0; yalnız sığınak650 otçul/0 avcı;
birleşik646/0. Erken16.000→20.000 birleşik597 otçul/45 avcı. Geç avcı nesli
hâlâ yenilenmez:4 doğum/45 ölüm (25 açlık/15 yaş/5 su). Uzun tarama/dağıtım yok.
76 Rust/33 Python +HTTP kurtarma geçti. Ham audit ve patch/SHA varyantları
`data/experiments/rendezvous-late-{audit,corrections}`. Son kaynak birleşik
varyant hash'leriyle aynı; geçici retry-only kaynak değişikliği geri alındı.
Sıradaki inceleme geç avcıların besine erişimi ve üreme rezervidir; otçulun
hatalı kaçışına geri dönerek dengeyi taklit etme. Eski V9 uzun kabul yeni temel
kaçış düzeltmesinin kabulü sayılamaz.

## V12 deneysel eş buluşması

Kaynakta `policy.mate_rendezvous` varsayılan false; sadece AI otçulu,
hafıza/türdeş hafızası açıkken uygular. `entity/social.rs`, tek hedef30tick,
retry10tick, visited32; gereksinim/tehlike/gerçek çiftleşme önceliği korunur.
V1–V11 kayıt göçünde kapalı/boş başlar; SQLite V2 değişmez. Kontrol noktası
örneklerinde bayrak göçten SONRA açıkça açılır. Ayrıntı CONTROL.md V12.
Bu adayın uzun denge kabulü henüz yok; canlı servisler güncellenmez.

## Soy arayüzü ve son aday sonucu

Soy arayüzü web/archive.js üzerinden mevcut salt okunur query API'sini kullanır;
CONTROL.md soy bölümü kapsam/sınırlar, scripts/test_archive.cjs SQL doğrulamasıdır.
UI değişikliği için uzun süreli dünyaları veya deney ikililerini güncelleme.

C4d2b35 mature adayı hedefli şokta 2/2 geçti, normalde 7/8 ile reddedildi.
Seed 2'de 17.660'ta otçullar tükendi (5 avcı kaldı). Normal kapısı geçilmediği
icin sekizli şok çalışmadı. Kaynak varsayılanı false kalır; yeni bir aday için
bu normal rejim hatasının teşhisi gerekir. Eski deney sonuçları korunur.

## Buluşma adayının kısa sonucu

`docs/reports/MATE_RENDEZVOUS.md` ve SCREEN.json. `bba1495` erken16.000
kopyasını20.000'e449 otçul/51 avcıyla taşır; geç17.400 kopyası toparlanmaz,
iki tür de biter.17.600 negatif kolunda hedef/sosyal karar0. Kapalı kontrolün
bütün eski alanları+RNG eşit; V12 aktif hedef devamı da eşit.71 Rust/30 Python
ve HTTP kurtarma kontrolleri geçti. Varsayılan kapalı, dağıtım ve yeni uzun
arama yok. Veriler `data/experiments/mate-rendezvous-checkpoints`; dört kol bitti.
Sıradaki dar inceleme geç koldaki üreme/tehlike/beslenme kesintileridir;
başarısız kol anlaşılmadan parametre veya ömür körlemesine ayarlanmaz.

## Seed 2 normal rejim teşhisi

Rapor `docs/reports/PREY_COLLAPSE_SEED2.md`; kanıt COMPARISON/REPLAY JSON'ları.
Ayrıntılı tekrar bütün biyolojik probe çıktısını birebir yeniden üretti.
16.000→17.000 checkpoint tekrarında tam durum+RNG eşit. 16.000'de hazırlığı
kapatmak 17.660'ta 535 otçul/44 avcı bırakır; asıl adayda 0/5. Bu, eski
reproductive kuralının tam tekrarı değildir. Son iki otçul fizyolojik hazır,
eş görüşü dışında; 17.634/17.660'ta yaşlılıktan ölür. Son 60 adımda 92
kararın hepsi fizyolojik hazır, doğum0. Av baskısını kopyada 17.400'te kaldırmak
78→606 otçul getirir; 17.600'de kaldırmak son ikiyi kurtarmaz. Müdahaleler
teşhistir, çözüm/kabul değildir. Canlı dünya ve varsayılan false korunur.

Ham veri ve sabit teşhis ikilisi/kontrol noktaları:
`data/experiments/seed2-collapse-diagnosis`. Koşular tamamlandı; yeni uzun
arama başlamadı. Sonraki öncelik küçük grupta yerel, güvenli eş aramasını
sınamak ve bu checkpointlerde demografik toparlanmayı ölçmek. Önce mekanizma,
sonra normal/şok/bağımsız seed kapıları. Yeni denge kanıtı yok.

## Seed 2 toparlanma takibi

D36a10a açık adayın normal 8/8, şok 7/8 deneyi bitti; seed 2 avcıları
13.752'de tükendi. Şokta hiç doğum olmadığı iddiası yanlış: 10.100–11.000
arasında 3 doğum/52 ölüm, 11.000–12.600 arasında 4 doğum/7 ölüm var.
Son 10 avcı ölümü OldAge. Kanıt POLICY_SEED2_FOLLOWUP.json ve açıklama
RESERVE_POLICY_ISOLATION.md son bölümünde.

Sonraki deneysel düzeltme, rezerv hazırlığında türdeşin reproductive yerine
mature olmasını arar. İki düşük rezervli yetişkinin karşılıklı hazır olma
beklentisini kaldırır; çiftleşmenin fiziksel eşiği değişmez. Varsayılan kapalıdır.
Hedefli ön elemede 2/3 seedleri aynı kaynak şokuyla sınanır; geçerse normal 8,
sonra şok 8. Kod üretme/otomatik parametre değiştirme veya dağıtım yoktur.
Çalışma yolu `data/experiments/policy-mature-validation`, servis
`curious-policy-mature-validation.service`; yalnız dosya/servis varsa başlamıştır.

## V11: reddedilen rezerv davranışının ayrılması

Dad97e9 normal kontrolde 7/8, kaynak şokunda 6/8 kaldı; başarısız adaydır.
Ana kaynak V11'de varsayılan açlık davranışı ae7f0df'ye geri getirildi.
`World.policy.reserve_opportunities` varsayılan false; deneysel yeni aday
`policy.rs` içinde sadece bitişik ve yeterli besinle tek adımlık fırsat uygular.
Uzak besine gidip eşten uzaklaşma yolu kaldırıldı. Kanıt/protokol
`docs/reports/RESERVE_POLICY_ISOLATION.md`; kontrol seçenekleri CONTROL.md V11.
Deneysel davranış kabul edilmeden varsayılanı true yapma. Dünya ayarı kayıtta
saklanır; eski V1–V10 kayıtlarda eksikse false. SQLite V2 değişmedi. Eski V9 ve
dad97e9 deney worktree/ikili/verilerini değiştirme; canlı dünyalar güncellenmedi.

## V10 geliştirme — gecikmiş su sonucu

V10 gecikmiş öğrenmeyi ekledi; güncel checkpoint V11, SQLite V2 değişmedi. `ae7f0df` araştırma
worktree'si hâlâ V9 ve temiz tutulur; ikilisi, config'i ve çalışan dünyalar
bu geliştirmede değiştirilmez. Yeni kaynak henüz canlı servise dağıtılmadı.
Su seçimi tek bir denemeyi en çok 20 yürütülmüş adım izler. Kesintiler puanlanmaz;
gerçek hareket enerjisi EntityMoved.energy_cost olayından hesaplanır. Ayrıntı
`docs/CONTROL.md` V10 ve `docs/LEARNING_NEXT.md`; genel av öğrenmesi henüz yok.
V10 kaydını eski ikiliyle açma. V1–V9 learner.pending atılır; değerler korunur.

## V9 av–avcı çalışması

- Kullanıcı 11 aşamalı kalıcı av–avcı planını uygulamayı onayladı. Plan:
  `docs/COEXISTENCE_PLAN.md`; kanıt ve koşullu akış:
  `docs/reports/COEXISTENCE_IMPLEMENTATION.md`.
- Seed12345/1 V8 teşhisi tamamlandı. Kaynak V9'da ciddi yaralanma rezervi,
  zamana/enerjiye göre kaynak seçimi ve görülen yetişkin türdeş hafızası var.
- Bedenin yeni alanları: `max_meal` (eski kayıt/default10),
  `reproduction_interval` (25), `reproduction_reserve_per_mille` (500).
  Varsayılan türler uzun ömürlü deney profiline çevrilmedi. V1–V8 okunur;
  SQLite V2 değişmedi. V9 kaydını V8 ikiliyle açma.
- Bir prototipte20.000 tick /65 avcı nesli var; aynı koşul diğer seedlerde
  tükendi. Kalıcı denge,27/30 uzun kabul veya gerçek gün/hafta geçti denmez.
- `prepare_coexistence_candidates.py`36 adayı önceden tanımlar. Kalibrasyon
  seedleri1,2,3,4,5,42,12345,2026; görülmemiş uzun test10001–10030.
- `research_coexistence.py`: aday tarama →8×20k →±%10 duyarlılık →30×100k
  kurallı → kaynak şoku → öğrenme/mutasyon karşılaştırmaları →30×100k birlikte.
  Bütün kapılar geçerse ayrı7882 servisi; eski dünyalara dokunmaz.
- Uzun iş için kaynak ayrı temiz Git worktree'sinde, ikili/senaryo hash ile
  sabitlenir. `data/coexistence-research/state.json`, aşama alt dizinlerindeki
  `progress.json` ve `seed-*.log` gerçek durumu gösterir. Sonuç dosyası yokken
  koşuyu tamamlanmış sayma. Ajan arka planda kod yazmaz; program tanımlı işleri
  yürütür. Biyolojik ret, altyapı hatasından ayrıdır; ret sonrası yeni ayar yazmaz.
- Araştırma bildirimi ayrı kalıcı outbox ve retry timer'ıyla; heartbeat/outbox
  bağımsız watchdog ile izlenir. Sayfa yalnız içerik değişince güncellenir.
- Koşullu7882, nitelikli seedlerden ilkini kullanır. Türlerden yalnız birinin
  kaybı da uyarılır.32GiB kayıt bütçesi, gerçek1/7/30 gün ölçümü; sınırsız disk
  veya sonsuz ekolojik süreklilik vaat edilmez.
- Son doğrulama45 Rust,21 Python ve HTTP kontrol/auth/SIGKILL/arşiv testi.
  Derleme temizlenebilir; çalışan/sabitlenmiş `bin/releases`, araştırma
  worktree'si, deney ikilileri ve veriler temizlenmez.

## Aktif araştırma devri — 2026-10-15 16:06:41 UTC

- Kod commit'i `ae7f0df4500b4df829b59ab7f119ba21ad94f84b`.
  Temiz, detached kaynak `/root/curious-research/revision-ae7f0df`;
  `/root/curious-research/current` buraya bağlı. Araştırma sürerken değiştirme.
- `curious-coexistence-research.service` etkin ve çalışıyor; ilk aşama
  `screen-n3-steady-q890`, seed 2/5. İki gerçek probe süreci doğrulandı.
  `curious-coexistence-research-notify.timer` etkin; kalıcı kuyruk boş,
  sayfa running. Sonraki gerçek sonucu `state.json` ve aşama sonuçlarından oku.
- Config `/root/curious/data/coexistence-research/config.json`; 36 aday ve
  SHA kimlikleri aynı dizinde. İkililer `bin/releases/coexistence-*-ae7f0df`.
  Araştırma, ana repo sonraki commitlerinden bağımsızdır.
- Rapor: https://notify.novastrum.dev/p/p_48ad8f4609f2fa26 .
  Başlangıç bildirimi `n_6c271f4398903599`; ayrıca çalışan süreç bildirildi.
  Sayfa ekleri ve ara bulgular korunur. Global proje sayfasını tamamlanmış yapma.
- 7882 kapalı; dünya unitleri henüz kurulmadı. Yalnız bütün kapılar geçerse
  program ayrı dünyayı kurar. `bin/curious-coexistence` şimdiden hazırlanmış
  ikili bağlantısıdır, çalışan dünya anlamına gelmez.
- Gerçek ikiliyle 71/72 seed, 500 tick çalıştırıcı smoke + resume doğrulandı;
  ikisi de biyolojik kabulü geçmedi (bu beklenen protokol kontrolüdür).
- Son `cargo clean`: 1267 dosya / 398,9 MiB kaldırıldı. Deneyler, veriler,
  sabit ikililer ve worktree korundu. Önceki debug temizliği ayrıca 209,4 MiB.
- Başlatma sonrası 7878/7880 pause; 7879/7881 0,1 tick/s ve storage_error yok.
  Eski dünyaların hızları ve ikilileri değiştirilmedi.

## Hedef ve dürüst kapsam

- Kullanıcı basitleştirilmiş bir dünyada belgesel gibi gözlenen canlılar, bireysel
  öğrenme, nesiller ve evrim istiyor. Sadece sabit kalan nüfus durma noktası değil.
- Dünya, beden/yetenek, algı, politika, hafıza, öğrenme, kalıtım ve gözlem ayrı
  tutulur. Yeni bir yeteneğin bedeli fiziksel çözümleyicide doğrulanır.
- Yaş otomatik zekâ bonusu değildir. Öğrenilmiş bilgi yavruya kopyalanmaz.
- Kaynak yenilenmesi nüfustan bağımsızdır; yok oluşu gizlice canlı ekleyerek düzeltme.
- Güncel özet `docs/reports/ROADMAP_RESULTS.md`. Kalıcı karma avcı–av dengesi
  çözülmedi. Sığınaklı seed 12345 etçil 20. nesle ulaştı, sonra tükendi.
- Su seçimi banditi ve tek hareket alleli için kalıtım/mutasyon çalışır; genel
  zekâ, genel öğrenme veya açık uçlu evrim tamamlandı denmez.
- 10.000 kayıtlı tick / 304. nesil kısa testtir; gerçek gün/hafta testi yerine geçmez.
- Kullanıcı 11 başlığı sırayla istedi; `ROADMAP.md` açık kısımları korur. 1/3'ün
  kalıcı soy/denge kısmı ve 9'un gerçek süreli doğrulaması hâlâ açık.

## Sunucudaki dünyalar ve ikililer

Repo `/root/curious`. Gerçek durumu `/status` ile oku; aşağıdaki tickleri güncel sanma.

| Port | Unit / veri | İkili ve durum |
|---|---|---|
| 7878 | curious.service / data/main | eski dünya, pause, korunuyor |
| 7879 | curious-meadow.service / data/meadow | eski sürekli çayır, eski motor, 0,1 tick/s |
| 7880 | curious-lab.service / data/lab | V8, bin/curious-lab → releases/curious-c06ed79, pause |
| 7881 | curious-evolution.service / data/evolution | V7, bin/curious-candidate → releases/curious-ef72f80, 0,1 tick/s |

- 7881 gerçek süreli koşu 2026-10-15 11:47:50 UTC'de başladı. Öğrenme ve kalıtım
  açık otçul çayırıdır. Sığınaklı karma deney bu dünyaya uygulanmadı.
- 7879 ve 7881 hızını gerekçesiz artırma. Testleri bu veri dizinlerinde yapma.
- Yeni sürümlerde `/` görsel gözlem ekranı, `/observation` atomik durum+karar+status.
  Diğer uçlar CONTROL.md'de. Uzak tarayıcı için SSH tüneli kullanılabilir.
- Güncel geliştirme checkpoint V9, lab V8, sürekli aday V7; SQLite şeması V2.
  Eski kayıt yüklemek eski motorla aynı geleceği değil, durum göçünü garanti eder.
- `bin/curious-live` eski 7878/7879 ikilisine işaret eder. Derleme çıktısına bağlı
  servis yok; `target` temizliği ikilileri silmez. `bin/releases` dosyalarını silme.
- `data/lab/deployment.json`, revision/hash/gerçek yedek yolunu kaydeder.
  Son lab yedeği `data/backups/lab-20261015T120041225349Z/`: V7→V8, tick30,
  pause ve tüm önceki alanlar/RNG doğrulandı. Önceki V3→V7 yedeği de korunuyor.
- `scripts/deploy_lab.py` clean commit, değişmez ikili, kapalı veri yedeği,
  integrity_check ve eklemeli durum doğrulaması uygular. Başarısız veriyi de
  koruyarak rollback yapar. Eski ikiliyi yeni veri üstüne körlemesine açma.
- Kapanmış `.sqlite.gz` dosyaları kayıpsız geçmiş; silme. API geçici açıp okuyabilir.
  V2 öncesi yedek `data/backups/pre-checkpoint-v2/` hâlâ korunur.

## Gerçek süreli gözlem ve uyarılar

- `curious-monitor.timer`: eski çayır, dakikalık sağlık/Notify.
- `curious-evolution-monitor.timer`: 7881 için ayrı sağlık/outbox.
- `curious-sampler.timer` / `curious-evolution-sampler.timer`: 15 dakikalık salt
  okunur RSS/fd/status örnekleri `data/soak` / `data/soak-evolution` içinde.
- `curious-watchdog.timer`: iki monitörün 3 dk, örnekleyicilerin 35 dk sessizliğini
  ve 15 dk boşalmayan bildirim kuyruğunu bağımsız kontrol eder.
- `curious-soak-report.timer`: saatlik kontrol; gerçek 1/7/30 gün ve yeterli ayrı
  15 dakikalık ölçüm dilimi oluşunca Notify. Hızlandırılmış tick gün sayılmaz.
- Monitörler kendiliğinden dünya değiştirmez veya yeniden canlı eklemez. Bir host
  tamamen durduğunda içerideki gözetim de durur; host dışı izleme henüz yok.
- V7 ölçümünde 0,1 tick/s için ~88 MB/gün / 2,6 GB/30 gün geçmiş tahmini var.
  Bu nüfus/tam algı/olay yoğunluğuna bağlıdır. Bütçede veya yazma hatasında pause
  beklenen davranış; geçmişi sessizce silip devam etme.

## Notify ve kullanıcı iletişimi

Kullanıcı önemli aşamalarda bildirim istedi; ayrıca izin isteme. Her komutta
bildirim gönderme. Tamamlanan aşama, anlamlı bulgu, hata veya müdahale gereği için kullan.

- Notify `/root/notify` → `/var/www/notify.novastrum.dev`. Güncel
  `docs/AGENT_INTEGRATION.md` ve `docs/OPERATIONS.md` bu sistemden okunur.
- CLI `node /root/notify/bin/notify.js -t /root/notify/token.json ... --json`.
- Yalnız memo / `u_e049630155fc` hedeflenir. Toplu gönderim yok.
- Token içeriğini çıktıya, Git'e, rapora veya sayfaya koyma. Sabit idempotency
  anahtarı ve aynı canonical body ile retry yap.
- Güncel 11 başlık sayfası `p_6c6647a23f563ace` / running: gerçek süreli gözlem
  sürüyor ve kalıcı karma denge açık. Bunu bütün projenin tamamlandığı diye kilitleme.
- Önceki completed aşama sayfaları: temel `p_32dc9b49b5101fc1`, karar
  `p_803d7f4d871633ff`. Completed, o aşamanın rapor durumudur.
- Gerçek 7881 ekran görüntüsü Notify raporuna eklendi; ajan arka planda yeni kod
  yazmıyor. Servisler simülasyon/ölçüm/uyarıları sürdürüyor.

## Doğrulama, not, commit ve temizlik

- Anlamlı değişiklikleri commit et. Push açıkça istenmedikçe yapma.
- `cargo test --all-targets`; `cargo build --release`; kontrol değişikliğinde
  `python3 scripts/check_control.py target/release/curious`.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`.
- UI testi `scripts/check_ui.cjs`: dışarıdan Playwright gerekir.
  `CURIOUS_PLAYWRIGHT` modül yolunu, `PLAYWRIGHT_BROWSERS_PATH` tarayıcı yolunu
  gösterebilir. Test kendi geçici dünyasını açar; 3. argüman senaryo yoludur.
- Son doğrulama 40 Rust (öğrenme örneği dahil), 11 Python, HTTP/SIGKILL/auth/
  Origin/arşiv/terrain ve gerçek Chromium desktop/mobile kontrolleri geçti.
- Kaynak indeksi optimizasyonunda 500 ticklik State+RNG önce/sonra aynıdır;
  bin kurucuda p95 71–84 ms → 19–29 ms (SQLite hariç). Rapor ROADMAP_10.md.
- Deneyler `data/experiments` altında, Git dışında korunur. Ham JSONL/SQLite'ı
  build temizliği sanıp silme. Yeni ölçüm dizini mevcut dizini ezmez.
- Son temizlik `cargo clean`: 363,0 MiB; yalnız bu görev için kurulmuş geçici
  Chromium/Playwright araçları 696,2 MiB kaldırıldı. Öncesinde ayrıca 205,1 MiB
  debug temizliği yapılmıştı. Target şu an yok; test için yeniden derle.
- Cargo registry, başka projeler, kalıcı geçmişler ve çalışan sürümler temizlik
  hedefi değildir. Güvenli aralıklarla build çıktısını temizleme döngüsünü sürdür.

## Karma gözlem gerçekten açıldı — 20 Ekim21:18:47UTC

curious-coexistence.service: localhost7882,0,03tick/s,seed20001; frozen kaynak
/root/curious-research/observation-7b157af, release curious-observation-7b157af.
Kalıcı açık politika/öğrenme/genetik başlangıcı doğrulandı; --resume kullanılır.
data/coexistence-deployment.json ve docs/reports/QUALIFIED_DEPLOYMENT.json
kanıtlarıdır. Üç ayrı monitor/sampler/soak-report timer enable/active; watchdog
yeni dünyayı görür. İlk578H/32C,~75MiBRSS,9fd; bunlar güncel sanılmamalı.
Gerçek1/7/30gün henüz dolmadı; mevcut sonraki iş bu gözlemi kontrol etmek,
kapasiteyi gerçek örneklerle değerlendirmek ve sınırlı av öğrenmesine geçişi
LEARNING_NEXT sözleşmesine göre ele almak. Yeni uzun denge taraması yok.

## Salt okunur kayıt izleyicisi — Luna alt ajanı

Kullanıcı Luna'ya delegasyonu açıkça istedi. scripts/archive_replay.py +
web/replay.html/js/css ayrı salt okunur viewer; deploy/curious-replay.service
7883loopback kullanır. GET sabitAPI, readonlySQLite/WAL, gzip/zstd desteği;
oynatma sadece tarayıcıda. Veri diziniOSseviyesinde ReadOnlyPaths ve
ProtectSystem=strict ile korunur; geçici cache256MiB/ençok2shard.
Simülasyon7882/frozen motor değişmez. Ayrıntı docs/ARCHIVE_REPLAY.md;
55Python ve bağımsızChromium desktop/mobile/read-only/baythash testi geçti.
Kaynak data/replay-runtime.env içindeki frozenworktree'den çalıştırılır.

## 21 Ekim — üç ajanla deneyim/av öğrenmesi geliştirmesi

Kullanıcı üç alt ajanla başlamayı onayladı. Su deneyimi için dört kollu
EXPERIENCE_PROTOCOL.md; deneysel av seçimi için entity/hunt_learning.rs ve
systems/hunt_outcomes.rs; ortak web/experience.js görünümü eklendi.
Geliştirme checkpointV13/SQLiteV2: hunt_learner varsayılan kapalı, eski
V1–V12 göçünde boş/kapalı. Su update_enabled true default; false yalnız
güncellemeyi durdurur, tercih/keşif/sonuç muhasebesi sürer. Resolver olayları
saldırıenergy_cost/killed ve yemekte corpse_id audit alanları taşır. Bunlar
fiziksel kuralı değiştirmez. Avın başka bireyce öldürülmesi/belirsizlik başarı
ödülü değildir; tablonun yavruya aktarılması yoktur.

Çalışan7882world hâlâ7b157af/V12dir; yükseltme yok. Su16k kontrollü tick
bedenleri açıkça sıfırlar, gerçek yaş/yaşam üstünlüğü değildir. Deneysource
SHA/manifest/ham sonuçlar korunur. Avın kontrol/frozen/naive fiziksel
karşılaştırması ve ayrı ekosistem kapısı henüz açık; genelML/evrim denmez.
Yeni uzun denge taraması başlatma. Yeni presenter hemcanlıUI hemreplay'de
kayıttan deneyim/maliyet/kesinti gösterir; eski kayıtta av verisi yok denir.

## Deneyim ilk tur sonucu ve av adayının durumu

Rapor docs/reports/EXPERIENCE_FIRST.md/JSON. Su ölçümü temizbac0b53: deneyimli
ilk100tam su edinimi96,375%, deneyimsiz95,375% (yalnız1pp). Çevretersi sonrası
son100online98,25%, aynıtrainedtablofrozen53,125%; kurallı100%. Öğrenme her
koşulda üstün değil; yaş herdeneme açıkça reset, gerçek yaşam avantajı denmez.

Av ilk smoke küçük tarihsel senaryoda control121/124adımda, aday87/95'te
tükendi. V2eşitQ'da temel maliyet sırasını korur:82/109. V3yalnızaday sıralaması
sözleşmesi için20tickhedefkilidini kaldırır; muhasebe aynıhedefte korunur,
değişimdepuansızkesilir. V3online96/121,frozenboş98/119; üstünlükkanıtı yok.
Bu iki görülmüş seed küçük teşhistir, formal kabul/ret veya büyükqualified
senaryo testi değildir. Bütünhamlar/checkpointler, immutableikili vehashlar
experience-v1-20261021/hunt-learning-smoke* altında korunur. Başarısız diye
silme veya yeni görülmemiş kabul diye sunma. Av varsayılan kapalı kalır.

110Rust/58Python+Node presenter, gerçek Chromiumlive/replay/GETonly/hash ve
HTTP/SIGKILL geçti. FinaldisabledV13biyoloji/RNG100/200'de frozen32ddfb2 ile
eşit; ek alanların kapalı/boşluğu doğrulandı. StateV13/SQLiteV2; çalışan dünya
V12/7b157af korunur. Yalnız7883reader yeni presenterla güncellenir; kaynak
replay-runtime.env'deki frozenpath. İlktercihyeniuzun tarama değil, av için
öncedensabit kontrollütrain/naive/trained-frozen deneyi ve nedenselölçüm.

## Av laboratuvarı v1

Kullanıcı 'av lab kur' istedi. Önceden docs/HUNT_LAB_PROTOCOL.md sabitlendi.
examples/hunt_lab.rs gerçekWorld/resolver ile passive/defensive iki iz,8ayrı
eğitim/eval çift ve kurallı/naive/deneyimli/frozen dört kol kullanır.80eğitim
+4×80eval, trialençok20tick; toplam128000ticküst sınır, sonlu300skomut.
Başlangıç beden/yaş/algı eşit; ilk40evalsonrası kaçan/bekleyen av rolleri değişir.
Ödül/epsilon/biyolojikparametre değiştirilmez. Bütünsonuçlar HuntEpisodeEnded
auditolayıyla izlenir (aynıtickte2sonuç dahil); transientqueue checkpointte
yok, World.tick sonundaboşalır. StateV13/SQLiteV2 değişmez.

run_hunt_lab.py temizsource/freshoutput ister; immutableikili/protokol ve
raw/manifest/summary +16kısaSQLite örneği saklar. Triallimit kesintisi motor
başarısızlığı diye puanlanmaz. Bedenenerjibütçesi gerçek food/actioncost ile
ayrı denetlenir. Sonuçlar görülmeden üstünlük iddiası yok. Çalışan7882/7883
source/ayarları değişmez. Kullanım docs/HUNT_LAB.md.

## Av lab v1 tamamlandı — sonuç

2672710kaynak/protokol ile6400trial79849tick yaklaşık9,72s, batchtamamlandı.
Kök data/experiments/hunt-lab-v1-20261021;33MiBham+manifest+immutablebinary+
16SQLite örneği. Öğrenici politikası/ödülü/epsilon değiştirilmedi; yalnız tam
audit eklendi. Rapor docs/reports/HUNT_LAB_V1.md/JSON.

Passive-normal eğitimlionline41/320beslenme (12,81%), naive9/320(2,81%),
rule0. Ama eğitimli751kapanışın710'u puansız (94,54%):434hedefdeğişimi,
275HuntTooCostly,1HuntLowEnergy. Defensive-ters320başarılıöğünün303'ü hasar
ambiguous nedeniyle puansız. İkiizdede8eğitimmodelinin4'ü boşkaldı. Genelav
faydası/ekosistem kabulü denmez.41001–41008artıkgörüldü; finalunseen diye
yenidensunma. Sonraki hipotez: kendi bütçe kesintisiyle dışmüdahaleyi ve
kendiavının savunmasıyla başkasının hasarını ayırmak; yeni protokol gerekir.

117Rust/62Python+HTTPkurtarma+closed100/200State/RNG+16SQLite/rawaudit
eşitliği geçti. Canlı7882/7883hiçgüncellenmedi. Bu jobarkaplanda süreklideğil,
completed; yeni uzun tarama başlatma. Deney örnekleri manualreadonlyreplay
ile açılabilir; docs/HUNT_LAB.md komutları var.

## 22 Ekim — Luna av geri bildirim düzeltmesi

Kullanıcı Luna alt ajanıyla uygulama, root denetimi istedi. Kendi görünür avının
savunma hasarı üçüncü taraf hasarından ayrılır; belirsiz/eski ambiguous sonuçlar
sansürlü kalır. Görünür hedef için doğrulanmış HuntLowEnergy/HuntTooCostly
kesintisi yalnız önceki resolver masraflarını mevcut formülle puanlar. Hedef
değişimi puansız; yeni tick eski bölüme yazılmaz. V13/SQLiteV2 ve varsayılan
kapalı korunur. Ödül hasarın maliyetini ölçmez; genel av faydası iddiası yok.
Protokol docs/HUNT_LAB_PROTOCOL_V2.md sonuçlardan önce sabitlenir; --v2 runner
42001–42008 değerlendirme seedlerini kullanır. V1 veri/rapor korunur. Canlı
7882/7883 ve frozen kaynaklar değiştirilmez; uzun ekoloji taraması yok.

Av lab v2 tamamlandı: source34973de,6400trial56384tick; ham veriler
hunt-lab-v2-20261022 altında. Rapor reports/HUNT_LAB_V2.md/JSON. Boş eğitimli
model0/8; defensive ters online307Fed tamamı puanlı. Deneyimli her koşulda
naive'den iyi değil; canlıya geçiş/kabul yok.123Rust/64Python+Node+HTTP+
kapalı100/200State/RNG+16SQLite/raw audit eşitliği geçti. Root denetimi kayıt
hedefinin predecision kesintiden sonraki ticke taşınmasını ve v2 summary
hesabını düzelttirdi. Kaynak frozenhunt-feedback-34973de, immutablecandidate
bin/releases/curious-hunt-feedback-34973de; dağıtılmadı. cargo clean490,2MiB;
hamlar, release, worktree korunur. Yeni arkaplan deneyi yok;42001–42008 görüldü.
