# Deneyim, sınırlı av öğrenmesi ve görünüm — 21 Ekim

Üç alt ajan ölçüm, av öğrenicisi ve ortak deneyim görünümünü hazırladı; ana
ajan resolver/checkpoint entegrasyonu, bağımsız test, deney ve incelemeyi yaptı.

## Kontrollü su deneyimi

Önceden sabitlenen EXPERIENCE_PROTOCOL.md: sekiz eğitim/değerlendirme çifti,
400 eğitim + dört kol ×400 değerlendirme; toplam16000 kontrollü tick.
Beden, yaş, su, kaynak ve konumlar açıkça sıfırlanır. Gerçek ekosistem veya
yaşlı hayvan üstünlüğü deneyi değildir. Deney sourcebac0b53 temiz commitinden
koştu; manifest/ham/özet SHA'ları EXPERIENCE_FIRST.json içindedir. Hamlar
`data/experiments/experience-v1-20261021` altında korunur.

Aşağıdaki oranlar100denemede tam10su edinme yüzdesinin sekiz seed ortalamasıdır:

| Kol | İlk100 | Değişimden sonraki ilk100 | Değişimden sonraki son100 |
|---|---:|---:|---:|
| Kurallı |50,00|100,00|100,00|
| Deneyimsiz, güncelleme açık |95,38|92,88|98,25|
| Deneyimli, güncelleme açık |96,38|92,88|98,25|
| Aynı deneyimli tablo, güncelleme kapalı |96,38|52,50|53,13|

Deneyimli bireyin başlangıç avantajı yalnız1yüzde puanı; güçlü/kalıcı bir
tecrübe avantajı diye sunulamaz. Çevre tersine döndüğünde güncellemeye devam
eden kol eski tercihinde kalan frozen kolu son pencerede45,125yüzde puanı
aşar. Frozen tablolar/değerler/visits bayt karşılığı olarak aynı kaldı.
Güncelleme açık kolun o pencerede hareket enerjisi7,25/100deneme, frozen187,25;
kurallı kol0. Kurallı tercih yeni koşulda zaten doğru, öğrenici ise keşif
bedeli öder ve%100'e ulaşmaz. Bu nedenle genel üstünlük iddiası yoktur.
Bütün ayrı seed farkları, su/maliyet/ret/kesinti/puanlanmışdeneme sayıları
JSON raporunda; yalnız ödül artışına bakılmadı. Konum tablosunun farklı
haritaya genellemesi veya yaşam/üreme faydası bu deneyde ölçülmedi.

## Av öğrenicisi: çalışan mekanizma, henüz kabul değil

Varsayılan kapalı HuntLearner108görünür bağlam ve ençok20tick bölümle sınırlı.
Seçim yalnız var olan HuntPrey içinde, fizikçe izinli adayları sıralar.
Yüksek öncelikli susuzluk/dışkomut korunur. SeedliRNG; yaş veya birey/koordinat
tablo anahtarı değildir. Saldırı enerji maliyeti ve ölümvuruşu resolver'dan,
yenen leşin kimliği gerçek Eat olayından gelir. Kendi öldürdüğü hedefin
besinine doğrulanmış kredi verilir; başka avcının başarısı, dış hasar veya
belirsiz sonuç puana çevrilmez. Yeni fiziksel maliyet/kaynak/bedenbonus yok.

V13 checkpoint aktif bölümü ve tabloyu saklar; eskiV1–V12 göçünde av öğrenmesi
boş/kapalıdır. Çocuklar mod bayraklarını sınırlı devralır, öğrenilmiş tabloyu
ve bölümleri devralmaz. Altı gerçekWorld testinde öldürme→beslenme, tam maliyet,
başkasının öldürmesine kredi vermeme, dışöncelik, gerçek doğum ve checkpointten
aynı olaylar/durum/RNG doğrulandı. Bunlar öğrenmenin ekolojik fayda kanıtı değildir.

Küçük tarihsel `scenarios/mixed-refuge.json` ile seed1/2×500tick kısa smoke:
kurallı kontrolde avcılar121/124, ilköğrenicide87/95adımda tükendi. İki kolda
da bu küçük senaryoda süreklilik yok; bu, büyük kabul senaryosunun testi veya
formal ret kapısı değildir. Olumsuz ham sonuçlar/checkpointler
`data/experiments/hunt-learning-smoke-20261021` altında korundu.

İncelemede ilk öğrenicinin puanlar eşitken enyakın avı seçerek temel
maliyet sırasını değiştirdiği bulundu. V2 eşitpuan durumunda aynı tahmini
maliyet→mesafe→kimlik sırasını korur; değerfarkı varsa öğrenilmiş sıralama
çalışır. Bu bir başlangıç politikası karışıklığı düzeltmesidir, sonuçtan
sonra biyolojik eşik veya beden ayarı değildir. Eski başarısız veri silinmez.
V2 tekrarda kontrol121/124, online82/109 ve boş/frozen82/109adımda avcı
tükenmesi verdi; olumsuz sonuçlar v2dizininde/immutableikiliyle korunur. Bu
görülen iki seedde bir teşhistir, bağımsız kabul değildir.

İkinci mekanizma karışıklığı, bölüm hedefini20tick kilitleyerek temel
politikanın her adımda hedefi yeniden değerlendirmesini engellemesiydi. V3
yalnız aday sıralaması sözleşmesine döner: her tick tekrar seçer, aynı hedefte
muhasebeyi korur, hedef değişiminde eski bölümü puansız kapatır/yeni maliyeti
sıfırdan başlatır. Yeni süre/epsilon/ödül veya biyolojik parametre ayarı yoktur.
Boş/frozen kontrol hem keşfi hem bölüm muhasebesini paylaşır. V3 kısa tekrarda kontrol121/124, online96/121 ve boş/frozen98/119adımda
avcı tükenmesi verdi. Sıralama dışındaki takip değişikliği kaldırıldı; bu iki
görülmüş seedde öğrenmenin faydası gösterilemedi. Başarılı diye sunulmaz.

Eğitimli/frozen/naive av deneyi, görülmemiş seedler
ve ayrı karma ekosistem kabulü hâlâ açık; canlı dünyada av öğrenmesi kapalıdır.

## Görünüm ve işletim

Ortak web/experience.js hem canlı UI hem replay'de bireyin tablosunu, sınırlı
puanlanmış denemelerini, bölüm maliyet/sonucunu ve kesilme/belirsizlik ayrımını
gösterir. Yaş/nesil öğrenme başarısı diye sunulmaz. Eski kayıtta bulunmayan av
verisi açıkça yok olarak gösterilir. GET-only replay ve kaynak bytehash
kontrolleri, hızlıseek yanıt sırası, shardlararası seçim ve masaüstü/mobil geçti.

Yalnız7883okuyucu sourcebac0b53'e geçirildi; ProtectSystem=strict ve veri
mountu salt okunur kaldı. GerçekV12kayıt deneyimi masaüstü/mobil görüntülendi.
7882karma dünya hâlâ7b157af/V12frozen motorla çalışır; biyoloji/ayarları ve
7878–7881eski dünyaları değiştirilmedi. V13motor adayrelease olarak korunur.
Yeni kapalı-öğrenici motorun100/200adım biyolojikState/RNG alanları eski
kabul motoruyla aynıdır; ek öğrenme alanları boş/kapalı olduğu ayrıca
kontrol edildi. Bu20.000/100.000adımlık yeni bir kabul testi değildir.

Son doğrulama:110Rust,58Python, deneyim presenter Node testleri, HTTP/auth/
writerlock/SIGKILL, gerçek Chromium canlı ve read-only replay desktop/mobile
kontrolleri başarılı. Yeni deneyim tablosu440px içinde kaydırılır;32konum
listesi bütün harita düzenini gereksiz büyütmez. Final aday varsayılan kapalı.
