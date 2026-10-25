# Kabul tanısı ve V12/V19 motor köprüsü — 25 Ekim 2026

Birincil alan K8; K3/K5/K6 ile kesişir. Bu çalışma yeni özellik veya canlıya dağıtım değildir. Önceki sonuçlarda görülen soruları açıklayan **keşifsel tanıdır**; kabul eşiği ayarlama veya yeni bağımsız held-out başarı iddiası yapılmaz.

## Sorular ve sabit kapsam

1. Güncel `c5ca361` sonrası yetenek-bayrağı aktarımı bulunan kaynakta, daha önce görülmüş 401..410 seedlerinin 150-tick fiziksel vektörleri seed başına ne gösteriyor? Sağkalım üçlüsünün herhangi birinde kontrol altına düşen seed ayrı işaretlenir. Toplam vektörde eşitlik nötrdür, kanıtlanmış fayda değildir. Eski toplu sınıflandırma yalnız tarihsel karşılaştırma için korunur.
2. Aynı V12 checkpoint'inden frozen V12 kaynak (`8dc23de`) ve güncel V19 kapalı kontrol 1.000 tick ilerlediğinde tür bazlı doğum/ölüm, yaşayan birey-tick, final fiziksel durum ve RNG nasıl ayrılıyor? Bu ikisi farklı motor sürümüdür; fark otomatik olarak bug değildir. Aynı runner kaynağı her iki crate'e bağlanır; yeni bir V12 derlemesi mevcut canlı release binary'siyle bit-for-bit eşdeğer sayılmaz.
3. Güncel V19 kapalı kontrol, yalnız bakım, yalnız ilk-aralık refuge ve birleşik paket için aynı checkpoint'ten 1.000 tick'lik mevcut tanı koşusu; tür bazlı doğum/ölüm ve birey-tick, 100-tick örnekler ve endpoint restore doğrulaması kalıcı JSON'a yazılır. Mevcut dört kolun tekrarıdır; bağımsız tekrar örneklemi değildir.

Girdi: kapalı `history-000870.sqlite.zst`, SHA-256 `307312648f3fabe39d3e038d702de26e312fe9f822ac92eb664c4e7709118856`. Yerel ayrı arşiv index'i bulunmadı. Read-only geçici SQLite kopyasında integrity_check=ok; son record id14, tick12639, V12, 630 slot. JSON SHA `357870bd9a3d414704ed6befbea050852028f54af304c27cdca482dc31abdad2`. Orijinal kapalı shard ve canlı dosyalar değişmez.

## Kaynak, bütçe ve durdurma

Kaynak ve bu protokol koşudan önce commit edilir. Binary/runner/protokol/input SHA'ları yerel manifestte korunur. Kaynak değişirse yeni kimlik açılır; aynı sonucu almak için parametre ayarlanmaz.

En fazla 18.000 küçük-fixture tick (12 kol ×10seed×150), 6.000 büyük-dünya ana tick (iki köprü +dört mevcutkol) ve köprüde açıklanan kısa resume doğrulama çatalları. İki farklı source için ayrı Cargo target, jobs1; büyük-dünya koşuları sırayla. Her komuta 10 dakika duvar-saat sınırı; toplam hedef 20 dakika, disk göreve ait 3 GiB altında. Panik, checkpoint reddi, disk yetersizliği veya süre sınırında koşu başarısız kaydedilir; canlıya veya otomatik tuning'e geçilmez. Örnekleme sonu hayatta kalma geçici son noktadır, aylarca işletim kanıtı değildir.

Ham stdout/stderr, seed vektörleri, özetler, input/endpoint dosyaları, kullanılan ikililer ve manifest yeni `data/experiments/qualification-diagnostic-20261025/` altında korunur. History/SQLite kayıt işletim yükü, salt World tick hızından ayrı ve bu testin dışındadır. Frozen kaynak değiştirilmez; eski binary canlı servis olarak başlatılmaz.
