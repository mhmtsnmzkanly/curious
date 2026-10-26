# Curious — güncel çalışma talimatları

Bu dosya geçmiş günlüğü değil, bugünkü çalışma sözleşmesidir. Tarihsel deney,
dağıtım ve karar kanıtları ilgili raporlarda korunur; eski kayıtları silmeyin.

## Ürün amacı ve mimari sınırlar

- Kullanıcının hedefi belgelenebilir bir yaşayan dünya ve her bireyin kendi
  deneyiminden oluşan özel bellektir. Öğrenme için yaş bonusu veya yaşlılık
  ödülü eklemeyin. Yaş ve hayatta kalan bireylerin seçilimi ayrı ölçülür.
- Öğrenilmiş bellek bireye aittir; yavruya kopyalanmaz. Yeni birey boş/varsayılan
  bellekle başlar. Genetik aktarım ile deneyim belleğini karıştırmayın.
- Dünya, beden, algı, politika/içgüdü, öğrenici, fiziksel çözücü ve kalıcı durum
  ayrı sorumluluklardır. Öğrenici yalnız gördüğü ve tanımlanmış deneyiminden
  aldığı bilgiyi kullanır; gizli beden, gelecekteki RNG veya çözücü dışı
  sonuçlara erişmez. Hareket, saldırı, maliyet ve yaşam sonuçlarında son söz
  fiziksel çözücü ve dünyadadır.
- Sonuç bileşenlerini fiziksel birimleriyle ayrı tutun. Kapanmamış, dış hasarlı,
  ilişkilendirilemeyen veya eksik gözlemli bölümü başarı ya da sıfır maliyet
  saymayın; sansürlü/bilinmeyen olarak kaydedin. Ödül toplamı tek başarı ölçüsü
  değildir.

## Kaynak sürümleri ve canlı çalışma

- Geliştirme kaynağı checkpoint V21 / SQLite V2'dir. Yeni deneysel seçenekler
  varsayılan kapalıdır. V21 checkpoint'i eski V12/V20 ikiliyle açmayın.
- Canlı dünya V12'dir; V19–V21 araştırma sürümleri canlıya yükseltilmiş değildir.
  Canlı servis `curious-coexistence.service`, `127.0.0.1:7882`,
  `/root/curious/data/coexistence` üzerinde çalışır. Nitelikli ikili
  `bin/curious-qualified` şu an `bin/releases/curious-observation-budget-8dc23de`
  dosyasına çözülür; SHA-256
  `35930838981679ab1b937324df6dda73d2b3bcc6c57706d69f2c11d2803959af`.
- V12 canlı dünyasındaki temel water learning ve mevcut genetik yaşam
  mekaniği, V17 su-artığı ve V19 deneysel seçeneklerinden ayrıdır.
  Birbirinin bayrak/default durumunu varsaymayın; canlı ve geliştirme
  checkpoint'lerini gerçek State/API'den doğrulayın.
- 25 Ekim 2026 04:03 UTC salt-okunur gözleminde 7882 durumunda tick 11057,
  567 entity / 557 canlı, 0.03 tick/s, 64 MiB shard ve 32 GiB **soft** history
  bütçesi, 2,833,337,708 history byte, `storage_error=null`, `paused=false`
  görüldü. Bu zaman damgalı anlık görüntüdür; sonraki görevde tick veya servis
  durumunu buradan varsaymayın, gerçek `/status` ve systemd durumunu okuyun.
- Salt-okunur arşiv okuyucu 7883'te `curious-replay.service` ile çalışır;
  25 Ekim gözleminde PID 1137 ve GET arayüzü erişilebilirdi. Bu arşiv
  okuyucusudur, 7882 dünyasını duraklatmaz. Üç timer etkin:
  `curious-coexistence-monitor.timer`, `curious-coexistence-sampler.timer` ve
  `curious-history-retention.timer`. 7878–7881 eski servislerini yeniden
  başlatmayın. 7882, 7883, veri dizinleri ve frozen ikililer yalnız açıkça
  yetkilendirilmiş, yedekli bir dağıtım göreviyle değiştirilebilir.
- Arşiv politikası 7 günlük saklama, kapalı sıkıştırılmış shardlarda 3 GiB
  hedef ve koşu başına en çok 2 sharddır. Bu toplam disk kotası değildir; aktif
  shard, WAL, ham kayıt, en yeni shard ve hata/uyumsuz arşivler korunur.
  V12 backport'u cache hesabı yeni yazmayı reddedecekse kapalı shard
  metaverisini yeniden tarar; sürekli disk kotası değildir ve WAL/geçici arşiv
  alanını kapsamaz. Ayrıntı dağıtım raporundadır.

## Öğrenmenin mevcut kanıt sınırı

- [Su uyum V2](docs/reports/WATER_ADAPTATION_V2_20261025.md): gıda-zengin
  fixture geçerlidir; üç learner kolu aynı fiziksel yörüngeyi verdi. Ek
  deneyim faydası kabul edilmedi. Seed 62206'nın prefix desteği düşük olup
  öğrenme kabulü için yetersizdir. [V1](docs/reports/WATER_ADAPTATION_V1_20261025.md)
  başarısız kalibrasyonu korunur.
- [Nitelik tanısı](docs/reports/QUALIFICATION_DIAGNOSTIC_20261025.md): seed
  401–410 aynı fixture'da bakım/paket kollarında 2 iyileşme, 8 nötr sonuç
  verdi; refuge kolu 10/10 nötrdü. V12 başlangıcından 1.000 tick'te paket
  V19 kapalı kontrole göre endpoint'te 13 daha fazla canlı, fakat toplamda
  1,428 daha az birey-tick üretti. Canlı fayda/Pareto kabulü yoktur.
- V17 su artığı ve V13/V14 av-risk seçenekleri default-off'tur; kayıtlı
  bounded utility / Hunt Lab sonuçları yarar kabulü değildir.
- Refuge tam-rota beklenen hasarı `Unknown`: full-route raporunda altı
  pozitif hasarlı bölüm sansürlüydü. İlk resolver aralığındaki tamamlanmış
  etiketler ayrı hedef ufkudur; route predictor veya kabul edilmiş öğrenici
  değildir. `REFUGE_RESIDUAL_CANDIDATE` World/karar/checkpoint yoluna
  bağlanmamış saf prototiptir.
- [Su sinyali tanısı](docs/reports/WATER_SIGNAL_DIAGNOSIS_20261026.md), V2'de
  mevcut context residual faydası görülmediğini; [kaynak güvenilirliği
  sözleşmesi](docs/SOURCE_RELIABILITY_TARGET_20261026.md) ise yalnızca önerilmiş
  K3 hedefini belirtir. Bu hedef uygulanmadı, test edilmedi veya kabul edilmedi.
- [Ekoloji transfer V1](docs/reports/ECOLOGY_TRANSFER_V1_20261026.md) sınırlı
  taramasında seyrek kaynaklı dünyalar tüm kollarda söndü; karma habitatta
  etçiller tüm koşullarda kayboldu, alarm koşullarından birinde tüm popülasyon
  söndü. Genel ekolojik kol üstünlüğü kabul edilmedi.
- [Enerji/paket tanısını](docs/reports/ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.md) izleyen
  [yem arama bütçesi](docs/FOOD_SEARCH_RESERVE.md) V20 kaynağına varsayılan kapalı
  eklendi. Bu bir enerji harcama sınırıdır; dönüş veya yemek garantisi değildir.
  [36 koşuluk lab](docs/reports/FOOD_SEARCH_RESERVE_V1_20261026.md) tek yemli
  fixture'da 4/6 seedde dar yarar buldu; tüm bireyler yine öldü ve diğer iki
  fixture'da kural tetiklenmedi. Genel fayda kabulü yoktur. Bakım+sığınak
  paketi davranışsal fayda kabulü değildir.
- [Offline operasyonel smoke](docs/reports/OPERATIONS_READINESS_20261026.md)
  V12→V19 migration, State/RNG ve izole depolama kontrollerini geçti; promotion
  izni vermez. Eski p50/p95 alanları geçersizdir; ayrıntı [düzeltme
  notunda](docs/reports/OPERATIONS_READINESS_TIMING_CORRECTION_20261026.md),
  orijinal koşu artifaktları ise değiştirilmeden korunur.
- Mekanizma testleri, P3 ablasyon ve Wave 7–10 yeniden denetimleri kod yolu ve
  sınırlı protokolleri kapsar; evrensel ekolojik üstünlük göstermez. Ayrıntı
  [ROADMAP](docs/ROADMAP.md) ve bağlantılı tarihli raporlardadır.
- Yeni deney aynı sonucu tekrarlamamalı veya sonuçtan sonra eşiği ayarlamamalı;
  sabit protokol, sınırlı kaynak ve bağımsız gözden geçirme gerektirir. Uzun
  gerçek-zamanlı soak ayrı işletim araştırmasıdır, özellik çalışmasının
  önkoşulu değildir.

- [Su öncesi öğün](docs/WATER_MEAL_PRIORITY.md) V21 kaynağında bağımsız varsayılan
  kapalı seçenektir. Yalnız ödenemeyen ilk su adımı yerine mevcut bitişik bitki
  öğününü seçer; kesilen su denemesi ödül/başarı sayılmaz. Kayıtlı kıyas ve
  genel fayda kabulü ayrıdır.

## Güncel sonuçlar

Tarihsel denetim ve soak ayrıntıları [docs/README.md](docs/README.md) üzerinden
bulunur; bu dosya güncel kuralları ve operasyonel gereklilikleri taşır.

## Kod, deney ve operasyon çalışma usulü

- İşe başlamadan ilgili bağımsız skill'i okuyun: [skill kataloğu](.agents/skills/README.md).
  Beş skill model/sağlayıcıdan bağımsızdır ve kullanıcı talimatını geçersiz
  kılamaz.
- Belge oluşturma/güncelleme için
  [curious-documentation](.agents/skills/curious-documentation/SKILL.md) ve
  [belgeleme standardını](docs/DOCUMENTATION_STANDARD.md) izleyin.
- Ana kod sahibi olarak Luna6 kaynak/probe değişikliklerini kendi worktree'sinde
  yapar; root kaynakları, sınırları ve test kanıtını gözden geçirir. Bu iş bölümü
  kullanıcı tarafından değiştirilebilir.
- Her worktree için ayrı `CARGO_TARGET_DIR` kullanın ve `--jobs 1` ile çalışın.
  Worktree'ler arasında paylaşılan Cargo target V12/V18 cache'lerini karıştırdı.
  Testi yalnız değişikliğe uygun ölçekte çalıştırın; dokümantasyon-only işte Rust
  derlemesi yapmayın.
- Frozen protokolü, hash'i, ham veriyi, manifesti veya eski başarısız sonucu
  sonradan değiştirmeyin. Düzeltme/tekrar gerekiyorsa yeni kimlikli deney ve yeni
  rapor oluşturun; orijinal kanıt başarısız kaldığı gibi gösterilir.
- Servis veya checkpoint işlemi öncesi gerçek process, binary, State/RNG,
  SQLite ve yedekleri inceleyin. Deployment için temiz kaynak, değişmez binary
  hash'i, kapalı ve doğrulanmış yedek, migration, State/RNG devam testi ve geri
  dönüş yolu zorunludur; yetkiyi mevcut oturumdan değerlendirin.
- Yalnız çalışmadığı doğrulanmış, göreve ait build cache'ini temizleyin. Önce
  `rustc`/`cargo` süreçlerinin olmadığını ve target'ın başka worktree tarafından
  kullanılmadığını kanıtlayın. Ham deney, arşiv, index, backup, frozen source ve
  `bin/releases` ikililerini silmeyin.
- Commit yalnız anlamlı, gözden geçirilebilir bir birim tamamlanınca yapılır.
  Push ancak kullanıcı bu konuşmada açıkça yetkilendirirse yapılır. Başka
  kişilere Slack/e-posta/harici mesaj göndermeyin.
- Notify yalnız anlamlı kilometre taşında ve tek hedef
  `u_e049630155fc` için kullanılabilir. Token dosyası `/root/token.json`;
  içeriğini açmayın, yazdırmayın, kopyalamayın veya rapora koymayın.

## Bakım noktaları

- `World::with_seed` varsayılan olarak `Logger::disabled()` kullanır; birim ve
  entegrasyon testleri `logs/simulation_*.log` sızıntısı yapmaz (dosyaya loglama
  yalnız `World::enable_timestamped_file_logger()` ile açıkça başlatılır).
- Bildirim, seed, deney ve karar kanıtı için tarihsel raporları tutun; bu giriş
  güncel kuralları kısa tutar. Ek araştırma planı [docs/ROADMAP.md](docs/ROADMAP.md)
  içindedir.
- Arşiv restore edilmeden eski okuyucu/validator çalıştırmayın; kayıpsız archive
  index ve SHA'yı doğrulayın. Başarısız run'a archive/repair sonucu ekleyerek
  geçmişini yeniden yazmayın.
- Her görevde çalışma ağacını ve gerçek servis durumunu tekrar doğrulayın.
  Bu talimatlardaki tarih damgalı tick, PID ve boyutlar daha sonraki çalışmanın
  canlı durumu sayılamaz.
