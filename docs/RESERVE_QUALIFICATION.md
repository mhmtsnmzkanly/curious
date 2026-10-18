# Rezerv adayının bağımsız uzun doğrulaması

## Başlangıç kanıtı

Biyolojik kaynak32ddfb2; rezerv hazırlığı ve otçul buluşması açık, sabit genetik,
öğrenme kapalı. Hedefli şok2/2, normal8/8, şok8/8 tamamlandı. Her koşu20000
adımda iki tür, son bölüm doğumları ve üreyen yavru soyu kapısını geçti.
3/2026 hedefli şokları shock8 içinde de vardır;18 bağımsız koşul diye sayılmaz.
Tam sonuç ve SHA'lar `reports/RESERVE_TRAINING_ACCEPTANCE.json` içinde.
Bunun kalıcı/sonsuz denge veya gerçek gün/hafta kabulü olduğu söylenmez.

## Sonuç görülmeden sabitlenen sıra

1. Eğitim seedleri1/2/3/4'te avcının max_energy ve reproduction_interval
   alanlarında ayrı ayrı%90/%110: dört senaryo×4seed×20000 adım. Her varyant4/4.
   Kapasite değişiminde kurucu enerjisinin doluluk oranı korunur.
2. Yeni ayrılan20001–20030:30seed×100000 adım, fixed. En az27/30.
3. Aynı ikili/senaryo/seçeneklerle geçmiş shock8 kanıtı yeniden doğrulanır.
4. Aynı sekiz eğitim seedinde learning, mutation, both ayrı ayrı20000 adım.
5. Sabitlenen aynı30seedde both modu100000 adım; en az27/30.

Her koşulda türlerin varlığı yanında son%10'da doğum ve tür başına en az2
üreyen yavru-soyu bireyi gerekir. Sonuç sonrasında eşik veya tohum değiştirilmez.
20001–20030 mevcut deney manifestlerinde kullanılmamıştı; ilk kullanım sonrası
aynı kümede koda tekrar tekrar uyum sağlamak bağımsız doğrulama sayılmaz.
Fixed ve both eş seedleri donmuş protokolün eşleştirilmiş karşılaştırmasıdır.
Öğrenme karşılaştırması mevcut su banditini, mutasyon karşılaştırması mevcut
bedelli hareket allelini sınar. Genel av öğrenmesi ve açık uçlu evrim hâlâ açık.

## Çalıştırıcı ve işletim

`scripts/research_coexistence.py` seçenekleri bütün aşamalara geçirir.
`scripts/research_protocol.py` protokolü sabitler, eğitim kanıtlarını salt okunur
kontrol eder. Geçmiş normal8/shock8 sonuçları ikili+senaryo SHA, derleme kaynağı,
seçenekler, temiz manifest ve her ham seed sonucu yeniden doğrulanınca kullanılır.
Özet tek başına kabul edilmez; eski veriler değiştirilmez. Yeni koşul çıktısının
seçenekleri de çalıştırma ve resume sırasında doğrulanır.

Araştırma Python kaynak commit'i ve ikilinin derlendiği Rust commit'i ayrı
kimliklerdir. Bu tur Rust kaynağını değiştirmez; src/Cargo.toml/Cargo.lock Git
kimlikleri derleme kaynağıyla eşit olmalıdır. İkili32ddfb2'den korunur.
Yeni çalışma ayrı temiz worktree'de, değişmez ikili/senaryo ve `protocol.json`
ile çalışır. Protokol değiştiğinde resume reddedilir. Teknik hata biyolojik
başarı sayılmaz; ret sonraki kapıyı durdurur. Kod/parametre kendiliğinden yazılmaz.

Çalışma kökü `data/experiments/reserve-qualification`:
`config.json`, `protocol.json`, `state.json`; aşamalarda `progress.json`,
`seed-*.log`, ham sonuç ve manifestler. `curious-reserve-qualification.service`
iki işçi, Nice10, toplam CPU%180 üst sınırı,512MiB bellek sınırıyla çalışır;
restart/resume tamamlanan seedleri korur. Host açılışında devam edebilir.
Önceki verilerden süre tahmini: duyarlılık yaklaşık2–3saat; bütün kapılar
geçerse uzun fixed+both ile toplam yaklaşık2gün. Nüfus ve host yükü süreyi değiştirir.

Notify sayfası dakika başına aşama, tamamlanan seedler ve son adımlarla güncellenir.
Aynı bildirim tekrarlanmaz; önemli aşamalar kalıcı kuyrukla iletilir.
Final qualified durumu da timer tarafından tek başarı bildirimi olarak kuyruğa
alınır. Bildirim kaynağı ana deneyden bağımsız sabitlenebilir; kimliği
`source.json` ve configte `notification_revision` ile kaydedilir.
`curious-reserve-qualification-notify.timer` tekrar/heartbeat sağlar.
Mevcut bağımsız watchdog `experiments/*/protocol.json` çalışmalarının running/error
heartbeat'ini ve bildirim kuyruğunu ayrı izler. Aynı hostun tamamen durması için
host dışı izleme konusu hâlâ açıktır.

Bu kampanyada otomatik dağıtım kapalıdır. Mevcut live_deploy yolu deneysel
seçenekleri başlangıçta açmıyor; açık seçeneklerle o yoldan dağıtım reddedilir.
Bütün kabul kapılarından sonra ayrı karma dünyada aynı ayarların kayıtla
başlatılması ve gerçek1/7/30gün gözlemi sıradaki işletim aşamasıdır.7878–7881
canlı dünyaları ve eski araştırma worktree/ikili/verileri korunur.

## Doğrulama

49 Python testi; gerçek32ddfb2 probe ile iki seedlik100adım treatment/resume
kontrolü; Node arşiv kontrolü geçti.100adım kontrolü keşif amaçlıdır, biyolojik
kabul sayılmaz. Rust kaynakları aynı olduğu için yeni Rust derlemesi yapılmadı;
önceki83 Rust+HTTP kurtarma kanıtı değişmez motor sürümüne aittir.
