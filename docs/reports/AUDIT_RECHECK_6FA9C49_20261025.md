# 6fa9c49 düzeltmelerinin bağımsız yeniden denetimi — 25 Ekim 2026

Birincil kategori K8; K1–K5 ile kesişir. Üç paralel Luna 6 incelemesi root tarafından kaynak ve test tanımlarıyla doğrulandı. Kapsam `bd0ceb2..6fa9c49`; kaynakta düzeltme, canlıya dağıtım veya yeni formal deney yapılmadı. Önceki raporlar değişmedi.

## Açık bulgular

### P1 — F1 bağlam artığının ölçeği hâlâ yanlış

[episode_physical_residual](../../src/entity/learning.rs#L334) artık bölüm ödülü yerine `water - movement_energy - incoming_damage - choice_baseline` hesaplıyor. Ancak [foraging::value](../../src/entity/foraging.rs#L16) baseline'ı `(2*water - movement_energy - ticks)*100/ticks` ölçeğinde tutar. Bu iki nicelik birbirinden çıkarılamaz; yanlış artık yeniden baseline'a eklenerek kararları değiştirir.

Kaynak formüllerinin basit sayısal karşılaştırması: 10 su, 0 hareket bedeli, 1 tick ve doğru baseline 1900 için gerçekleşen aynı-ölçek skor da 1900, artık 0 olmalıdır. Yeni yardımcı `10-1900` sonucunu -1000'e kırpar. Bu bir Rust reproducer çalıştırması değil, incelenen formüllerin aritmetik karşılaştırmasıdır. `choice_baseline=None` durumunda da sıfır varsayılır; kanıtsız baseline için artık üretmemek gerekir. Mevcut legacy residual hesabı [learning.rs](../../src/entity/learning.rs#L485) aynı-ölçek hesap için referanstır. Gelen hasarın nedensel maliyet sayılması ayrıca sözleşmeyle açıklanmalıdır.

Özdeş sonuçları atlayan değer-eşitliği filtresi kaldırılmıştır. [water_outcomes](../../src/systems/water_outcomes.rs#L16) yalnız aktif bölüm bulunan bireyi işlediği için tamamlanmış eski sonucu her tick tekrar kaydetmez; bu alt sorun düzeldi.

### P2 — F8 kanıtı, tam kapanış raporundaki iddiaları karşılamıyor

[Karşılaştırma](../../tests/k8_throughput_audit.rs#L364) gerçekten eş seedli kontrol/treatment dünyalarını 80 tick çalıştırıyor. Fakat [assert_ne](../../tests/k8_throughput_audit.rs#L422) tam checkpoint JSON'larını karşılaştırır; açık/kapalı bayraklar JSON'a zaten yazıldığı için bu koşul davranış farkı oluşmadan da sağlanır. RNG ayrıca karşılaştırılmaz. Treatment'ta sektör kaydı için pozitif assert vardır; `context_entries` ve `first_interval_outcomes` toplamları hesaplanıp zorunlu tutulmaz, sinyal sayısı zorunlu tutulmaz, in sayısı toplanmaz.

[Düzeltme raporu](AUDIT_RECHECK_REMEDIATION_20261025.md) her seedde bu beş kaydın üretildiğinin ve State+RNG ayrışmasının exact assert'lerle kanıtlandığını söyler; test bunu göstermiyor. Dünyaların gerçekten çalışması ve merak sektörlerinin oluşması doğrulanan daha dar sonuçtur. Kabul için konfigürasyondan ayrı dinamik sonuçlar, özellik başına gözlenen olay/öğrenme güncellemesi ve gerekiyorsa ayrı RNG karşılaştırması ölçülmelidir. Ekolojik yarar veya nüfus sürekliliği bu incelemenin koşulu değildir.

## Diğer düzeltmeler ve sınırlar

| Bulgu | Durum |
|---|---|
| F2 pusu | Önceki erişilemezlik düzeldi. [Örtü yardımcısı](../../src/systems/perception.rs#L182) sıradan avcının ulaşabildiği bitki/duvar bitişiğini kabul eder; algı ve saldırı aynı yardımcıyı kullanır. |
| F3 checkpoint | Sürüm kapısı önceki incelemede düzeldi. Yeni [ham JSON testi](../../tests/persistence.rs#L291) V19 alanları olmayan el yapımı V18 şema fixture'ını okur; tarihsel binary'den alınmış artifakt değildir. |
| F4 kış maliyeti | Önceki düzeltme korunuyor. |
| F5 inler | World harita API'sinde sıfır TTL, sınır dışı ve dolu hücre reddi düzeldi; kışta otomatik kazı eklendi. Fakat [from_map](../../src/environment.rs#L287) hâlâ boş burrow metadata'sı üretir: statik harita ini, `has_active_burrow` üzerinden kış koruması almaz. Metadata-only `Environment::dig_burrow` harita doğrulaması yapamaz; harita API'siyle aynı garantiye sahipmiş gibi belgelenmemeli. |
| F6 leşler | Sıfır tüketimde hasar düzeldi. [Çürüme](../../src/systems/life_cycle.rs#L20) yalnız decrement sonrası remaining 1..3 olduğunda işler: yaşlanma/çürüme döneminin üç tick'i. Bu kasıtlı olabilir; “her tick” tam yaşam iddiası kaynakla uyuşmaz. |
| F7 taşma | Önceki sınır ve i64 toplam düzeltmesi korunuyor. |

Otomatik kazıda stamina, in ekleme sonucundan önce kesilir; kapasite doluyken başarısız girişim de bedellidir. Bunun başarısız kazı bedeli olarak tasarım mı yoksa hata mı olduğu belirlenmeden ayrı kusur sayılmadı. Normal World erişilebilirliği doğrulanmayan farklı-hedef başarısız içme atfı şüphesi de doğrulanmış bulgulara eklenmedi.

## Doğrulama

`cargo test --locked --all-targets --jobs 1`: 396 geçti, 0 başarısız; 68 özet grubu. Ayrı `CARGO_TARGET_DIR` ve jobs 1 kullanıldı. Günlük yerel `data/operations/recheck-6fa9c49-20261025/rust-tests.log` dosyasında korundu. Testlerin geçmesi yukarıdaki puanlama ve assertion kusurlarını kapatmaz.

Canlı ve replay servisleri systemd'de aktifti; değiştirilmedi. Yeni kalan bulgular statik çağrı/formül ve assertion incelemesine dayanır; yeni bağımsız biyolojik deney veya her kusur için ayrı dinamik repro çalıştırılmadı. Sonuç: tüm bulguların eksiksiz kapandığı söylenemez; F1 ve F8 önceliklidir.
