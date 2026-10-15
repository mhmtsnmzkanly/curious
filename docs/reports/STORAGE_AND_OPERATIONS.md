# Kayıpsız arşiv ve canlı gözlem — 15 Ekim 2026

## Kayıt maliyeti

Aynı çayır, seed 1, 10.000 tick:

| Ölçüm | Sıkıştırmasız | Kapanmış shardlar gzip |
|---|---:|---:|
| Disk | 146.751.488 bayt | 26.456.234 bayt |
| Koşu süresi | 5,326 s | 6,421 s |
| En yüksek nesil | 230 | 230 |
| Doğum / ölüm | 774 / 770 | 774 / 770 |

Bütün olay ve niyet sayıları **aynı**, son dünya checkpointi **birebir aynı**.
Bu koşuda boyut **5,55 kat** azaldı. Arşiv deneyinde hızlıca çok sayıda geçişi
sınamak için 1 MiB shard kullanıldı; aktif gözlemde 16 MiB. Sıkıştırma oranı ve
süre tek bir küçük senaryonun ölçümüdür; tüm dünyalara sabit oran vaat edilmez.

Kayıt örneklenerek veya olaylar atılarak küçültülmedi. Tam JSON durum, niyetler,
olaylar, metrikler ve müdahaleler SQLite içinde korunur. Kapanmış dosya gzip
olur; aktif dosya normal SQLite/WAL olarak kalır.

## Dayanıklılık sözleşmesi

1. Eski dosyada WAL checkpoint tamamlanmadan arşiv oluşturulmaz.
2. Yazıcı bağlantısı kapatılır; arşiv aynı dizindeki geçici dosyaya yazılır.
3. Arşiv yeniden açılır ve kaynakla bayt bayt karşılaştırılır; CRC/footer doğrulanır.
4. Dosya ve dizin senkronize edilir, atomik isimle yayımlanır.
5. Ancak bundan sonra sıkıştırmasız kopya kaldırılır. Mantıksal geçmiş silinmez.

Kesintide kaynak veya yayımlanmış arşiv korunur. Kaynakla arşiv birlikte kalırsa
kaynak önceliklidir. Tamamlanmamış `.curious-archive-*` dosyaları sonraki açılışta,
run yazıcı kilidi tutulurken temizlenir. Sıkıştırma hatası normal kayıt hatası gibi
dünyayı pause eder; geçmiş sessizce kaybedilerek ilerlenmez.

`/databases` sıkıştırılmış dosyaları da gösterir. `/query` arşivi özel geçici dizine
açıp salt okunur sorgular; sonra geçici veri kaldırılır. 2 GiB açılmış dosya sınırı
vardır. SQL yürütme bütçesi açma süresinden sonradır. Eski büyük arşivleri sorgulamak
ve dosya geçişinde sıkıştırmak kısa süre tick ilerlemesini bekletebilir; asenkron
arşiv işçisi henüz yok. Geçici disk alanı/WAL için bütçenin dışında pay gerekir.

## Çalışan servisler

| Servis | Adres/veri | Davranış |
|---|---|---|
| `curious.service` | `127.0.0.1:7878`, `data/main` | Önceki dünya korundu, 10. tickte pause; V2'ye yükseltildi |
| `curious-meadow.service` | `127.0.0.1:7879`, `data/meadow` | Seed 1, yeni çayır; 0,1 tick/s; restartta kayıtlı dünyadan çalışır |
| `curious-monitor.timer` | Dakikada bir | Çayırı salt okunur kontrol eder; gerektiğinde Notify gönderir |

Çayır kasıtlı olarak **yavaş gözlem hızında**: her tick yaklaşık 10 saniye.
Ölçülen disk oranıyla bu yaklaşık 23 MB/gün; gelecekteki nüfus/özellik değişiklikleri
bu tahmini değiştirebilir. 10 GiB kayıt bütçesi korunur. Bu başlangıç,
aylarca dayanıklılığın şimdiden kanıtı değildir; gerçek süreli koşu şimdi başladı.
Hız API'den değiştirilebilir. Hızı artırmak kayıt büyümesini de artırır.

Ana dünyanın yükseltme öncesi SQLite yedeği, eski çalışan binary ve JSON durumu:
`data/backups/pre-checkpoint-v2/`. Yükseltmede tick, RNG, mevcut beden değerleri,
konumlar, kaynaklar ve bekleyen komutlar korunarak kontrol edildi.

## Otomatik bildirim

İzleyici motorun parçası değildir ve dünyaya müdahale etmez. Şu durumlarda haber verir:

- Kontrol portuna üç kontrol üst üste erişilemiyorsa.
- Kalıcı kayıt hatası varsa.
- Yaşayan birey kalmadıysa; yeniden canlı eklemez.
- Kayıt bütçesi %80'i aştıysa.
- Önceki hata/yok oluş durumu sonradan düzelmişse.

Aynı durum her dakika tekrar gönderilmez. Tam bildirim gövdesi/idempotency anahtarı
önce `data/meadow-monitor.json` içinde saklanır; teslim onayı gelmeyenler yeniden
denenir. Notify başarısı cihazda gösterim garantisi sayılmaz. İzleyicinin kendi
servis hataları systemd journal'da görünür; kendisini denetleyen ayrı bir sistem yok.

## Doğrulama

- 22 Rust testi: kaynak/yaşam sözleşmeleri, çok nesilli koşu, bağımsız RNG,
  checkpoint göçü, arşiv doğrulama, bozuk arşiv reddi, arşivden devam ve geçici temizliği.
- HTTP entegrasyonu: kontrol, auth, arşiv sorgusu, arşiv sonrası SIGKILL devamı ve yazıcı kilidi.
- 5 Python izleyici testi: arıza eşiği, tekrar bastırma, düzelme, yok oluş, bütçe ve eksik disk ölçümü.

Sıradaki biyolojik çalışmalar avcı–av döngüsü ve karar gözlemi; ardından bireysel
hafıza/öğrenme. Yeni bir öğrenme/genom alanı eklenirken checkpoint ve uzun süreli
bellek/disk maliyeti de güncellenmeli.
