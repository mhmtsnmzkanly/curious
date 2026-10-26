# Resolver Order V1: kayıt öncesi açıklama eki

Bu ek, dondurulmuş [protokoldeki](RESOLVER_ORDER_V1_PROTOCOL_20261026.md) kayıt ve yorum ayrıntılarını açıklar. Protokolün kendisi değiştirilmemiştir. Bu ek, ilk laboratuvar World'ü başlatılmadan önce protokol ile birlikte hash'lenip kayıt dosyasına eklenmelidir; protokol hash'i `e8357e8d2d0795ae110579befb416585f50161355f140ff5d80252dc4402f6ff` olarak sabitlenmiştir. Ek'in SHA-256'sı da her koşu kaydına yazılır.

## Girdi ve kaynak kimlikleri

Beş hedef, eski raporda reddedilmiş öğün örnekleri diye değil, yaşam/sonuç kaybı görülen koşullardan seçilmiş beş **olumsuz seed vakası** olarak tanımlanır. Beş vakanın her birinde hedef tikteki override ve başarılı fiziksel öğün aynı şey değildir; rapor actor ve event düzeyindeki sonucu verir.

`d55d6b1d4fb27ec1abc87d088a8d7616c1dad96a442c7144e9cc3a4f6a075d5a`, orijinal V1 deney kaynak içeriğinin SHA-256'sıdır (`edb76316d7002f0533d4c038b664a796ec293c41`); bu yeni laboratuvar uygulama ağacının hash'i değildir. Her koşu kaydı ayrıca gerçek laboratuvar source commit'ini, temiz source tree hash'ini, binary hash'ini, protokol ve bu ekin hash'lerini tutar. Bu hash ve ham manifest kimlikleri koşu öncesi kayıt içindir; mevcut test/build işlemleri geçmişe dönük olarak bu koşu sayılmaz.

Ham JSONL başlığında başlangıç `initial_state` bulunur, fakat her tik için tam State/RNG checkpoint'i yoktur. Hedef öncesi durum başlıktaki State/RNG'den sıradan tikleri replay ederek yeniden kurulur; replay edilen prefix'in ham kayıtta mevcut event ve fiziksel/stok metrikleri doğrulanır. Tarihsel ara-tik State/RNG ile byte-eşitlik iddiası yapılamaz. Yeniden kurulan hedef-öncesi `State` ve RNG'nin capture/restore/recapture eşitliği ve hedef tik referansları arası karşılaştırmalar koşu içi bütünlük kapılarıdır.

## Aday uygulama ile olağan referans ayrımı

Üç kollu ortak hazırlık, normal politika bayrağı kapalı `control` kararlarını bir kez toplar; A/B/C dönüşümleri bu ortak karar girdilerinden üretilir. Fork hazırlığı içinde bayrak açık ikinci politika seçimi veya ikinci preamble çalıştırılmaz. Ayrı eşitlik referansı olarak, aynı yeniden kurulmuş hedef-öncesi State'ten bir olağan bayrak-kapalı ve bir olağan bayrak-açık `World::tick()` yürütülür. A, ilk referansa; C, ikinci referansa event/RNG/`last_intents`/iz/State alanlarında eşit olmalı; yalnız kayıtlı özellik bayrağı karşılaştırmada normalize edilir. B ve C'nin learner interruption ve audit alanları çözücü öncesinde eşleşmelidir. Bu eşitliklerden herhangi biri bozulursa run durur ve başarısız kalır.

## Kaynak tavanları ve kapsam

398 çözücü-tik çağrısı (373 prefix tick + 15 fork resolver çağrısı + 10 olağan off/on referans tick'i) ve beş paylaşılan kısmi preamble, dondurulmuş protokoldeki laboratuvar çalışmasının sınırıdır. Her child process 45 saniyeyi, 1 GiB sanal adres alanını ve 64 MiB çıktıyı aşamaz. Bunlara ek olarak bütün run klasörünün toplamı **64 MiB** ile sınırlıdır; registration, manifest, çocuk çıktıları, hata kayıtları ve ek metadata bu toplamın içindedir. Toplam tavana yaklaşıldığında yeni child başlatılmaz; taşma olursa ilk hata korunur, run fail-closed durur.

Kayıtlı laboratuvar çalışması başlamadan önce protokol/addendum/hash girdileri sabitlenir. Önceden talep edilmiş veya bu protokol çalışmasından bağımsız derleme, unit, regresyon ya da mekanizma testleri ayrı validasyon işlemleridir; bunlar 398 World-tik bütçesine dahil edilmez ve laboratuvar sonuç verisi sayılamaz. Herhangi bir lab koşusundan sonra kaynak, protokol, ikili, girdi veya sınırlar değişirse aynı run yeniden kullanılamaz; yeni deney kimliği ve yeni frozen protokol gerekir.
