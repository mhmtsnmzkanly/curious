# Av laboratuvarı v1: kurulum, ölçüm ve bulgu

Kaynak2672710, temiz commit; protokol sonuç görülmeden sabitlendi. Çalışma kökü
`data/experiments/hunt-lab-v1-20261021`. Manifest, değişmez ikili/protokol,33MiB
hamJSONL, özet ve16kısaSQLite örneği korunur.6400kontrollü denemede79849gerçek
fiziksel tick yürüdü;128000üst sınır aşılmadı. Bütün koşu yaklaşık9,72s sürdü.
Bu bir sürekli simülasyon servisi veya yeni uzun denge taraması değildir.

## Aynı koşullarda dört kol

Her izde sekiz eğitim/evaluation çifti; kurallı, deneyimsiz-online,
eğitimli-online ve aynı eğitimli tablo-frozen karşılaştırıldı. Yaş/beden/ilk
görüş her trial dört kolda eşit; update kapalı kolun tablosu ve visits değişmedi.
Normal koşulda düşük sağlıklı yakın av kaçar, uzak av bekler.40evaluation
sonrası roller tersine döner. Passive ve bitişikte gerçekten saldıran defensive
izleri ayrıdır. Trial içinde ışınlama/reset yok; Player av komutları da normal
resolver ile maliyet öder. Bedenler yalnız trial sınırında açıkça sıfırlanır.

Besin edinilen trial oranları (her hücre8seed×40trial=320deneme):

| İz / koşul | Kurallı | Deneyimsiz | Eğitimli online | Eğitimli frozen |
|---|---:|---:|---:|---:|
| Passive, normal |0%|2,81%|12,81%|12,81%|
| Passive, ters |100%|100%|100%|100%|
| Defensive, normal |0%|2,81%|2,81%|2,81%|
| Defensive, ters |100%|100%|100%|100%|

Pasif zor koşulda geçmiş deneyimle gözlenen avantaj vardır fakat320denemenin
279'unda eğitimli avcı da besin alamaz. Bu yeterli bir avcı/ekosistem kabulü
veya istatistiksel genel üstünlük değildir. Her iki izde sekiz eğitim
modelinin dördünün tablosu80eğitim sonunda tamamen boştur; etki seedler arasında
homojen değildir. İlk/son20pencereler ve seed sonuçları JSON özetinde ayrı durur.

Ters passive koşulun son20denemesinde eğitimli-online toplam103,5enerji,
frozen129enerji/20deneme harcar; kurallı100, deneyimsiz103,5. Aynı besin oranı
olmasına rağmen eski tercihi güncellememenin ek bedeli görülebilir. Her koşulda
öğrenme üstün değildir. Toplam enerji başlangıç+edinilenbesin−sonrezerv ile,
hareket/saldırı ayrıca resolver olaylarıyla denetlendi; iki kez çıkarılmadı.

## Asıl tıkanıklık: çok az puanlanabilir sonuç

Passive-normal eğitimli-online kolunda751motor bölüm kapanışı var:41puanlı
Fed,710puansız kesinti; **%94,54puanlanmıyor**. Kesintiler434TargetChanged,
275HuntTooCostly,1HuntLowEnergy. Bu düzenekte dış hasar yoktur; yani tek
sorun yaralanma değildir. Çok yürüyüp sonra maliyet sınırına takılmak veya
hedef değiştirmek tabloya başarısızlık/bedel bilgisi taşımıyor. Labın20tick
sınırındaki kesintiler ayrı origin=lab_boundary sayıldı; motor sonucu diye
puanlanmadı. Bu iki farklı kesinti türü aynı torbaya atılmadı.

Defensive-ters eğitimli-online kolu320kez gerçekten besin aldı; **303'ü** avcı
hasar aldığı için ambiguous ve reward=None kaldı. Başarılı beslenmenin%94,69'u
öğrenmeye yansımadı. Bu izde savunan avın verdiği hasar gerçek Attack olayıdır;
öğrenici mevcut tasarımında kendi avının savunmasını da dış nedensellik
belirsizliği sayar.17puanlıbeslenme320ambiguous bölüm sayısıyla karıştırılmaz:
başka kesintiler de ambiguous olabilir. Bütün auditler hamda mevcut.

Bu sonuçlar yalnız 'daha uzun eğitelim' veya ödül katsayısını rastgele artırmak
önerisini desteklemez. Sonraki dar çalışma: denetlenmiş kendi maliyeti yüzünden
biten takip ile dış müdahaleyi ayırmak; hedef avın savunma hasarını başka
bireylerin saldırısından ayırmak. Bu v1'de öğrenici/ödül/epsilon/süre veya
biyolojik kurallar sonuçlara göre değiştirilmedi. Yeni hipotez ayrı kaynak ve
protokol ister;41001–41008artık görülmüş evaluation seedleridir.

## Doğrulama ve sınırlar

117Rust/62Python test geçti. Audit aynıtickte iki kapanışı kaybetmez, her tick
kuyruğu boşalır, checkpoint devamı aynı olay/durum/RNG'yi verir. Kuyruk transient
ve serde dışında; checkpointV13/SQLiteV2 korunur. Öğrenme kapalı100/200tick
biyoloji veRNG frozen32ddfb2 ile eşit; yeni alanların kapalı/boşluğu kontrol edildi.
HTTP/auth/writer-lock/SIGKILL kontrolü geçti.16SQLite örneği hamJSONL ile tick,
son beden ve bütün bölüm sonları açısından yeniden karşılaştırıldı; replay
okuyucusu V13örneklerini açtı. Deney modeline ayrıcalıklı dünya bilgisi verilmedi.

Kontrollü/senaryolaştırılmış avlar ve trial reseti doğal ekosistem değildir;
ölüm/üreme/gerçek yaş avantajı ölçülmez. Belli kapanışlar (örneğin susuzluk veya
hedef kaybı) bu düzende baskın çıkmadı; yokmuş gibi genellenmez. PRNG ve yeniden
seedleme düzeni sabittir; trial'ların IID olduğu veya güven aralığı/p-değeri
iddia edilmez. Kaynak/ham/ikili hash'leri HUNT_LAB_V1.json ve manifestte.

Canlı7882dünya ve7883replay kaynak/ayarları değiştirilmedi. Av öğrenicisi
canlıda yok ve geliştirmede varsayılan kapalı. Bu laboratuvar yeni avcıyı
üretime kabul etmek için değil, eksik geri bildirim yolunu teşhis etmek içindir.

Kullanım ve örnek replay komutu: ../HUNT_LAB.md.
