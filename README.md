# Curious

Curious, Rust ile yazılmış, canlıların algı, beden, bellek, karar ve soy
ilişkilerini gözlenebilir kılmayı amaçlayan bir ekosistem simülasyonudur.
Özellik varlığı veya uzun yaşam tek başına öğrenme ya da ekolojik başarı kanıtı
sayılmaz.

## Güncel durum

Geliştirme kaynağı checkpoint V19 / SQLite V2'dir. Canlı dünya V12 checkpoint
üzerindedir; V19 canlıya yükseltilmemiştir. Nitelikli ikili ve operasyon
durumunun zaman damgalı ayrıntıları [AGENTS.md](AGENTS.md)'de tutulur.

[V2 su uyum ölçümü](docs/reports/WATER_ADAPTATION_V2_20261025.md) üç learner
kolunda aynı fiziksel eylem ve yörüngeleri buldu; ek öğrenme faydası kabul
edilmedi. [Nitelik tanısında](docs/reports/QUALIFICATION_DIAGNOSTIC_20261025.md)
bakımın kısa fixture sinyali, 1.000-tick V12/V19 gölge koşusundaki 1.428 daha
az birey-tick ödünleşimini ortadan kaldırmadı. Canlı fayda veya yükseltme kabulü
yoktur. Tam kaçış refuge riski `Unknown` kalır. Diğer mekanizma ve tarihli
sonuçların kapsamı [ROADMAP](docs/ROADMAP.md)'de özetlenir.

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
