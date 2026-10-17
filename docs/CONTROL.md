# Kalıcı kayıt ve kontrol servisi

## V12: deneysel otçul buluşması

Checkpoint V12, SQLite V2. `policy.mate_rendezvous=false` varsayılanı;
rezerv hazırlığı bayrağından bağımsızdır. Kabul edilmeden canlı dünyaya açılmaz.
V1–V11 göçünde buluşma kapalı, bireysel `memory.social` boş başlar; eski rezerv
bayrağı korunur. V12 eski motorla açılmaz. Yeni eylem/fiziksel avantaj yoktur.

Yalnız AI otçulları, hafıza ve türdeş hafızası açıkken kullanır. Ayrı
`entity/social.rs`, algı ve en fazla sekiz mevcut türdeş anısından kısa bir
buluşma hedefi çıkarır. Yetişkin ama henüz hazır olmayan türdeşin yanında
bekleyebilir; hazır eş fırsatı önce gelir. Açlık, susuzluk, tehlike, toparlanma,
fiilî çiftleşme, özel politika ve dış komut korunur. Kaynak ararken hedefin
saati durmaz. Hedef en çok30 dünya adımı sürer; ardından10 adım yeniden
başlama arası vardır. Körü körüne süresiz bekleme yoktur.

Hedefe algılanmış yolun tek adımıyla yaklaşılır. Güncel/son30 adımdaki bilinen
avcı konumunun iki hücre çevresi sosyal yaklaşımda dışlanır; sığınak fiziksel
koruması korunur. Bilinmeyen avcı için güvenlik garantisi değildir. Yeterli
kalan ömür ve enerji bütçesi yoksa yolculuk başlatılmaz. Kayıp türdeşin son
bilinen çevresinde sınırlı, ziyaret edilmiş hücreleri hatırlayan arama yapılır;
koordinat güncellemek için dünya veya küresel nüfus bilgisi kullanılmaz.
Birey başına tek hedef ve en çok32 ziyaret konumu; yavruya boş hafıza geçer.

Gözlem gerekçeleri `SeekCompanion` ve `WaitForMate`; arayüz hedef kimliğini,
son görülme yerini/zamanını ve arama evresini gösterir. `ecology_probe` için
`--mate-rendezvous`; doğrulayıcı ve kapılı karşılaştırıcı da aynı bayrağı alır.
Ayar manifest kimliğine katılır; değişirse resume reddedilir ve probe ayarı
uygulamamışsa sonuç kabul edilmez. `collapse_branch` için `rendezvous` deney kolu vardır.
Bu ilk sürüm yerel adım seçer; genel güvenli rota planlayıcısı veya bütün
popülasyonda kalıcı denge değildir. Kabul öncesi küçük mekanizma testleri,
aynı kontrol noktalarının açık/kapalı karşılaştırması ve sonra normal/şok/
bağımsız seed kapıları gerekir. Eski uzun deneylerin kaynak ve ikilileri korunur.


## V11: deneysel politikanın ayrılması

Checkpoint V11, SQLite V2. Varsayılan `policy.reserve_opportunities=false`.
Başarısız dad97e9 rezerv kuralı ortak açlık değerlendirmesinden kaldırıldı;
normal içgüdü kodu ae7f0df ile aynıdır. Bu, kaynak şokunun çözüldüğü anlamına gelmez.

Deneysel davranış ayrı `policy.rs` katmanında yalnız açıkça etkinleştirilince
uygulanır. Sağlıklı, yetişkin, bekleme süresi bitmiş birey; bitişikte yetişkin bir türdeş
ve yerinden ayrılmadan tüketebileceği yeterli besin varsa rezerv tamamlar.
Bir sonraki adımın bazal maliyeti için bir enerji payı gerekir. Uzak veya küçük
bir besine yönelmez; tehdit, susuzluk, mevcut açlık, dış komut ve özel politika
kararlarını değiştirmez. Gerekçe `PrepareReproduction`; fiziksel doğrulama ve
ödemeler aynı çözümleyicide kalır. Rakip besini önce tüketirse başarı garanti edilmez.

`ecology_probe` ve `validate_coexistence.py --reserve-opportunities` ile
etkinleştirilir; seçeneğin değeri deney manifestine girer ve değişirse resume
reddedilir. Türdeşin zaten üremeye hazır olması rezerv hazırlığı için şart değildir;
aksi halde iki rezervi düşük yetişkin birbirinin hazır olmasını bekler.
Gerçek çiftleşmede iki bireyin fizyolojik uygunluğu yine çözümleyicide aranır.
Canlı HTTP kontrolüne yeni bir açma eylemi eklenmedi. Policy bir dünya
ayarını temsil eder, kalıtsal bir beden özelliği veya öğrenilmiş anı değildir.
Yeni senaryolar kapalı başlar. V1–V10 kayıtta alan yoksa kapalı yüklenir;
V11'de açık kaydedilmiş dünya açık sürer. V11 eski motorla açılmaz.

## V10 geliştirme ekleri

Checkpoint V10, SQLite V2. Mevcut servisler ve `ae7f0df` sabit araştırması
bu değişikliği kullanmaz. V1–V9 durumlar okunur; V10'u eski ikiliyle açmayın.

`learner.pending` yerini `episode` ve `last_episode` alanlarına bırakır.
Eski tamamlanmamış `pending` atılır; öğrenilmiş su değerleri korunur.
`episode`: hedef, yürütülmüş adım sayısı, gerçek hareket enerji maliyeti ve
alınan dış saldırı hasarı. En fazla 20 yürütülmüş adım; daha öncelikli karar,
görünür hedef kaybı, dış komut, öğrenmenin kapatılması veya ölüm denemeyi keser.
Kesintiler puan güncellemez. Hedefe su içerek ulaşma, kaynak bulunamaması veya
20 adımlık süre sonu puanlanır; bir deneme yalnız bir deneyim sayılır.

Ödül: `10 × içilen su − 2 × hareket enerjisi − adım sayısı − başarısızsa 10`,
[-1000,1000] aralığına kırpılır. Bu bir başlangıç hipotezidir; ekolojik başarı
ölçütü değildir. Metabolizma/iyileşme/üreme net enerji değişiminden tekrar
çıkarılmaz. Dış saldırı hasarı bağlam olarak gösterilir, nedensellik iddiasıyla
su kaynağının puanından düşülmez. Süre dolan yavaş ama geçerli bir rota yine
negatif puan alabilir; sınırlı ufkun bilinen bedelidir.

`last_episode` son denemenin sonucu ve varsa puanıdır; geçmiş denemeler için
kalıcı durum kayıtları okunur. Gövde ve aktif/son deneme boyutları sınırlıdır,
checkpoint yüklemede ve `update_entity` sırasında doğrulanır.
`EntityMoved.energy_cost` çözümleyicinin gerçekten harcadığı enerjidir; eski
olaylarda alan eksikse 0 okunur. Eski olaylar yeni öğreniciye yeniden oynatılmaz.
Yeni hareket olayında alan fiziksel kuralları değiştirmez.

## V9 geliştirme ekleri

Kaynak checkpoint V9, SQLite V2. Çalışan7880 V8,7881 V7 korunuyor.
Yeni beden alanları `max_meal` (1–1.000.000), `reproduction_interval`
(1–1.000.000.000) ve `reproduction_reserve_per_mille` (250–1000).
Eksik eski alanlar10/25/500'dür. Üremede ebeveynin dörtte birlik enerji/su
ödemesi korunur; rezerv alanı yalnız gerekli başlangıç enerjisini belirler.
`update_entity` bu alanları da tam nesneyle taşır. `spawn` yerleşik türün
varsayılan bedenini kullanır; deneysel senaryo bedenini global varsayılan yapmaz.

`memory.companions`: görülen yetişkin türdeşlerin kimlik/son konum/zamanı,
en fazla8 kayıt ve30 tick. `companions_enabled` bunun bağımsız kontrolüdür.
`RecallMate` gerekçesi son görülen konumu arama kararıdır; güncel konuma
omniscient erişim değildir. Algıda `mature` yerel yetişkinlik gözlemidir.

Koşullu yeni7882 servisi ancak araştırmanın bütün kabul kapıları geçince başlar.
Durum/çalıştırma sözleşmesi `reports/COEXISTENCE_IMPLEMENTATION.md` içindedir.

## Başlatma

```bash
cargo build --release
./target/release/curious --serve --paused --data-dir data/main
# Son kalıcı durumdan devam:
./target/release/curious --serve --paused --resume --data-dir data/main
```

Varsayılan adres `127.0.0.1:7878`, hedef hız 3 tick/s. `--tps`, `--seed`,
`--bind`, `--shard-mb` (varsayılan 256), `--max-history-mb` (varsayılan 10240)
ile değiştirilebilir. `--resume-or-new`, kayıt varsa devam eder, yoksa dünya kurar.
Kayıtlı dizini yanlışlıkla yeni bir dünyayla ezmek reddedilir. Aynı dizinin ikinci
bir süreç tarafından açılmasını işletim sistemi dosya kilidi engeller.

`--paused` olmadan tick ilerler. Pause/speed çalışma kontrolleridir; yeniden
başlatmada bu CLI seçenekleri uygulanır. Dünya durumu, bekleyen canlı komutları
ve RNG durumu checkpointten alınır. Seed, devam edilen dünyayı yeniden tohumlamaz.

Terminal demosu `cargo run` ile hâlâ kullanılabilir; SQLite ve HTTP için `--serve`
gereklidir. Servis motorun tek sahibidir; sorgu ve müdahaleler tick sınırlarında
uygulanır. Hız hedefi işlem süresi nedeniyle tam duvar saati garantisi değildir.

## HTTP

| Yöntem / yol | İşlev |
|---|---|
| `GET /status` | Tick, pause, hız, depolama hatası ve aktif shard |
| `GET /decisions` | Son tickin gerekçe ve sonuç içeren kararları |
| `GET /state` | Bütün canlılar, tüm yaşam özellikleri, kaynak tür/miktarları, sınırlar, RNG, bekleyen komutlar |
| `GET /databases` | Tarihsel SQLite dosyaları ve boyutları |
| `POST /control` | Aşağıdaki müdahaleler |
| `POST /query` | Seçilen shard üzerinde salt okunur SQL |

```bash
curl -s http://127.0.0.1:7878/status
curl -s -H 'Content-Type: application/json' http://127.0.0.1:7878/control \
  -d '{"action":"pause"}'
curl -s -H 'Content-Type: application/json' http://127.0.0.1:7878/control \
  -d '{"action":"step","ticks":10}'
curl -s -H 'Content-Type: application/json' http://127.0.0.1:7878/control \
  -d '{"action":"speed","ticks_per_second":30}'
curl -s -H 'Content-Type: application/json' http://127.0.0.1:7878/control \
  -d '{"action":"resume"}'
```

Diğer kontrol gövdeleri:

```json
{"action":"set_cell","pos":{"x":0,"y":0},"cell":{"Water":{"amount":100}}}
{"action":"spawn","species":"Herbivore","pos":{"x":1,"y":1}}
{"action":"intent","id":1,"intent":{"Move":{"steps":["Up","Right"]}}}
{"action":"remove","id":1}
{"action":"shutdown"}
```

`update_entity` için `/state` içindeki bir bireyin **tam nesnesini** `entity`
alanında gönderin: `{"action":"update_entity","entity":{...}}`.
Konum, yaşam özellikleri, tür, faz ve controller değiştirilebilir. Konum ve kapasite
kontrolleri yapılır; görüş/hız en fazla 32, kaynak miktarı en fazla 1 milyon,
HTTP üzerinden spawn sınırı 10 bin mevcut kayıttır. Bu son sınır doğal üremeyi
sınırlamaz. Müdahaleler doğal simülasyon olaylarından ayrı kaydedilir.
`step`, pause gerektirir ve tek istekte 1–100 tick kabul eder. Komut verme
canlıyı o tick için yönlendirir; kalıcı controller değişimi `update_entity` ile yapılır.
`remove` fazı Removed yapar; fiziksel temizlik bir sonraki tickte olur.

Kontrol API'si henüz idempotency anahtarı desteklemez; özellikle step/spawn
isteklerini bağlantı hatasında körlemesine tekrar göndermeyin, önce durumu okuyun.

## SQLite geçmişi

Her dosyada aynı şema vardır:

- `records`: Tick veya müdahale sonrası tam JSON checkpoint, zaman, metrikler ve
  motorun niyet/plan/olay izleri. `(shard, id)` kaydın global adresidir; tick tekil
  değildir, aynı tickte birden fazla müdahale olabilir.
- `events`: Tick olayları; doğum/çiftleşme olayları ebeveyn ilişkilerini de içerir.
- `decisions`: Her karar veren bireyin seçtiği niyet; boşta beklemeler de dahildir.
- `interventions`: Kabul edilen kontrol istekleri ve sonuçları. Step isteği batch
  öncesinde de kaydedilir; kesintide tamamlanan tickler görülebilir. Reddedilen
  istekler dünyayı değiştirmez ve bu tabloya yazılmaz.

Checkpoint şu anki somut canlılarda bulunan tüm yaşam durumunu içerir.
Yeni bir canlıya hafıza/model/genom gibi alanlar eklendiğinde bunlar da sürümlü
checkpoint sözleşmesine eklenmelidir. Güncel geliştirme/laboratuvar SQLite şeması V2, checkpoint V8; gelecekteki
uyumsuz sürümler sessizce yüklenmez. Yeni ölüm nedeni alanı eski kayıtlarda boş kabul edilir. RNG artık dünya başınadır.
Bu bir gelecekteki tüm sürümler için otomatik
migrasyon sistemi değildir.

```bash
curl -s -H 'Content-Type: application/json' http://127.0.0.1:7878/query \
  -d '{"shard":0,"sql":"SELECT id,tick,kind,metrics_json FROM records ORDER BY id DESC LIMIT 20"}'
curl -s -H 'Content-Type: application/json' http://127.0.0.1:7878/query \
  -d '{"shard":0,"sql":"SELECT entity_id,intent_json FROM decisions WHERE entity_id=1 ORDER BY record_id DESC LIMIT 20"}'
```

Bireylerin tarihsel özellikleri SQLite `json_each(state_json, '$.entities')`
üzerinden, kaynaklar `$.resources` üzerinden sorgulanabilir. Sorgular 1000 satır,
4 MiB sonuç ve yaklaşık 200 ms SQLite yürütme bütçesiyle sınırlandırılır.
Arşivlerin tamamını birleştiren tek SQL uç noktası yoktur; `/databases` listesindeki
shardlar ayrı sorgulanır. Yerel SQLite araçlarıyla da incelenebilir.

**Yeni tablo dosyayı küçültmez.** Dosya hedef boyuta ulaştığında yeni
`history-000001.sqlite` dosyasına geçilir. Eski dosyada WAL checkpoint yapılır.
Eşik yaklaşık değerdir: bir transaction kadar aşılabilir. Eski dosyalar otomatik
silinmez. Toplam geçmiş bütçesi dolduğunda veya kayıt yazılamadığında simülasyon
pause olur ve `/status.storage_error` dolar; kayıt kaybını gizleyerek ilerlemez.
Bütçe kontrolü dosya geçişinden önceki mantıksal SQLite boyutlarını kullanır;
aktif WAL, dosya sistemi boş alanı ve tek transaction taşması için pay bırakın.

Böyle bir hatada bellekteki son tick/müdahale kalıcı kayıttan ileride olabilir.
Yeni müdahaleler reddedilir. Alan sorununu düzeltip son **başarılı transactiondan**
yeniden başlatın. Son commit edilmemiş tick tekrar hesaplanır. HTTP başarı cevabı,
kaydı da commit edilmiş müdahale anlamına gelir.

## Erişim ve işletim

Varsayılan servis yalnızca loopback dinler. İsteğe bağlı `CURIOUS_TOKEN` ortam
değişkeni ayarlanırsa bütün uçlar `Authorization: Bearer ...` gerektirir.
Loopback dışı dinlemek için en az 24 karakter token zorunludur. Uzak erişimde SSH
tüneli veya TLS proxy kullanın. Bu yönetim portudur; genel kullanıcı API'si değildir.

Örnek systemd birimi `deploy/curious.service`. Bilerek pause ile başlar;
inceleme sonrası `resume` gönderilir. Her yeniden başlatmada tekrar pause olur.
`systemctl stop curious` süreci durdurur; API `pause` dünyayı duraklatır,
`shutdown` süreci başarıyla kapatır. `Restart=on-failure` nedeniyle API shutdown
sonrası otomatik yeniden başlamaz.

## Doğrulama

```bash
cargo test
cargo build --release
python3 scripts/check_control.py target/release/curious
```

Kayıt/rotation, geçmiş shard sorgusu, read-only sınırı, müdahale doğrulama,
RNG/checkpoint devamı, HTTP auth, pause/hız/step, ikinci süreç kilidi ve SIGKILL
sonrası kalıcı müdahalelerin korunması kontrol edilir. Aylar süren dayanıklılık
ve büyük nüfus performansı henüz doğrulanmış değildir. Ekosistemin ilk nesilden
sonra devam etmesi, öğrenme ve evrim bu değişiklikte tamamlanmış sayılmaz.

## Tick sözleşmesi

Temizlik → faz süreleri/metabolizma → ölüm kontrolü → doluluk/algı/karar → planlar
→ eylemler → eylem sonrası ölüm kontrolü → kayıt. Yeni doğan bir bireyin ilk
metabolizma adımı sonraki ticktedir. Uyuyan birey de yaşlanır ve kaynak tüketir.
`EntityDied.cause`, ölüm nedenini; `ActionRejected`, saldırı/üreme reddinin nedenini
bildirir. `CorpseConsumed` ikinci bir ölüm olarak sayılmaz.

Bu sözleşmedeki değişiklikten önceki kayıtların yüklenmesi desteklenir; eski
ikiliyle aynı geleceği üretmek garanti edilmez. Aynı ikili/sözleşme ve seed altında
tekrar üretim test edilir.

## Yenilenen çevre ve senaryolar (checkpoint v2)

Çevre V2 yükseltmesinde eklendi. Güncel geliştirme/laboratuvar SQLite şeması V2, checkpoint V8. V1 kayıtları okunur:
mevcut kaynak hücrelerinden varsayılan yenilenme konumları çıkarılır; daha önce
bitmiş kaynakların eski konumu V1 checkpointten geri kazanılamaz. Mevcut birey
yaşam değerleri korunur; eksik soy bilgisi kurucu nesil olarak kabul edilir.
Bu yükseltme eski ikiliyle aynı geleceği garanti etmez.

V2 `environment.sites` alanında konum, tür, kapasite, yenilenme miktarı/sıklığı;
bireylerde `generation`, `parents`, `corpse_energy` alanları saklanır. Boş kaynak
konumları da checkpointte kalır. `EntityAte.pos` ve `EntityDrank.pos` tüketilen
kaynağın konumudur; canlı aynı hücrede olmak zorunda değildir.

```json
{"action":"set_resource_site","site":{"pos":{"x":0,"y":0},"kind":"Water","capacity":30,"replenish":4,"interval":1}}
{"action":"remove_resource_site","pos":{"x":0,"y":0}}
```

`set_cell` anlık miktarı değiştirir; tek başına yenilenme konumu oluşturmaz.
`remove_resource_site` yenilenmeyi kaldırır, eldeki kaynağı silmez.

```bash
./target/release/curious --serve --paused --seed 1 \
  --scenario scenarios/meadow.json --data-dir data/meadow --bind 127.0.0.1:7879
```

Senaryo JSON'u açık sınırlar, kurucular, kaynak konumları ve başlangıç miktarları
belirtir. Devam edilen kayıt varsa `--resume-or-new` kayıtlı çevreyi kullanır;
senaryo dosyasını değiştirerek mevcut dünyayı sessizce değiştirmez.

Kütüphane `SimulationEngine::snapshot()` artık aynı tam checkpoint temsili döndürür:
eski `map_resources` yerine türü koruyan `resources` alanı kullanılır.
Gözlem araçları bu alan değişimine göre güncellenmelidir.

## Sıkıştırılmış geçmiş ve sürekli gözlem

Kapanmış shardlar artık `history-NNNNNN.sqlite.gz` olarak kayıpsız saklanır.
`/databases` her shard için `compressed` alanını döndürür. `/query` aynı shard
numarasıyla çalışır; arşiv geçici olarak açılır. 2 GiB açılmış dosya sınırı vardır.
Sıkıştırma öncesi WAL checkpoint, arşivi bayt bayt doğrulama ve atomik yayımlama
uygulanır. Eski kaydın içeriği atılmaz. SQL JSON sorguları değişmez.

`/status` artık `alive`, `history_bytes` (yaklaşık mantıksal aktif + sıkıştırılmış
geçmiş boyutu) ve `history_budget_bytes` verir. WAL/geçici arşiv alanı için pay bırakın.

Çalışan ayrı gözlem dünyası: `127.0.0.1:7879`, `curious-meadow.service`,
`data/meadow`, 0,1 tick/s. `7878` üzerindeki önceki dünya pause durumunda korunur.
`curious-monitor.timer` çayırda hata/yok oluş/bütçe durumlarında Notify gönderir.
Ayrıntılar: [STORAGE_AND_OPERATIONS.md](reports/STORAGE_AND_OPERATIONS.md).

## Karar kanıtı (SQLite şema V2)

`GET /decisions`, son tickin kararlarını döndürür. Her kayıtta `before`,
`seen`, `decision.reason`, `decision.intent`, `target` ve `outcome` bulunur.
Gerekçe, seçimi yapan kod dalından gelir; sonuç görüldükten sonra üretilmez.
`outcome.events` bireyin eylemini veya ona gelen etkileri içerir.

```json
{"action":"observation","full_all":false,"full_for":[1,20]}
```

Varsayılan: bütün bireylerde kompakt algı özeti ve gerçek sonuç. Seçilen bireylerde
`perception` tam algıyı içerir. `full_all:true` bütün bireylerde ayrıntıyı açar ve
kayıt büyümesini artırır. Gözlem ayarı checkpointte korunur. Restart sonrasında
`/decisions` ilk ticke kadar boştur; geçmiş SQLite'tadır.

Yeni `decision_traces(record_id,entity_id,trace_json)` tablosu tick transactionıyla
birlikte yazılır. SQLite şeması **V2**, dünya checkpointi **V8**. Eski shardlar
salt okunur kalır, içlerinde yeni tablo yoktur. Aktif eski shard açılırken tablo
atomik eklenir; tanınmayan gelecek şema sürümü reddedilir.

Servis binary'leri artık `bin/releases/` altında sürümlenir; systemd
`bin/curious-live` bağlantısını kullanır. `target/release` içinde deney derlemek,
çalışan dünya beklenmedik yeniden başlarsa yeni davranışı sessizce devreye sokmaz.

## Dayanıklılık (checkpoint V3) ve deney servisi

Yaşam durumuna `max_stamina`, `stamina`, `exertion` eklendi. V1/V2 içinde eksik
alanlar 16/16/0 olarak yüklenir; sonraki kayıt V3 olur. Eski fizikle aynı geleceği
garanti etmez. Ayrıntılar: [deney raporu](reports/DECISION_AND_PREDATION.md).

`curious-lab.service`, `127.0.0.1:7880`, `data/lab`, yeni fizik/karar sürümüyle
pause başlar ve checkpointten devam eder. `7878` ve `7879` eski sürümde kalır.
Laboratuvarda `step` ile davranış gözlenebilir; `bin/curious-lab` canlı çayırın
`bin/curious-live` bağlantısından ayrıdır.

## Hareket V4

Checkpoint V4, duvarları da `resources` içinde saklar. `set_cell` için `"cell":"Wall"`
kullanılır. Her gerçek hareket adımı ara hücre doluluğu dahil yeniden doğrulanır.

## Gözlem ekranı ve V7 durum

`GET /` gözlem ekranını açar. Veri/komut uçları token varsa korunur; statik
arayüz anahtar girilebilmesi için açıktır. `/observation` atomik gözlem yanıtıdır.
Checkpoint V7: bireysel `memory`, `learner`, `genome`; dünya `genetics`.
`update_entity` tam nesnesinde bu alanlar da saklanır.

```json
{"action":"genetics","enabled":true,"mutation_per_mille":20}
```

Hafıza ve öğrenme bireyin yaşamında kalır; yavrulara genom aktarılır. Yeni kayıt
eski motorla açılamaz; geri dönüş için yükseltme öncesi veri yedeği gerekir.

## V8 habitat deneyi ve sürekli V7 koşusu

`Cell::Burrow` / `set_cell` için `"cell":"Burrow"`: otçulun girebildiği dar
sığınak; büyük avcı giremez veya içine saldıramaz. Enerji/su ve yaşam desteği
vermez. Senaryolar `mixed-refuge.json` ve sığınaksız eş kaynak/kurucu kontrolü
`mixed-refuge-open.json`. Sonuçta avcı soyunun kalıcı olmadığı raporda açıktır.

Güncel kaynak ve 7880 laboratuvar V8; 7881 sürekli öğrenme/kalıtım çayırı V7
ikilisiyle sabit kalır. 7879 eski çayır eski sürümdedir. Gerçek süreli raporlar
1/7/30 gün ve yeterli 15 dakikalık kapsam oluşmadan gönderilmez. Operasyon
komutları ve güncel sürümler AGENTS.md'de tutulur.

## Soy ve yaşam arşivi

Gözlem ekranında birey numarasıyla arama yapılabilir. Ebeveyn, seçili birey
ve yavru bağlantıları arasında geçiş yapılır; ölmüş bireyler de seçilebilir.
Güncel dünyada bulunmayan bireyin son kayıtlı bedeni açıkça arşiv görüntüsü
olarak gösterilir. Bu seçim haritada canlı oluşturmaz; takip ve ayrıntılı güncel
algı kaydı devre dışıdır. Arşiv kararı, güncel kararla karıştırılmaz.

“Soy ve geçmişi getir” en yeni kayıtlardan başlar. “Daha eski kayıtlara devam et”
her istekte en çok 50 karar ve dört boş arşiv parçası tarar. SQLite ve gzip
arşivleri mevcut salt okunur `/query` API'siyle açılır; yeni tablo yoktur.
Soy bağlantıları güncel ebeveyn alanlarından ve kayıtlı çiftleşme olaylarından
çıkarılır. Henüz karar vermemiş yavru, doğum kaydındaki bedeninden açılabilir.
Tarama tamamlanmadan yavru bulunmaması, yavru olmadığı anlamına gelmez;
arayüz tarama kapsamını gösterir. En çok sekiz bireyin arşiv okuyucusu ve
okuyucu başına 200 arşiv yavru bağlantısı bellekte tutulur. Soy görünümü bütün
nesilleri aynı anda çizen sınırsız bir grafik değildir; bağlantılarla ilerlenir.

Ölüm kaydı son kararın ardından 32 kayıtlık aralıkta ve gerekirse hemen sonraki
parçanın ilk 32 kaydında aranır. Bulunursa adım ve neden gösterilir; bulunamaması
hayatta olduğu sonucunu doğurmaz. Karar vermeden ölen ve hiçbir bilinen doğum
bağlantısıyla açılmayan bireyler için numara araması kayıt bulamayabilir.
Arşiv sorguları her saniyelik gözlem döngüsünde tekrarlanmaz. Açık okuyucu
arşiv envanterini sabit tutar; yeni kayıtlar için “En yeni kayıtlara dön” kullanılır.

Doğrulama: `node scripts/test_archive.cjs` gerçek SQLite üzerinde SQL, sayfalama,
parça sınırında ölüm, henüz karar vermemiş yavru ve eksik kayıt durumlarını sınar.
`scripts/check_ui.cjs` izole dünyada masaüstü/mobil ve ölmüş ata–yavru geçişlerini
sınar; dışarıdan Playwright gerekir. Canlı dünyaları ve deney verilerini değiştirmez.
