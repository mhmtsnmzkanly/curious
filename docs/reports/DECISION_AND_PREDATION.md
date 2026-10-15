# Karar kanıtı ve ilk avcı–av deneyi

Bu aşama öğrenme veya evrim içermez. Amaç, kurallı bireyin davranışını kayıtla
incelemek ve yaşam döngüsünü gizli nüfus desteği olmadan düzeltmekti.

## Seed 2: neden soy kesiliyordu?

[İlk teşhis](SEED2_DECISIONS.md): son yavru 164. tickte doğuyor; ebeveynleri
181/183'te ölüyor, kendisi 184'te olgunlaşıyor. 270'te yaşlılıktan ölüyor.
Karar anındaki 96 bitişik, üremeye hazır eş fırsatının 80'inde bireyler %75
rezerv hedefini doldurmayı seçiyordu. Bunlar birey-karar sayılarıdır; 80 olası
başarılı doğum demek değildir.

Düzeltme: tehdit önceliği korunurken, bedenin mevcut üreme güvenlik sınırlarını
karşılayan birey bitişik hazır eş fırsatını değerlendirir. Enerji/su/sağlık
sınırları, olgunluk, bekleme süresi ve ebeveynlerin yavruya ödediği kaynaklar
korunur. Kaynak miktarı veya kurucu nüfus değişmedi; seed'e özel kural yok.

Yorgunluk dahil son otçul kurallarıyla 8 seed × 2.000 tick: **8/8 hayatta**,
48–55. nesil. Seed 2: 168 doğum, 165 ölüm, 52. nesil, sonda 11 canlı.
Bu, sonsuz süre garantisi veya evrim kanıtı değildir.

## Avlama ve beden sınırları

Eski av koridoru: dört seed'de sıfır savaş ölümü; 38–40 enerjisiz saldırı reddi.
Avcılar açlıktan ölmüş bedenden yiyebilse de kendi avını öldüremiyordu.

Yetenek profili artık saldırı hasarı/enerji/dayanıklılık maliyetini taşır.
Yerleşik hasarlar otçul 6, etçil 20, hepçil 14; saldırı 3 enerji ve 2 dayanıklılık
harcar. Tür uygulaması profili değiştirebilir. Hareket bir adımda bir enerji ve
bir dayanıklılık, üreme iki dayanıklılık harcar. Başlangıç dayanıklılık kapasitesi
otçul 16, etçil 24, hepçil 20. Efor olmayan önceki adımdan sonra 4, eforlu adımdan
sonra 1 dayanıklılık geri gelir. Dinlenme besin enerjisi üretmez; metabolizma sürer.

Avcı dayanıklılığı düşükken avı bırakıp dinlenir; en az yaklaşma + bir saldırı
bedelini karşılayamıyorsa ava başlamaz. Sıkışmış av karşı saldırabilir; açık
kaçışta kaçmayı seçer. Ceset toplam besini max_health/2, süre 5 tick ve öğün
10 ile sınırlıdır. Bunlar ilk kalibrasyon değerleridir; doğal dünyanın ölçülmüş
sabitleri veya öğrenilmiş davranış değildir.

`hunt-corridor.json`, tüm fiziksel başlangıç koşulları açık olan iki bireylik
laboratuvardır. Önceki explicit beden JSON'larında dayanıklılık alanı olmadığından
iki birey de belgelenmiş 16 varsayılanıyla yüklenir. Dört seed'de artık bir av
savaşta ölür, avcı ardından beslenir. Yalnız bir av bulunduğu için avcı daha
sonra açlıktan ölür; bütün dünya 70–72. tickte boşalır. Sürdürülebilirlik deneyi değildir.

## Karma çayır: henüz süreklilik yok

Aynı çayıra iki etçil kurucu ekleyen `predator-meadow.json`; 4 seed × 1.000 tick.

| Seed | Otçul doğum | Etçil doğum | Son yaşayan etçilin ticki | Bütün dünyanın yok oluşu |
|---|---:|---:|---:|---|
| 1 | 7 | 1 | 120 | 192 |
| 2 | 83 | 1 | 120 | yok, 1.000'e kadar |
| 3 | 76 | 0 | 108 | yok, 1.000'e kadar |
| 42 | 79 | 1 | 120 | yok, 1.000'e kadar |

Avlama, beslenme ve üç başlangıçta etçil yavru doğumu gerçekleşir. Yavru etçiller
66–100 aralığında açlık/susuzluktan ölür; kalıcı avcı soyu oluşmaz. Seed 1'de
artan av baskısı altında tüm dünya da boşalır. İstenen sonuca ulaşıldı denemez.
Sıradaki araştırma genç etçilin besine/suya erişimi, aile paylaşımı ve avı seçme/
bırakma kararlarıdır. Önce bu darboğazlar açıklanmalı; sırf nüfus devam etsin diye
otomatik yeniden doğurma, koruma veya kaynak desteği eklenmedi.

## Gözlem, kayıt ve sınırlar

`GET /decisions`: önceki beden, görünür hedefler, karar anındaki gerekçe, seçilen
niyet, hedef ve gerçek sonuç. `observation` kontrolü seçilen bireylerin tam
algısını açar. Karar izi ve tick aynı SQLite transactionında saklanır. Tam/kompakt
gözlem aynı RNG ve aynı dünya geleceğini üretir; testle doğrulandı.

SQLite şeması V2; checkpoint V3. Eksik eski dayanıklılık alanları açık varsayılan
16/16/0 alır. Mevcut biyolojik değerler korunur; eski motorla aynı gelecek
vaat edilmez. Aktif eski shardın tablo göçü atomiktir; eski arşivler değişmez.

Kompakt iz + checkpoint + olaylar, sekiz çayır koşusunda 2.000 tick başına
10,09–11,26 MB geçmiş üretti (1 MiB shard eşiği, kapananlar gzip). 0,1 tick/s için
aynı içerik yoğunluğuyla yaklaşık 44–49 MB/gün, 1,3–1,5 GB/30 gün eder.
Bu gerçek ay süren disk ölçümü değildir; nüfus, olaylar, shard boyutu ve tam algı
ayarı değiştirir. Önceki 23 MB/gün hesabı yeni izler için geçerli sayılmaz.

Doğrulama: 31 Rust testi, 5 izleyici testi, ayrı geçici dünyada HTTP auth,
karar/algı ayarı, ayarın SIGKILL sonrası korunması, pause/speed, salt okunur arşiv
sorgusu ve ikinci yazıcı reddi. Kaynak dosyalar ve özet sayılar
[DECISION_AND_PREDATION.json](DECISION_AND_PREDATION.json) içinde; ham SQLite
`data/experiments/` altında, Git dışında korunuyor.

Sürekli çayır eski sürümde devam eder; deney fiziği bu uzun koşuya karıştırılmaz.
Yeni karar uçları ayrı, pause başlayan `curious-lab.service` / `127.0.0.1:7880`
üzerinden incelenebilir. Deney ikilisi `bin/curious-lab` altında ayrı sürümlenir.
