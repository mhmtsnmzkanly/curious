# Kalıcı kayıt ve kontrol servisi

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
checkpoint sözleşmesine eklenmelidir. Güncel SQLite şeması V2, checkpoint V3; gelecekteki
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

Çevre V2 yükseltmesinde eklendi. Güncel SQLite şeması V2, checkpoint V3. V1 kayıtları okunur:
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
birlikte yazılır. SQLite şeması **V2**, dünya checkpointi **V3**. Eski shardlar
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
