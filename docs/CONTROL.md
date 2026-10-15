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
checkpoint sözleşmesine eklenmelidir. Şema/checkpoint sürümü 1'dir; gelecekteki
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
