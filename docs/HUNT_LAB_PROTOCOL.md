# Av laboratuvarı v1 — sonuç görülmeden sabitlenen protokol

Bu deney mevcut V3 av-adayı öğrenicisini ölçer; ödül, keşif (%10), bölüm sınırı
(20 tick) veya ekosistem parametreleri sonuçlara göre değiştirilmez. Laboratuvar
her denemede bedenleri/konumu/yaşı ve öğrenme dışı hafızayı sıfırlar. Bu bir
hayatta kalma, genel zekâ, evrim veya ekosistem kabul deneyi değildir.

## Kollar, seedler ve süre

Eğitim/evaluation çiftleri: (1,41001),(2,41002),(3,41003),(4,41004),
(5,41005),(42,41006),(12345,41007),(2026,41008). Değerlendirme seedleri bu
protokolde eğitime girmez; projenin hiçbir yerinde kullanılmadıkları iddia edilmez.
İki ayrı iz: passive ve defensive. Her iz/çiftte 80 eğitim denemesi, ardından
aynı değerlendirme başlangıç seed çizelgesinde dört ayrı kol×80 deneme vardır:
kurallı, boş/online, eğitimli/online, aynı eğitimli tablo/frozen. Eğitimli tablo
aynıdır; frozen'da seçim/keşif sürer, update_enabled=false olur. İlk40 evaluation
normal, son40 ters koşuldur. Her deneme en çok20 fiziksel tick. Üst sınır
2×8×(80+4×80)×20=128000tick; sonsuz koşu/parametre taraması yoktur.

Bir trial'ın world seed'i Random(seed)'den next_u64 ile önceden üretilir;
dört kol aynı trial için aynı world seed'i alır. Kollarda karar sayıları ve
RNG çağrılarının aynı olması şart değildir; kaynak ayrımı açıkça kaydedilir.

## Fiziksel düzen

Boş 13×13 harita (0..12), kaynak yenilenmesi/genetik kapalı; üç gerçek beden:
AI Carnivore #1 (2,2), yakın Herbivore #2 (3,2), uzak Herbivore #3 (2,5).
Avcı standart beden, energy40, water=max, speed1; yaş0. Yakın av health20,
uzak av health60. Her iki avın max_energy300/energy60, max_stamina64/stamina64,
speed2; diğer alanlar varsayılan. Bu açık beden ayarları enerji/heal/fatigue
karışıklığını azaltmak için sabittir, kollara farklı uygulanmaz. Su öğrenmesi
bütün aktörlerde kapalı; avcının diğer hafızası her trial yeniden başlar.

Normal koşulda yakın av kaçar, uzak av bekler; ters koşulda roller değişir.
Kaçan av iki gerçek Move adımı/tick ile başlangıcından sağa4/aşağı4/sola4/yukarı4
kare parkuru tekrarlar. Bu önceden yazılmış Player komutudur; trial içinde
ışınlama veya bedensel reset yoktur. Bekleyen av passive izde Idle kalır;
defensive izde avcı fiziksel olarak bitişikse gerçek Attack komutu verir.
Savunma bedeli/hasarı ve hareketler motorun normal çözümleyicisinden geçer.
Avcı hiçbir öğretmen etiketi, iz adı veya avın görünmeyen bedenini almaz.
İlk görüş/bedenler aynı trial'da dört kolda aynıdır; yalnız öğrenilmiş tablo
ve seçimi/güncellemeyi açan bayraklar farklıdır.

Trial ilk gerçek besin edinimi, avcının ölümü veya20tick ile biter. Sınırda
hala açık bir bölüm laboratuvar kesintisi olarak ayrıca puansız kapatılır;
bu doğal başarısızlık veya motor timeout'u diye sunulmaz. Tamamlanmış sonuçlar
HuntEpisodeEnded audit olaylarıyla alınır; son sonucu sürekli tekrar sayma ya
da aynı tickte hedef değişimi+ölüm gibi iki sonucu kaybetme yapılmaz.

## Önceden belirlenen ölçümler

Ham her trial: başlangıç beden/görünür avlar, seed, tick seçimleri/hedefleri,
resolver hareket/saldırı enerjisi, gerçek edinilen besin, gelen hasar, reddedilen
eylemler, ölüm/ilk besin zamanı, puanlanan/puanlanmayan bütün bölüm sonuçları,
tablo ziyaret değişimi. Toplam fiziksel enerji harcaması başlangıç enerjisi
+gerçek edinilen besin−son enerji ile ayrıca denetlenir; hareket/saldırı
maliyetleri bundan tekrar çıkarılıp toplamda iki kez sayılmaz. Kalan enerji
harcaması ayrı non_action_energy alanıdır; bu denklemi öğrenici ödülü kullanmaz.
Başarısız süre = besin almayan trial'da harcanan tick.
Bölüm end, updated/reward, belirsizlik, motor/lab-boundary kaynağı ayrıdır.
Interrupted sonuçlarında mevcut reason (susuzluk, stamina, hedef değişimi vb.)
ayrıca kaydedilir. Yalnız toplam ödül başarı ölçütü değildir.

Eğitim ilk/son40; evaluation normal ilk/son20, ters ilk/son20 pencereleri
raporlanır. Besin, enerji, zarar ve başarısız zaman için seed başına fark ve
sekizli ortalama verilir; frozen tablo ve visits değişmemelidir. Başlangıç
avantajı küçük, fayda sıfır veya olumsuz çıkabilir. Kabul/üstünlük için sonradan
bir eşik seçilmez. Görülmeyen nedenler kodun bildiği bilgiymiş gibi etiketlenmez.

İlk evaluation çifti için her kolun normal ilk trial'ı ve ters ilk trial'ı
SQLite'a da kaydedilir (iki izde toplam en çok16kısa kayıt dizini). Diğer trial'lar JSONL audit
olarak korunur. Örnek dizinleri mevcut salt okunur replay okuyucusuyla açılabilir.
Kaynak/ikili/protokol hash'leri ve temiz commit kimliği manifestte tutulur.
Canlı7878–7883 servisleri ve verileri bu deneyin parçası değildir.
