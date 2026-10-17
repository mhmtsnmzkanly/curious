# Yerel buluşma: ilk deneysel aday

Amaç, küçülen otçul grubunda bilinen yetişkinin yanında kısa süre kalmak,
kaybolduğunda son gözlenen çevresinde sınırlı arama yapmak. Bu kurallı davranış;
yeni makine öğrenmesi veya türün yok oluşunu engelleyen dünya müdahalesi değildir.

Kaynak adayı `bba1495`, checkpoint V12 / SQLite V2. Ayrı `entity/social.rs`;
`policy.mate_rendezvous=false` varsayılanı. Rezerv bayrağı bağımsızdır.
V1–V11 kayıtlarında yeni davranış kapalı, hedef boş başlar. Canlı servisler,
eski deney kaynak/ikili/verileri değiştirilmedi. Kullanım ve sınırlar CONTROL.md V12.

## Davranış sözleşmesi

- Yalnız AI otçulu ve açık bireysel hafıza/türdeş hafızası. Konumlar gözlemden
  veya sınırlı mevcut türdeş anılarından gelir; küresel nüfus/konum okunmaz.
- Yetişkin türdeşin yanında hazır olmasını bekleyebilir. Mevcut hazır eş fırsatı,
  fiilî çiftleşme, açlık/susuzluk/tehlike ve dış/özel komut korunur.
- Tek hedef30 dünya adımı, ardından10 adım yeniden başlama arası; öncelikli
  gereksinim hedefin saatini dondurmaz.32 ziyaret konumu. Boş hafıza doğuştan gelir.
- Sosyal hareket tek adım, bedeni/enerji maliyetini aşmaz. Yolculuk için kalan
  ömür ve enerji sınanır. Algılanan/hatırlanan tehlikenin iki hücre çevresine
  sosyal yaklaşım yapılmaz; sığınak fiziksel koruması korunur.
- Hedef kaybolunca son gözlenen çevrede sınırlı arama; bilinmeyen hücrelerin
  veya hareket etmiş eşin gerçek koordinatını öğrenme yoktur. Yerel adım seçimi
  genel rota planlayıcısı değildir; bilinmeyen tehlikeden koruma garantisi yoktur.
- SeekCompanion/WaitForMate ve aktif hedef gözlemden incelenebilir.

## Önce küçük testler

11 mekanizma testi: hazır olmayan yetişkinle bekleme; gerçek doğum; hazır
alternatif eşi kaçırmama; bilinmeyen hedef uydurmama; temel öncelikler;
çapraz tehlike ve yeniden deneme güvenliği; maliyet/işgal; kayıp hedef araması,
zaman aşımı; yaş sınırı; bozuk hafıza reddi; aktif hedefle tam durum+RNG
checkpoint devamı; yalnız avcılı dünyada bayrağın davranışı değiştirmemesi.
Bazı testler birden fazla sözleşmeyi kapsar. Toplam71 Rust,30 Python testi;
HTTP/auth/kontrol/yazıcı kilidi/SIGKILL kurtarma ve arşiv SQL kontrolleri geçti.

## Aynı kontrol noktaları

Önceki çöküşün seed2 fixed kaydı kullanıldı. Rezerv hazırlığı bütün kollarda
**açık kaldı**; canlı çıkarma/ekleme veya kaynak değişimi yapılmadı. Deney
bayrağı göçten sonra açıldı. Bütün kolların ikilisi ve giriş SHA'ları sabit.

| Kol | Başlangıç → son | Son otçul | Son avcı | Sonuç |
|---|---|---:|---:|---|
| Kapalı kontrol |16.000 →17.000|274|56|Eski tam durum+RNG ile eşit|
| Erken buluşma |16.000 →20.000|449|51|Bu kopyada çöküş önlendi|
| Geç buluşma |17.400 →20.000|0|0|Toparlanmadı|
| Bilinmeyen eş negatif kontrolü |17.600 →17.800|0|0|Hedef uydurulmadı|

Kapalı kontrol karşılaştırmasında yalnız açıkça eklenen şema farkı normalize
edildi: sürüm12→11; false politika alanı ve boş social alanları çıkarıldı.
Yeni alanların gerçekten false/boş olduğu ayrıca doğrulandı. Diğer bütün
alanlar ve RNG eşit. Aktif hedefli V12 devamı ayrı testte normalize edilmeden eşit.

Erken kolda son400 adımda2.072 otçul/21 avcı doğumu var. Sosyal kararlar gerçekten
çalıştı:34.263 yaklaşma/arama ve42.867 bekleme. Bu, tüm sürenin bekleyerek
güvende tutulması değildir; normal ihtiyaçlar ve avlanma sürer. Geç kolda50
otçul doğumu olmasına rağmen toparlanma yok;17.700 örneğinde otçul görülmüyor,
20.000'de iki tür de yok. Örnek sıklığı100 olduğu için17.700 kesin yok oluş
adımı diye sunulmaz. Negatif kolda sosyal karar0; geçmişte bilinmeyen eşin
koordinatını bulma veya son iki bireyi zorla birleştirme eklenmedi.

Ham kayıtlar `data/experiments/mate-rendezvous-checkpoints`; plan/ikili/giriş
SHA'ları manifestte, kısa makine raporu `MATE_RENDEZVOUS_SCREEN.json` içinde.
Tekrar: `collapse_branch CHECKPOINT TICKS rendezvous YENI_SON_DURUM.json`.
Kapalı kol için `unchanged`. Eski V11 checkpoint bayrağı göçte kapatılır;
örnek program rendezvous kolunda bu göçten sonra açıkça etkinleştirir.

## Kabul ve sıradaki sınır

**Aday ekolojik olarak kabul edilmedi.** Bilinen bir seed'in belirli geçmişi
üzerinde erken etkili, geç toparlanmada başarısız. Başlangıçtan açık seed2,
sekiz normal/şok seed ve görülmemiş uzun küme henüz doğrulanmadı. Eş aramasını
iyileştirmek her evrede av baskısını yenmeye yetmiyor. Başarısız geç kolu gizleyip
yeni uzun tarama açılmadı; varsayılan false, canlı dünyalar eski ikililerde.

Bir sonraki dar inceleme,17.400 kopyasındaki üreme/beslenme/tehlike kesintilerinin
hangi bireyleri fırsatsız bıraktığıdır. Bu başarısızlık anlaşılmadan hedef süresini
veya yaşları körlemesine ayarlama. Mevcut sonuç başlangıçtan açık davranışa veya
kaynak şokunun çözülmesine genellenmez.

Doğrulama araçları `validate_coexistence.py` ve `check_reserve_policy.py`
`--mate-rendezvous` alır; açık ayar manifest kimliğine katılır. Bayrak değişince
resume reddedilir, probe ayarı uygulamamışsa sonuç reddedilir. Bu seçeneklerin
hazır olması uzun araştırmanın başlatıldığı anlamına gelmez.
