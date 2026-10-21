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
