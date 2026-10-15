# 9 — Dayanıklılık: hızlandırılmış sonuç ve gerçek süreli gözlem

V7 motoru: bir seed, **10.000 tick tam SQLite kaydı**, shard geçişi ve gzip.
Yaşam sonda devam ediyor, 304. nesle ulaştı. 1.249 doğum, 1.243 ölüm; 14 yaşayan.
Duvar saati 13,877 s; örneklenmiş en yüksek RSS 9.020 KiB, dosya tanıtıcısı 9;
en yavaş 100 tick HTTP batchi 0,1932 s. Kayıt 101.533.181 bayt. Bu ölçüm tek
nüfus/çevre içindir ve kısa süreli RSS tepe örneklemesi tam bellek sınırı değildir.

Aynı yoğunlukla 0,1 tick/s, yaklaşık 88 MB/gün ve 2,6 GB/30 gün yapar. Tam algı,
nüfus ve kayıt ayarı bu hesabı değiştirir. Eski disk tahminleri yeni V7 izlerine
aynen uygulanamaz. 10 GiB bütçede hata/taşma gizlenmez: sistem durur ve uyarır.
Sınırsız disk veya yıllarca sıfır bakım vaat edilmez.

`curious-sampler.timer`, gerçek çayırın durumunu, RSS, fd sayısını ve yanıt
süresini 15 dakikada bir `data/soak/YYYY-MM.jsonl` içine fsync ile ekler. İlk
örnekte servis başlangıcı 2026-10-15 00:59:31 UTC, tick 3836, RSS 8620 KiB,
fd 8; hata yok. Bu eski kurallarla çalışan çayırdır; V7'ye ait gerçek süreli
kanıt olarak kullanılamaz. Yeni sürüm için ayrı koşu dağıtım aşamasında açılacak.

**Gerçek gün/hafta testi sürüyor; tamamlanmadı.** Hızlandırılmış koşu bunun yerine
geçmez. Aşama 9'un altyapısı ve kısa testleri bitti; süre bekleyen gözlem çalışırken
10. aşamadaki bağımsız nüfus ölçekleme ölçümüne geçilir. SIGKILL/kalıcı devam,
yazıcı kilidi, arşiv hasarı ve disk hatasında durma mevcut testlerle korunur.
