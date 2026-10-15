# Av–avcı planı: teşhis, ilk düzeltmeler ve sınanacak adaylar

Kalıcı denge **henüz sağlanmış kabul edilmiyor**. Mevcut dünyalara müdahale
edilmedi. Kaynak V9; çalışan laboratuvar V8, gerçek süreli çayır V7 olarak kaldı.

## Kanıt ve değişiklik

V8'in seed12345 ve seed1 koşuları tam SQLite geçmişiyle yeniden üretildi.
Son avcı soyundaki av daralması ile ilk yavrunun üremeye hazır olamaması ayrı
zincirlerdir. Ayrıntı COEXISTENCE_BASELINE.md ve JSON'unda.

- Ciddi yaralanmada (%50 sağlık ve altı) beslenme hedefi %90'a yükselir.
  %75'te beslenmeyi bırakıp yine %75'te bedelli iyileşme isteme çatışması
  azaltılır. Hafif yaralıların sürekli fazladan avlanmasını teşvik etmek için
  bütün yaralılara uygulanmaz. Bazal tüketim ve iyileşme bedeli korunur.
- Kaynak seçimi yenebilecek gerçek miktarı, yol enerjisini ve gerekli tickleri
  birlikte karşılaştırır. Uzak dolu yığın her zaman yakın öğünden iyi değildir.
- Yalnız görülen yetişkin türdeşlerin son konumu hatırlanır: en fazla8, ömür30
  tick. Görülen eski konumda birey yoksa unutulur. `RecallMate` gerçek karar
  dalıdır; dünyadaki görünmeyen eşlerin konumuna erişim yoktur. Yeni doğan bilgi
  devralmaz. `companions_enabled` bağımsız karşılaştırma sağlar.
- Bedenin `max_meal`, `reproduction_interval`, `reproduction_reserve_per_mille`
  alanları ayrıldı. Eski kayıtlar sırasıyla10/25/500 alır. Yerleşik türlerin bu
  varsayılanları değiştirilmedi. Öğün sonlu kaynağı ve kapasiteyi aşamaz; üreme
  hâlâ her ebeveynin kapasitesinin dörtte birini yavruya aktarır.
- UI, hatırlanan türdeşleri, öğün sınırını, üreme rezervini ve beklemeyi gösterir.

Checkpoint V9, V1–V8'i eklemeli alan varsayılanlarıyla okur. Eski motor V9'u
açamaz. Aynı V9 ikili/seed/komut deterministik devam eder; V8 ile aynı gelecek
vaat edilmez. Şema SQLite V2'dir. Çalışan dünyalar yükseltilmedi.

## Denenenler ve sınırlar

COEXISTENCE_EXPLORATION.json, **262 seçili tarama koşusunu** içerir. Bunlar
geliştirme sırasında farklı ara politika sürümleriyle yapılmıştır; tek sürümün
262 tekrarı veya birleştirilebilir başarı oranı değildir. Ham veriler
data/experiments/coexistence-* dizinlerinde korunur.

Ceset ömrünü5'ten20'ye uzatma tek başına çözmedi; kaynakta5 korundu. Yalnız
kaynak artırma, daha büyük öğün, uzun ömür veya büyük sığınak da çözmedi.
Üreme yavaşlatıldığında kısa ömrün soy yenilenmesini engellemesi ayrıca
sınandı. Deneysel beden değerleri global tür varsayılanı yapılmadı.

Yavaş üreyen/yüksek rezervli etçil ve dört bağlantılı habitatın seed1'i20.000
tickte sürdü:5 etçil/155 otçul, etçil65. nesil; son2.000 tickte11 etçil doğumu
ve yavru kuşaklardan15 üreyen birey. **Aynı koşul seed2'de16.564, seed3'te12.799,
seed4'te9.558'de avcı kaybetti.** Dokuz ve16 bölgeye büyütmek de seed2'de
sırasıyla19.175 ve9.358'de avcı kaybını önlemedi. Tek iyi koşu başarı değildir.

Yüksek üreme eşiği soy yenilenmesini azaltırken daha düşük eşikte avcı artışı
avı tüketebiliyor. Bundan sonraki sınırlı tarama bu iki başarısızlığın arasını
ve başlangıç rezervi/üreme fazını ayırır. Sonuçlar düzeltme gerektiren yeni bir
mekanizmayı gösterebilir; bir aday bulunacağı garanti edilmez.

## Koşullu, tekrar başlatılabilir deney akışı

`prepare_coexistence_candidates.py`36 adayı **önceden** tanımlar:

- 2×2,3×3,4×4 bağlantılı habitat; her bölgede aynı36 otçul/2 etçil ve aynı
  nüfustan bağımsız kaynaklar. Kaynak üretimi artırılmaz.
- Etçil bedeninde1200 ömür,270 enerji kapasitesi,60 öğün,350 üreme aralığı.
  Bunlar açık deney koşullarıdır; yüksek kapasite yavru yatırımını da artırır.
- Üreme rezervi875/880/885/890/895/900 permil. Sıra dosyada sabittir.
- Yaşları dağılmış/full rezervli başlangıç ile ayrıca rezervi%60–100 arasında,
  üreme bekleme fazı yaşına göre dağılmış başlangıç karşılaştırılır. Yavru
  kuruculara geçmiş üreme beklemesi verilmez. Sonradan canlı eklenmez.

`research_coexistence.py` adayları yalnız araştırma seedleriyle seçer:

1. Seed2/5 ×5.000 tick ön tarama; geçen adaya8 seed ×20.000 tick.
2. Aynı adayın enerji kapasitesi ve üreme aralığında ayrı±%10,4 seed ×20.000.
   Rezerv yüzdesi korunur; kapasite değişince bütün kurucular bedelsiz doldurulmaz.
3. İlk dayanıklı aday için **10001–10030 ×100.000 tick**, en az27 geçiş.
4. Araştırma seedlerinde20.000 tick;10.000'de kaynak yenilenmesi1.000 tick
   boyunca%50 azalır, sonra eski coğrafi kurala döner. Nüfusa göre yardım yoktur.
5. Öğrenme/mutasyon ayrı ve birlikte8 seed ×20.000; ardından birlikte aynı30
   seed ×100.000. Kaynak/parametre bu testlerin sonuçlarına göre değiştirilmez.
6. Bütün kapılar geçerse ayrı7882 servisi; doğrulanan seedlerden ilkiyle yeni
   dünya. Önceki7878–7881 dünyaları korunur. Gerçek1/7/30 günlük gözlem başlar.

Bir koşunun geçmesi, iki türün hiç tükenmemesi, sona en az ikişer bireyle
ulaşması, son%10'da iki türde doğum ve en az ikişer yavru kuşak bireyinin üremesi
demektir. Başlangıçtaki uzun ömürlü bireylerin kalması yetmez.

Ön tarama adayları tükenirse veya seçilen aday sonraki bir kapıda kalırsa akış
durur ve raporlar. Görülmemiş seedler kalibrasyon için kullanılmaz. Başarısız
uzun testten sonra aynı seedlere bakarak yeni aday seçilip yeniden "görülmemiş"
denmez. Başarı eşiği sonuçtan sonra düşürülmez.

## İşletim

Kaynak ayrı, temiz ve sabit Git worktree'sinden çalışır. İkililer ve senaryolar
SHA256 ile bağlanır; her koşu kendi değişmez kopyasını saklar. Seed sonuçları
atomik yazılır; resume farklı sürüm/ikili/koşulu reddeder. İki süreç, düşük
öncelik, toplam%180 CPU ve512MiB sınırı kullanılır. Uzun doğrulama günler
sürebilir. Ara ilerleme seed loglarında, kapı sonuçları JSON'dadır.

Notify kalıcı outbox, sabit anahtar, rapor sayfası ve ayrı retry timer'ı kullanır.
İzleyici araştırmanın heartbeat'ini ve bildirim kuyruğunu bağımsız denetler.
Program arka planda yeni kod yazmaz; yalnız tanımlı deneyleri ve koşullu ayrı
dağıtımı yürütür. Sonraki kod kararı için sonuçların ajan tarafından incelenmesi
gerekir. Başarısızlığı otomatik canlı ekleyerek veya yeni ayarla saklamaz.

Yeni dünyada tek tür kaybı da uyarılır; toplam canlı sayısının sıfırlanması
beklenmez. Kalıcı kayıt bütçesi32GiB, boş disk sonsuz değildir. Gerçek günlük
örnekler miktarı ölçmeden aylarca depolama garantisi verilmez. Bütçede pause
ve bildirim korunur; geçmiş sessizce silinmez.

Doğrulama:45 Rust testi,21 Python testi, HTTP kontrol/auth/SIGKILL/arşiv testi.
Bu sonuçlar biyolojik denge kabulü değildir. Yeni araştırma süreci ve sayfa
kimliği çalıştırma sonrası AGENTS.md'ye eklenir.
