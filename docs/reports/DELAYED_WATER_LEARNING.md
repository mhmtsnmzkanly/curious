# Gecikmiş su denemesi muhasebesi

## Değişiklik

V9 su öğrenicisi kararın verildiği tek tickteki sonucu değerlendiriyordu.
Hızı 1 olan birey 5 hücre uzaklıktaki kaynağa yönelince ilk adımda su içemediği
halde kaynağın değerini düşürebiliyordu. Aynı denemede sonraki adımları birleştiren
bir kayıt yoktu; toplam beden enerjisi değişimi farklı maliyetleri karıştırıyordu.

V10'da görünür hedefe yönelik bir su denemesi en çok 20 yürütülmüş tick sürer.
Daha öncelikli kurallı karar korunur. Aynı hedef görünürken yeniden keşif seçimi
yapılmaz; kalan tek görünür kaynak olsa bile başlatılmış deneme devam eder.
Motor, hareket olayına gerçekten ödenen enerji maliyetini ekler; öğrenici
mesafeden tahmin yapmaz. Böylece kıvrılan veya başladığı yere dönen rotanın
bedeli de kaybolmaz.

Su içilince tek sonuç güncellenir. Drink/MissingResource kesin başarısızlıktır;
Approach/OutOfRange tek başına başarısızlık değildir. Süre sınırında bitmeyen
deneme negatif puanlanır. Kesilen, kapatılan veya ölümle sonuçlanan denemeler
puan tablosunu değiştirmeden gerekçesiyle saklanır. Kaynak görünmez olduğunda
varlığı/yokluğu dünya üzerinden sorgulanmaz.

Ödül ve varsayımlar `../CONTROL.md` V10 bölümünde. Dış saldırı hasarı yalnız
bağlamdır; su seçimine nedensel ceza yazılmaz. Metabolizma/iyileşme/üreme,
hareket maliyetine ikinci kez eklenmez. Bir aktif ve bir son deneme tutulur;
öğrenilmiş su tablosu en çok 32 kayıt olarak kalır. Anılar yavruya aktarılmaz.

## Doğrulama

`tests/learning_episodes.rs` sekiz anlamlı davranışı kapsar:

- Dört ticklik gerçek yolculuk: ara adımlarda değer yok, sonuçta tek ziyaret,
  10 su ve 4 hareket enerjisi. Puan 88. Ara checkpointten devamda durum/RNG eşit.
- Yalnız aynı aktör/hedefin sonucu değerlendirilir; dış saldırı hasarı ayrı kayıttır.
- Tehdit ve dış komut öğrenmeyi keser, kaynağı yanlış biçimde cezalandırmaz.
- Bekleme 20 tickte sınırlanır; yaklaşma gecikmesi kesin kaynak kaybından ayrılır.
- Kapatılan öğrenici ve karar veremeden ölen bireyde açık deneme kalmaz.
- V9 alanları yeni varsayılanlarla okunur; geçersiz deneme checkpointte reddedilir.
- Tek görünür kaynağa devamda yeni rastlantısal seçim yapılmaz.
- Aynı konuma dönen dört adımlık rotada gerçek hareket maliyeti korunur.

Tüm hedeflerde 54 Rust testi ve 21 Python testi geçti. HTTP kontrol/auth,
yazıcı kilidi ve SIGKILL kurtarma kontrolü geçti. Arayüz JavaScript sözdizimi
kontrol edildi; bu aşamada tarayıcıyla görsel test tekrar çalıştırılmadı.

## Kapsam

Bu bir sonuç muhasebesi düzeltmesidir; av öğrenmesi, genel zekâ veya uzun vadeli
ekolojik fayda kanıtı değildir. Sabit rekabet ve ortam değişimi laboratuvarları
geçmeye devam eder. Yeni öğrenici ayrıca ayrı karma dünya deneyine ihtiyaç duyar.
20 ticklik ufuk yavaş ama geçerli rotayı olumsuz değerlendirebilir; bu sınırlı
ufkun açık bedelidir, sonsuz bekleyen deneme bırakılmaz.

Kaynak checkpoint V10 üretir ve V1–V9 okuyabilir. V10 eski motorla açılmaz;
SQLite V2 değişmedi. Eski tamamlanmamış pending atılır, öğrenilmiş değerler
korunur. Çalışan servisler ve ae7f0df araştırma worktree'si değiştirilmedi.
Dolayısıyla sürmekte olan denge testleri V9 davranışını ölçmeye devam eder;
V10'a ait bir sonuçmuş gibi yorumlanamaz.
