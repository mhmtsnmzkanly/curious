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

- Geliştirme kaynağı checkpoint V19 / SQLite V2'dir. Yeni deneysel seçenekler
  varsayılan kapalıdır. V19 ile uyumsuz eski ikiliyi V19 checkpoint'e açmayın.
- Canlı dünya V12'dir; V19 araştırma kaynağı canlıya yükseltilmiş değildir.
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

- V17 su artığı varsayılan kapalıdır. `WATER_RESIDUAL_V1` raporunda kayıtlı
  bounded utility kapısı iki karşılaştırıcıya karşı başarısızdır; üç seedde
  mekanizma şahidi olması yarar/üstünlük kabulü değildir.
- V13/V14 av deneyimi ve risk alanları ayrı deneysel seçeneklerdir. Hunt Lab V3
  riskli kol yaralanmaları azaltmadı; risk özelliği kapalı tutulur.
- V16 exact-key ve V18 genelleştirilmiş refuge belleği kaynakta default-off'tur.
  25 Ekim tek-run baseline çalışması 623 attempt/67 owner kaydetti: 182
  attributable arrival'ın tümü sıfır hasarlı, altı pozitif threat-damage
  attempt ise sansürlüydü. Bu nedenle route beklenen hasarı her yerde Unknown;
  sıfır MAE başarı veya kalibrasyon değildir. İlk resolver aralığı ayrı raporda
  617 sıfır ve altı 20 hasar gösterir; full-route beklenen risk değildir.
- `REFUGE_RESIDUAL_CANDIDATE` yalnız saf, default-off mekanizma prototipidir;
  World, karar seçimi, checkpoint veya canlıya bağlanmamıştır. Kalibre edilmiş
  route-damage baseline yoktur. Hiçbir risk öğrenicisi veya route faydası kabul
  edilmedi. Neural model veya entegrasyon iddiası kullanmayın.
- Uzun aylar/yıllar soak'u uzun dönem işletim araştırmasıdır; her yeni özellik
  için önkoşul veya mevcut kabul kanıtı değildir. Yeni fiziksel çalışma ancak
  sabit protokol, sınırlı kaynak ve bağımsız denetimle başlar; aynı sonucu
  tekrarlamayın veya sonuçtan sonra eşiği/parametreyi ayarlamayın.
- Dalga 1–6 motor geliştirmeleri ve Wave 7 entegrasyon düzeltmeleri tamamlandı:
  `ca5971b` bağımsız denetiminde tespit edilen sekiz bulgu (F1–F8) düzeltildi.
  Öğrenme/merak döngüsü, ebeveyn bakımı, pusu vuruş hasarı, yerel alarm yayılımı,
  kış termal maliyeti, dinamik in hücre senkronizasyonu ve leş toksisitesi doğrudan
  aktif `World::tick` yürütme yoluna bağlandı; checkpoint sürümü `V19`'a yükseltildi.
  Disabled-path `State+RNG` denkliği ve checkpoint-resume determinizmi tam doğrulanmıştır.
  Epistemik sınır: Mekanizma ve `World::tick` entegrasyonu doğrulanmış olup,
  evrensel çok-habitatlı ekolojik üstünlük kanıtlanmamıştır / kapsam dışıdır
  ([AUDIT_CA5971B_20261025](docs/reports/AUDIT_CA5971B_20261025.md),
  [AUDIT_CA5971B_REMEDIATION_20261025](docs/reports/AUDIT_CA5971B_REMEDIATION_20261025.md)).

Güncel sonuçlar ve tarihçe:

- [Arşiv-bütçe V12 dağıtım raporu](docs/reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md)
- [Refuge fiziksel baseline V1](docs/reports/REFUGE_RISK_BASELINE_V1.md)
- [İlk çözücü aralığı](docs/reports/REFUGE_FIRST_RESOLUTION_V1.md)
- [Önceki teknik tamamlama doğrulaması](docs/reports/REFUGE_REPAIR_VALIDATION.md)
- [ca5971b entegrasyon denetimi (F1–F8 bulguları)](docs/reports/AUDIT_CA5971B_20261025.md)
- [Wave 7: ca5971b entegrasyon denetimi (F1–F8) düzeltme ve V19 doğrulama raporu](docs/reports/AUDIT_CA5971B_REMEDIATION_20261025.md)
- [P2/P3 ampirik protokol ve K8 denetimi (Tarihsel dondurulmuş rapor)](docs/reports/P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)
- [Öğrenme sınırları ve sonraki adımlar](docs/LEARNING_NEXT.md)
- [Güncel roadmap](docs/ROADMAP.md) · [doküman dizini](docs/README.md)

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
