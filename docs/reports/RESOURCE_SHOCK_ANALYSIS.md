# Kaynak şokunda avcı soyunun kırılması

## Sonuç

Seçilen `n4-steady-q885`, V9/ae7f0df profilinde normal koşulların uzun doğrulaması
30/30 × 100.000 tick geçti. Eğitim seedlerindeki kaynak şoku 4/8 geçti.
Başarısız koşullarda otçullar toparlanırken avcı soyu tükendi. Bu sadece
son avcının açlıktan ölmesi şeklinde bir sorun değildir: son kuyruklarda
rezervi olan yaşlı avcılar ve az sayıda uygun eş gözleniyor. Üreme sürekliliği,
fizyolojik hazır olma penceresi ve yerel eş bulma birlikte araştırılmalı.

Bu bir kanıta dayalı teşhistir; tek bir davranış değişikliğinin çözüm olacağı
veya eş bulmanın tek neden olduğu henüz gösterilmedi.

## Karşılaştırma ve uygulanan şok

`scripts/analyze_resource_shock.py` sekiz kayıt çiftini salt okunur inceler.
Şok ve kontrol manifestlerinde commit, ikili SHA, senaryo SHA, mod ve seedler
aynıdır. Şok öncesindeki kaydedilmiş nüfus/enerji örnekleri sekiz çiftte de aynıdır.
Bu tam durum eşitliği testi değildir; kayıtlı örneklerin eşitliğidir.
Kontrol koşulları `canary-n4-steady-q885`, şok koşulları `resource-shock` altındadır.
Her ikisi de fixed moddur; V10 öğrenme değişikliğinin etkisi bu veride yoktur.

Şok 10.000. adımda başlar, 11.000. adımda eski yenilenme geri gelir. İkisi de
aynı anda azaltılır: 784 besin alanında 2→1/adım, 192 su alanında 4→2/adım.
Tamsayı yuvarlamasının şoku beklenenden sertleştirdiği ihtimali kontrol edildi;
bu senaryoda iki kaynak da gerçekten %50 azalır. Stoklar doğrudan silinmez,
kapasite değişmez, dünyaya canlı eklenmez. Yalnız besin mi, yalnız su mu daha
etkiliydi sorusu mevcut birleşik müdahaleden ayrılamaz.

## Nüfus akışı

Hücrelerde avcı/otçul sayısı. 9.900 şok öncesi, 10.100 şok sırası,
11.000 yenilenmenin geri geldiği örnektir. Sonuç satırındaki süreye kadardır.

| Seed | 9.900 | 10.100 | 11.000 | Son durum | Son adım |
|---|---|---|---|---|---:|
| 1 | 53/524 | 51/141 | 2/326 | 0/687 | 11.163 |
| 2 | 53/525 | 50/136 | 10/302 | 54/579 | 20.000 |
| 3 | 66/489 | 61/151 | 1/299 | 0/411 | 11.041 |
| 4 | 48/574 | 48/168 | 7/318 | 0/657 | 13.941 |
| 5 | 47/551 | 43/140 | 7/316 | 37/581 | 20.000 |
| 42 | 45/507 | 41/141 | 7/307 | 43/505 | 20.000 |
| 12345 | 47/480 | 40/159 | 3/306 | 0/648 | 12.521 |
| 2026 | 49/519 | 46/149 | 10/304 | 46/524 | 20.000 |

Otçul düşüşü bütün seedlerde var. Şok geçince otçullar toparlanabiliyor;
avcıların toparlanması başlangıç koşuluna bağlı. Tek başına nüfusun 7'ye
inmesi kesin yok oluş değildir: 4 ve 5 aynı 11.000 örneğinde 7 avcıya sahipken
4 daha sonra tükeniyor, 5 toparlanıyor. Başarılı seed 5 örneklemde en az 3
avcıya kadar inmiş; dolayısıyla düz bir asgari nüfus eşiği açıklaması yetersiz.

## Son ölümler ve rezervler

Başarısız dört koşunun her birinde son 10 avcı ölümü tutulmuştur. Bu 40 ölümün
37'si OldAge, 3'ü Starvation'dır; açlık ölümlerinin üçü de seed 3'tedir.
Dört koşunun son avcısı OldAge ile ölür. Bunlar yalnız son kuyruk sayılarıdır;
şok boyunca yaşanan bütün ölümlerin dağılımı değildir.

Son karar anları, ölüm sonrası sıfırlanan sağlıkla karıştırılmadan incelendi:

- Seed 1: son avcı 213/270 enerji, 54/70 su ve 140 sağlıkla Rest kararı veriyor.
- Seed 3: son avcı 201/270 enerji, 51/70 su ve 140 sağlıkla SeekWater kararı veriyor.
- Seed 4: son avcı 215/270 enerji, 50/70 su ve 140 sağlıkla SeekWater kararı veriyor.
- Seed 12345: son avcı 254/270 enerji, 52/70 su ve 140 sağlıkla NoEligibleMate kararı veriyor.

Dördünde de yaş 1200; azami ömrün sonundalar. Terminal ölüm nedeninin OldAge
olması önceki kaynak sıkıntısının etkisiz olduğu anlamına gelmez. Yeni bireylerin
azlığı ve yaş dağılımı hakkında ek kayıt olmadan 'şokta hiç doğum olmadı' denemez.

## Politika ile fizyoloji arasındaki dar pencere

Bu profilin üreme enerji eşiği `floor(270 × 885 / 1000) = 238`.
Sağlıklı bireyin beslenmeyi önceliklendirme eşiği ise `floor(270 × 3 / 4) = 202`.
Dolayısıyla 202–237 enerjideki yetişkin ne açlık gerekçesiyle rezerv yükseltmeye
ne de fizyolojik olarak üremeye yöneliyor. Büyük lokma eşiği aşarsa üremeye
hazır olabilir; fakat enerji yeniden düşer. Bu kodda görülen gerçek bir politika
uyumsuzluğudur. Tek başına bütün yok oluşu açıklaması henüz doğrulanmadı.

Başarısız dört koşunun son 10 ölüm × son 10 karar kuyruklarında 400 karar var.
Bunların 55'inde birey üremeye hazırdır; hiçbirinde görünür ve üremeye hazır
türdeş yoktur. Bu görünmeyen veya henüz hazır olmayan eş bulunmadığı anlamına
gelmez. Üstelik başarılı koşulların son ölüm kuyruklarında da aynı yerel eş
sayısı sıfır olabiliyor: bu istatistik tek başına başarısızlığı ayırt etmez.
Önemli farkın tüm daralma dönemi boyunca hazır çiftlerin buluşması ve yavru
üretmesi olup olmadığını ölçmek gerekir.

## Sonraki sınırlı deney

Yeni genel tarama veya rastgele profil ayarı yerine aynı sekiz eğitim seedinde,
aynı başlangıç ve şokla aşağıdaki kayıtlar eklenmeli:

1. Her 100 adımlık aralıkta doğumlar, ölüm nedenleri ve yetişkin/yavru sayısı.
2. Avcıların yaş, enerji eşiği ve üreme bekleme dağılımı; hazır eş çiftlerinin
   mesafesi, hazır kalma süresi ve gerçekleşen çiftleşme. Eş uzaklığı tanı amacıyla
   motor dışı ölçülür; canlı politikasına dünya bilgisi olarak verilmez.
3. Besin ve su şokunu ayrı uygulayarak daralmanın hangi kaynaktan geldiğini ayırma.
4. İki davranış hipotezini ayrı kollarla karşılaştırma: yetişkinin üreme rezervini
   bilinçli tamamlaması ve görülen türdeş bilgisinin daha tutarlı takip edilmesi.
   İkisini birlikte değiştirip neden düzeldiğini belirsiz bırakmama.

Daha fazla rezerv için avlanmak otçullara baskıyı artırabilir. Bu nedenle yalnız
avcı hayatta kaldı diye kabul edilmez: iki türün devamı, edinilen besin/harcanan
enerji, yavru doğumu ve yerel kararlar birlikte incelenir. Dünya nüfusa bağlı
kaynak üretmez, eş ışınlamaz ve gizlice yeni canlı eklemez. V10 su öğrenmesini
bu sabit kural teşhisine karıştırmadan ayrı tedavi olarak ölçmek gerekir.

Mevcut 30 seedlik uzun küme artık görülmüştür. Yeni davranışın son kabulü için
aynı kümeye tekrar tekrar uyum sağlamak yerine yeni bir görülmemiş küme ayrılmalı.
Bu incelemede yeni koşu veya yeni davranış başlatılmadı.

Makinece okunur bulgular ve girdi SHA'ları `RESOURCE_SHOCK_ANALYSIS.json` içinde.
Ham veriler korunmuştur. 100 adımlık örnekler ara nüfus diplerini kaçırabilir;
son 10 ölüm kaydı tam soy/üreme muhasebesi değildir.
