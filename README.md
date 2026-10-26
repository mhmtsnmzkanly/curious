# Curious

Curious, Rust ile yazılmış, canlıların algı, beden, bellek, karar ve soy
ilişkilerini gözlenebilir kılmayı amaçlayan bir ekosistem simülasyonudur.
Özellik varlığı veya uzun yaşam tek başına öğrenme ya da ekolojik başarı kanıtı
sayılmaz.

## Güncel durum

Geliştirme kaynağı checkpoint V20 / SQLite V2'dir. Canlı dünya V12 checkpoint
üzerindedir; V20 canlıya yükseltilmemiştir. Nitelikli ikili ve operasyon
durumunun zaman damgalı ayrıntıları [AGENTS.md](AGENTS.md)'de tutulur.

[V2 su uyum ölçümü](docs/reports/WATER_ADAPTATION_V2_20261025.md) üç learner
kolunda ek öğrenme faydası göstermedi. [Nitelik tanısı](docs/reports/QUALIFICATION_DIAGNOSTIC_20261025.md)
bakım/bakım+sığınak için kısa fixture sinyali ile 1.000-tick gölge
ödünleşimini ayırır; paket kabul edilmedi. [Su sinyali tanısı](docs/reports/WATER_SIGNAL_DIAGNOSIS_20261026.md)
yalnızca uygulanmamış, öneri aşamasındaki yeni K3 hedefini gerekçelendirir.
[Ekoloji aktarım taraması](docs/reports/ECOLOGY_TRANSFER_V1_20261026.md) üç
habitatta sınırlı 600-tick karşılaştırmadır: seyrek kaynaklı kohortlar tüm
kollarda söndü; karma avcı fikstüründe etçiller 15/15 koşuda kayboldu ve bir
alarm koşusunda tüm popülasyon söndü. Genel ekolojik üstünlük kabul edilmedi.
[Çevrimdışı operasyon probu](docs/reports/OPERATIONS_READINESS_20261026.md)
teknik/storage kontrollerini geçti; promotion kapısı kapalıdır ve ölçüm
raporundaki [timing düzeltmesi](docs/reports/OPERATIONS_READINESS_TIMING_CORRECTION_20261026.md)
kabul iddiası oluşturmaz. [Yem arama bütçesi](docs/FOOD_SEARCH_RESERVE.md) V20 kaynağında varsayılan
kapalı uygulandı; [36 koşuluk lab](docs/reports/FOOD_SEARCH_RESERVE_V1_20261026.md) tek yemli ortamda
dar yarar buldu; kalıcı yaşam veya habitat genellemesi göstermedi. Canlı fayda ya da V19/V20 yükseltmesi kabulü yoktur;
tam kaçış refuge riski `Unknown` kalır. Kapsam ve kalan sınırlar
[ROADMAP](docs/ROADMAP.md)'de özetlenir.

26 Ekim 2026'daki önceki paralel inceleme turunun
özeti: [birleştirilmiş sonuç raporu](docs/reports/CONSOLIDATION_20261026.md).

## Yerel çalıştırma

Yeni bir veri dizininde duraklatılmış çayır dünyası başlat:

```bash
cargo run --release -- --serve --paused --scenario scenarios/meadow.json \
  --data-dir data/my-world --bind 127.0.0.1:7878
```

Gözlem ekranı `http://127.0.0.1:7878/` adresindedir. Aynı dünyadan sürdürmek
için `--resume` kullan. Mevcut servis veya veri dizinine dokunmadan önce
[CONTROL](docs/CONTROL.md) sözleşmesini incele. Kayıt, checkpoint ve shard'lar
SQLite üzerinde tutulur.

## Doğrulama

Worktree adına göre ayrı target dizini kullan:

```bash
export CARGO_TARGET_DIR="${TMPDIR:-/tmp}/curious-target-${PWD##*/}"
cargo test --locked --all-targets --jobs 1
cargo build --release --locked --jobs 1
python3 -m unittest discover -s scripts -p 'test_*.py'
```

Yalnızca Markdown değişiklikleri Rust derlemesi gerektirmez.

## Mimari sınırlar

- Beden ve resolver fiziksel maliyetleri ve yaşam sonuçlarını belirler.
- Algı karar anında bireyin görebildiği bilgiyle sınırlıdır.
- Öğrenilmiş bellek bireyseldir, sınırlıdır ve yavruya aktarılmaz.
- Kalıtsal özellikler deneyim belleğinden ayrıdır.
- Eksik, sansürlü veya atıfsız sonuçlar `Unknown` kalır.
- Arayüz kaydedilmiş olguları ve belirsizliği gösterir; okumak dünyayı
  değiştirmez.

## Belgeler

- [Güncel durum, K1–K8 yol haritası ve özgün 11 aşama](docs/ROADMAP.md)
- [Belge dizini ve kanıt bağlantıları](docs/README.md)
- [Dokümantasyon standardı](docs/DOCUMENTATION_STANDARD.md)
- [Çalışma ve operasyon kuralları](AGENTS.md)
- [Kontrol ve işletim](docs/CONTROL.md)
- [Öğrenme sözleşmesi ve açık kanıt sınırları](docs/LEARNING_NEXT.md)
- [Sıradaki işler](TODO.md)
