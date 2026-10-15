# 10 — Nüfus ve harita ölçekleme

Release motorunda aynı seed ile 10/100/500/1.000 kurucu, 16/32/64 kenarlı harita,
30 tick ve iki yük: kaynak olmayan başlangıç; besin aranan başlangıç. Nüfus
sabitlenmez; doğum/ölüm sürer, sonuçtaki nüfus JSON'dadır. **SQLite/HTTP hariç**
motor süresi; kalıcı kayıt hızıyla karıştırılmamalı. Bu tek sunucu/tek kısa seri,
mutlak kapasite/SLO garantisi değildir.

Darboğaz: her birey için algıda bütün nüfusu yeniden taramak. Artık tick başına
bir konum indeksi kurulur; büyük nüfusta yerel adaylar alınır. 256'dan az bireyde
basit tarama korunur. Adaylar eski birey sırasına konur, HashMap sırası davranışı
etkilemez. 500 ticklik dünya/RNG checkpointi optimizasyon öncesi ve sonrası
**bayt bayt aynı**. Bütün hedef testleri de geçti.

| Kurucu nüfus | Başlangıç yükü | Önce p95 | Sonra p95 |
|---:|---|---:|---:|
| 500 | kaynaksız | 21,0 ms | 8,4 ms |
| 500 | besin arama | 22,8 ms | 11,1 ms |
| 1.000 | kaynaksız | 70,9 ms | 19,3 ms |
| 1.000 | besin arama | 84,5 ms | 29,5 ms |

1.000 kurucuda RSS high-water 7,5–7,7 MiB; son checkpoint 613–671 KB. Bu tam
izlerin ve geçmişin disk yükünü içermez. Az nüfusta alt milisaniye farklar/gürültü
var; indeks her nüfusta hız kazandırır iddiası yok. Karar sonuçlarını olaylara
bağlama ve planların bazı taramaları hâlâ nüfusa bağlı; çok daha büyük nüfus
bu sonuçlardan tahmin edilmemeli. Kayıt katmanı ayrı ölçülmeli.

Kaynak: `examples/benchmark_world.rs`, ROADMAP_10_BEFORE/AFTER.json. Komut:
`cargo run --release --example benchmark_world -- 1000 64 30 forage`.
