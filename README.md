# Curious

Basitleştirilmiş bir dünyada canlıları, kararlarını ve nesillerini gözlemlemek
için geliştirilen Rust simülasyonu. Hedef belgesel gibi izlenebilen, deneyimden
öğrenen ve kalıtsal özellikleri değişebilen bir ekosistemdir.

Şu anda deterministik motor, yenilenen kaynaklar, üç tür, yaşam döngüsü,
bireysel hafıza, su seçimini öğrenme ve bedelli bir hareket özelliği için kalıtım/
mutasyon var. **Kalıcı avcı–av dengesi ve aylar süren dayanıklılık henüz kanıtlanmadı.**

## Çalıştırma

```bash
cargo run --release -- --serve --paused --scenario scenarios/meadow.json \
  --data-dir data/my-world --bind 127.0.0.1:7878
# Aynı veriden devam: --resume veya --resume-or-new ekle.
```

`http://127.0.0.1:7878/` görsel gözlem ekranını açar. Birey seçimi, karar/
sonuç geçmişi, hafıza, öğrenilmiş su tercihleri, nesil/ebeveyn, pause/adım/hız
kontrolleri bulunur. `CURIOUS_TOKEN` ayarlanırsa veri ve yönetim uçları anahtar
ister. Çalışan dünya yanlışlıkla yeni senaryoyla ezilmez.

SQLite her tickin durumunu, kararlarını, olaylarını ve müdahalelerini kaydeder;
dosya boyutuyla shard geçişi, kayıpsız arşiv ve checkpointten devam desteklenir.

## Ayrımlar

- `map`, `environment`: arazi ve nüfustan bağımsız kaynak döngüsü.
- `entity`: beden, algı, kurallı davranış, hafıza, öğrenme ve genom ayrı modüller.
- `systems`: niyet toplama, planlama, fiziksel maliyet/çakışma, yaşam döngüsü.
- `simulation/checkpoint`: sürümlü dünya durumu; SQLite'tan bağımsız.
- `persistence`, `control`, `web`: kayıt, yönetim ve gözlem.

Yeni yeteneğin eylem maliyeti/dünya doğrulaması motorda, karar tercihi politikada
olmalı; öğrenme dünyaya doğrudan kaynak yazmaz. Bireysel deneyim ve kalıtım ayrıdır.

## Doğrulama

```bash
cargo test --all-targets
cargo build --release
python3 scripts/check_control.py target/release/curious
python3 -m unittest discover -s scripts -p 'test_*.py'
# Playwright kurulu ortamda:
node scripts/check_ui.cjs target/release/curious
```

Hedef: [VISION](docs/VISION.md). Açık işler: [ROADMAP](docs/ROADMAP.md).
Kullanım/işletim: [CONTROL](docs/CONTROL.md). Kanıt ve sınırlar:
[11 başlığın sonuçları](docs/reports/ROADMAP_RESULTS.md).
Ajanlar için sunucu, bildirim, commit ve temizlik kuralları: [AGENTS](AGENTS.md).
