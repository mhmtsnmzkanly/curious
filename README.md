# Curious

Curious, Rust ile yazılmış, canlıların algı, beden, bellek, karar ve soy
ilişkilerini gözlenebilir kılmayı amaçlayan bir ekosistem simülasyonudur.
İhtiyaçlara göre davranış, bireysel deneyim ve kalıtsal değişim ayrı ölçülür;
bir özellik ya da uzun yaşam tek başına başarı kanıtı sayılmaz.

## Güncel durum — 25 Ekim 2026

- V17 su-artığı, av-risk öğrenmesi ve V19 deneysel seçenekleri varsayılan
  kapalıdır. Bu, temel bellek/su davranışlarının veya canlı V12 genetiğinin
  tümünün kapalı olduğu anlamına gelmez. Canlı 7882 dünyası V12 checkpoint
  üzerinde, yalnızca arşiv bütçesi düzeltmesini içeren
  `curious-observation-budget-8dc23de` ikilisiyle çalışır. 7883 replay ikilisi
  dondurulmuştur; ana V19 dünyası canlıya yükseltilmedi.
- Aynı 30 seed, kayıtlı habitat fixture'ında `fixed` ve `both` kollarında
  100.000 tick kapısını geçti. Bu koşullu bir coexistence sonucudur; bütün
  habitatlarda kalıcı denge veya aylar/yıllar dayanıklılığı değildir.
- V17 su artığı seçeneği varsayılan kapalıdır. Kayıtlı bounded utility ölçütü
  **FALSE** çıktı. Habitat-vizyon karşılaştırması karışık betimleyici kanıt
  verdi; tutarlı bir avantaj göstermedi.
- Tam kaçış için beklenen tehdit hasarı `Unknown`. Ayrı ilk-resolver-aralığı
  çıkarımı 623 exact gözlem buldu (617 sıfır, altı 20 hasar, dört birey); bu
  per-route predictor değildir ve öğrenme modeli kabul edilmedi. Tam rota
  sansürü, tamamlanmış ilk aralığın exact etiketini geçersiz kılmaz.
- Aylar/yıllar süren soak kullanıcı isteğiyle ertelendi; sonraki özellik için
  engel değildir.
- Dalga 1–6 motor geliştirmeleri, Wave 7–9 denetim düzeltmeleri ve Wave 10
  kanıt sınırı kapanışı ile P3 ablasyon/K7 telemetrisi tamamlandı: `World::with_seed`
  varsayılan olarak `Logger::disabled()` kullanır (testlerde dosya sızıntısı önlendi);
  `Memory::toward` yerel BFS baypas aramasıyla duvara takılmayı çözer. Bağımsız
  denetimlerde tespit edilen bulgular ve dar kanıt sınırları eksiksiz kapatıldı: su
  bağlam artığı doğrudan `foraging::value` formülü ve legacy residual ile birebir
  aynı ölçekte `(actual - baseline).clamp(-1000, 1000)` hesaplanır, baseline yoksa
  artık üretilmez ve başarısız su bölümlerinde hedef doğrulaması gözetilir (F1);
  pusu örtüsü sıradan avcıların erişebildiği bitki örtüsü ve duvar bitişiğine
  genişletildi (F2); checkpoint `V19` ham V18 JSON göç testiyle doğrulandı (F3);
  kış mevsimi ek BMR maliyeti bağlandı (F4); statik inler `from_map` ile kaydedilip
  kış koruması sağlandı, dinamik in girdileri doğrulanarak kış otonom kazısında
  kapasite doluyken stamina kaybı önlendi (F5); ceset enerjisi ilk tick taze besin
  paritesini koruyup izleyen 3 tick'te kademeli çürümeye ve sıfır besin alan bireyler
  için toksisite muafiyetine bağlandı, `CorpseStage::Putrid` varyant adlandırması
  hizalandı (F6); bağlam artığında i64 toplamı ve sınır doğrulaması sağlandı (F7);
  konfigürasyondan bağımsız dinamik dünya durumu (`DynamicWorldSnapshot`), RNG ayrışması,
  yeni otonom kazılan inlerin (`treatment_newly_excavated_burrows_total`) başlangıç statik
  inlerinden kesin ayrımı ve mükerrer sayım olmaksızın benzersiz yayımlanan sinyallerin
  (`treatment_unique_signals_emitted_total`) pozitif assert'leri kanıtlandı (F8).
  P3 tekil özellik ablasyon matrisi (`tests/p3_multi_habitat_ablation.rs`) ile 3 habitat
  × 5 held-out tohum (`301..=305`) × 7 kolda hedef mekanizmaların tam seçiciliği ve
  izolasyonu kanıtlandı; K7 Web Replay & Experience V19 çevre/beden telemetrisi
  (`#env-telemetry`, `CuriousExperience.environment`) arayüze bağlandı. Mekanizma
  ve `World::tick` entegrasyonu doğrulanmış olup, evrensel çok-habitatlı ekolojik üstünlük
  kanıtlanmamıştır / kapsam dışıdır
  ([ca5971b denetim](docs/reports/AUDIT_CA5971B_20261025.md),
  [Wave 7 düzeltme ve V19 raporu](docs/reports/AUDIT_CA5971B_REMEDIATION_20261025.md),
  [2002ee4 yeniden denetim](docs/reports/AUDIT_RECHECK_2002EE4_20261025.md),
  [Wave 8 tam kapanış](docs/reports/AUDIT_RECHECK_REMEDIATION_20261025.md),
  [6fa9c49 yeniden denetim](docs/reports/AUDIT_RECHECK_6FA9C49_20261025.md),
  [Wave 9 kapanış ve hizalama](docs/reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md),
  [9c7d3f0 denetim](docs/reports/AUDIT_RECHECK_9C7D3F0_20261025.md),
  [Wave 10: P3 ablasyon ve 9c7d3f0 kapanış](docs/reports/P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md),
  [Tarihsel P2/P3 raporu](docs/reports/P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)).

Sekiz kategori ve yedi A–G yetenek izinin durumu ile öncelikleri:
[ROADMAP](docs/ROADMAP.md). Belgelerin girişi:
[docs/README](docs/README.md).

## Yerel çalıştırma

Yeni bir veri dizininde duraklatılmış çayır dünyası başlat:

```bash
cargo run --release -- --serve --paused --scenario scenarios/meadow.json \
  --data-dir data/my-world --bind 127.0.0.1:7878
```

Gözlem ekranı `http://127.0.0.1:7878/` adresindedir. Aynı dünyadan sürdürmek
için `--resume` kullan. Bir servisin ya da mevcut veri dizininin üzerine
yazmadan önce [CONTROL](docs/CONTROL.md) sözleşmesini incele. Simülasyon
kayıtları, checkpoint ve shard işlemleri SQLite üzerinde tutulur.

## Doğrulama

Worktree adına göre ayrı target diziniyle:

```bash
export CARGO_TARGET_DIR="${TMPDIR:-/tmp}/curious-target-${PWD##*/}"
cargo test --locked --all-targets --jobs 1
cargo build --release --locked --jobs 1
python3 -m unittest discover -s scripts -p 'test_*.py'
```

Yalnızca Markdown değişiklikleri Rust yeniden derlemesi gerektirmez.

## Mimari sınırlar

- **Beden:** sağlık, enerji, metabolizma ve ifade edilmiş fiziksel maliyetler.
- **Algı:** karar anında bireyin görebildiği sınırlı bilgi; gizli dünya durumu
  karar girdisi değildir.
- **Bellek ve öğrenme:** bireye özgü, sınırlı deneyim. Hedef ufku içinde
  sansürlü veya belirsiz sonuçlar bilinmeyen kalır; öğrenme yavruya aktarılmaz.
- **Karar:** algı, beden ve bireysel bellekten niyet üretir.
- **Resolver:** eylemi doğrular, çatışmayı çözer, maliyeti öder ve sonucu
  kaydeder.
- **Kalıtım:** üreme sırasında sonlu allel/özellik aktarımıdır; öğrenilmiş
  bellekten ayrıdır.
- **Kayıt ve arayüz:** simülasyon durumundan bağımsız gözlem, kontrol ve
  dayanıklı geçmiş sağlar.

## Belgeler

- [Durum, öncelikler ve kabul kapıları](docs/ROADMAP.md)
- [Belge dizini](docs/README.md)
- [Dokümantasyon standardı](docs/DOCUMENTATION_STANDARD.md)
- [Kontrol ve işletim](docs/CONTROL.md)
- [Öğrenme sözleşmesi](docs/LEARNING_NEXT.md)
- [Arşiv bütçesi düzeltmesinin canlı dağıtım kanıtı](docs/reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md)
- [İlk resolver aralığı sonucu](docs/reports/REFUGE_FIRST_RESOLUTION_V1.md)
- [ca5971b entegrasyon denetimi (F1–F8 bulguları)](docs/reports/AUDIT_CA5971B_20261025.md)
- [Wave 7: ca5971b entegrasyon denetimi (F1–F8) düzeltme ve V19 doğrulama raporu](docs/reports/AUDIT_CA5971B_REMEDIATION_20261025.md)
- [2002ee4 düzeltmelerinin bağımsız yeniden denetimi](docs/reports/AUDIT_RECHECK_2002EE4_20261025.md)
- [Wave 8: 2002ee4 yeniden denetim bulgularının (F1, F2, F5, F6, F8) tam kapanış raporu](docs/reports/AUDIT_RECHECK_REMEDIATION_20261025.md)
- [6fa9c49 düzeltmelerinin bağımsız yeniden denetimi](docs/reports/AUDIT_RECHECK_6FA9C49_20261025.md)
- [Wave 9: 6fa9c49 yeniden denetim bulgularının kapatılması ve tam hizalama raporu](docs/reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md)
- [9c7d3f0 düzeltmelerinin bağımsız denetimi](docs/reports/AUDIT_RECHECK_9C7D3F0_20261025.md)
- [Wave 10: P3 ablasyon ve 9c7d3f0 kanıt sınırı kapanış raporu](docs/reports/P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md)
- [P2/P3 ampirik protokol ve K8 denetimi (Tarihsel dondurulmuş rapor)](docs/reports/P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)
- [Tüm raporlar](docs/reports/)
