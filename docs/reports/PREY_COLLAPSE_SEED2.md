# Seed 2: mature rezerv adayının normal rejimde otçul çöküşü

Bu inceleme bir düzeltme/kabul koşusu değildir. Canlı dünyalar, eski deney
ikili/verileri ve varsayılan kapalı politika değiştirilmedi. Aday `c4d2b35`,
önceki aday `d36a10a`; aynı senaryo, fixed öğrenme/mutasyon kapalı, türdeş
hafızası açık, rezerv politikası açık ve kaynak şoku yok.

## Kayıtlı sonuç ve nüfus muhasebesi

Yeni aday hedefli şokta seed 2/3 için 2/2 geçti, normal sekizlide 7/8 kaldı.
Seed 2 otçulları 17.660 adımda tükendi; 5 avcı kaldı. Sekizli şok başlatılmadı.
Önceki aday aynı normal seed'i 20.000'e götürdü: 524 otçul/46 avcı.

Aşağıdaki aralıklar başlangıç adımından **sonra**, bitiş dahil olaylardır.
Her satırda önceki nüfus + doğum − bütün ölümler = sonraki nüfus doğrulanır.

| Aralık | Otçul önce → sonra | Doğum | Avlanma ölümü | Yaşlılık | Açlık | Susuzluk |
|---|---:|---:|---:|---:|---:|---:|
| 16.000–16.800 | 485 → 366 | 4.274 | 953 | 3.291 | 91 | 58 |
| 16.800–17.000 | 366 → 274 | 732 | 255 | 551 | 10 | 8 |
| 17.000–17.200 | 274 → 175 | 512 | 241 | 361 | 6 | 3 |
| 17.200–17.400 | 175 → 78 | 356 | 211 | 242 | 0 | 0 |
| 17.400–17.600 | 78 → 2 | 64 | 70 | 70 | 0 | 0 |
| 17.600–17.660 | 2 → 0 | 0 | 0 | 2 | 0 | 0 |

Son bölüm otçul açlığı değildir. Doğum/ölüm dengesi bozulur; avcı nüfusu
arkadan çöker. 17.400–17.600'de avcı doğumu 0, avcı açlık ölümü 19'dur.
Bu aşamada avcı açlığı otçul kaybını izler; başlangıç sebebi diye sunulamaz.

Sadece “çok fazla avcı” açıklaması yeterli değil: yeni adayın sağ kalan diğer
seedlerinde örneklenen avcı maksimumu 66–71 iken seed 2'de 66. Tüm maksimumlar
100 adımlık örneklerdir, mutlak tepe iddiası değildir. 17.200–17.600'de kaynak
kıtlığından ölüm olmaması da daha önceki fizyolojik etkileri dışlamaz.

## Kaynaklar ve tekrarlanabilirlik

`data/experiments/seed2-collapse-diagnosis/comparison.json`, sekiz seed için
iki manifesti/senaryo SHA'sını doğrular; doğum/ölüm farklarını ve kaynak JSON
SHA'larını içerir. Üretim komutu:

```sh
python3 scripts/analyze_prey_collapse.py --output YENI_SONUC.json
```

Ayrıntılı tekrar ayrı dizinde, aynı seed ve ayarla çalışır. Manifest bütün
Rust kaynak hash'lerini, ikili ve senaryo hash'ini içerir. Gözlemci 16.000'den
itibaren karar/üreme engelleri, kaynak stoku, son bireyler ve son 64 otçul ölümü
kaydeder. 16.000/16.800/17.000/17.400/17.600 kontrol noktaları sonraki teşhislerin
baştan çalıştırılmasını önler. Gözlemci RNG kullanmaz/dünyayı değiştirmez;
regresyon testi her adımda gözlem açık/kapalı tam durum+RNG+olay eşitliğini sınar.

```sh
PROBE SCENARIO 20000 2 fixed --reserve-opportunities --stop-on-extinction \
  --progress-every=1000 --diagnostic-dir=YENI_DIZIN --diagnostic-from=16000
```

Kontrol noktası karşı-olguları yalnız kopyada çalıştırılır:
`collapse_branch KONTROL_NOKTASI ADIM unchanged|reserve-off|without-predators`.
`without-predators` yapay müdahaledir; bir çözüm veya ekolojik kabul değildir.
Öğrenme ve mutasyon kapalı bu protokole özeldir; genel checkpoint oynatıcısı değildir.

## Ayrıntılı tekrar sonucu

Tekrar 591,4 saniyede aynı 17.660 yok oluşuna ulaştı. Yalnız dosya yolu ve
süre alanları hariç bütün eski probe çıktısı birebir eşleşti: nüfus/enerji
örnekleri, bütün doğum/ölüm ve karar sayaçları, son avcı ölüm öyküleri dahil.
Ayrıca 16.000 kontrol noktasından 1.000 adım değiştirilmeden oynatılan
kopyanın **tam durum ve RNG'si**, asıl tekrarın 17.000 kaydıyla birebir eşleşti.

16.200'de 16 harita bölgesinde 566 otçul var. 16.500'de (3,3) bölgesinde
2, 16.800'de 0 kalıyor. 17.000'de birkaç bölgede yalnız birer birey var;
17.600'de iki birey iki ayrı bölgede. Bölgeler kapalı ada değildir; burada
14 hücrelik senaryo yerleşiminin koordinat kutuları sayılır. Bazı kutulara
sonradan gezginler de gelir; bu bölge başına kalıcı yok oluş iddiası değildir.

Toplam bitki stoku aynı süreçte 7.266→15.666 artıyor. Bu, yerel erişim ve
tehlike maliyetini dışlamaz; **dünyada yiyecek kalmadı** açıklamasını dışlar.
Son 64 otçul ölümünün 34'ü yaşlılık, 30'u avlanma; avlananların 15'i henüz
olgunlaşmamış. Bu son örneklemdir, bütün koşunun yaş dağılımı değildir.

17.600'de son iki birey:

- #92153: (38,24), yaş72, sağlık106/120, enerji59/80, su53/60.
- #92156: (31,38), yaş46, sağlık89/120, enerji59/80, su54/60.

İkisinin de bekleme süresi0, dayanıklılığı yeterli; ikisi de fizyolojik üreme
şartlarını sağlıyor. Görüş6, aralarındaki en az geometrik hareket14 hücre;
her ikisinin o anki son algısında **hiç birey yok**. Sığınaktalar, kaynakları
var; son bölümdeki 92 birey-kararının tamamında fizyolojik üreme engeli yok.
54 karar NoEligibleMate; kalanlar besin/su/sığınak. Doğum0. #92153 17.634'te,
#92156 17.660'ta yaş106 ile ölür; son kararlarında da eş görünmez. Son anlık
sağlıkları120, enerjileri61/58, suları50/45'tir. Son neden yaşlılık,
soyun devam etmeme mekanizması ise mevcut yerel eş aramasının ayrılmış son
bireyleri kalan ömürlerinde buluşturamamasıdır.

## Kopya dünya karşı-olguları

| Başlangıç / müdahale | Son adım | Otçul | Avcı |
|---|---:|---:|---:|
| Asıl aday, değişiklik yok | 17.660 | 0 | 5 |
| Aynı 16.000 kaydı, rezerv hazırlığı kapalı | 17.660 | 535 | 44 |
| Aynı 17.400 kaydı, avcılar kopyadan çıkarıldı | 17.700 | 606 | 0 |
| Aynı 17.600 kaydı, avcılar kopyadan çıkarıldı | 17.800 | 0 | 0 |

İlk karşı-olgu aynı başlangıçtan yalnız politika bayrağını değiştirir.
Eski d36a10a kuralının tam tekrarı değildir: rezerv hazırlığının tamamını
kapatır. Sonraki davranış ve RNG kullanımı doğal olarak ayrışır; farklı
gelecekler aynı rastgele olayları birebir yaşamış gibi karşılaştırılamaz.

Avcıyı çıkarmak yapay bir teşhis müdahalesidir; canlı dünyaya yapılmadı ve
çözüm önerisi değildir. 17.400'deki grubun hâlâ toparlanabilecek beden/üreme
kapasitesi olduğunu; 17.600'de yalnız mevcut av baskısını kaldırmanın bu
iki bireyi kurtarmadığını gösterir. Kısa ufuk tek seed içindir, kalıcı
birlikte yaşam veya kaynak şokuna dayanıklılık kabulü değildir.

## Teşhis ve sonraki düzeltmenin sınırı

Yeni mature hazırlığı mantıksal bir karşılıklı beklemeyi kaldırıyor, fakat
bu davranışın ekosistem dengesi güvenli değil. Normal rejimde av baskısı ve
doğumların zamanlaması değişerek yerel gruplar küçülüyor; dağılma/eş bulamama
küçülmeyi pekiştiriyor. Avcıların uzun ömrü nedeniyle av baskısının azalması
geriden geliyor. Otçulun kalan üreme fırsatı bitince avcı açlığı da artıyor.
Bu zincirin bütün ara etkileri tek bir parametreye indirgenemez; yalnız bir
"66 avcı sınırı" koymak gözlemlerle desteklenmiyor.

Yeni aday otomatik kabul edilmez, varsayılan false kalır. Eski kaynak şoku
hatası bu teşhisle çözülmüş değildir. Sonraki değişiklik için öncelik, küçük
ve dağılmış otçul gruplarının **yerel bilgiyle, tehlikeyi ve kalan zamanı
hesaba katarak eş aramasını** ayrı küçük senaryolarda sınamak; ardından bu
kayıtlı kontrol noktalarıyla doğum/ölüm dengesini ölçmektir. Ekosistemi bilen
küresel nüfus tanrısı, gizli takviye veya avcıyı otomatik silme eklenmez.
Sadece eş aramasını değiştirmenin tek başına dengeyi sağlayacağı da henüz
kanıtlanmış değildir. Önce dar mekanizma testi, sonra normal/şok ve bağımsız
seed kapıları gerekir; yeni uzun tarama bu teşhiste başlatılmadı.

Kanıt: `PREY_COLLAPSE_COMPARISON.json`, `PREY_COLLAPSE_REPLAY.json`.
Ham veriler/ikili/kontrol noktaları `data/experiments/seed2-collapse-diagnosis`.
60 Rust testi (gözlemci nötrlüğü dahil), 29 Python testi ve gerçek checkpoint
tekrar/karşı-olgu koşuları geçti. Fizik, karar politikası, checkpoint ve
SQLite şeması değiştirilmedi.
