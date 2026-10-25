# 9c7d3f0 düzeltmelerinin bağımsız denetimi — 25 Ekim 2026

Birincil kategori K8; K1/K3/K5 ile kesişir. Üç paralel Luna 6 kaynak incelemesi root tarafından gözden geçirildi. Kapsam `68d4fc6..9c7d3f0`, önceki [denetim](AUDIT_RECHECK_6FA9C49_20261025.md) ve yeni [düzeltme raporu](AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md). Kaynak düzeltilmedi, canlıya dağıtım veya yeni formal deney yapılmadı.

## Sonuçlar

- **F1 düzeldi.** [Gerçekleşen skor](../../src/entity/learning.rs#L333) artık `(2*water-movement_energy-ticks)*100/max(1,ticks)` ile baseline'ın ölçeğine uyuyor. Baseline yoksa None; hasar nedensel su maliyeti olarak çıkarılmıyor. [Kayıt yolu](../../src/systems/water_outcomes.rs#L59) başarısız içmeler için hedef doğrulaması istiyor. Özdeş tamamlanmalar korunuyor, aktif bölüm yoksa eski sonuç tekrar yazılmıyor. Formula/censor/duplicate testleri mevcut.
- **F5'in önceki kusurları düzeldi.** [from_map](../../src/environment.rs#L258) statik inleri sınırlı metadata listesine alıyor; sezon açıkken inde duran aktif birey için kayıt eksikse [World](../../src/world/mod.rs#L113) kayıt yapıyor. Başarısız otomatik kazıda stamina kesilmiyor. [Test](../../tests/p2_environment_integration.rs#L649) statik kış koruması ve kapasite doluyken bedel kesilmemesini kontrol ediyor. Otonom kazı için ayrıca [World testi](../../tests/p2_environment_integration.rs#L426) mevcut. Sınır: kayıt kapasitesi 32; statik inler kalıcı değil, TTL 1.000.000 tick ile kaydediliyor. Bu belgelenen sınır, yeni doğrulanmış kusur sayılmadı.
- **F6 davranış/belge ayrımı giderildi.** Fresh'in ilk tick'inde enerji korunması, sonraki remaining 1..3 aralığında birer enerji çürümesi ve sıfır tüketimde toksisite muafiyeti açıklanmış. Yeni düzeltme raporundaki `CorpseStage::Rotten` adı yazım hatasıdır; gerçek enum `Putrid`.
- **F8'in ana karşılaştırma kusuru düzeldi.** [DynamicWorldSnapshot](../../tests/k8_throughput_audit.rs#L366) konfigürasyon bayraklarını içermiyor. Kontrol ve treatment aynı başlangıç bedeni/harita/seed kullanıyor; gerçek tick'lerden sonra dinamik özet ve RNG ayrı karşılaştırılıyor. İki beş-seed grubunda sektör, bağlam ve ilk-aralık kayıtları ile sinyal/in metadata'sı için pozitif assert'ler var. Bu artık yalnız bayrak farkıyla geçen JSON karşılaştırması değildir.

## F8'de kalan dar kanıt sınırı

[treatment_burrows_total](../../tests/k8_throughput_audit.rs#L572), tick'ler boyunca toplam metadata sayısının maksimumudur. Fixture zaten iki fiziksel in hücresi koyar; World'ün mevcut inde duran canlı için metadata kaydı yapması bu sayıyı artırabilir. Bu nedenle pozitif in sayısı **yeni otonom kazının her seedde gerçekleştiğini tek başına kanıtlamaz**. Yeni kazı konumlarının başlangıç haritasına göre farkı veya kazı olayı sayılmalıdır. Ayrı otonom kazı testi mekanizmanın uygulanmış olduğunu doğrular; bu not mekanizmanın yokluğu iddiası değildir.

Sinyal sayacı da her tick mevcut sinyal listesinin uzunluğunu toplar; aynı sinyali birden çok kez sayabilir. Başlangıçta boş listede pozitif olması sinyal varlığını destekler, benzersiz yayım sayısını vermez. Diğer sayaçlar anlık tablo büyüklüklerinin maksimumlarıdır; sayısal toplam/öğrenme yararı gibi yorumlanmamalıdır. Çoklu-seed testleri paket halinde etkin mekanizma ve yörünge farkını gösterir; özellik başına bağımsız nedensel fayda veya uzun dönem ekolojik üstünlük kabulü değildir.

## Doğrulama ve işletim

`cargo test --locked --all-targets --jobs 1` ayrı target dizininde tamamlandı: 397 geçti, 0 başarısız, 68 özet grubu. Yerel kanıt: `data/operations/recheck-9c7d3f0-20261025/rust-tests.log`. Yeni bağımsız reproducer veya formal ekoloji koşusu yapılmadı. Belge bağlantıları ve `git diff --check` kontrol edildi.

Canlı dünya ve replay systemd'de aktifti; servisler, binary'ler ve canlı veriler değiştirilmedi. Önceki F3/F4/F7 düzeltmelerine bu farkta müdahale yok. Yeni bir çalışma zamanı engeli saptanmadı; F8 otonom kazı sayacıyla ilgili dar kanıt sınırı korunmalıdır. İnceleme bütün projede kusur bulunmadığı garantisi değildir.
