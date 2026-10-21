# Su deneyimi: önceden sabitlenmiş ölçüm protokolü

Bu protokol sonuçlar çalıştırılmadan önce yazıldı. Yeni av öğrenicisinin,
yaşam süresinin veya ekolojik dengenin kabul testi değildir. Kontrollü bir
laboratuvarda yalnız su tercih tablosunun etkisini ölçer. Negatif bulgu çıktığında
seed, süre veya başarı tanımı değiştirilmez; yeni hipotez ayrı protokol ister.

## Sabit tasarım

`cargo run --release --example experience_trial` sekiz eşleşme üretir:

| Eğitim | Ayrı değerlendirme |
|---|---|
| 1 | 30001 |
| 2 | 30002 |
| 3 | 30003 |
| 4 | 30004 |
| 5 | 30005 |
| 42 | 30006 |
| 12345 | 30007 |
| 2026 | 30008 |

Bunlar bu deneyin eğitim/değerlendirme ayrımıdır; 30001–30008'in projenin başka
bir deneyinde hiç kullanılmadığı iddia edilmez. Parametre seçimi veya seed arama
yapılmaz. Her eşleşmede 400 eğitim tick'i ve dört kolun her birinde 400
değerlendirme tick'i vardır: toplam 2.000 tick/eşleşme, 16.000 tick/tam deney.
Her kol 400 tick ile sınırlı; varsayılan çalışma kısa, sonlu bir komuttur.

5×5 haritada deney bireyi (0,0), su kaynakları (1,0) ve (0,3) konumundadır;
her kaynak her deneme başında 10 su içerir. İkinci birey dış komutla önce
(2,0)'dan yakın kaynağı içer. Değerlendirmede 200. deneme tamamlandıktan sonra
rakip (0,4)'e geçerek uzak kaynağa yönelir. Başka çevre veya parametre araması
yoktur. Eğitimde bu tersine dönüş yoktur.

Dört kol aynı değerlendirme seed'iyle yeni dünyada başlar:

- `rule_based`: öğrenici tamamen kapalı, varsayılan kurallı su seçimi.
- `naive_online`: tablo boş, su öğrenmesi ve güncelleme açık.
- `experienced_online`: 400 eğitim denemesinin tablosu kopya, güncelleme açık.
- `experienced_frozen`: aynı eğitilmiş tablo, öğrenilmiş tercih açık,
  `update_enabled=false`; sonuç kaydı sürer fakat tablo ve deneyim sayıları değişmez.

Aktif üç öğrenici aynı mevcut %10 keşif mekanizmasını kullanır. Dondurma keşfi
kapatmaz; böylece keşif azaltımının etkisi öğrenme faydası diye sunulmaz.
Başlangıç seed'i eşitliği, kollarda aynı miktarda RNG tüketildiğini garanti etmez;
özellikle kurallı kol farklı çağrılar yapabilir. Tek bir iyi seed seçilmez.

Her deneme bir tick'tir. Kaynaklar, iki bireyin bedeni/yaşı/suyu/konumu/evresi ve
öğrenme dışındaki hafızası açıkça sıfırlanır; yalnız su değerleri ve deneyim sayısı
aktarılır. Deneme başlangıcında yaş her kolda sıfırdır; motor karar öncesinde aynı bazal
yaşlanmayı uygular (karar kaydında yaş 1). Yaş bonusu yoktur. Tick sayacı/RNG denemeler
arasında ilerler. Her kolda aynı varsayılan beden kullanılır. Tamamlanmamış bir
su bölümü laboratuvar sınırında `Interrupted` olarak kesilir ve ayrı sayılır;
tam ekosistemde su kaynağına ulaşamama olarak yorumlanmaz. Kaynaklar görüşte,
fiziksel çözümleyici gerçektir; bu deney nüfus, hayatta kalma veya üremeyi ölçmez.

## Önceden belirlenmiş çıktılar

Her deneme için hedef, edinilen su, olaylardan gerçek hareket enerji maliyeti,
gelen hasar, reddedilen eylem sayısı, bölüm sonu/ödülü ve önceki/sonraki tablo
deneyim sayısı JSONL'a yazılır. 10 su tam başarı, 1–9 kısmi başarı, 0 sıfır su;
bu üç sınıf birbirini dışlar. Kısmi içme `WaterEnd::Reached` olabileceğinden,
motorun bölüm başarısı ile laboratuvarın tam su başarısı aynı sayılmaz.

Değişim öncesi ilk/son 100 ve değişim sonrası ilk/son 100 deneme ayrı özetlenir.
Yakın/uzak/diğer tercih sayıları, bedel, su ve deneyim sayısı ayrı gösterilir;
toplam ödül başarı ölçütü değildir. Başlangıç, tersine dönüş ve bitiş tabloları
korunur. `experienced_frozen` değerleri ve ziyaret sayıları değişmemelidir.

Ana karşılaştırmalar: deneyimli/deneyimsiz kolun ilk 100 denemesi; aynı eğitilmiş
tabloyla güncelleme açık/kapalı kolların tersine dönüş sonrası ilk/son 100'ü;
kurallı kolun aynı pencereleri. Her seed'in farkı ve sekizli aritmetik ortalama
raporlanır. Önceden kabul edilmiş bir üstünlük eşiği veya p-değeri yoktur.
Düşüş, deneyim yararı bulunmaması ve sabit kuralın daha iyi olması geçerli
sonuçlardır. Farklı haritaya genelleme bu ilk deneyde ölçülmez; aynı konumlara
bağlı tablo olduğu açık kalır.

## Kaynak kimliği ve çalıştırma

`scripts/measure_experience.py --binary target/release/examples/experience_trial
--output data/experiments/experience-v1` yeni bir dizin oluşturur; var olan
çıktıyı ezmez. Çalıştırmadan önce protokol SHA256'sını, Git revision/dirty
bilgisini, `src` dosyaları/Cargo dosyaları/örnek SHA256'larını ve ikili SHA256'sını
manifestte saklar. stdout ham JSONL, stderr ayrı log, sonuç özeti ayrı JSON'dur.
SHA'lar ikilinin tam o kaynaktan derlendiğini tek başına ispatlamaz; derlemeyi
ana ajan temiz ve kayıtlı doğrulama akışında yapar. Sonuç bozuk veya komut
başarısızsa manifest tamamlanmış kabul edilmez. Canlı veri veya servise yazılmaz.
