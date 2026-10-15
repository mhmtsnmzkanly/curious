# 7 — Kalıtım, mutasyon ve özellik dağılımı

Genom (`entity/genetics.rs`) bireysel hafıza/öğrenmeden ayrı. İlk özellik hareket
alleli 0/1/2: temel hıza 0/1/2 ekler, bir adımın enerji bedeli 1/2/3 olur.
Üstün hız bedelsiz değildir; gerçek hareket çözümleyicisi bu bedeli uygular.
Yavru ebeveynlerden birinin allelini alır; varsayılan %2 olasılıkla komşu allele
mutasyon olur. Sınırlar açık ve testlidir. Öğrenilmiş değerler yavruya geçmez.

Genel gelişmiş evrim motoru değil, bir özellik için kalıtım/çeşitlilik/seçilim
altyapısıdır. Mutasyon kapatılabilir. HTTP `genetics` kontrolü ile `enabled` ve
`mutation_per_mille` değiştirilir. Genom ve ayarlar checkpoint V7'de korunur.

`evolution_trial`: aynı otçul çayırında 4 normal + 4 hızlı kurucu, sekiz seed,
2.000 tick. Bireysel öğrenme bu karşılaştırmada kapalı; mutasyon 0 ve 20/1000
ayrı koşular. Başlangıç allele oranı 50/50; nesiller boyunca dağılım değişir.
Bu ortamda maliyetli hızlı allel çoğu koşulda azalır. Mutasyonlu koşuda yeniden
ortaya çıkabilir. Sayısal dağılımlar ROADMAP_07.json içinde.

Sonuç ortam ve maliyet kuralına bağlıdır; her dağılım değişimi tek başına uyum
kanıtı değildir, genetik sürüklenme de vardır. Açık uçlu yenilik veya sürekli
karmaşıklık artışı vaat edilmez. Aynı matris av baskısı altında ayrıca ölçülmelidir.
