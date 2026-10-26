# Su–öğün önceliği: ham kayıt tanısı

**Durum:** Tamamlanmış V1 koşusunun salt-okunur, keşif amaçlı yeniden çözümlemesi. Yeni World çalıştırılmadı; yeni kabul eşiği veya nedensel kabul yoktur. `water_meal_priority_enabled` varsayılan kapalı kalır; canlı V12 değiştirilmedi.

## Soru ve kapsam

V1 kıyasındaki habitat/seed sağkalım farklılıkları, doğrudan müdahale sonuçları, bireysel sağkalım, üreme/soy değişiklikleri ve ayırt edilemeyen sonraki etkiler bakımından ayrıştırıldı. Girdi, V1 raporundaki 72 World × 300 tick koşusunun ham kaydıdır. Bu yeniden çözümleme yalnız mevcut JSONL, resolver olayları, soy kayıtları ve raporlanmış durumları okudu; World, checkpoint ya da canlı servis çalıştırmadı.

Önceki raporun fiziksel sonuçları bu raporda yeniden değerlendirilir, değiştirilmez. Ham koşu: `data/experiments/water-meal-priority-v1-runnerfix-20261026/`; manifest SHA-256 `db0a490b5f3a62ed9c0f9b8db72b9d5851d1659356d3423dcdec256d9d80134c`. Dünya kaynağı revizyonu `edb76316d7002f0533d4c038b664a796ec293c41`, kaynak içeriği SHA-256 `d55d6b1d4fb27ec1abc87d088a8d7616c1dad96a442c7144e9cc3a4f6a075d5a`; kayıtlı ikili SHA-256 `81d6044ca3f838234643d072e23d07ce4856f70953d91c0642e783883e1f28e0`; protokol SHA-256 `2b602b2dc294b49585dc2235c669af80d598479090774fce9390ee92bd0a6e4b`. Salt-okunur analiz/probe geçmişi `41b75e9caf78c035f34b86198b774c8d8c594629`, `e7132ffc8aaadb6e55d467bc16e4e046ec8e3ce8` ve son probe kaynağı `f17e10d236a55a804329cd34bf5c02d73fee572e`; son betik `scripts/analyze_water_meal_negative_outcomes.py` SHA-256 `ea7a26605aed9cb86b26223ad51b39dbf5358879565de06a05b70c7d4162bbf4`. Root'un `5ee9d330b2560237da7554426ebeeee81e77f3f6` üzerinden bağımsız yeniden üretimi aynı analizi ve JSON SHA-256 `ee8f8004067debad67e84b5a22e9205cf04ee525da5b0d9018e82d37fbe62f40` çıktısını doğruladı. Bu hash ile bu raporun [JSON eşlikçisi](WATER_MEAL_PRIORITY_DIAGNOSIS_20261026.json) byte-byte aynıdır; manifestle doğrulanan 72 ham JSONL girdisinin adı, SHA-256 ve byte boyutu eşlikçide korunur.

Karşılaştırmalar seed 62501–62506'nın üç fixture'daki tekrarlı kullanımıdır; 18 fixture-seed bloğu, 18 bağımsız seed değildir. Buradaki ana kontrast `water_meal_only − control` olup her kol altı World ve World başına 300 tick içerir.

## Sonuçlar

| Fixture | Yaşayan birey-tick, W−C | Bitki öğünü enerjisi, W−C | Su, W−C | Ödenen hareket enerjisi, W−C | Tick 300'de canlı World, W−C |
|---|---:|---:|---:|---:|---:|
| Tek-yem stresi | 638 − 638 = 0 | 578 − 578 = 0 | 190 − 190 = 0 | 591 − 591 = 0 | 0/6 − 0/6 |
| Çayır | 25.242 − 25.028 = **+214** | 49.166 − 48.130 = **+1.036** | 27.760 − 27.588 = **+172** | 17.171 − 16.560 = **+611** | 6/6 − 6/6 |
| Seyrek kaynak | 5.865 − 5.871 = **−6** | 5.182 − 5.180 = **+2** | 3.336 − 3.366 = **−30** | 3.111 − 3.103 = **+8** | 0/6 − 0/6 |

Tek-yem fixture'ında yeni seçenek su-öğün müdahalesi üretmedi ve kontrolle aynı kaldı. Çayırdaki toplu artış, her seed'de artış anlamına gelmedi: seed bazında yaşayan birey-tick farkları `+266, −106, −173, +268, −167, +126`; altı World de ufukta canlıydı. Seyrek kaynak fixture'ında altı World'ün tamamı iki kolda da t300'den önce söndü. Birleşik ölçüler kısa ufukta heterojen sonuç gösterir; dayanıklı sağkalım veya genel ekolojik fayda göstermez.

### Seçilen öğün ile gerçekleşen öğün

Water-meal-only kolunda 120 müdahale denemesi kaydedildi: çayırda **69/73** deneme resolver tarafından kabul edildi (526 enerji), 4'ü `MissingResource` ile reddedildi; seyrek kaynakta yalnız **11/47** kabul edildi (22 enerji), **36/47 (%76,6)** `MissingResource` oldu; tek-yem fixture'ında deneme yoktu. `water_meal_only` kolundaki toplam, 80 başarılı ve 40 reddedilmiş girişimdir. Seçilmiş/görünür öğün miktarı gerçekleşmiş alım değildir; gerçekleşen enerji yalnız `EntityAte` resolver kaydıyla sayıldı.

Örnek olarak seyrek kaynak seed 62503'te actor 5, t55–63 arasında dokuz ardışık seçimde görünür komşu bitki öğününü seçti; dokuzunun tamamı `Eat/MissingResource` ile reddedildi. Aynı hedefte diğer actor'ların önceki resolver tüketimleri kayıttadır. Actor t64'te gerçekten 2 enerji aldı; t65'te 1 hareket enerjisi ödeyip `Approach/OutOfRange` aldı. Seed 62505 actor 6 için t90–93'te dört öğün seçimi reddedildi; t94'te 2 enerji alımı gerçekleşti. Bu olaylar anlık kaynak rekabetini gösterir. Kontrol kolunda aynı koşulda alternatif Drink'in başarılı olacağını göstermez; bu nedenle reddedilen öğünün geciken/kaçırılan suya veya ölüme neden olduğu sonucu çıkarılamaz.

### Resolver sırası ve ilk müdahale

Kaynakta `src/world/mod.rs:236,437–450`, her tick `Eat` plan sırasını `tick % plan.len()` kadar döndürür. Su kararının `Eat` ile değiştirilmesi o tick'teki `Eat` vektörünün üyeliğini/uzunluğunu değiştirebildiğinden, aynı hücredeki öğün yarışının kazananı değişebilir. Eşleştirilmiş ilk müdahale tick'lerinde bu doğrudan gözlendi:

- Çayır seed 62502, t94: aynı hücrede kontrol actor 24'e 8 enerji verirken, müdahaleli kolda actor 19'a 8 enerji verdi; override actor 7 başka hücrede 6 enerji aldı.
- Seyrek kaynak seed 62505, t65: aynı hücrede kontrol actor 3'e, müdahaleli kol actor 4'e 2 enerji verdi.
- Seyrek kaynak seed 62503, t55: sıra `[3,4,7,9]` iken `[7,9,3,4]` oldu; o tikte başarılı alıcılar/miktarlar aynı kaldı ve override actor 5'in öğün denemesi reddedildi. Bu örnek sıra değişikliğinin tek başına fiziksel fark garantilemediğini gösterir.

Bu ilk tick örnekleri aynı tick'te kaynak tahsis mekanizmasını doğrular; sonraki nüfus farklarının tek açıklaması olduklarını doğrulamaz. 2 enerjilik alımın pasif iyileşme döngüsü başlattığı da desteklenmiyor: metabolizma karar/çözümlemeden önce çalışır ve pasif iyileşme için enerji en az azami enerjinin dörtte üçü olmalıdır. Anılan tekrar eden seçimlerde sorun çoğunlukla başarılı beslenme değil, kaynak bulunmadığı için reddedilmedir.

## Bireysel iz ve nedensel sınır

Odaklı negatif eşleşmelerde ilk müdahaleye kadar iki koldaki toplam yaşayan birey-tick aynıdır: çayır seed 62502/62503/62505 için sırasıyla 1.423/1.358/923; seyrek kaynak seed 62503/62505 için 537/602. Müdahale anından ufka kadar farklar çayırda −106/−173/−167, seyrek kaynakta −8/−1'dir. İlk resolver olay listesi ayrışması müdahale tick'inde gerçekleşmiştir.

Başlangıçtaki kurucu bireylerin toplam yaşam maruziyeti ile popülasyon toplamının geri kalanını ayrıca ayırınca, bu beş negatif seed'de farkların çoğu kurucu grupta görünmez:

| Fixture / seed | Kurucu birey-tick farkı | Yavru/sonraki nesil birey-tick farkı | Toplam fark |
|---|---:|---:|---:|
| Çayır 62502 | 0 | −106 | −106 |
| Çayır 62503 | 0 | −173 | −173 |
| Çayır 62505 | +14 | −181 | −167 |
| Seyrek kaynak 62503 | −1 | −7 | −8 |
| Seyrek kaynak 62505 | −1 | 0 | −1 |

Kurucu grubu başlangıçtaki founder ID'leridir; `death_lineage` kaydına göre ölüm tick'inden önceki (`death_tick−1`) canlı tick'ler sayılır, ufukta yaşayanlar ufka kadar sağdan sansürlüdür. İkinci grup, toplamdan kurucu maruziyeti çıkarılarak elde edilir; sonraki nesil bireyleri kollar arasında aynı offspring kimliğiyle eşleştirmez. Bu ayrım çayırdaki üç negatif toplamın iki seed'de kurucularda fark olmadığını, birinde kurucuların +14 iken sonraki nesil maruziyetinin −181 olduğunu gösterir. Neden veya aracılık kanıtı değildir.

Daha sonra soy/doğum ve ölüm kayıtları da ayrışır; bu sıralama bir mekanizma zinciriyle uyumludur, ancak aradaki aynı-birey karşı-olgusunu tanımlamaz.

Odak birey ölümü, neden ve tick `death_lineage` kaydında bulunabilir; doğum ebeveynleri ve nesli `birth_lineage` ile izlenebilir. Ancak non-focal actor'lar için tam resolver sonrası beden/enerji durumu ham kayıtta yoktur. Kollar ilk dünya/eylem farkından sonra aynı birey durumu ve seçim dizisi olarak devam etmez; sonraki aynı entity ID tek başına birey eşleşmesi sayılmaz. Doğrudan müdahale→ölüm aracılığı, sapmanın yaşam boyu sonucu ve habitat toplamındaki farkın nedeni **Unknown** kalır. Ayrıca t300'de yaşayan bireyler bu ufukta sağdan sansürlüdür.

## Yorum ve sonraki dar ölçüm

İki yakın mekanizma kanıtlandı: görünür ortak öğün, resolver'a gelene kadar başka actor tarafından tüketilebilir; ayrıca müdahalenin `Eat` plan kümesine eklenmesi deterministik sıralamayı değiştirip aynı tick kaynak kazananını değiştirebilir. Kaynakta bu davranışla çelişen doğrulanmış bir uygulama kusuru bulunmadı. Bu yerel mekanizma ve 300 tick'lik habitat ölçümleri, nüfus kaybının genel nedensel açıklaması değildir.

Önerilen sonraki çalışma, **yeni öğrenici veya politika tuning'i olmadan**, ilk müdahale öncesi aynı durumdan yeniden oynatılacak dar bir resolver ablasyonudur. Mevcut ham kayıt tüm actor'ların müdahale öncesi tam intent/plan listesini içermediğinden, bu listeyi ve kaynak stokunu yakalayan yeni probe gerekir. Dondurulmuş aynı durum ve plan listesinden (a) baseline su planı/doğal döndürme, (b) suyu öğüne çevirip override actor'ı sabit bir sıraya koyma, (c) aynı override plan kümesini doğal `tick % plan.len()` döndürmesiyle çözme karşılaştırılır. (b) ile (c) aynı plan kümesini tuttuğu için sıra rotasyonunun etkisini ölçer; (a) ile (b) ise sabit sırada plan üyeliği/intent değişimini gösterir. Bu tek-tick ölçüm yaşam/soy nedenselliğini çözmez; bireysel yaşam iddiası için tüm aktörlerin karar-sonrası bedenini ve sonraki kararlarını kaydeden ayrı tasarım gerekir. Bu rapor deney başlatmaz veya formül/default değişikliği önermez.

## Kanıt ve doğrulama

- [V1 kıyas raporu](WATER_MEAL_PRIORITY_V1_20261026.md) ve [sabit V1 protokolü](../WATER_MEAL_PRIORITY_V1_PROTOCOL_20261026.md) asıl koşunun kayıtlı tasarımını ve sonuçlarını taşır; değiştirilmemiştir.
- Keşif analizi JSON'u bu raporun eşlikçisidir; detaylı mekanizma incelemesi `data/operations/water-meal-diagnosis-20261026/review.json` ve intake sayımı `root-intake-audit.json` içinde kalır. Bu ek operasyon dosyaları yerel/ignore edilen ham kanıttır ve kaynak deposuna eklenmemiştir. Probe/test doğrulama commit'i `f17e10d236a55a804329cd34bf5c02d73fee572e` üzerinde yazar ve root `python3 -m unittest discover -s scripts -p test_analyze_water_meal_negative_outcomes.py -v` komutunu 8/8 geçirdi; analiz çıktısı değişmedi.
- Bu belge yazarı World/build/test çalıştırmadı. Root'un salt-okunur analiz tekrarında aşağıdaki CLI kullanıldı; unittest discovery komutu 8 testi geçti. Rust build ve yeni World çalıştırması yapılmadı. Markdown bağlantıları ve `git diff --check` geçti.

```sh
python3 scripts/analyze_water_meal_negative_outcomes.py \
  --input /root/curious/data/experiments/water-meal-priority-v1-runnerfix-20261026 \
  --output /root/curious/data/operations/water-meal-diagnosis-20261026/diagnosis.json

python3 -m unittest discover -s scripts \
  -p test_analyze_water_meal_negative_outcomes.py -v
```

**Birincil roadmap alanı:** K2 karar/beden sınırı; çapraz bağımlılıklar K5 kaynak rekabeti ve K6 üreme/soy ölçümü. Bu rapor yeni bir backlog veya kabul kararı oluşturmaz.
