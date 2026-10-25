# Su koşullarının değişimine bireysel uyum — V1 protokolü

Birincil alan K3; K1/K7/K8 ile kesişir. Amaç V19 bağlamsal su öğrenmesinin kendi deneyiminden sonra fiziksel fayda ve karar farkı üretip üretmediğini ölçmektir. Yeni fizik/ödül, nüfus yardımı veya canlıya dağıtım yoktur. Önceki [ömür boyu su V2](reports/LIFELONG_WATER_V2.md) ve [kabul tanısı](reports/QUALIFICATION_DIAGNOSTIC_20261025.md) faydanın genel olmadığını gösterir; eski sonuçlar değişmez.

## Hipotez ve karşılaştırma

Aynı bireyin ortak200tick öğrenme geçmişinden sonra sourceA'nın yenilenmesi yavaşlayınca güncellemeleri açık bağlamsal öğrenici, aynı öğrenilmiş tabloyu donduran kola göre seçimini değiştirebilir. Fiziksel fayda öğrenme tablosunun dolmasından veya yalnız B'ye ulaşmaktan çıkarılmaz. Kurallı ve naif-online kontroller de aynı başlangıç bedenine ve çevreye sahiptir.

Tek ordinary AI herbivore: başlangıç(0,0), harita[-6,6]×[-3,3], varsayılan beden; yalnız max_age=2000 ve maturity_age=2000 başlangıçta ayarlanır. Böylece 400tick ufukta yaşlılık ve üreme etkisi ayrılır. A(-2,0), B(4,0). Başlangıç su miktarı her ikisinde12; A kapasite12/refill4/2tick, B kapasite12/refill1/12tick. Gıda(0,-1) kapasite30/refill2/2tick. Rastgele ilk harita kaynakları silinir, World::with_seed çağrısının tükettiği RNG adımları korunur ve gerçek RNG kaydedilir. Normal World/karar/çözücü kullanılır; kaynak deposu ve beden her tick elle doldurulmaz.

V19 context_ttl_enabled=true; policy.water_residual_enabled=false. Legacy value ve residual tabloları kaynakta normal şekilde güncellenebilir ama V17 artık seçimi açılmaz. Merak, mevsim ve diğer deneysel seçenekler kapalıdır.

Tick200 ortak checkpoint'inden dört kol:

- trained_online: öğrenilmiş tablolar korunur, update açık.
- trained_frozen: aynı tablolar korunur, update kapalı. Değerler, sayımlar ve context kayıtları değişmez; 48tick TTL tahminde doğal olarak işler, zaman durdurulmaz. Episode gibi geçici işlem durumu frozen tablo değildir.
- naive_online: yalnız learner değer/residual/context ve episode geçmişi temizlenir, güncelleme açık; beden/yaş/RNG/ordinary memory korunur. Bu bellek ablasyonudur, doğal yaşamda unutma iddiası değildir.
- rule_based: learner kapalı; beden/ordinary memory/harita aynı.

Fork öncesi açık su bölümü her kola eşit şekilde Interrupted olarak kapatılıp başlangıç müdahalesi açıkça kaydedilir; bu yapay sansür başarısız kaynak etiketi değildir. Tick201'den önce tek çevre müdahalesi: A amount1/kapasite12/refill1/12tick, B amount12/kapasite12/refill4/2tick. A tamamen kaldırılmaz; algıda iki seçenek fırsatı korunabilir. Bundan sonra beden/konum sıfırlaması veya ortam ayarı yoktur. Her kol tick400 veya bireyin doğal ölümünde biter.

## Ölçüm ve yorum

200tick suffix için resolver'dan su kazanımı ve hareket enerjisi; tick başındaki water<max_water/4 susuzluk tick'i, water==0 sıfır-su tick'i; yaşayan suffix tick sayısı, son sağlık/enerji/su; karar, görülen kaynak miktarı/yolu, gerçek seçili hedef ve context kayıtları tutulur. Saldırı beklenmez; sağlık azalmasının atfı eksikse açlık/susuzluk diye uydurulmaz.

B'den ilk doğrulanmış içme gecikmesi yardımcı ölçüdür; tek içme sürdürülebilir uyum değildir. İçme yoksa gözlenen pencere ve ölüm/ufuk sansürüyle null tutulur; otomatik200 veya0 değeri verilmez. RNG'nin explore/greedy dalı yeniden oynatılarak doğrulanmadıkça etiketlenmez. Base reason çıkarımsa kayıtta inferred belirtilir.

Ağırlıksız vektörde sağkalım tick'i, terminal health/energy/water ve toplam su kazanımı büyük; hareket enerjisi, susuzluk ve sıfır-su tick'leri küçük daha iyi sayılır. Online-frozen ve online-rule ayrı Pareto karşılaştırılır: hiçbir bileşen kötüleşmeden en az biri iyileşirse o seedde fiziksel hakimiyet; hepsi eşitse nötr, aksi durumda ödünleşim/karşı hakimiyet. İlk B gecikmesi bu ana vektöre eklenmez.

Öğrenme kaynaklı karar için online-frozen arasındaki ilk farklı seçim, eş gerçek algı/beden ve RNG koşullarında kayıtlı tablo farkıyla desteklenmelidir. Farklı bedenler altında sonraki karar farkları tek başına mekanizma atfı değildir. En az bir seedde bu mekanizma şahidi ve her iki kontrole karşı Pareto hakimiyet bulunursa yalnız varlık örneği raporlanır; genel avantaj kabulü yapılmaz. Yokluğu veya karşı hakimiyet başarısız/negatif sonuç olarak korunur; sonuçtan sonra eşik değiştirilmez.

## Kalibrasyon ve formal kapısı

Tek kalibrasyon seed62199, ardından ancak geçerlilik kapısı geçerse formal62101..62106. Ön tarama `data/experiments` altındaki1375 metin dosyasında bu seedlerin daha önce ilan edildiğini bulmadı; bu tarama kapsamı dışında küresel kullanılmamışlık iddia edilmez. Kalibrasyon sonuçları formal fayda sayılmaz.

Kalibrasyon kapısı: birey tick200'de canlı; prefixte en az5 taze iki-kaynak seçimi; fork learner'ında en az1 context kaydı; online ve frozen suffixte ayrı ayrı en az3 taze iki-kaynak seçimi; başlangıç fiziksel beden/ordinary memory/harita/RNG eşitliği; frozen öğrenilmiş tablolarında sıfır değişiklik. Kapı geçmezse formal seedler tüketilmez; sonuç tanımlanamaz/fixture geçersiz olarak raporlanır. Kalibrasyondan sonra parametre ayarı veya aynı kimlikle tekrar yoktur. Ölüm kısa-ufuk sonucu olabilir, başarısız lab'ı gizlemek için yeniden birey eklenmez.

Kaynak, protokol ve koşucu koşudan önce commit edilir; calibration ve formal aynı binary/source/protokol SHA ile yürür. Kaynak değişirse yeni kimlik ve ayrı kalibrasyon gerekir. Supervisor koşmadan önce temiz Git ağacı ve no-overwrite çıktı dizini ister. Her seed yeni JSON +stderr, summary, replay ve manifest saklar. File-based viewer salt okunurdur; tarayıcı dosyayı yerel açar, sunucuya yüklemez.

## Kaynak sınırı ve durdurma

Bir kalibrasyon1000 ve altıformal×1000 olmak üzere azami7000 fiziksel World tick; erken ölümler toplamı düşürür. Seed başına30s, subprocess adres alanı1GiB, raw kayıt64MiB, toplam disk512MiB; ayrı Cargo target/jobs1. Panik, şema/freeze eşitsizliği veya kaynak sınırı altyapı başarısızlığıdır; ham çıktı korunur, formal sürdürülmez. Geçmiş sınırlı öğrenme lab'ı sonucundan genel ekolojik fayda, yıllarca işletim veya açık uçlu evrim çıkarılmaz. Çalışma bittiğinde rapor, commit/push, bildirim ve yalnız göreve ait inaktif build cache temizliği yapılır; ham lab kanıtı korunur.
