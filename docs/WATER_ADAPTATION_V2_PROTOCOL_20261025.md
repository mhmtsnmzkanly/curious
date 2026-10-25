# Su koşullarına uyum V2 — gıda erişimi ayrıştırılmış protokol

Birincil alan K3; K1/K7/K8 ile kesişir. Kullanıcı V1 geçersiz kalibrasyonundan sonra alt ajanla yeni fixture kurulmasını açıkça istedi. V1 protokolü, kodu, sonucu ve ham verisi değişmez. [V1 sonucu](reports/WATER_ADAPTATION_V1_20261025.md): gıda mevcutken hareket enerjisi sıfırlandığı için birey fork öncesi öldü. Bu yeni kimlik, su kararını bu gıda erişim kilidinden ayırır; hareket maliyeti, metabolizma, ödül veya öğrenici değiştirilmez.

## Tek fiziksel tasarım değişikliği

Tek ordinary AI otçul, başlangıç(0,0); harita[-6,6]×[-3,3]. V1 gibi varsayılan beden; yalnız max_age=2000 ve maturity_age=2000 başlangıçta ayarlanır. A(-2,0), B(4,0); her ikisi başlangıçta12 su ve kapasite12. Prefix A refill4/2tick, B refill1/12tick. Food hücreleri yürünebilir; her bir89 su-dışı hücre ordinary Food amount/kapasite30, refill2/2tick içerir. Normal Environment yeniler; her tick beden/konum/kaynak sıfırlama yoktur. Bu gıda-zengin lab, gerçekçi kıtlık veya ekolojik yarar testi değildir.

Gıda su hücresinin üzerinde yoktur, fakat her su hücresinin bitişiğinde vardır. Normal Eat planı son etkileşim adımını hareketten ayırdığı için sıfır enerjiyle bulunduğu veya bitişik hücreden yemek mümkündür; yeni ücretsiz hareket veya rescue kuralı eklenmez. Bütün kollarda aynı başlangıç haritası/beden/ordinary memory/RNG kullanılır. Rastgele ilk harita silinir; World::with_seed kaynak-popülasyon RNG adımları geri alınmaz.

## V1 ile aynı deney ve geçerlilik koşulları

V19 context_ttl_enabled=true, policy.water_residual_enabled=false; merak, mevsim ve diğer deneysel seçenekler kapalı. Ortak200tick online prefix, tick200 aynı checkpoint'ten dört200tick suffix: trained_online, trained_frozen, naive_online, rule_based. [V1 protokolündeki](WATER_ADAPTATION_V1_PROTOCOL_20261025.md) learner ablasyonları aynen geçerlidir. Her kola eşit Interrupted sansürüyle açık prefix bölümü kapatılır; bu kaynak başarısızlığı değildir.

Tek fork müdahalesi: A amount1/refill1/12tick, B amount12/refill4/2tick; kapasiteler12 kalır. Bundan sonra fiziksel sıfırlama veya ortam ayarı yoktur. Her kol tick400 veya doğal ölümde biter. Frozen stored values/residual/context tabloları değişmez; tahminde48tick TTL yaşlanır. Legacy tablolar normal kaydedilebilir ama V17 artık seçimi açılmaz.

Hipotez: güncellemeleri açık birey kaynak yenilenmesi değişince kendi deneyimiyle seçimini değiştirebilir ve bazı fiziksel sonuçları iyileştirebilir. Yalnız tablo kaydı veya B'ye varma öğrenme yararı değildir. Su kazanımı, resolver hareket enerjisi, tick başında water<max_water/4 susuzluk tick'i, water==0 sıfır-su tick'i, yaşayan suffix tick'i, terminal sağlık/enerji/su kaydedilir. Sağlık kaybı atfı eksikse açlık/susuzluğa dağıtılmaz. İlk B içme gecikmesi yardımcı ölçüdür; gerçekleşmemişse null ve ölüm/ufuk sansürüyle korunur.

Aynı ağırlıksız Pareto vektörü: sağkalım, su kazanımı, terminal stoklar büyük; hareket, susuzluk ve sıfır-su küçük daha iyi. Online-frozen ve online-rule ayrı karşılaştırılır: hiçbir bileşen kötüleşmeden en az biri iyileşirse o seedde hakimiyet, hepsi eşitse nötr; diğer durumlar ödünleşim/karşı hakimiyet. Aggregate toplamlar yalnız betimseldir. En az bir seedde iki kontrole hakimiyet ve öğrenme mekanizması şahidi varsa yalnız o seed/fixture için varlık örneği raporlanır; genel avantaj veya canlıya kabul çıkarılmaz.

Mekanizma şahidi: trained_online ve trained_frozen arasında farklı seçili kaynaklı taze iki-kaynak kararında learner dışındaki gerçek State, RNG ve aday konum/miktar/yol listesi eşit; kayıtlı öğrenilmiş tablolar farklı olmalıdır. Farklı beden/RNG altında sonraki karar farkı şahit değildir. Greedy/explore sınıfı gerçek RNG dalı doğrulanmadan iddia edilmez. Şahit olmaması başarısızlığın gizlenmesi için eşik veya parametre değiştirme nedeni değildir.

Kalibrasyon seed62299; ancak geçerse formal62201..62206. Seed ön taramasında canonical data/experiments metin kayıtlarında kullanım bulunmadığı dosyayla kaydedilir; küresel kullanılmamışlık garantisi verilmez. Seedler V1'in gözlenen62199 sonucunun tekrarı değildir. Tek kalibrasyon ve tek formal batch; sonuçtan sonra tuning veya aynı kimlikle yeniden koşu yoktur.

Geçerlilik kapısı DEĞİŞMEZ: tick200'de canlı; prefix en az5 taze iki-kaynak seçimi; fork context en az1 kayıt; online ve frozen suffix ayrı ayrı en az3 taze iki-kaynak seçimi; dört kol mevcut; başlangıç fizik/ordinary memory/harita/RNG eşit; frozen tablolar değişmez. Kalibrasyon geçmezse formal başlamaz. Formalda düşük fırsat veya ölüm ölçüm sonucu/sansürdür; ham kimlik, frozen tablo ya da başlangıç eşitliği hatası altyapı reddidir ve kalan seedler durur.

## Kimlik, kaynak bütçesi ve sunum

Yeni example ve V2 supervisor, V1 metrik helper'ını kaynak SHA haritasına dahil ederek kullanır; V1 dosyasına yazmaz. Protokol/kaynak koşudan önce commit edilir. Temiz Git ağacı, binary/source/protokol SHA eşliği ve başarılı kalibrasyon formal önkoşuludur. Yeni çıktı dizini gerekir; var olan raw/stdout/stderr üzerine yazılmaz.

En fazla7000 fiziksel Worldtick: kalibrasyon200+4×200, formal6×1000. Seed30s, child adres alanı1GiB, raw64MiB, replay20MiB, phase çıktı512MiB; jobs1 ve ayrı Cargo target. Panik/kaynak/şema hatasında raw korunarak durulur; yeni sweep veya canlıya geçiş başlatılmaz. Sınıra rağmen raw tutma başarısızsa kabul kanıtı yoktur.

Raw JSON tam State/algı/olayları korur. Replay yalnız kaydedilmiş tick/bounds/resources/entities ve id/pos/decision alanlarının sunum projeksiyonudur; statik Environment kaynak listesi ve tam algı raw dosyada kalır. Aynı water-adaptation-replay-v1 sunum şeması, experiment=water-adaptation-v2 etiketi ve gerçek source commit'iyle mevcut yerel görüntüleyicide açılır. Sonuçlar kayıt/dondurma/müdahale farklarıyla açıklanır; hiçbir fake fizik olayı eklenmez.

Manifest, ham kayıtlar, negatif sonuçlar, SHA'lar, seed başına analiz ve replay yeni dizinlerde saklanır; rapor/commit/push/bildirim yapılır. Yalnız göreve ait inaktif target temizlenir; canlı7882 ve replay7883 değiştirilmez. Başarısız kalibrasyon da çalışmanın dürüst sonucu olarak yayımlanır.
