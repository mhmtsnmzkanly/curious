# Resolver sıra laboratuvarı V1 protokolü

**Kimlik:** `resolver-order-v1-20261026`  
**Durum:** Dondurulmuş dar mekanizma protokolü. Sonuç, genel fayda ya da ekolojik kabul eşiği tanımlamaz. Protokol hash'i, kaynak ve ikili hash'leri ile girdilerin hash'leri herhangi bir World çalıştırılmadan önce yeni koşu kaydına yazılır.  
**Sahip:** K2 karar/beden sınırı; bağımlılıklar K5 kaynak tahsisi ve K6 soy ölçümü.

## Soru ve kapsam

Su–öğün önceliği tanısında ilk müdahale anında görülen aynı-tik öğün kazananı değişiminde, `Eat` plan kümesi ve `Drink` sıralamasındaki doğal rotasyonun payını aynı ön-durum ve aynı karar girdileriyle ayırmak. Bu ölçüm yalnızca belirtilen beş reddedilmiş-öğün olayına ait tek resolver tiklerini kapsar. Yaşam, üreme, genel politika yararı, ekoloji transferi veya canlıya alma iddiası değildir.

Girdi kaynağı değiştirilemez V1 ham koşusudur: `/root/curious/data/experiments/water-meal-priority-v1-runnerfix-20261026/`. Kaynak kayıt manifest SHA-256'sı `db0a490b5f3a62ed9c0f9b8db72b9d5851d1659356d3423dcdec256d9d80134c` olmalıdır. Manifest girdilerinin adı, SHA-256'sı ve byte boyutu koşu kaydına kopyalanır; veri bulunamaz veya manifest doğrulanamazsa çalışma başlamaz. Olay tanısı ve beş hedefin tanımı [WATER_MEAL_PRIORITY_DIAGNOSIS](reports/WATER_MEAL_PRIORITY_DIAGNOSIS_20261026.md) içindedir.

| Fixture | Seed | Hedef tik | Başlangıç kaynağı |
|---|---:|---:|---|
| meadow | 62502 | 94 | Aynı seed'in eski `control` ham kaydı |
| meadow | 62503 | 92 | Aynı seed'in eski `control` ham kaydı |
| meadow | 62505 | 72 | Aynı seed'in eski `control` ham kaydı |
| sparse_resource | 62503 | 55 | Aynı seed'in eski `control` ham kaydı |
| sparse_resource | 62505 | 65 | Aynı seed'in eski `control` ham kaydı |

Her hedefte ham `control` JSONL header'ındaki tam V21 `initial_state` ve RNG geri yüklenir; dosyada tick-başına tam `State` veya RNG checkpoint'i yoktur. World, sıradan `World::tick()` ile hedef tikten önceki tik sonuna kadar deterministik olarak replay edilir. Her prefix tick'in elde edilen resolver olayları ve kaydedilmiş fiziksel/stok özetleri eski `control` JSONL satırlarıyla karşılaştırılır; hedef öncesi yeniden kurulan `State`/RNG yakalanır ve restore/recapture round-trip'inde eşitlik zorunludur. Bu, tarihsel ara-tik tam State/RNG eşitliği kanıtı değildir; ham kayıtta böyle bir karşılaştırma değeri yoktur. Yeniden seed ederek yaklaşık aynı dünya kurmak kabul edilmez. `water_meal_priority_enabled` ve diğer politika bayrakları ham `control` değerlerinde kalır. Kararlar/planlar için politika bayrağını açan ikinci bir World çalıştırılmaz.

## Ortak hazırlık ve üç kol

Her hedef için tam bir ortak hazırlık yapılır: hedef tikin normal zaman ilerletmesi, çevre ve beden/metabolizma güncellemeleri, algı ve normal niyet/karar toplama. Önceden hazırlanmış görünür girdiler ve kaydedilmiş temel karar izleriyle baseline planları bir kez kurulur. Aday override kararları saf dönüşüm/yardımcı işlevle aynı görünür girdilerden türetilir; yeniden algı, metabolizma, politika seçimi veya RNG çekimi yapılmaz. Karar/intent ve her plan vektörü rotasyon öncesi ve sonrası kaydedilir. Kollar B/C için learner interruption ve audit alanları çözümleme öncesi aynı olmalıdır; bunlar aynen kaydedilir.

Hazırlık tamamlandığında aynı ortak `State`/RNG, karar izleri, niyetler, doluluk haritası ve çözücü öncesi beden/envanter kaydedilir. Her kol bu aynı noktadan bağımsız geri yüklenir ve aynı üretim çözücü kuyruğunu bir kez çalıştırır. Kollar arası çözücü RNG çekimleri kendi koluna aittir; her kol aynı başlangıç RNG'sini alır. Bir kolun çözücü sonucu diğer kolun başlangıcına taşınmaz.

| Kol | Karar/plan üyeliği | Plan sırası | Ne ile karşılaştırılır |
|---|---|---|---|
| A — baseline | Doğal temel kararlar | Her plan türünde üretim rotasyonu (`tick % len`) | Kontrol davranış referansı |
| B — aday/sabit yerleşim | Aday override planları | `Eat`: A'da kalan actor'ların döndürülmüş sırasını koru, yeni `Eat` actor'larını artan ID ile sona ekle. `Drink`: A'daki döndürülmüş sıranın kalanlarını koru. Diğer plan türlerinde A sırasını aynen tut. | A–B: aday plan üyeliği ile burada önceden belirlenmiş yerleşimin bileşik farkı |
| C — aday/doğal rotasyon | B ile aynı aday karar/plan üyeliği | Her plan türünde aday vektörlerine üretim rotasyonu (`tick % len`) uygula | B–C: aynı plan üyeliğinde ortak plan rotasyon düzeninin (özellikle `Eat` ve `Drink`) farkı |

Bu kontrastlar algoritmik olarak tanımlı müdahalelerdir. A–B yalnız saf “karar değişikliği” etkisi değildir; yeni `Eat` actor'larının belirtilen konuma yerleştirilmesini de içerir. B–C, `Eat` rotasyonunu tek başına yalıtmaz: `Drink` dahil tüm plan vektörlerinin doğal döndürülmüş sırasını B'deki kurala göre değerlendirir. A–C aday karar/plan üyeliği ile tüm doğal rotasyonun toplam bir-tik farkıdır. Üç kolun hiçbirine genel veya benzersiz bir karar-nedensel yorumu atanmaz.

## Kaydedilecek ölçümler

Her hedef ve kol için şunları sakla:

- Tam hedef-öncesi yeniden kurulmuş `State` ve RNG, ortak hazırlık sonrası `State` ve RNG, kol başlangıç restore doğrulaması ve kol sonrası tam `State`/RNG.
- Tüm canlı actor'ların ortak hazırlıktaki beden/metabolizma sonrası girdileri, görünür algı/gözlem girdileri, karar/niyetleri, karar gerekçeleri; kolun aday karar/intent farkları ve karar öncesi/sonrası beden alanları.
- Her plan türünün (`Move`, `Eat`, `Drink`, `Mate`, `Attack`, `Flee`, `Sleep`) rotasyon öncesi/sırası, actor ID'leri ve gerekli kaynak/hedef, yol, maliyet alanları.
- Çözücü öncesi/sonrası her actor bedeni (enerji, su, sağlık, stamina, konum, canlılık/faz ve ilgili sayaçlar); çevre kaynak stoku; her resolver olayı ve actor/konum/miktar/maliyet alanları; tüm fiziksel olay vektörleri.
- Her kol için olay sayısı ve türü, `EntityAte` ile alınan bitki enerjisi ve `EntityDrank` su miktarı, ödenmiş hareket enerjisi, reddedilen `Eat`/`Drink` ve nedenleri; ayrıca üç kol arasındaki actor bazlı olay farkları.

Birincil mekanizma çıktısı her hedefte `EntityAte` alıcı kimliği ve miktarı ile `Eat`/`Drink` sırası ve resolver stok değişimidir. Her kolun her son-çözücü alanı betimsel olarak raporlanır; fiziksel toplamlar birimler halinde verilir. Beş hedef ayrı tutulur ve destek `n=5` eşleşmiş tek-tik vakadır. Ortalama, p-değeri, pass/fail bileşik puanı veya nüfus projeksiyonu tanımlı değildir. Sonuçlar 300-tik yaşam/soy farklarını doğrulamaz.

## Replay denetimi, kaynak ve çalışma bütçesi

Kaynak tabanı, başlangıç revizyonu `e424a45` (`curious` V21) ve başlangıçta kullanılacak resolver davranışının kaynak içerik SHA-256'sı `d55d6b1d4fb27ec1abc87d088a8d7616c1dad96a442c7144e9cc3a4f6a075d5a` olarak kaydedilir. Uygulama commit'i, temiz kaynak ağacının SHA-256'sı, protokol SHA-256'sı, önceden derlenmiş laboratuvar ikilisinin SHA-256'sı, senaryo/hash'leri, ham girdi manifesti/hash'leri, komut satırı ve koşu kimliği ilk çalıştırmadan önce koşu kaydına yazılır. Bu protokol bir ikili hash'i henüz sabitlemez; run registration sırasında üretilip ilk World çalıştırmasından önce kaydedilmelidir. World çalıştırdıktan sonra kaynak, protokol, ikili, seed, input veya eşik değişirse o koşu geçersizdir; yeni kimlikli yeni protokol ve kayıt gerekir.

Her hedefte hedef öncesi ortak `State` iki ek referans çalışması için de kullanılır: biri `water_meal_priority_enabled=false`, diğeri `true` ayarıyla olağan `World::tick()` çalıştırır. Kol A, `false` olağan tick'i; kol C, `true` olağan tick'i eşlemelidir. C deney kolunun politika bayrağı kapalı tutulabilir; açık davranış için karşılaştırma native bayrak açık olağan tick'tir. Olay listesi, RNG, `last_intents`, karar izleri ve State alanları tam eşit olmalı; yalnız `water_meal_priority_enabled` ayarı State/iz karşılaştırmasında normalize edilir ve bu normalizasyon raporda görünür tutulur. Eşleşmeme, aday dönüşümün normal bayrak açık yürütmeyle aynı olmadığını gösterir ve koşuyu fail-closed durdurur; bunu yeni bir sonuç sayıp devam etmek yasaktır. A baseline kolu ayrıca hedef tikin ve önceki replay'in eski `control` ham kaydındaki gözlenen olay/stok/fiziksel toplam başlıklarıyla karşılaştırılır. Eski ham kayıt ara-tik tam State/RNG içermediğinden replay edilen ticklerde yalnız ham kayıtta mevcut olay ve metrik alanları doğrulanır. Eşleşmeyen girdi, tick, olay veya kayıtlı toplam, üç kolu çalıştırmadan önce fail-closed durdurma sebebidir. Bir eşleşme yalnız replay'in bu girdilerde tutarlılığını gösterir; B için karşı-olgusal ham kayıt sağlamaz.

Replay üst sınırı: ham header `initial_state` değerinden hedef öncesi duruma toplam 373 World-tik replay adımıdır (`93+91+71+54+64`). Her hedefte A/B/C toplam 15 resolver-tik çözümü, ayrıca bayrak kapalı/açık iki olağan `World::tick()` referansı (10 tick) çalışır; toplam **398 çözücü-tik yürütümü**. Beş hedef için birer ortak preamble ayrıca hazırlanır ve bu beş kısmi tik adımı 398 çözücü-tik sayımına dahil değildir. İki normal referans, replay ile yeniden oluşturulmuş ve ayrıca restore edilmiş ön-tik State'lerinden yürütülür. Kurulum/restore maliyeti ayrıca duvar saati ve bellek sınırına tabidir. Her koşu en çok 180 saniye; her child process en çok 45 saniye, 64 MiB çıktı ve 1 GiB sanal adres alanı kullanabilir. Süre veya bellek sınırının aşılması altyapı başarısızlığıdır. Canlı servis, canlı checkpoint, World dışı veri kaynağı ve uzun soak kullanılmaz.

## Durdurma, bilinmeyenler ve yorum sınırı

Herhangi bir hash/manifest uyuşmazlığı, replay başlık farkı, restore eşitsizliği, eksik actor girdisi/kararı/planı, karar/plan listesinin beklenmeyen dönüşümü, event/state bütünlük hatası, child timeout, kaynak sınırı ihlali veya eksik çıktı ilk hatada tüm koşuyu durdurur. İlk hata ve kısmi dosyalar değiştirilmeden saklanır; otomatik retry, seed değiştirme, eşik ayarı, vaka çıkarma veya protokol değiştirme yoktur. Doğal kaynak tükenmesi/boş kaynak, reddedilen öğün ya da sıfır fiziksel etki ise altyapı hatası değildir; gözlenen sonuç olarak kaydedilir.

Bu çalışma her actor'ın yalnız bu ortak, tek-tik seçiminden sonraki resolver/finalize sonuçlarını görür. Kol ayrışmasından sonra bir sonraki tik için aktör kararlarının aynı olacağı varsayılmaz ve takip edilmez. B/C sıra kuralının başka plan türleri veya başka dünya/seed/tiklerdeki etkisi, uzun vadeli beden/sağkalım/soy aracılığı, öğrenme veya ekoloji yararı **Unknown** kalır. Beş olay seçilmiş negatif tanı vakalarıdır; rastgele örnek veya genellenebilir nüfus örneklemi değildir. Bu protokol kabul, default değişikliği, deployment veya canlı kullanım yetkisi vermez.

**Kabul ölçütü:** Protokolün başarı eşiği yoktur. Kayıt ancak bütün beş hedef, üç kol, zorunlu bütünlük kontrolleri ve artefakt kimlikleri eksiksizse *tamamlanmış betimsel mekanizma ölçümü* olarak raporlanabilir. Eksik veya erken durmuş kayıt başarısız/eksik kalır; içerik yorumla tamamlanmaz.
