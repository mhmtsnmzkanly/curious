# Sürdürülebilir habitat ve bedelli kalıtsal özellik tasarımı

**Durum:** Root incelemesi için bounded prototip tasarımı. Buradaki kod yalnız
yerel tanı aracı taslağıdır; kök preregistrasyonu/temiz kaynak commit'i öncesi
formal world batch'i başlatılmaz. `src/`, ortak protokol, canlı dünya ve
arşivler bu çalışmanın dışında.

## Sorulan biyoloji sorusu

Ekoloji-traits v1'in sekiz seed × on kolunda (80 dünya) hiçbir dünyada tick
2.000 sonunda yaşayan ve üremiş yavru-soy bulunmadı. Her kol sekiz aynı yaştaki
otçul kurucuyla başladı; kaynaklar sekiz siteyle sınırlıydı; çevrimli koşullar
aynı küçük haritada kaynak yenilenmesini dar pencerelere indirdi. Bazı static
kontrol/mobility/vision kollarında F1 yavrular doğdu, fakat hiçbir F1 üremedi;
vision+stamina kolu hiç doğum yapmadı. Trait maintenance istekleri oluştu,
ancak kurucuların kaynak kısıtı altında bunun önemli bölümü ödenemedi. Bu veri
bedelin mevcut olduğunu gösterir; hangi özellik avantajlı veya fixture'ın
nerede darboğazlı olduğunu ayırmaz.

Amaç, sıfır soy sonucunu parametreyi sonuçtan sonra ayarlayarak “düzeltmek”
değil. İlk adımda aynı kısa çalışma için habitat ve biyolojik throughput
tanımlanır: tüketilen/yenilenen besin ve su, başlangıç stoku, yaşayan kurucu,
F1/F2 doğumları, üreyen yavru sayısı ve canlı üreyen torunlar ayrı ölçülür.
Kaynak haritasının dolu olması kaynakların erişildiği veya metabolizmayı
karşıladığı anlamına gelmez.

## Fixture seçimi ve yeniden kullanım

Kalıtım/trait için başlangıç referansı, kabul görmüş `32ddfb2` kabul çalışmasının
tam scenario kopyasıdır: `qualification-both/scenario.json` SHA-256
`33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac`, adı
`mixed-refuge 4x4 food=1 water=1 age=steady`, 608 kurucu (576 otçul, 32 etçil),
976 yenilenebilir site ve 1.232 başlangıç kaynağı hücresi. Kopya ve ilk
koşunun koşulları manifest/protokole hash ile bağlanmalı; orijinal arşiv
değişmez. Bu 30 seed × 100.000 tick sonucu **sınırlı biyolojik başlangıç
kanıtıdır**. Seedler tüketilmiş/görülmüştür; yeni held-out sonuç gibi yeniden
kullanılamaz.

Kabul senaryosunu tekrar etmek, trait kolunu onunla “kabul ettik” demek değildir.
Önce yeni, önceden belirlenmiş ve kullanılmamış seedlerde kısa fixture-validity
kolu çalışır. Aynı kabul edilmiş kaynak, yaş/enerji dağılımı, 4×4 yerleşim ve
aynı dünya kurallarıyla kontrol armı, daha önceki performans arşivi için
başlangıç kalibrasyonu verir. Sonra bir birincil habitat kolu ve önceden
belirlenmiş düşük kaynak veya coğrafi/yenilenme şoku kolu eklenebilir. Her
faktör için yalnız bir değişken ve küçük kayıtlı bir matris kullanılır; sonuç
görüldükten sonra kaynak, üreme, yaş veya şok parametresi seçilmez. Daha büyük
harita/aktör sayısı istendiğinde ayrı, root tarafından önceden tanımlanan
kapasite benchmark'ı kullanılır.

Ölçülen demografik kategoriler:

- **Kurucu:** tick 0'da var olan birey; kurucu yaşamı ve ölümü ayrı sayılır.
- **F1/F2 doğumu:** generation/ebeveyn bağlantısı üzerinden nesil başına.
- **Üreyen yavru:** en az bir `EntityMated` olayında ebeveyn olmuş F1.
- **Yaşayan üreyen soy:** ufukta hâlâ yaşayan ve daha önce kendisi üremiş
  descendant; founder sayısına eklenmez.
- **Doğal yok oluş:** türün/kolun popülasyonu sıfırda kalır; replacement yok.

İki popülasyonun yalnız toplam doğumunu veya yalnız en son popülasyonunu
karşılaştırmak soy sürekliliği değildir. Kayıp ebeveynler için soy bağı event
akışından tutulur. `EntityMated` ebeveyn/çocuk ID'leri doğum ve yavrunun
üremesini bağlar; tür, ölüm nedeni, yaş, genotype/allele, phenotype, sağlık,
enerji, su, hareket ve trait maintenance istenen/ödenen maliyeti de raporlanır.

## Bedel ve faydayı ayırma

Başlangıç phenotype'ları gövdeye gerçekten ifade edilen allellerden kaydedilir.
Her expressed trait için `requested` ve `paid` maintenance ayrı ayrı
toplanır; `paid < requested` ise eksik talep, bedelin tamamen ödendiği anlamına
gelmez. Hareket için resolver'ın gerçek `EntityMoved.energy_cost`, hasar için
gerçek saldırı/sağlık olayları, beslenme için `EntityAte`/`EntityDrank` kullanılır.
Metabolizma, hareket veya ödül aynı kaynağa iki defa yüklenmez. Kontrol ve
özellik kolunda aynı seed başlangıç eşleşir, fakat kararlar farklılaşınca RNG
ve sonraki dünya izlerinin de farklılaşabileceği açıkça raporlanır.

Sabit allele kolları (örn. control vs vision vs stamina) bakım maliyetini ve
fenotip etkisini **mekanizma/ekoloji karşılaştırması** olarak gösterebilir;
popülasyon içi seçilimi göstermez. E kolunda seçilim iddiası için aynı üreyen
popülasyonda en az iki kalıtsal fenotip, soy bazında farklı üreme katkısı,
allele sıklığının kuşaklar içinde ölçümü ve ayrı no-mutation/control armı
gerekir. Mutasyon oranı kendi başına adaptasyon değildir; bir feature dağılımı
arttı diye açık uçlu yenilik veya complexity artışı söylenmez.

## Fixture gate, bütçe ve durdurma

İlk, henüz preregistrasyona dönüşmemiş gate taslağı:

1. Seed/ham arşiv envanteri ve source/scenario hash eşleştirilir; bütün
   `20001–20030` kabul seedleri, 50001–50008 ecology-traits seedleri ve diğer
   mevcut seedler görülmüş kabul edilir.
2. Tek bir short calibration world çevriminde başlangıç kaynakları, per-tick
   yenilenme, gerçek tüketim, hareket, metabolik basal cost, ölüm ve doğum
   event muhasebesi doğrulanır. Bu yalnız runner/schema/muhasebe gate'idir; tür
   sürdürülebilirliği iddiası olmaz.
3. Küçük, yeni-seedlik sabit kalibrasyon grubu aynı pristine 32ddfb2 profilinde
   her türün kaynak bulma ve yavru üretme fırsatı olduğuna bakar. Biyolojik
   başarısızlık parametre düzenleme izni değildir; fixture yeterliliği
   tanımlanamıyorsa preregistration yeniden tasarlanır ve root onayı bekler.
4. Yalnız temiz, reviewed source ve preregistrasyon sonrasında sınırlı formal
   comparison başlar. Tick cap, duvar saati, RSS, raw çıktı boyutu, kuyruk ve
   yeni dosya yolu önceden belirlenir. Önceden tanımlı ret/altyapı hatasında
   durur; seed değiştirme, ek sweep, rescue, beden reseti veya replacement yok.

Kesin seed, arm sayısı, tick sınırı ve kabul eşiği henüz seçilmemiştir; bu
taslakta bunlar kökün preregistrasyon kararından önce dondurulamaz. Başlangıç
prototipi çoklu batch, parametre taraması veya büyük nüfus çalıştırmaz.

Örnek [`ecology_candidate.rs`](../examples/ecology_candidate.rs) tek bir input
senaryosunu değiştirmeden tek dünya çalıştırabilir; CLI sınırı 1–100.000 tick,
örnek sayısı en çok 1.001 ve output dosyası `create_new` ile açılır. Her
çağrıda seed, scenario adı, kurucu/descendant census'u, F1 doğum/üreme
sayısı, toplam beslenme/içme/hareket maliyeti, başlangıç/son kaynak stoku,
yenilenen toplam kaynak miktarı, bakım istek/ödeme toplanır. Bu kod yalnızca
derlenmiş bir taslak prototiptir; runtime fixture/muhasebe validasyonu yapılmadı,
preregistration veya seed audit yerine geçmez. Seed argümanı audit edilene kadar
kullanım biçimi placeholder bırakılmıştır:

```sh
cargo run --example ecology_candidate -- \
  scenarios/ecology-traits-lab.json SEED TICKS SAMPLE_EVERY candidate.json
```

Bu komut yalnız biçimi gösterir; bu aşamada çalıştırılmadı. Candidate yalnız
sabit input senaryosunu çalıştırır; seed tazeliğini, protokolü ve temiz source
hash'ini root-owned runner doğrulamalıdır. Sonuç dosyası başka dosyanın üstüne
yazılmaz.

## Prototipte henüz ölçülmeyenler

Derleme, ölçüm doğruluğunu veya fixture geçerliliğini doğrulamaz. Aşağıdaki
eksikler kapatılmadan throughput ya da biyolojik sonuç iddiası yapılmaz:

- Sayaçlarda tick-0 ve final toplamları şimdilik bulunur; besin/su açılış
  bakiyesinden yenileme, tüketim ve kapanışa conservation reconciliation
  henüz uygulanmadı. `EntityReplenished` miktarı ile `EntityAte/EntityDrank`
  miktarları event sırasına göre doğrulanmalıdır.
- `EntityAte`/`EntityDrank` toplamları tür bazında ayrıştırılmaz; gövde soy
  kimliği kaybolduğunda olay türünü doğru yakalamak için ID→tür/nesil sözlüğü
  kurulmalıdır.
- `EntityMetabolized` yalnızca ekstra vision/stamina istek/ödemelerini verir;
  bazal metabolizma, açlık/susuzluk kaybı, üreme rezervi ve trait bedeli sonrası
  gerçek enerji hesabı ayrıca resolver/life-state üzerinden tutarlı
  doğrulanmadı.
- Başlangıç ve yavru allele/genotype dağılımı ve fenotip ifade edilmesi henüz
  kaydedilmez; bu nedenle şu an cost/fitness trade-off karşılaştırması
  üretilemez.
- Kurucu/descendant türetimi generation alanı ve `EntityMated` ebeveynleriyle
  başlatılmıştır; ölmüş ebeveynlerin soy/nesil sınıflaması, aynı tick doğum-
  ölüm olay sırası ve yaşam durumu için event replay doğrulaması gerekir.

## Sonraki bağımlılık

C fixture ve throughput ölçümü netleştikten sonra A/B'nin su/sığınak deneyleri
aynı fiziksel maliyetleri kullanabilir, fakat ayrı subject/context/seed/protocol
ve analiz tutar. D için her eylem parçası perception-only input, intent,
resolver maliyeti, attribution ve bounded memory kontratı gerektirir. Sosyal
parça en fazla komşu/olay tabanlı, finite ve ölçülebilir olmalıdır. E, yalnız
önceki kapılarda gerçekten çoğalıp üreyen descendant olduğunda değerlendirilir.
F, ayrı ayrı geçilmiş ölçüm bileşenlerini küçük bir birleşik gösterimde izler;
birleşik koşuda olumlu sonuç önceden belirlenen fiziksel/soy ölçütü olmadan
kabul sayılmaz.

F'nin kullanıcıya dönük teslimi salt-okunur belgesel görünümüdür: aynı
bireyin tarihçesinde önceki gözlenen su/kaçış episode'u, daha sonraki gerçekten
değişen kaynak veya rota seçimi, resolver'ın doğruladığı içme/hasar/hareket
maliyetleri ve ardından gelen öğrenme güncellemesi kronolojik gösterilir.
Kayıttaki mevcut alanlarla ilerleme ve checkpoint sonrası resume/replay
yapılabilir. Eski kayıtta alan yoksa “unknown/not recorded” gösterilir; olay
kayıtları olmayan bir düşünce veya gerekçe geriye dönük oluşturulmaz. UI karar
deneyini ispatlamaz; ham karar/perception/resolver/state kayıtlarına salt-okunur
bağlantı sağlar.

Bu tasarım ne bütün seedlerde devamı ne her habitatta iki türün bir arada
kalmasını garanti eder. Doğal yok oluş, tanımlı matris içinde geçerli olumsuz
sonuçtur; kök/rapor nedenini kaynak sıkışması, üreme fırsatı, av baskısı,
phenotype maliyeti, çevresel şok veya yetersiz gözlem olarak ayırmalıdır.
