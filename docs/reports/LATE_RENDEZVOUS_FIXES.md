# Geç kopya: dolu sığınak ve kaçırılan görünür eş

Geç17.400 kopyası tam bireysel algı ve eylem sonuçlarıyla yeniden oynatıldı.
Gözlem ayarı dışında tam durum+RNG eski koşuyla eşit; bütün100 adımlık çıktı
satırları da birebir eşit. Gözlem kaynaklı başka bir dünya incelenmedi.
İnceleme dünya/ömrü/kaynakları değiştirmedi, canlı servislere dağıtım yapılmadı.

## Somut bulgular

5.283 otçul kararının1.109'u sığınma.486 kararda en yakın sığınak bilinen bir
canlı tarafından işgal edilmiş;476'sında algıda boş alternatif de var.442
sığınma reddinde seçilen sığınak bilinen dolu hedef. Algı, canlıların işgal
ettiği uç hücreleri onları görmek/etkileşmek için erişilebilir uç olarak taşır;
politika bunu **içine girilebilir sığınak** diye kullanmış. Bu, gelecekteki
çakışmayı tahmin edememe değil; o an bilinen doluluğu göz ardı etmedir.

Örnek17.401, #91937: (17,11)'den dolu (17,10)'a adım niyeti NoPath ile
reddediliyor. Görüşünde altı adım aşağıda boş sığınak var. Diğer iki örnek
ve bütün sayaçlar JSON raporundadır.

Bir ikinci hata, buluşma yeniden deneme arasının yeni ve görünür gerçek eş
fırsatına da uygulanması.330 fizyolojik hazır beklemede hazır eş görünür;
66'sında mevcut bilgiyle bedeli ödenebilir, tehlike dışı ilk adım var. Bunların
**57'si yeniden deneme aralığının içinde**. Geri kalan9 kararın hepsi bu
aralık yüzünden engellendi denmez; yaş/yolculuk bütçesi gibi başka sınırlar da
var. Güvenli ilk adım tüm yolun veya çiftleşmenin güvenli olduğunu kanıtlamaz.

Bireylerin post-karar hedefi varken467 besin,286 sığınma,226 su kararı var.
Kesintiler gerçek; fakat hepsi hata sayılmıyor. Açlık, susuzluk ve tehdit
önceliklerini kaldırmak yerine iki doğrulanmış karar boşluğu düzeltildi.

## Düzeltmeler

1. Otçulun sığınma politikası, gördüğü canlıların konumlarıyla dolu olduğunu
   bildiği sığınakları eler. Yol dizileri farklı olsa da uç koordinat karşılaştırılır.
   Boş sığınak kalmadığında olağan kaçış/savunma kararı kullanılır. Hareket,
   çakışma ve enerji doğrulaması fiziksel çözümleyicide kalır. Bu temel karar
   düzeltmesi kaynakta **buluşma bayrağından bağımsızdır**; sadece deneysel
   buluşmanın açılmasıyla çalışan bir hile değildir. Canlı eski motorlar korunur.
2. Deneysel buluşmada yeniden deneme arası, hâlihazırda görülen hazır eşe
   güvenli/ödenebilir tek adım yaklaşmayı engellemez. Yeni hedef açılmaz,
   hedefin30 adımlık süresi veya10 adımlık tekrar arası sıfırlanmaz. Bilinmeyen
   eş araması hâlâ süreli ve sınırlıdır; tehlike ve fiziksel üreme şartları korunur.
3. Gerçekten türdeş yanında bekleme WaitForMate; engelli/ertelenen arama
   PauseMateSearch olur. Önceden aynı gerekçe, türdeş yanında olunmadığı halde
   “yanında bekliyor” diye gösterilebiliyordu. Arayüz açıklamaları ayrıldı.

Checkpoint V12 / SQLite V2; yeni kayıt alanı yok. Buluşma bayrağı false kalır.
Karar düzeltmesi yeni motorun geleceğini değiştirdiğinden eski sürümün uzun
kabulü bu kaynağa taşınmaz. Fiziksel eylem/maliyet veya yaşam süresi değişmedi.

## Aynı başlangıçtan ikiye iki karşılaştırma

Dört varyant aynı17.400 kaydı, fixed modu, açık rezerv ve buluşmayla20.000'e
oynatıldı. Eski kontrol önceki çalışmadan; denetimli tekrar bunun bütün çıktısını
yeniden doğruladı. Araştırma varyantları uncommitted patch olarak açık etiketli,
ikili/kaynak SHA'ları ve patch dosyalarıyla saklıdır. Son kaynak birleşik
varyantın bütün motor/örnek dosya hash'leriyle birebir eşleşir. Bunlar ekolojik
kabul veya canlı dağıtım ikilisi değildir.

| Düzeltme | Son otçul | Son avcı | Sonuç |
|---|---:|---:|---|
| Yok |0|0|Önceki başarısızlık|
| Yalnız eş fırsatı |0|0|60 otçul doğumu; toparlanma yok|
| Yalnız boş sığınak seçimi |650|0|Otçul toparlandı, avcı soyu bitti|
| İkisi birlikte |646|0|Otçul toparlandı, avcı soyu bitti|

Birleşik geç kolda4 avcı doğumu /45 ölüm var:25 açlık,15 yaşlılık,5 susuzluk.
Bu yüzden “bütün avcılar açlıktan öldü” denmez. Otçulun gerçek kaçışı düzeldikten
sonra avcıların beslenme/üreme başarısı nesli yenilemiyor. Sırf eski sayısal
dengeyi korumak için bilinen dolu sığınağa koşma hatası geri konmaz; dengenin
bu başarısız kaçıştan ne kadar yararlandığı ayrıca değerlendirilmelidir.

## Erken kopyada gerileme kontrolü

Birleşik düzeltme16.000'deki aynı geçmişten20.000'e oynatıldı: **597 otçul /45
avcı**. Son400 adımda her iki tür de yavru üretiyor; kesin sayılar JSON'da.
Böylece önceki erken kopya kazanımı bu kısa ufukta kaybolmadı. Bu, sıfırdan
başlayan seed2, diğer seedler veya kaynak şoku kabulü değildir. Geç müdahale
kolunda avcı soyu hâlâ bittiği için yeni çoklu uzun tarama başlatılmadı.

## Kanıt ve doğrulama

- `scripts/analyze_rendezvous_audit.py AUDIT --output YENI_JSON`: tam algılı
  kaydı akışla okur; eksik algıyla güvenli yol iddiası üretmeyi reddeder.
- `collapse_branch CHECKPOINT TICKS rendezvous YENI_STATE [YENI_AUDIT]`:
  isteğe bağlı audit tam algıyı açar. Bu gözlem ayarı duruma yazılır; yalnız
  bu alan normalize edilerek nötrlük doğrulanır. Audit açık yazımı ayrıca disk
  tüketir; bu incelemenin çıktısı36MiB, ham deney verisi olarak korunur.
- Ham veriler `data/experiments/rendezvous-late-audit` ve
  `data/experiments/rendezvous-late-corrections`; eski deneyler değişmedi.
- 76 Rust ve33 Python testi geçti. Yeni testler dolu sığınak/boş alternatif,
  farklı yol dizisiyle aynı uç, boş sığınak yokken kaçış, gerçek dünyada boş
  sığınağa2 enerji maliyetiyle erişim ve yeniden denemede güvenli eş fırsatını
  kapsar. HTTP/auth/kilit/SIGKILL kurtarma ve JS sözdizimi de doğrulandı.

Sıradaki biyolojik inceleme, düzeltilmiş geç kolda avcıların hangi besini
algılayıp erişemediği ve üreme rezervini neden yenileyemediğidir. Otçulun
kaçışını bilerek bozma, gizli takviye veya salt popülasyona göre dünya müdahalesi
çözüm değildir. Bu rapor tamamlanan iki karar düzeltmesini, açık kalan avcı
soy dengesinden ayrı tutar.
