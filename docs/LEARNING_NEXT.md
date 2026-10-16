# Bireysel öğrenmenin sonraki adımı

## Mevcut sınır

`entity/learning.rs` görünür su kaynaklarını konuma göre puanlayan, en çok
32 kayıtlı bir bandit içerir. Yalnız kurallı karar SeekWater olduğunda ve en az
iki su kaynağı göründüğünde devreye girer. Sonuç aynı tickte içilen su, enerji
kaybı ve yaralanmadan hesaplanır. Yaş bonusu veya yavruya anı aktarımı yoktur.

Bu yapı genel bir durum–eylem öğrenmesi değildir. Uzak kaynağa doğru giderken
henüz su içilmemesini başarısızlık sayabilmesi, aynı tickteki başka yaralanmaları
seçime bağlayabilmesi ve konum bilgisini yeni bir bölgeye taşıyamaması açık
sınırlardır. Sonraki tasarımda bunlar sessizce doğru kabul edilmemelidir.

## Önce öğrenmeyi ölç

`examples/learning_trial.rs` sabit ortam deneyine ek olarak 400 denemeden sonra
rakibin yarıştığı su kaynağını değiştirir. Aynı fiziksel çözümleyici kullanılır.
Öğrenme açık/kapalı, sekiz seed; değişim öncesi ve sonrası ilk/son 100 denemede
alınan su raporlanır. Bu kontrollü laboratuvarda bedenler ve kaynaklar her denemede
açıkça sıfırlanır; yaşayan ekosistem veya yaşlı hayvan başarısı diye sunulmaz.

Sabit politika değişen çevrede zaten iyi olabileceğinden yalnız onu yenmek şart
koşulmaz. Eski tercihin sürmesi, toparlanma süresi ve kalan keşif maliyeti birlikte
incelenir. Negatif sonuç motorun değişmesini gerektiren bulgudur, eşik gevşetme
nedeni değildir. Sekiz eğitim seedinden sonra ayrı seedlerde değerlendirilir.

## İlk genişletme: av tercihi

Büyük hazır model indirmek yerine yerel, sınırlı tablolardan başlanır. Motorun
fiziksel yetki sınırı korunur: öğrenici sadece uygun adayları sıralar; hareket,
saldırı, harcanabilir enerji ve diğer gerçek kurallar çözümleyicide doğrulanır.
Öğrenici dünya nesnesine, RNG'nin geleceğine, avın görünmeyen bedenine erişemez.

İlk sürüm yeni üst düzey amaçlar öğrenmez. Var olan avlanma kararı içindeki
aday tercihini öğrenir. Kurallı politika kapatılabilir karşılaştırma olarak kalır.
Sonradan avı bırakma ve başka davranışa geçiş ayrı değişiklikler olarak ölçülür.

Bağlamda kendi enerji/dayanıklılık oranının üç bandı, görünür avın mesafesinin
üç bandı ve algıda varsa yaralanma bilgisi kullanılır. Kimlik veya tam koordinata
bağlı puan yerine durum sınıfı kullanılır. Bilinmeyen sağlık ayrı sınıftır;
canlı göremediği bilgiyi öğrenici için üretmeyiz. Tablo ve günlük üst sınırları
kodda doğrulanır. Kesin tablo boyutu bağlam kodlamasıyla birlikte test edilir.

## Sonuç ve zaman sınırı

Bir saldırı başarısı tek başına pozitif ödül değildir. Takip başlangıcı, hedef,
başlangıç rezervleri ve ödenen maliyetler sınırlı bir bölümde izlenir. Bölüm;
beslenme, hedef kaybı, vazgeçme veya en çok 20 tick sonunda kapanır. Süre sınırı
ilk deney parametresidir, ekolojik gerçek diye sunulmaz. Hedef ölümü veya
leşten beslenmenin kredisi yalnız kimliği doğrulanmış olaylarla ilişkilendirilir;
başkasının öldürdüğü avı kendisinin başarısı saymayız.

Başlangıç ödül hipotezi: gerçekten edinilen besin eksi hareket/saldırı maliyeti
ve yaralanma. Aynı maliyet hem net rezerv farkından hem olaylardan iki kez
çıkarılmaz. Açlık, iyileşme, üreme ve dış saldırı etkileri muhasebede ayrılır;
ayıramadığımız sonuç belirsiz diye kaydedilir. Ağırlıklar eğitim seedleriyle
seçilir ve değerlendirmeden önce sabitlenir. Toplam ödül tek başarı metriği değildir.

## Kanıt ve dağıtım sırası

1. Aynı algı/beden, farklı deneyim: farklı tercih gözlenmeli; yaş eşit tutulmalı.
2. Aynı birey, değişmiş çevre: kötüleşen eski tercih güncellenmeli.
3. Öğrenme kapalı/açık ve eğitimli politikanın güncellemesi kapalı karşılaştırma:
   bedelsiz rastgelelik ile öğrenme faydası ayrılmalı.
4. Gerçek ölçütler: av başına enerji maliyeti, edinilen besin, yaralanma,
   başarısız takip süresi; sonra yaşam ve üreme. Sadece ödül artışı yeterli değil.
5. Checkpointten devam aynı durumu/RNG'yi vermeli; yavrunun tablosu boş başlamalı.
6. Eşit işlem bütçesiyle eğitimde kullanılmamış seedler ve farklı kaynak düzeni.
7. Ayrı karma dünya doğrulaması; mevcut frozen araştırmayı veya canlı dünyayı
   yerinde değiştirme. Başarısız öğrenici varsayılan olarak açılmaz.

Gözlem ekranında öğrenilmiş tercih, deneyim sayısı, son bölümün maliyet/sonucu
ve eski karar yerine neden bu adayın seçildiği gösterilir. Kullanıcı bir yaşlı
bireyin deneyiminin işe yaradığını karar geçmişinden görebilmelidir. Yaş ile
hayatta kalan bireylerin seçilim etkisi ayrı değerlendirilir.

Bu dosya uygulama sözleşmesi taslağıdır. Genel av öğrenmesi henüz uygulanmadı;
uzun denge taraması yalnız mevcut su öğrenmesi ve hareket mutasyonunu içerir.
