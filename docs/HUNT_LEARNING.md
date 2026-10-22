# Sınırlı av-adayı öğrenicisi

Bu modül genel zekâ veya yeni bir avlanma fiziği değildir. Varsayılan `enabled=false`;
kapalı `choose` ve `finish` öğrenici durumuna veya RNG'ye dokunmaz. Üst katman yalnız
AI bireyin mevcut kararını verir. Yüksek öncelikli kararlar değiştirilemez. Canlı
7878–7883 servisleri bu geliştirmeyle yükseltilmez.

`HuntLearner::choose(species, life, view, base, rng)` yalnız `HuntPrey` kararında,
görünür ve hareket+ilk saldırı maliyeti karşılanabilen adayları sıralar. Mevcut
saldırı/dayanıklılık ve uzak av tahmin eşikleri korunur. Yürütme ve son yetki
çözümleyicidedir. Eşit öğrenilmiş puanda kurallı politikanın tahmini maliyet, mesafe ve kimlik
sırası bağı kararlı çözer; kimlik öğrenilmiş
tablonun anahtarı değildir. En az iki aday varken %10 keşif dünya RNG'sini kullanır.
`update_enabled=false` aynı keşif politikasını ve bölüm muhasebesini koruyup tabloyu
dondurur; böylece rastgele arama ile bilgi güncellemesi ayrı karşılaştırılabilir.

Tablo anahtarı kendi enerji/dayanıklılık oranlarının üç bandı, görünür avın
mesafesinin üç bandı (0–1, 2–3, 4+) ve görünür sağlık sınıfıdır. Sağlık sınıfı
bilinmeyen / bir darbede / iki darbede / daha fazla darbedir. Görünmeyen maksimum
sağlık veya gerçek yaralanma oranı üretilmez. Böylece en çok 108 benzersiz hücre,
bir açık bölüm ve bir son sonuç saklanır. Yaş anahtar değildir; yavruya yeni boş
`HuntLearner::default()` verilmelidir.

Bölüm ilk hedef ve bağlamla başlar; sonradan bağlam veya hedef değiştirilip eski
maliyetler yeni karara yüklenmez. Görünür hedef kaybı ve daha önemli amaca geçiş
ödülsüz kapanır. Kendi öldürdüğü hedefin kimlikli leşini mevcut `Eat` kararıyla
alma aşaması devam edebilir. Her bölüm en çok 20 yürütülmüş tick sürer.

`finish(actor_id, &HuntLedger)` yalnız denetlenmiş çözümleyici muhasebesi alır;
dünya veya çözümleyiciye referansı yoktur. Üst katman actor'un gerçek hareket ve
saldırı enerjisini, aldığı hasarı, bölüm hedefine ait öldüren kimliğini ve yediği
leş kimliğini/gerçek miktarını toplar. Eksik eski olay alanından maliyet veya
öldürme başarısı uydurulmaz; `ambiguous=true` gerekir. Besin kredisi hem kendi
öldürmesi hem yediği leşin hedef kimliğiyle eşleşmesini gerektirir. Başkasının
öldürmesi, ölüm, kesinti, belirsiz muhasebe ve dış yaralanma `reward=None` olur.
Çözümleyici, etkin hedefin saldırısını (kendi avının savunması) diğer
saldırılardan ayırır. Hedef savunması `ambiguous` sayılmaz, ancak mevcut toplam
`incoming_damage` alanında bağlam olarak kalır ve ödüle maliyet eklenmez.
Başka veya atfedilemeyen hasar puansız bırakır. Hasar alt toplamı gelen hasara
eşit değilse muhasebe bozuk sayılır ve puan verilmez. Başkasına ait leşten gerçek
beslenme ekosistemde geçerlidir; bu ilk öğrenici ona kendi av başarısı kredisi
vermez.

İlk sabit ödül hipotezi `10*food - 2*movement_energy - 2*attack_energy - ticks`,
[-1000,1000] aralığına sınırlıdır. Açlık/iyileşme/üreme rezerv farkı ikinci kez
çıkarılmaz. İlk örnek doğrudan hücre değeridir; sonra farkın dörtte biri ile
güncellenir. Bu ağırlıkların ekolojik yararı kanıtlanmış değildir; başarılı
mekanizma testleri yeni uzun kabul sonucu sayılmaz. Tabloyu dondurma kıyası,
enerji/besin/maliyet/yaralanma/başarısız süre metrikleri, bağımsız seedler ve
ayrı karma dünya değerlendirmesi dağıtımdan önce gereklidir.

## İşletim ve kayıt sınırı

Geliştirme kaydı V13'tür; SQLite şeması değişmez. V12 ve daha eski kayıtlardan
geçişte av öğrenicisi kapalı/boş, su öğrenicisinin güncellemesi açık başlatılır.
V13 kaydını eski motorla açmak desteklenmez. Aktif dünyalar ve replay'in frozen
kaynağı otomatik yükseltilmez; yalnız geliştirme testlerinin geçmesi dağıtım
kararı değildir. Yeni bir gözlem dünyası veya açıkça yedekli göç ayrı iştir.

Doğumda ebeveynlerin etkinlik ve güncelleme bayraklarının mantıksal VE'si yavruya
geçer; hücreler, açık bölüm ve son bölüm kaydı geçmez. Oyuncu/dış komut veya
susuzluk gibi üstün ihtiyaç bölümü ödülsüz keser. Gerçek çözümleyici entegrasyon
testleri öldürme→kimlikli öğün kredisi, başka avcının öldürmesine kredi vermeme,
bölüm ortası JSON/checkpoint devamı, bu öncelikler, doğum ve sürüm göçünü kapsar.
Bunlar sınırlı mekanizma/kurtarma kontrolleridir; av öğrenmesinin hayatta kalma
ve üreme yararını veya aylarca ekolojik dengeyi kanıtlamaz. Yeni uzun tarama
başlatılması bu değişikliğin parçası değildir.

Probe --hunt-learning öğreniciyi açıkça etkinleştirir. --frozen-hunt ise yeni
boş tabloyu güncellemeden kullanır; eğitimli tabloyla frozen değerlendirme
yerine geçmez. Doğru eğitimli/frozen av karşılaştırması ayrı deney olarak
yapılmalıdır. Şu ana kadarki testler mekanizma doğrulamasıdır; av öğrenmesinin
yiyecek edinme, hayatta kalma veya üreme üstünlüğü kanıtlanmış değildir.

## V2: eşit puanda kurallı sıralama

İlk aday boş tabloda eşit puanlı avları yalnız mesafe/kimlikle sıralıyordu. Bu,
öğrenme gerçekleşmeden kurallı tahmini av maliyeti tercihini değiştirerek kontrol
kıyasını kirletiyordu. V2 yalnız eşit öğrenilmiş puanda mevcut kurallı
`estimated_cost, distance, id` sırasını korur. Farklı öğrenilmiş puan ve açıkça
yapılan %10 keşif değişmez. V2 sırasında kalan hedef kilidi V3 ile kaldırılır.
Bu düzeltme kısa negatif smoke sonucunu başarı
veya ekolojik kabul olarak yeniden yorumlamaz.

## V3: bölüm muhasebedir, hedef kilidi değildir

V1/V2 açık bölümün hâlâ görünen hedefini otomatik sürdürüyordu. Bu, yalnız aday
sıralama sözleşmesine ek bir takip stratejisi getiriyordu. V3 her `HuntPrey`
kararında aynı Q/epsilon/kurallı maliyet sırasını yeniden uygular. Aynı hedef
seçilirse birikmiş maliyet ve ilk bağlam korunur. Yeni hedef seçilirse eski bölüm
ödülsüz `Interrupted`, eski hedef artık görünmüyorsa `TargetLost` ile kapanır;
yeni hedefe eski maliyet taşınmaz. Kendi öldürdüğü leşe normal `Eat` geçişi
korunur. Ödül, keşif oranı veya süre sınırı değiştirilmez.

## V4: doğrulanmış kendi-bütçe kesintisi ve hasar kaynağı

V1 laboratuvarında passive eğitimli-online kapanışlarının çoğu `TargetChanged`
veya `HuntTooCostly` olarak puansız kalmıştı; defensive ters kolunda avın kendi
savunma hasarı dış saldırı gibi sansürlenmişti. `HuntLowEnergy` veya
`HuntTooCostly` kararında eski hedef hâlâ görünür ve aynı görünür beden/enerji
hesabı hedefin hareket/saldırı bütçesini karşılamadığını doğrularsa eski bölüm
`Interrupted` kapanır. Yalnız önceki ticklerde çözümleyicinin kaydettiği
hareket/saldırı maliyeti varsa ve bölümde belirsiz/dış hasar, kendi öldürüp henüz
yememiş av veya besin yoksa mevcut ödül formülüyle kapanış puanlanabilir. Kesinti
kararının ticki eski bölüme eklenmez. Maliyet doğrulanamıyorsa puan yoktur.

`HuntEnd::Interrupted` wire değeri V13 okuyucularıyla uyum için korunur; tetik
kararı `HuntLowEnergy` veya `HuntTooCostly` olarak laboratuvar auditinde
görünür. `TargetChanged`, hedef kaybı, üstün ihtiyaçlar ve başka kesintiler
puansız kalır; maliyet yeni hedefe aktarılmaz. `HuntEpisode`, `HuntResult`,
checkpoint alanları ve SQLiteV2 değiştirilmemiştir. Eski V13 bölümündeki
`ambiguous=true` bayrağı korunur; tarihsel hasar nedeni yeniden atanmaz ve bu
bölüm yeni kuraldan puan alamaz. Böylece eski ikili kayıt formatını okuyabilir;
yeni kodla yazılan puan anlamı eskiden farklı olabilir, serileştirilmiş sonuç
türü değişmez.

Kapanış puan katsayıları ve keşif oranı değişmez. Kendi avının savunma hasarı
ayrı geçici ledger sayacında izlenir, ancak kanıtlanmış hasar maliyeti
olmadığından ödüle eklenmez. Üçüncü taraf/atfedilemeyen hasar ve alt toplamların
uyuşmaması puansız bırakılır. Bu düzeltme fizik, varsayılan kapalı durumu ve
yavruya tablo aktarımını değiştirmez.

Kısa iki-seed V2 smoke'unda hem çevrimiçi hem boş/dondurulmuş kol kontrolün
avcılarından erken tükendi; ikisi aynı sonucu verdi. Bu, öğrenme yararı değildir.
V3 bir semantik düzeltmedir; önceki olumsuz sonuçları kabul sonucuna dönüştürmez.
Bu mekanizma hâlâ sık hedef değiştirme nedeniyle puansız kapanan çok bölüm,
sadece kendi öldürdüğü avdan ilk öğüne kredi ve dış yaralanmayı belirsiz sayan
muhasebe gibi sınırlara sahiptir. Öğrenilmiş tablonun yararı ayrı eğitimli/frozen
ve bağımsız koşullarda henüz kanıtlanmamıştır. Varsayılan kapalı kalır.
