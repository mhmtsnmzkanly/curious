# Kalan ekosistem ve yetenekler için paralel çalışma planı

**Güncel durum (24 Ekim 2026):** A'nın V17 water-residual formal çalışması
tamamlandı. Frozen source `506ed6f` main `c855154` içindedir; V17 default-off
kalır. Calibration seed 52099 1.400 tickte 8/8 fırsat kapısını geçti (20 prefix,
35 trained-online suffix fresh choice). Formal seedler 52001–52008 toplam
11.200 tick ve 164.554.619 byte raw ile tamamlandı; 32 focal branch 500 tick
yaşadı. Root 32 physical vector'ı ve üç full normalized state/perception/body/
RNG greedy-score/resolver witness'ını doğruladı: 52002/247, 52003/415,
52006/241.

Preregistered bounded utility **FALSE**. Trained-online − rules: acquired water
−9, paid movement energy +32, terminal energy +4, terminal water −9,
health/survival 0. Trained-online − frozen: acquired water +5, movement energy
−13, terminal energy −3, terminal water +5, health/survival 0. Rules karşısında
8/8 tradeoff; frozen karşısında bir dominance, iki tradeoff, beş identical.
Genel fayda, qualification, deployment veya retune iddiası yoktur; yeni batch
yoktur. Dokuz resmi seed rolü artık seen. Ayrıntı: [WATER_RESIDUAL_V1.md](reports/WATER_RESIDUAL_V1.md)
ve [WATER_RESIDUAL_V1.json](reports/WATER_RESIDUAL_V1.json).

B refuge-schema 13 sentetik check, C resource/lineage audit 9 check ve 13
physical World tick ile tamamlandı; ikisi de schema/measurement prototipidir,
production integration veya fitness kabulü değildir. B/C'nin sonraki adımı root
kararıdır. C body energy unknown bırakır; detail-row cap 2.000'dir, ancak
identity/event koleksiyonlarına total-memory guarantee sağlamaz. Seed123
engineering twin 2×1.400 tick/1.400 matched frame'dir; formal evidence değildir.
Root combined checks: 211 Rust, 140 Python, release probe/app passed. Bu formal
lab için process/pending wake yok; coexistence/replay servisleri aktif bırakıldı.
Root 967.6 MiB logical inactive target cleanup yaptı; experiment artifacts
korundu. Gelecek A hipotezi context-stale residual/current-baseline redundancy
mekanizmasını yeni preregistration ile sınamalıdır. Aşağıdaki ilk-dalga notları
handoff tarihindeki durumdur.

## İlk dalga teslimi ve mevcut doğrulama

- **A — su politikası:** [`PARALLEL_WATER_DESIGN.md`](PARALLEL_WATER_DESIGN.md),
  [`water_candidate.rs`](../examples/water_candidate.rs) ve
  [`water_candidate.rs` helper](../examples/support/water_candidate.rs).
  Bunlar ilk prototipin tarihsel araçlarıdır (`769f683`). Root review sonrası
  production integration adayı `src/entity/learning.rs` ve resolver-atıf helperı
  `src/systems/water_outcomes.rs` içindedir; tam `i64 sum/u32 count` kullanır.
  Odak testleri 18/18, genetics inheritance testi 2/2 geçti. Core politika /
  checkpoint entegrasyonu tamamlandı; formal ölçüm hâlâ bekliyor.
- **B — sığınak bağlamı:** [`PARALLEL_REFUGE_DESIGN.md`](PARALLEL_REFUGE_DESIGN.md),
  [`refuge_candidate.rs`](../examples/support/refuge_candidate.rs) ve
  [`refuge_candidate.rs` testleri](../tests/refuge_candidate.rs).
  Root isolated prototype'i inceledi; ajan 11 targeted kontrol bildirdi.
  Bağımsız root `cargo test --all-targets` sonucu 181 passed / 0 failed olup
  A'nın 7 ve B'nin 11 testini içerir. Eski kayıtlı çalışmada 94 çoklu-sığınak
  kararının hiçbirinde tamamlanmış bağlam eşleşmesi yoktu; bu calibration
  sorusudur, geçmişte başarı veya yeni kullanıcı onayı değildir.
- **C — fixture/throughput:** [`PARALLEL_ECOLOGY_DESIGN.md`](PARALLEL_ECOLOGY_DESIGN.md)
  ve [`ecology_candidate.rs`](../examples/ecology_candidate.rs). Targeted
  `cargo check` geçti; runtime/muhasebe validasyonu yapılmadı.
- Root final doğrulaması tamamlandı: `cargo test --all-targets` 191 passed / 0
  failed (39 summaries), Python 132/0, incremental release build exit 0,
  `scripts/check_control.py` HTTP/control/auth/crash-resume exit 0 ve scoped
  Rust format check passed. Root disposable target build outputlarını temizledi;
  retained formal binary yoktur.

Henüz A/B/C için registered ecology batch'i, yeni seed koşusu veya deployment
yapılmadı. A'nın core integration adayı biyolojik sonuç veya kabul değildir.

## Mevcut kanıt ve sınır

Başlangıç noktası temiz `8d23ea5` kaynağıdır. Kabul edilen 32ddfb2 biyolojisi,
fixed ve both kapılarında aynı 30 seed için 30/30 × 100.000 tick sürmüştür;
bu sınırlı, sabitlenmiş koşullarda soy devamı kanıtıdır. Bununla birlikte
ekoloji-traits çalışmasında 80 dünyanın hiçbirinde üreyen yavru-soy sonuca
ulaşmadı; sabit genotip kollarında mutasyon yoktu ve çevrim kolu özellikle
zayıf kaldı. Bu sonuç genetik avantajı sıralamaz. Yeni iş önce başarısızlığın
fixture kaynaklı mı yoksa özellik/bedel kaynaklı mı olduğunu ayırmalıdır.

Önceki su çalışması aynı bireyde güncellenen tablonun sonraki seçimi değiştirdiği
ve resolver fiziksel farkı oluşturduğu tek bir örnek verdi; 8 seedin yalnız
birinde bütün fiziksel bileşenler aynı anda iyiydi, güçlü kural tabanlı kolu
geçmedi. Sığınak protokolündeki 94 çoklu-sığınak kararının hiçbirinde tamamlanmış
bir öğrenilmiş bağlam eşleşmesi yoktu; tekrarlı ve karşılaştırılabilir sonuç
yetersiz olduğundan etki belirlenemedi. Av risk V3 fayda göstermedi. Bu sonuçlar
yeni hipotezleri daraltır; özelliklerin genel yararlı veya gereksiz olduğunu
kanıtlamaz.

Hedef altı capability araştırma izi ve onlardan bağımsız bir kapasite izi olarak
tanımlanır. Her capability opt-in kalır;
öğrenilmiş bilgi bireyseldir, motor maliyetleri resolver veya metabolizma
sınırında uygulanır. Bir izin başarısı diğer iz için kabul yerine geçmez.

## İzler ve sahiplik

| İz | Soru / somut çıktı | Bağımlılık | İlk kabul kanıtı |
|---|---|---|---|
| **A — Su değerini mevcut koşulla birleştirme** (`water_policy_wave1_luna6`; V17 source integration adayı hazır) | Her görünür aday için güncel `foraging::value(amount, path, body)` tabanı + bireyin own-source exact residual sum/count ortalaması. Choice-time baseline episode boyunca tutulur; candidate censoring ve <=32 konum sınırı vardır. | Root formal preregistration, held-out seed audit ve güçlü rule baseline. | Aynı durum/algı/RNG'de online ve aynı tablosu dondurulmuş seçimi greedy biçimde ayrışır; resolver fiziksel fark izler. Rule-based, naive, online ve trained-frozen kolların ölçütleri ayrı raporlanır. Entegrasyon testleri bunu henüz göstermiş sayılmaz. |
| **B — Sığınak bağlamını genelleme; sonra av seçimi** (`refuge_policy_wave1_luna6`) | Kalıcı karar bağlamında tehdit ID'si ve tam alternatif listesi anahtar olmaktan çıkar; tehdit ID'si yalnız resolver hasarını atfetmekte tutulur. Bağlam görünür threat geometry, subject budget, habitat ve candidate route özelliklerinden kurulur. Uyumluluk eşiği henüz seçilmez; belirsiz destekte mevcut içgüdüye dönüş. | Mevcut algı ve resolver-atıf sözleşmeleri; sansürlü olayları bilinmeyen tutmak. | Uyumlu bağlamda bireyin kendi tamamlanmış, güvenli-varış kanıtı karar sırasını değiştirir; uyumsuz tür/habitat/geometri kanıtı reddedilir. Av öğrenmesi sığınak kabulüne karıştırılmaz ve ayrı sonraki preregistrasyon gerektirir. |
| **C — Sürdürülebilir fixture, habitat ve kalıtsal bedeller** (bu planın sahibi) | Kabul edilmiş 32ddfb2 tabanını tekrarlanabilir başlangıç olarak kullanarak sürdürülebilir fixture/throughput tasarımı. Kurucular, F1/F2, üreyen yavru-soyları, yaşayan üreyen soy ve doğal yok oluş ayrılır. Habitat kaynak şoku ve tür dağılımı için sınırlı matris tasarlanır. | Önce fixture geçerliliği; sonra D/E değişkenleriyle ayrı faktöriyel çalışmalar. | Temiz prototipte kaynak arzı/tüketimi, kurucu ve yavru-soy demografisi, maliyet istek/ödeme farkı doğrulanır. Sonra kökün preregistrasyonu ile held-out başlangıç biyolojisi testi; var olan kabul seedleri yeniden “yeni” sayılmaz. |
| **D — Sonlu ve birleştirilebilir eylem/yetenek parçaları** | Küçük tasarlanmış katalog: yönelme/kaçınma, dinlenme, su/yiyecek arama, rota takibi ve sınırlı sosyal koordinasyon gibi mevcut eylem bileşimleri. Yeni kod/strateji kendiliğinden doğmaz. Birey primitifleri deneyimle seçebilir; kalıtsal repertuvar değişimi ayrı, açıkça tasarlanmış gene/allele bağlıdır. | A/B ve C'nin karar, algı, maliyet, bellek, resolver sınırları. | Her primitif için uygun algı girdisi, izinli niyet, resolver doğrulaması, maliyet, sonlu kullanım belleği, devre dışı eşitliği ve nedensel davranış örneği. Sosyal davranış yalnız görünür komşu/iletişim olayıyla sınırlı. |
| **E — Kalıtımsal varyasyon ve seçim** | Sabitlenmiş sonlu allele/primatif repertuvarında varyasyonun üreme başarısıyla sıklık değiştirmesini ölçmek. Yeni strateji ortaya çıkabilir; karmaşıklık artışı veya sonsuz yenilik hedef/kabul garantisi değildir. Yenilik ölçümü (ör. önceden kayıtlı davranış/allele kombinasyonu) betimleyicidir; karmaşıklığı artırmaya zorlayan hedef değildir. | C'de yaşayan ve üreyen yavru-soy kapasitesi; D'de beden maliyetinin ölçülebilir olması. | Başlangıç allele sıklıkları, soy/aile, alele aktarımı/mutasyon, fenotip, kaynak ve enerji bedeli, üreme katkısı ayrı ölçülür. Sabit özelliğin tek-genotip kolu seçilim kanıtı değildir. Başarılı olmayan özellik “avantajlı” ilan edilmez. |
| **F — Aynı birey-deneyim ve ekolojik sonuç kanıtı** | Karar→niyet→resolver→sonuç→bellek zincirini tek kayıtta izleyip öğrenmenin bireysel fiziksel etkisini soy ve habitat sonuçlarından ayıran bütünleyici, küçük kanıt. Buna salt-okunur belgesel UI dahildir. | A/B ölçüm kontratları ve C fixture'ı; D/E ancak ayrı ayrı kabul edildiğinde. | Eşlenik başlangıç/çatallar, karar girdisi, öğrenme tablosu, fiziksel maliyet/sonuç, checkpoint devamı, doğum ve torun ölçümü. UI, aynı bireyin önceki gözlenen episode'unu sonraki değişmiş kaynak/rota kararına bağlar, gerçek maliyetleri ve bilinmeyeni gösterir, olay kronolojisinde resume/replay sağlar. Eksik alanlar unknown kalır; sistem düşünce/niyet uydurmaz. |
| **G — Bounded throughput, performans ve arşiv bütçesi** (sonraki, çapraz kesen iz) | Sabit kısa horizonlarda 10/100/1.000 aktör gibi ölçek noktalarında CPU, RSS, tick hızı, SQLite/HTTP maliyeti, olay ve arşiv bayt büyümesini ölçmek; kota, duraklama, retention/archival tier tasarımı. | Temiz benchmark fixture ve gözlenebilir eş iş yükü; C'den ekolojik iş yükü profili. | Kısa sabit tick ve donanım bütçesinde reproducible karşılaştırma; disk/CPU/RAM sınırında güvenli durma ve yazma hatası davranışı. Geçmiş, kullanıcı politikası olmadan silinmez. Bu çalışma aylar/yıllar reliability soak'u değildir. |

### Kapsam dışı

Aylar/yıllar süren reliability soak'u bu planın parçası değildir; zaten çalışan
uzun süreli izleme değiştirilmez. Buna karşılık 10/100/1.000 aktör gibi sonlu
ölçeklerde sabit kısa benchmark, disk/SQLite/HTTP throughput ölçümü ve bütçe
güvenli durma tasarımı G izinde kapsamdadır. Sonsuz nüfus, sınırsız dosya sistemi
büyümesi veya bütün habitatlarda hayatta kalma garantisi verilemez. Belirli
habitat ve önceden tanımlı şoklarda her türün
yaşamı garanti edilmez: doğal yok oluş geçerli sonuçtur ve teşhis edilmelidir.
Kurtarma amacıyla canlı eklemek, kaynakları nüfusa bakarak artırmak, üreme
bedelini gizlice kaldırmak veya yok oluşu başarıdan saymak yasaktır.
“AGI”, genel zekâ, açık uçlu evrim, garantili karmaşıklık veya sürekli yeni
strateji üretimi iddiası yoktur. Yeni stratejiler ancak sonlu, tasarlanmış
primatif kataloğu ve ölçülebilir bedellerle denenecektir.

## Dalga sırası ve karar kapıları

1. **İlk dalga — A/B/C paralel tasarım ve izole prototip.** A su skor taslağını,
   B sığınak genelleme yardımcı fonksiyonunu, C sürdürülebilir habitat/profil
   ölçüm taslağını kendi worktree'sinde yürütür. Ortak `src/`, checkpoint,
   SQLite veya protokol dosyaları değiştirilmez. A/B/C'den hiçbirinin formal
   koşusu bu iş kapsamında başlatılmaz. C, ek özellik başarısı iddia etmeden
   fixture nedenini teşhis edecek ölçümleri ve kaynak/besin throughput
   bütçesini somutlaştırır.
2. **Kök incelemesi ve tekil birleştirme.** A'nın V17 source entegrasyonu ve
   root migration/disabled/resume test kapıları tamamlandı; B/C ortak
   `src/`/checkpoint değişiklikleri yine root'un seri incelemesiyle yapılır.
   Her aday için temiz kaynak, migration, kapalı mod State+RNG eşitliği ve
   targeted test gerekir. Entegrasyon tek başına biyolojik kabul değildir.
3. **Her iz için protokol ve sınırlı kalibrasyon.** Hipotez, sabit fixture,
   arms, seed rolleri, eşik, örnek boyutu, tick/duvar zamanı/bellek/çıktı
   bütçesi, durdurma koşulu, ham şema ve analiz formal seed görülmeden önce
   kök tarafından sabitlenir. Kalibrasyon ölçüm/fixture geçerliliğini ölçer;
   parametre arama veya eşik seçme için kullanılamaz. İlk resmi batch için ayrı
   kök preregistrasyonu ve inceleme gerekir.
4. **Fiziksel ve soy kabulü.** Ödül/öğrenme tablosu değil; edinilen kaynak,
   ödenen enerji, yaralanma/sağlık, ölüm, yaşam süresi, doğum, F1/F2 ve
   gerçekten üreyen yavru-soyları ayrı raporlanır. Başlangıç kurucusu sayısı,
   doğum toplamı veya son anda yaşayan kurucular tek başına fitness değildir.
5. **İzler arası entegrasyon ve G benchmark planı.** F yalnız A/B'nin bireysel nedensellik kapıları
   ve C'nin fixture/ölçüm kapıları geçtikten sonra başlar. D/E değişiklikleri
   F'ye birden eklenmez; her yeni yetenek için ayrı kontrat ve ablation gerekir.
   “Özellik birlikte daha iyi” ancak önceden belirlenmiş birleşik karşılaştırma
   ile söylenebilir. G için herhangi bir sweep öncesi root ölçüm planı, sabit
   makine/ayar, workload, actor-count ve disk/CPU/RAM/çıktı limitlerini
   preregistre eder; bu plan bu turda geniş performans sweep'i başlatmaz.

## Ölçüm ve durdurma ilkeleri

- Her seed/arm durumunu ve aralıklarını göster; bir başarıseed'i genel üstünlük
  yapma. Pareto dışı sonuçları ağırlıklı puanda gizleme.
- Kaynak döngüsü için başlangıç/son stok, yenilenme, tüketim, tür başına
  tüketim ve kapasite doygunluğu kaydedilir. “Arz/throughput yeterli” demek için
  yalnızca haritada çok kaynak bulunması yetmez.
- Kök/dal/kuşak kimliği açık olmalı; F1 doğumu ile F1'in kendisinin üremesi
  farklı metriklerdir. Yavruların öğrenilmiş belleği boş kalır.
- Gelecek/hidden state, RNG'nin ilerideki çıktısı veya başka aktörün gizli
  fizyolojisi karar girdisi olamaz. Resolver'ın reddi, enerji kesintisi,
  belirsiz atıf veya lab sınırı ölçümde sansürlenir.
- Her protokolde doğal yok oluş ve çıkış kuralı bulunur. Bir kol yok olunca
  devam eden boş tickler sabit horizon için kaydedilebilir; yeni kurucu yoktur.
- Hash/seed/state uyumsuzluğu, yanlış maliyet muhasebesi, ham çıkış/bütçe
  taşması, schema bozulması veya canlı veri yoluna erişim halinde işi durdur;
  partial raw kaydı koru. Biyolojik olumsuz sonuç runner hatası değildir.

## Dosya ve sorumluluk sınırı

Bu plan, [`PARALLEL_ECOLOGY_DESIGN.md`](PARALLEL_ECOLOGY_DESIGN.md) ve A/B
bağlantıları ilk dalga teslimidir. Shared `src/`/checkpoint entegrasyonu, formal
protokoller, canlı servisler, release'ler, deney arşivleri ve root commitleri
ayrı root sahipliği ve incelemesine tabidir. Ana dalda `AGENTS.md`, `ROADMAP.md`
ve `TODO.md` içine güncel durum için kısa handoff eklendi. Kabul edilen 32ddfb2
kanıtı tarihsel bağlamdır; yeni kaynak/protokol kimliği veya seed listesi olarak
kullanılmaz.
