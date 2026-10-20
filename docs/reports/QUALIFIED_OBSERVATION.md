# Kabulden gerçek karma gözleme — 20 Ekim 2026

Biyolojik kaynak32ddfb2 için sabitlenen bütün11kapı yeniden ham sonuçlardan
hesaplandı. Normal8/8; dört avcı ±%10 duyarlılık varyantı ayrı4/4; fixed30/30
ve both30/30×100000; shock8/8; learning/mutation/both canary ayrı8/8.
Son%10 doğum ve her türden en az2üreyen yavru-soyu şartları sağlandı.
Fixed/both aynı20001–20030kümesidir;60bağımsız seed denmez. Geçmiş normal/şok
koşuları tekrar çalıştırılmadan SHA, manifest, ham satır ve yeniden hesaplanan
özet eşitliğiyle kabul edildi. Kanıtlar QUALIFIED_OBSERVATION.json içinde.
Bu sonuç sonsuz denge, genel öğrenme veya açık uçlu evrim kanıtı değildir.

## Başlangıç ve motor sadakati

Eski live_deploy açık deneysel politika bayraklarını taşımıyordu. Yeni
verify_qualification.py her11kapıyı salt okunur doğrular; prepare_observation
örneği yeni dizinde seed20001, reserve_opportunities/mate_rendezvous açık,
mevcut su öğrenmesi ve türdeş hafızası açık, genetik açık ve mutasyon20‰
ile tick0tam durumunu SQLite'a kaydeder. HTTP motoru yalnız --resume kullanır.
Var olan dizinde kurulum reddedilir; servis eksik veriden kurucuları yeniden
oluşturmaz. Yeni bin/curious-qualified kullanılır; eski ikililer korunur.

Yeni motorun biyolojik kaynakları kabul edilen commit ile aynıdır; yalnız
archive.rs/persistence.rs depolaması değişir. Gerçek HTTP+SQLite testi frozen
32ddfb2probe ile100ve200adımda bütünState/RNG eşitliğini ve SIGKILL sonrası
aynı durumdan devamı doğruladı. KontrolAPI/auth, Chromium masaüstü/mobil,
sıkıştırılmış arşivde ölmüş ebeveyn/yavru gezinmesi geçti.87Rust ve52Python
başarılı. Tarayıcı testinde dış step sonrası gözlem güncellemesi beklenerek
eski asenkron test yarışı düzeltildi; ürünün kontrolleri gevşetilmedi.

## Geçmiş kapasitesi ve ilk hız

Aynı200tickörneğinde gzip15kapalı shard/188tick için140650172bayt;
zstd54705127bayt: yaklaşık%61azalma. Yeni arşiv zstd seviye3,128MiBpencere,
checksum, her baytı aslıyla karşılaştırma ve fsync sonrası ham shardı kaldırma
uygular. Geçmiş kaybı yok; eski gzip okuması ve hasarlı arşivi reddetme sınandı.
SQLiteV2/StateV12 değişmedi. Eski motorlar zstd okuyamaz; eski dünyalar
bu sürüme geçirilmez. Python geçmiş araçları zstdCLI kullanır.

0,1tick/s örnek tahmini6,02→2,34GiB/gün. Yeni gözlem başlangıcı0,03tick/s,
64MiBshard,32GiBgeçmiş bütçesi: yaklaşık0,70GiB/gün,30günde21,1GiB.
Bu200tickten tahmindir; nüfus, hafıza ve olaylar hacmi değiştirebilir.
Aktif shard ve geçici açılmış arşiv ayrıca yer ister. Disk sonsuz değildir;
mevcut bütçe uzun vadeli sınırsız saklama çözümü sayılmaz. %80bütçe uyarısı
ve yazma/bütçe hatasında güvenli pause korunur. Gerçek ölçümlerle kapasite
planını yeniden değerlendirmek TODO olarak kalır; geçmiş sessizce silinmez.

## İşletim sözleşmesi

7882localhost üzerinde ayrı curious-coexistence.service; --resume ile kalıcı
başlangıç,0,03tick/s. Kurulum betiği temiz commit, frozen scriptsource ve
SHA'lı release/başlangıç/kanıt metaverisi hazırlar. Önce pausedtick0bütün
alanları HTTP ile doğrulanır, sonra sürekli gözlem açılır.7878–7881korunur.
Dakikalık tür/sağlık monitörü,15dakikalık RSS/fd/tür örnekleri, saatlik gerçek
1/7/30gün raporu ve bağımsız watchdog kullanılır. Bu süreçler kod yazmaz,
canlı eklemez. Gerçek1/7/30gün tamamlanmış sayılmaz; host dışı izleme açıktır.
