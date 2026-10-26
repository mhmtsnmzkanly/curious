# Açık işlerin birleştirilmiş sonucu — 26 Ekim 2026

Üç paralel Luna ajanı kaynak/kanıt/doküman çalışmalarını yaptı; root çapraz inceleme, derleme, kayıtlı koşular ve operasyon kontrolünü yürüttü. Bu turda motor davranışı değiştirilmedi. Teşhisler, gelecekteki hedef sözleşmesi ve sınırlı ölçümler tamamlandı; teşhis edilen bütün biyolojik sorunların çözülmüş olduğu iddia edilmez.

| İş | Sonuç | Kanıt |
|---|---|---|
| Öğrenmenin neden kararı değiştirmediği | 147 güncellemenin 146'sı sıfır; 43 taze seçimde sıralama değişmedi | [Su sinyali tanısı](WATER_SIGNAL_DIAGNOSIS_20261026.md) |
| Gözlenebilir öğrenme hedefi | Kendi ziyaretindeki hizmet tahmin hatası için bounded sözleşme; **öneri**, kodlanmadı | [Hedef sözleşmesi](../SOURCE_RELIABILITY_TARGET_20261026.md) |
| Sıfır enerjide yem erişimi | Görünen yem fiziksel olarak kalan enerjiyle erişilemez hale geliyor; bedava hareket çözümü değil. Reserve formülü henüz seçilmedi | [Enerji tanısı](ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.md) |
| Bakım/sığınak paketi | +13 endpoint canlı, fakat −1.428 toplam birey-tick; canlı fayda kabulü yok | [Paket tanısı](ENERGY_AND_PACKAGE_DIAGNOSIS_20261026.md) |
| Güncel dokümanlar | Tarihsel PASS ile bugünkü kabul ayrıldı; roadmap K1–K8 korundu | [Belge dizini](../README.md), [roadmap](../ROADMAP.md) |
| Habitat + işletim | 45 dünya / 27.000 tick ve offline 121 tick tamamlandı; rollout yapılmadı | [Ekoloji](ECOLOGY_TRANSFER_V1_20261026.md), [işletim](OPERATIONS_READINESS_20261026.md) |

## Sonuçlardan çıkan yön

Yeni bir özellik paketini topluca açmak için kanıt yok. Seyrek kaynakta 15/15 koşu tükendi; karma habitatta 15/15 koşunun sonunda etçil kalmadı. Alarm kolunun bir karma seedinde tüm nüfus tükendi. Bunlar ayar değiştirilerek saklanmadı. Genel birlikte yaşam kabulü verilmedi.

Ambush bayrağı yalnız saldırı hasarını değiştirmiyor: mevcut algı yolu örtüde tam dayanıklılığı olan diğer canlıların görünürlüğünü de azaltıyor. Bu yüzden otçul dünyasındaki farklılıkları av başarısı saymıyoruz. Ekstra özellik kolu hem beden ifadesini hem üremedeki RNG çekim sayısını değiştiriyor; kalıtım her kolda zaten var. Bu sınırlar [ekoloji raporunda](ECOLOGY_TRANSFER_V1_20261026.md) yer alır.

Öncelik, mevcut beden/algıyla ödenebilir yem rotasının korunmasını ayrı bir hipotez olarak tasarlamak ve seyrek kaynak/avcı tükenmesini açıklamak. Kaynak güvenilirliği hedefi henüz model veya World entegrasyonu değildir; öncelikle saf tahmin/etiket sözleşmesi doğrulanmalıdır. Uzun soak ya da canlı geçiş, bu açık soruları çözmez.

## Kontroller ve kayıt disiplini

- Ekoloji kaynağı: `bc60fa6`, temiz ağaç, değişmez protokol ve ayrı [scanner ek notu](../ECOLOGY_TRANSFER_V1_SCAN_ADDENDUM_20261026.md). İlk girişim seed tarayıcısında **0 World tick** ile başarısız oldu; ayrı girişim kaydı korundu.
- İki Rust probunda 2 + 4 test ve Python analiz/supervisor tarafında 4 + 6 test geçti (16 güncel test). İlk derleme/doğrulama hataları kayıtlı fiziksel koşudan önce düzeltildi.
- İşletim probunun ilk koşusundaki p50/p95 değerleri sıralama hatası nedeniyle geçersiz: ham dosya korunup rapordan dışlandı. Saf fonksiyon düzeltildi, sırasız girdi testi geçti, World tekrarlanmadı. [Düzeltme notu](OPERATIONS_READINESS_TIMING_CORRECTION_20261026.md).
- Kaynak, ikililer, protokol/fixture hash'leri, kayıtlar ve stderr yerelde korundu. Detaylı raw veriler ignore edilen `data/` altındadır; GitHub'a yüklenmiş gibi sunulmaz.
- Canlı V12 dünyası, servis ayarları ve replay değişmedi. Teknik smoke geçmesi davranışsal kabul anlamına gelmediğinden `promotion_allowed=false` kaldı.

Makine özeti: [JSON](CONSOLIDATION_20261026.json). Sıradaki işler tek yerde [TODO](../../TODO.md) içinde tutulur.

Temizlik: çalışan Cargo/rustc/probe olmadığı doğrulandıktan sonra yalnız `/tmp/curious-consolidation-20261026` kaldırıldı (192.705.723 byte). Deney kaynak arşivi, çalıştırılan ikililer, ham kayıtlar ve canlı veriler korundu. Son salt-okunur kontrolde tick 13644, 598 canlı, `paused=false`, `storage_error=null`; canlı PID 119175 ve replay PID 1137 değişmedi, iki serviste de NRestarts=0. Bu zaman damgalı gözlem `data/operations/consolidation-20261026/live-after.json` içindedir.
