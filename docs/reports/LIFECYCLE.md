# Yaşam döngüsü düzeltmeleri

Başlangıç raporu: `BASELINE.md`. İlk düzeltme paketi:

- RNG artık dünya başına. İki dünya ve checkpointten yüklenen dünya iç içe
  çalıştırıldığında aynı sonuçları üretmeleri test ediliyor.
- Metabolizma karar öncesinde; uyku yaşlanmayı/susuzluğu durdurmuyor.
- Açlık ve susuzluk düşük sağlıkta da hasar veriyor; açlık bedava enerji üretmiyor.
- Su/enerji ihtiyacı göreli eksiklikle karşılaştırılıyor; yaralanma suyu gizlemiyor.
- Cesetler canlı tehdit/eş listesine girmiyor; otçul tehditten kaçıyor.
- Yavru ebeveynin cooldown, ölüm nedeni ve kullanılan hareketlerini miras almıyor.
- Üreme tür ve uygunluk denetimleri, saldırı enerji/aktiflik denetimleri yapılıyor.
- Ölüm nedenleri ve saldırı/üreme retleri yapılandırılmış olaylarda tutuluyor.

Bu aşama kaynak yenilenmesi veya sürdürülebilir nüfus iddiası değildir. Eski
hataların kaldırılması bazı koşullarda daha erken gerçek ölüm doğurabilir.

Doğrulama: 11 Rust testi, HTTP kontrolü ve SIGKILL kurtarma kontrolü.
