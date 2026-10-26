# Çevrimdışı işletim hazırlığı — 26 Ekim 2026

**Sonuç:** Sınırlı geri yükleme/depolama kontrolü geçti. Canlı yükseltme kapısı kapalıdır (`promotion_allowed=false`). p50/p95 süre alanları hesaplama hatası nedeniyle geçersizdir; ortalama, toplam ve maksimum ölçümleri bu hatadan etkilenmez.

## Kapsam ve kimlik

[K8 protokolü](../OPERATIONS_READINESS_PROTOCOL_20261026.md) tek sabit V12 checkpoint kopyasında yürütüldü. Kaynak `bc60fa60ac060b7cc0dfae75f20fe16df5173461`, temiz çalışma ağacı; input SHA-256 `357870bd9a3d414704ed6befbea050852028f54af304c27cdca482dc31abdad2`. İkili SHA-256 `d65a61908996fe6b71ef11471098c2de748e88fa886549cf036b44f5079c7221`. Kaynak/binary/protokol ve komut kimlikleri `data/operations/consolidation-20261026/operations-registration.json` içinde korunur. Makinece okunabilir özet: [JSON](OPERATIONS_READINESS_20261026.json).

Canlı arşivden daha önce çıkarılmış tick 12639 / 630 slot girdisi salt okunur açıldı; kopya V19'a çevrildi. Yalnız `off` modu koşuldu; bakım ve her iki refuge bayrağı kapalıdır. Diğer modlar için bu raporda çalışma zamanı kanıtı yoktur. 120 ana World tick'i ve ayrı kopyada bir depolama tick'i: toplam **121 fiziksel tick**, ek yeniden başlatma/soak koşusu yoktur.

## Ölçülen sonuç

| Kontrol / ölçüm | Sonuç |
|---|---|
| İlk migration tick / slot korunumu | 12639 / 630, korundu |
| İlk V19 State/RNG restore–recapture | Tam eşit |
| Son checkpoint | V19, tick 12759, 615 slot (canlı sayısı değildir) |
| Son State/RNG restore–recapture | Tam eşit |
| Ana tick toplam süresi | 4.686542 s |
| Ortalama / maksimum World tick | 39.055 / 48.774 ms |
| Tüm süreç tepe RSS | 160,820 KiB / 157.05 MiB |
| Süreç duvar süresi / kullanıcı CPU | 5.23 s / 5.12 s |
| Koşu dizini toplam dosya boyutu | 19,938,734 byte |

Zamanlayıcı yalnız `World::tick` çevresindedir; ana tick süresine checkpoint serileştirme, Store yazımı ve migration dahil değildir. RSS ise bütün süreci, özel depolama kontrolünü ve geçici restore kopyalarını kapsar; sürekli daemon belleği tahmini değildir. Ortalama yaklaşık 25.6 tick/s hesaplama hızına karşılık gelir; bu ölçüm disk yazan üretim TPS garantisi değildir. Yeni karmaşık dünya/nüfus veya uzun süre için ölçekleme eğrisi ölçülmedi.

### Zaman yüzdelikleri düzeltmesi

Probe `percentile()` fonksiyonu örnekleri sıralamadan indekslediği için ham `world_tick_p50=41701 µs` ve `world_tick_p95=35965 µs` alanları geçersizdir. Süre örnekleri ilk koşuda saklanmadığından doğru yüzdelikler sonradan çıkarılamaz: **Unknown**. Ham `run.json` değiştirilmedi, koşu tekrarlanmadı. [Düzeltme notu](OPERATIONS_READINESS_TIMING_CORRECTION_20261026.md) sonraki kaynak düzeltmesi ve saf sıralama testini ayırır.

## SQLite ve arşiv

Özel `storage/` dizininde iki kayıt yazıldı; shard eşiği 1 MiB, history soft bütçesi 8 MiB idi. İlk kapalı shard 187,099 byte zstd; ikinci açık SQLite shard 5,582,848 byte oldu. Bu eşik yazılan checkpoint'i bölen katı kota değildir. Store yeniden açıldı; son State tam eşleşti. İki shard için `PRAGMA integrity_check=ok`; kapalı zstd shard `archive::Reader` üzerinden açıldı. Son kayıt tick 12640'tır.

Bu iki yazım retention silme, uzun süreli WAL büyümesi, history bütçesi baskısı, reboot veya runtime writer kilidi sınaması değildir. 256 MiB çıktı sınırı fazlar arasında kontrol edilir; önleyici disk kotası değildir. Süreç 1 GiB sanal adres alanı ve 120 s sınırında tamamlandı.

## Karar ve korunmuş kanıt

[Önceki nitelik tanısı](QUALIFICATION_DIAGNOSTIC_20261025.md) paket için toplam sağkalım ödünleşimini koruyor. Operasyonel kontrolün geçmesi bu davranışsal kapıyı açmaz. Canlı V12 ve replay servislerine müdahale edilmedi. Canlı geçiş, geri dönüş ve gerçek uyumluluk kontrolleri protokolün runbook bölümündedir; burada uygulanmadı.

Yerel/ignore edilen ham kanıtlar: `data/experiments/operations-readiness-20261026/` (başlangıç/son State, Store ve `run.json`); `data/operations/consolidation-20261026/operations.time.txt`, `operations-exit.json`, `operations-registration.json`, `frozen/`. Bunlar GitHub'a yüklenmiş veri değildir. Kaynak arşivi SHA-256 `2147bd6870545adb3de2e67370bc4d48c018cf830e3519a28d961b7d67bee365` (bc60fa6), ikililer ve önceki başarısız seed-scan kaydı korunmuştur.
