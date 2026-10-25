# Curious

Curious, Rust ile yazılmış, canlıların algı, beden, bellek, karar ve soy
ilişkilerini gözlenebilir kılmayı amaçlayan bir ekosistem simülasyonudur.
İhtiyaçlara göre davranış, bireysel deneyim ve kalıtsal değişim ayrı ölçülür;
bir özellik ya da uzun yaşam tek başına başarı kanıtı sayılmaz.

## Güncel durum — 25 Ekim 2026

- V17 su-artığı, av-risk öğrenmesi ve V18 genelleme gibi deneysel seçenekler
  varsayılan kapalıdır. Bu, temel bellek/su davranışlarının veya canlı V12
  genetiğinin tümünün kapalı olduğu anlamına gelmez. Canlı 7882 dünyası V12
  checkpoint üzerinde, yalnızca arşiv bütçesi düzeltmesini içeren
  `curious-observation-budget-8dc23de` ikilisiyle çalışır. 7883 replay ikilisi
  dondurulmuştur; ana V18 dünyası canlıya yükseltilmedi.
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
- [Belge dizini ve kaldırılan planların politikası](docs/README.md)
- [Kontrol ve işletim](docs/CONTROL.md)
- [Öğrenme sözleşmesi](docs/LEARNING_NEXT.md)
- [Arşiv bütçesi düzeltmesinin canlı dağıtım kanıtı](docs/reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md)
- [İlk resolver aralığı sonucu](docs/reports/REFUGE_FIRST_RESOLUTION_V1.md)
- [Tüm raporlar](docs/reports/)
