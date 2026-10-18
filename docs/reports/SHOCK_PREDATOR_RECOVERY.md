# Şok sonrası avcıların üreme rezervi

## Önceki adayın sonucu

C2d9da8 normal8/8 geçti, şok6/8 ile reddedildi. Seed3 avcıları13584,
seed2026 avcıları14751'de tükendi. Şok10000–11000 arasında besin/su
alanlarının yenilenmesini yarıya indirir; stok, maliyet ve üreme fiziği değişmez.
Şok sonrası3'te4 doğum/12 yaşlılık ölümü,2026'da8 doğum/15 yaşlılık+1 su
ölümü var. İki kolda da bu dönemde avcı açlık ölümü0. Son ölümün yaşlılık
olması bütün şok boyunca açlık yaşanmadığı anlamına gelmez.

## Aynı başarısızlığın ayrıntılı tekrarı

`ecology_probe` yalnız isteğe bağlı taze dizine, belirtilen adımlarda kontrol
noktası yazabilir: `--checkpoint-dir=DIR --checkpoint-at=9900,11000,...`.
Gözlemci RNG tüketmez veya politika girdisi vermez. Aynı senaryo/seed/kuralda
tekrar yapılır.15000 istenen adım sayısı yüzünden yalnız süre, requested_ticks
ve son yüzde10 penceresine bağlı sayaçlar normalize edilir. Diğer biyolojik
çıktılar önceki20000 adım talebiyle, tükenme zamanına kadar karşılaştırılır.

11000→13000 tam avcı audit'i eski tekrarın13000 kontrol noktasıyla bütün
durum+RNG açısından eşit (yalnız `observation.full_all` normalize edildi).
Seed3'te8704,2026'da10714 karar incelendi. Yeterli bitişik leş, tam sağlık,
yeterli su, enerji dışında fiziksel üreme engeli olmayan ve görünür yetişkin
türdeş bulunan ama türdeş bitişik olmadığı için hazırlanmayan27/23 karar var.
Bu fırsat sayısıdır; ayrı birey veya garantili çiftleşme sayısı değildir.
Eski hazırlık kararı aynı aralıkta yalnız3/4 kez verildi.

## Dar davranış değişikliği

`policy::prepare_reserve` artık avcı için mevcut algıdaki yetişkin türdeşi
yeterli sayar. Yemek hâlâ aynı adımda, yer değiştirmeden alınabilmeli ve öğün
bir sonraki bazal maliyetle birlikte üreme eşiğini tamamlayabilmelidir.
Uzak yemeğe yolculuk, görünmeyen eşi bilme, üreme eşiğini düşürme, fiziksel
avantaj veya nüfus takviyesi yoktur. Otçulun bitişik türdeş şartı korunur.
Hazırlık survival/thirst/hunger/threat ve özel politikaları geçersiz kılmaz.

Bu, rezerv davranışının opt-in araştırma adayıdır: `reserve_opportunities`
varsayılan false. Buluşma davranışı değiştirilmedi. V12 ve SQLiteV2 değişmez;
canlı dünyalar yükseltilmedi. Amaç önce hazırlanıp sonra eşe yaklaşabilmektir;
görünen yetişkinin hazır olduğu veya sonraki adımda yerinde kalacağı garanti edilmez.

## Doğrulama ve kabul sınırı

İki yeni Rust testi, görünür uzak yetişkin için yerinde hazırlığı; susuzluk,
uzak yemek ve eş yokluğu sınırlarını; otçulun eski şartını kapsar.
83 Rust/37 Python testi, Node arşiv ve HTTP/auth/pause/speed/kilit/SIGKILL
kurtarma kontrolleri geçti. Hazırlık kapalı kontrol500 adımda süre dışındaki
bütün probe çıktılarında eski ikiliyle aynı. İki yeni Python testi audit
sayımında farklı fırsatların birbirine karıştırılmamasını ve eksik algının
reddini doğrular.

| Kopya | Eski avcı tükenmesi | Yeni15000 nüfusu (otçul/avcı) | 11000→15000 avcı doğumu |
|---|---:|---:|---:|
| 3 | 13584 | 578/47 | 90 |
| 2026 | 14751 | 572/60 | 108 |

15000'de başlangıç11000 avcılarının hiçbiri yaşamıyor; iki kolda da yaşayan
yavru ebeveynler var. Böylece yalnız eski bireylerin ömrü uzamamış, soy yenilenmiş.
Bu iki seedin kısa toparlanması kalıcı denge değildir.

Karşılaştırmalı kısa kollar11000 kontrol noktasından başlar; değişikliğin
başlangıçtan beri açık olduğu bir koşunun yerine geçmez. Son metrikler ve
SHA'lar eşlik eden JSON'da. Ham kayıtlar `data/experiments/shock-predator-diagnosis`
ve `data/experiments/visible-adult-reserve` altında korunur.

Sonraki kapı temiz kaynak ve değişmez ikiliyle başlangıçtan çalışır:
önce3/2026 hedefli şok, geçerse normal8, o da geçerse shock8. Sabit genetik,
öğrenme kapalı, iki işçi. Ret sonraki aşamayı durdurur. Çalışmanın kimliği ve
güncel durumu `data/experiments/visible-adult-reserve-validation/{study,state}.json`,
unit `curious-visible-adult-reserve-validation`. Program kod/parametre değiştirmez
ve dağıtım yapmaz; bu kapılar kalıcı denge veya30×100000 uzun kabul değildir.
