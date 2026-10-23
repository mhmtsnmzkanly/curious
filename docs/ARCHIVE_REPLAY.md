# Salt okunur arşiv tekrarı

`scripts/archive_replay.py`, Curious'un mevcut SQLite kayıtlarından ayrı bir
tarayıcı izleyicisi sunar. Simülasyon kontrol uçları veya keyfi SQL uç noktası
yoktur. Tarayıcıdaki oynat/duraklat ve kayıt/sn seçeneği yalnız görüntüyü
ilerletir; simülasyon hızını ya da durumunu değiştirmez.

Python 3.10+ yeterlidir. `.sqlite.zst` geçmişi için sistemde `zstd` CLI bulunmalı;
eski `.sqlite.gz` shardları Python standart kütüphanesiyle açılır. Varsayılan
adres loopback `127.0.0.1:7883`:

```bash
cd /root/curious
python3 scripts/archive_replay.py \
  --data-dir /root/curious/data/coexistence \
  --bind 127.0.0.1 --port 7883 \
  --cache-dir /tmp/curious-replay-cache
```

Ardından `http://127.0.0.1:7883/replay` sayfasını açın. Uzak tarayıcı için
loopback adresine SSH tüneli kullanın. `--cache-dir` verilmezse çalışma
boyunca kullanılan geçici önbellek süreç kapanınca kaldırılır. Önbellek arşiv
dizininin dışında tutulmalıdır.

Shard listesi yalnız dosya adlarını/metaverisini okur. İçerik ancak kullanıcı
bir shard seçince açılır. Ham aktif SQLite `mode=ro` ve `query_only=ON` ile
okunur; WAL görünürlüğünü korumak için `immutable=1` kullanılmaz. Sıkıştırılmış
shard geçici dizine akış halinde çözülür; tek shard için açılmış boyut 2 GiB
sınırındadır ve toplam disk önbelleği 256 MiB'ı veya iki shardı aşmaz. Kayıt
zaman çizelgesi sabit sayfalarla okunur; API istek başına en çok 500 kayıt
verir. Servis simülasyonun writer lock dosyasını almaz, SQLite şeması/indexi
oluşturmaz. SQLite salt okunur sorgular için kendi okuma kilitlerini kullanır.

Yalnız şu GET uçları sunulur:

- `/api/shards`: bulunan shard numaraları ve sıkıştırma/boyut bilgisi
- `/api/shard-meta?shard=N`: seçilen shardın tick aralığı ve kayıt sayısı
- `/api/timeline?shard=N&after=ID&limit=200`: artan kayıt kimlikleriyle sayfalama
- `/api/record?shard=N&id=ID`: seçili kaydın dünya durumu, metrikleri, kararları ve olayları
- `/api/individual-history?shard=N&id=ID&before=ID&limit=50`: bireyin karar izini aynı shard içinde geriye doğru, en çok100 kayıtlık sabit bir sayfada okur. `before` yerine `after` verilirse ileri sayfalar okunur. Yanıt `{items, has_more, next_before, next_after, budget_limited}` zarfıdır; arayüz devam işaretçisini kullanır, satır sayısından arşiv sonunu çıkarmaz. Bütçe nedeniyle boş/kısmi sayfa dönerse `budget_limited` ve imleç, okumanın gerçekten sonlandığını ayırt eder. Sorgu `decision_traces(entity_id,record_id)` indeksini kullanır; V2 öncesi bireysel iz tablosu yoksa boş `items` ve `has_more:false` döner. Her satır için state en çok8MiB ve trace en çok1MiB ise gösterilir; çağrı toplamda64MiB okuma ve2s SQLite çalışma süresi ile sınırlıdır. Olay yanıtı bireyle ilişkili sabit event alanlarından en çok50 kayıt/64KiB olay alır. Eşik dışı veri bilinmiyor olarak kalır.

Bir bireyin yaşam izi yalnız seçili bireyin kimlikli karar kayıtlarından oluşur. Görülenlerin ayrıntısı yalnız tam algı kaydedilmişse gösterilir; özet veya eski kayıt eksik bilgiyi tamamlamaz. Bellek ve beden değerleri karar sonrası kayıt durumundandır; karar öncesi yaş ayrıca karar izindeki `before` alanından gösterilir. Gerçek maliyet yalnız resolver olaylarının yazdığı hareket/saldırı enerjisinden hesaplanır; eski olay alanı eksikse toplam bilinmiyor denir. Su tablosu için ardışık taranmış kayıtların saklı konum/değer/ziyaret snapshot farkı gösterilir; ilk satırda, eksik learner alanında veya kesilmiş arşiv kapsamı için fark bilinmiyor kalır. Bu snapshot farkı kaynak ediniminin nedeni veya yaşam faydası değildir. Besin/enerji Q öğrenme güncellemesi, her tickte yinelenebilen `last_episode` alanından sayılmaz; yalnız ilgili `HuntEpisodeEnded` olayının `updated` bayrağından gösterilir. Bu bayrak risk modelinin güncellenmesini ifade etmez. V14 risk alanları (`risk_enabled`, `risk_values` ve episode içindeki exposure/injury/damage alt alanları) varsa saklanan karar sonrası istatistik olarak görüntülenir; per-record farkı kanıtlanmadığı için artış/güncelleme denmez. V13/eskilerde alan yokluğu sıfır sayılmaz.

Benzer av karşılaşmalarının sabit eşleşme kuralı, kayıtlı HuntContext içindeki enerji, dayanıklılık, mesafe ve sağlık sınıflarının dördünün de biliniyor ve eşit olmasıdır. Context yalnız bireyin kayıtlı aktif episode'undan veya aynı kayıttaki kapanış olayının attempt'inden alınır; eksik/eski algıdan sağlık sınıfı tahmin edilmez ve bilinmeyen sınıflar eşleşmez. Aynı hedef/context ile artan attempt tick'leri tek episode grubu sayılır; yeni attempt tick'i veya kapanıştan sonra başlayan bölüm yeni grup olur. İlk/son karşılaştırma UI'nin taradığı kapsamla sınırlıdır, her iki başlangıç kaydında kimlik, tick ve karar öncesi yaş gösterilir. Benzerlik nedensel başarı veya öğrenme yararı kanıtı değildir. Daha eski sayfalar en çok50 karar satırıyla yüklenir; bütçe sınırında gerçek imleçle sürdürülür ve tarayıcı belleğinde en çok500 kayıt tutulur.

Bu servis `7882` veya simülasyonun kontrol API'sine bağlanmaz; doğrudan verilen
arşiv dizinini açar. `scripts/test_archive_replay.py` gzip/zstd okumasını, salt
okunur sınırını, sayfalamayı, sabit API listesini, yazma metodlarının reddini ve
arşiv dışına çıkan sembolik bağlantının reddini sınar. Gerçek tarayıcı akışı ve
mobil görünüm `node scripts/check_replay_ui.cjs` ile doğrulanır; bu kontrol
yalnız kendi geçici arşivini oluşturur.

## Doğrulama ve ayrı servis

Luna alt ajanı okuyucu ve arayüzü hazırladı; ana ajan inceleyip bağımsız
Chromium testini ekledi. Tam Python paketi55test geçti. Gerçek Chromium
masaüstü/mobil testinde gzip/zstd/hamSQLite, zaman seçimi, eski yanıtın yeni
seçimi ezmemesi, shardlar arasında oynatma, birey seçiminin korunması, pause,
sadece GET ve kaynak dosyalarının baytlarının değişmemesi doğrulandı.

`deploy/curious-replay.service`, ayrı7883portunda frozen kaynakla çalışır.
`data/replay-runtime.env` içindeki CURIOUS_REPLAY_SOURCE temiz kaynak worktree
yoludur. ProtectSystem=strict ve ReadOnlyPaths simülasyon veri dizinine
yazmayı işletim sistemi seviyesinde engeller; PrivateTmp geçici önbelleğe
yazma izni verir. Dünya7882servisi/ikilisi/verisi değiştirilmez. İzleyici
sayfasını yenilemek aktif shardın yeni kayıt listesini getirir; oynatma
mevcut listeyi izler, canlı takiple karıştırılmaz.

Uzak bilgisayardan erişim:

```bash
ssh -N -L 7883:127.0.0.1:7883 root@SUNUCU_ADRESI
```

Ardından http://127.0.0.1:7883/replay açılır. Baştan izle ile ilk kayda
gidilir; Oynat kayıtları tarayıcıda sırayla gösterir.
