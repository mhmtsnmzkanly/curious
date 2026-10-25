# Curious belgeleri

## Başlangıç noktaları

- [Ana README](../README.md): proje, yerel çalıştırma ve mimari sınırlar.
- [Dokümantasyon standardı](DOCUMENTATION_STANDARD.md): belge rolleri, kanıt
  dili ve bakım akışı.
- [ROADMAP](ROADMAP.md): sekiz K1–K8 kategori, özgün 11 aşama, A–G yetenek
  izleri, güncel durum ve ortak kabul kapıları.
- [TODO](../TODO.md): P0–P3 sıradaki işler.
- [CONTROL](CONTROL.md): HTTP kontrolü, pause/resume ve işletim sözleşmesi.
- [LEARNING_NEXT](LEARNING_NEXT.md): öğrenme alanları için davranış ve kayıt
  kontratları.
- [VISION](VISION.md): kullanıcı amacı ile mimari ilkelerin tarihsel taslağı.
- [HISTORY_RETENTION](HISTORY_RETENTION.md): arşivleme, retention ve depolama
  sınırları.

Kategorilere doğrudan git: [K1 Dünya ve çevre](ROADMAP.md#dunya-cevre),
[K2 Canlı ve beden](ROADMAP.md#canli-beden),
[K3 Bireysel davranış ve öğrenme](ROADMAP.md#bireysel-ogrenme),
[K4 Grup ve sosyal ilişkiler](ROADMAP.md#grup-sosyal),
[K5 Ekoloji ve etkileşim](ROADMAP.md#ekoloji-etkilesim),
[K6 Kalıtım ve evrim](ROADMAP.md#kalitim-evrim),
[K7 Gözlem ve arayüz](ROADMAP.md#gozlem-arayuz),
[K8 Motor ve işletim](ROADMAP.md#motor-isletim).

## Güncel kanıt

- [Su uyum labı V2](reports/WATER_ADAPTATION_V2_20261025.md) — kalibrasyon ve formal tamamlandı; deneyimin ek davranış/fayda şahidi bulunmadı.

- [Su uyum labı V1 kalibrasyonu](reports/WATER_ADAPTATION_V1_20261025.md) — birey fork öncesi öldü; formal çalıştırılmadı, fayda sonucu yok.

- [Seed başına kabul ve V12/V19 köprü tanısı](reports/QUALIFICATION_DIAGNOSTIC_20261025.md) — uzun ufukta paket ödünleşimli; canlı geçiş kabul edilmedi.

- [Wave 12: Çok-nesilli V19 yetenek bayrağı kalıtımı ve gerçek V12 canlı checkpoint üzerinde 1.000-tick (`Tick 12639 -> 13639`) izole gölge soak deneyi](reports/LIVE_V12_SHADOW_1000_TICK_SOAK_20261025.md) — `resolve_mates` içinde V19 yetenek bayraklarının sıfır öğrenilmiş bellek aktarımıyla yeni nesillere devri ve 630 bireyli gerçek V12 dünyasında 1.000 tick (`34+` yeni nesil, `~5.150` doğum) boyunca 4 kolun (`ControlFlagsOff`, `JuvenileParentalCareOnly`, `RefugeFirstIntervalOnly`, `QualifiedCandidateBundle`) karşılaştırmalı soak kanıtı.
- [Wave 11: Yazılmış V19 özelliklerinin davranışsal fayda (Pareto) elemesi ve gerçek V12 canlı checkpoint üzerinde izole gölge doğrulaması](reports/BEHAVIORAL_UTILITY_AND_SHADOW_QUALIFICATION_20261025.md) — 10 held-out tohum (`401..=410`) × 150 tick üzerinde 9 tekil özelliğin fiziksel sonuç vektörüyle Pareto sınıflandırması (`JuvenileParentalCareOnly` + `RefugeFirstIntervalOnly` canlı aday paketi), gerçek canlı V12 (`history-000870.sqlite.zst`, `tick = 12639`, `630` birey) üzerinde 20-tick V19 gölge devamlılık doğrulaması ve kontrollü canlıya taşıma prosedürü.
- [Wave 10: P3 ablasyon, K7 V19 telemetri ve 9c7d3f0 kanıt sınırı kapanış raporu](reports/P3_ABLATION_AND_9C7D3F0_CLOSURE_20261025.md) — çoklu-habitat tekil-özellik ablasyon seçiciliği, F8 otonom kış kazısı deterministik tanığı, `SignalTrace <= 64` sınırı ve K7 V19 web telemetrisi.
- [9c7d3f0 bağımsız kapanış kontrolü](reports/AUDIT_RECHECK_9C7D3F0_20261025.md) — ana düzeltmeler doğrulandı; F8 otonom kazı sayacının kanıt sınırı korunuyor.

- [Wave 9: 6fa9c49 yeniden denetim bulgularının (P1/F1, P2/F8, F5, F6) tam kapanış ve hizalama raporu](reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md) — su bağlam artığının `foraging::value` ölçeğiyle tam hizalanması, konfigürasyondan bağımsız dinamik snapshot ve RNG ayrışması, statik harita inleri metadata kaydı ve ceset çürümesi zamanlamasının tam kapanış kanıtı.
- [6fa9c49 bağımsız yeniden denetimi](reports/AUDIT_RECHECK_6FA9C49_20261025.md) — F1 ölçek hatası, F8 kanıt sınırı ve diğer kapanışların kontrolü.
- [2002ee4 düzeltmelerinin bağımsız yeniden denetimi](reports/AUDIT_RECHECK_2002EE4_20261025.md) — F3/F4/F7 düzeldi; diğer bulgulardaki kalan kusurlar ve kanıt sınırları.
- [Wave 8: 2002ee4 yeniden denetim bulgularının (F1, F2, F5, F6, F8) tam kapanış raporu](reports/AUDIT_RECHECK_REMEDIATION_20261025.md) — su bağlam ölçeği, pusu örtüsü, ham V18 JSON göç testi, dinamik in ve ceset çürümesi ile 5-seed World::tick karşılaştırmasının tam kapanış kanıtı.
- [25 Ekim ca5971b entegrasyon denetimi: sekiz bulgu](reports/AUDIT_CA5971B_20261025.md) — P2/P3 entegrasyon iddialarının sınırlarını ve tespit edilen eksikleri açıklar.
- [Wave 7: ca5971b entegrasyon denetimi (F1–F8) düzeltme ve V19 doğrulama raporu](reports/AUDIT_CA5971B_REMEDIATION_20261025.md) — sekiz bulgunun düzeltmelerini, `World::tick` entegrasyonlarını, V19 checkpoint yükseltmesini ve epistemik kapsam ayrımını belgeler.
- [Qualified coexistence ve canlı gözlem](reports/QUALIFIED_OBSERVATION.md)
- [Canlı V12 arşiv bütçesi backportu](reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md)
- [V17 su artığı sonucu](reports/WATER_RESIDUAL_V1.md)
- [Karışık habitat vision sonucu](reports/VISION_HABITAT_V1.md)
- [İlk resolver aralığı hasar ölçümü](reports/REFUGE_FIRST_RESOLUTION_V1.md)
- [Tam kaçış hasar baseline'ı](reports/REFUGE_RISK_BASELINE_V1.md)
- [Sınırlı ecology fixture sonucu](reports/ECOLOGY_FIXTURE_V1.md)
- [P2/P3 ampirik protokol ve K8 denetimi (Tarihsel dondurulmuş rapor)](reports/P2_P3_EMPIRICAL_AND_K8_AUDIT_20261025.md)

## Tasarım ve protokoller

Yürürlükteki ya da dondurulmuş ölçüm sözleşmeleri `docs/` içindedir; her rapor
kendi kaynak, seed, süre ve yorum sınırlarını belirtir. Öne çıkanlar:
[REFUGE_FIRST_RESOLUTION_PROTOCOL](REFUGE_FIRST_RESOLUTION_PROTOCOL.md),
[ECOLOGY_FIXTURE_PROTOCOL](ECOLOGY_FIXTURE_PROTOCOL.md),
[ECOLOGY_TRAITS_PROTOCOL](ECOLOGY_TRAITS_PROTOCOL.md) ve
[WATER_RESIDUAL_PROTOCOL](WATER_RESIDUAL_PROTOCOL.md). Protokol ya da ham veri
kimliği sonuç çıktıktan sonra sessizce değiştirilmez.

P0/P1 saf prototiplerinin ardından, P2 kapsamında temel mekanizmalar (mevsimsel döngü,
inler, kademeli cesetler, ebeveyn bakımı, sığınak pusu gizlenmesi, yerel sesli alarm,
su bağlamı, ilk-aralık hasar öğrenmesi ve mekânsal merak) çekirdek motora modüler ve
varsayılan kapalı (`default-off`) olarak entegre edilmiştir. Wave 7, Wave 8 ve Wave 9 denetimlerinde
([AUDIT_CA5971B_20261025](reports/AUDIT_CA5971B_20261025.md),
[AUDIT_RECHECK_2002EE4_20261025](reports/AUDIT_RECHECK_2002EE4_20261025.md),
[AUDIT_RECHECK_6FA9C49_20261025](reports/AUDIT_RECHECK_6FA9C49_20261025.md)) tespit edilen
tüm entegrasyon kusurları giderilmiş; özellikler aktif `World::tick` yürütme yoluna bağlanmış,
bağlam artığı `foraging::value` fiziksel net artık ölçeğine tam uyarlanmış, pusu örtüsü sıradan avcılara
genişletilmiş, in/ceset fizik kuralları senkronize edilmiş, checkpoint sürümü `V19`'a
yükseltilmiş ve konfigürasyondan bağımsız dinamik durum karşılaştırmasıyla 5'er tohumlu gerçek
`World::tick` Control vs Treatment karşılaştırmaları doğrulanmıştır
([AUDIT_CA5971B_REMEDIATION_20261025](reports/AUDIT_CA5971B_REMEDIATION_20261025.md),
[AUDIT_RECHECK_REMEDIATION_20261025](reports/AUDIT_RECHECK_REMEDIATION_20261025.md),
[AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025](reports/AUDIT_RECHECK_6FA9C49_REMEDIATION_20261025.md)).
K7 web replay ve deneyim ekranları (`refuge`, `causalChain`, `autoDirector`) arayüze bağlanmış;
P3 sabit 5-seed held-out protokolü ve K8 throughput denetimi (`SignalTrace <= 64`, `sites <= 8`,
`dangers <= 8`, `known_sectors <= 16`, `burrows <= 32`) tamamlanmıştır. Mekanizma entegrasyonu
doğrulanmış olup ekolojik üstünlük kanıtlanmamıştır / kapsam dışıdır. Ayrıntılar
[LEARNING_NEXT](LEARNING_NEXT.md), [ROADMAP](ROADMAP.md) ve [TODO](../TODO.md) içindedir.

Protokoller sabitlenmiş ölçüm tasarımlarını, raporlar ise tarihli kanıt ve
sınırlarını anlatır; ikisi de güncel servis işletim talimatlarının yerine
geçmez. Canlı kontrol ve operasyon için [CONTROL](CONTROL.md) kullanılır.

## Yerel deney artifaktları

`data/` deney verileri ve `bin/` derlenmiş ikililer yerel/ignore edilen
artifaktlardır; Git'e push edilmez. Bazı tarihsel rapor bağlantıları bu hostta
bulunabilen ham JSONL dosyalarına işaret eder. Şu anda dört eski raw JSONL
bağlantısı kayıpsız Zstandard arşivi ve indeksle saklanmaktadır; özgün raw
yolunu gerektiren analizden önce arşiv indeksini
[EVIDENCE_ARCHIVES](EVIDENCE_ARCHIVES.md) yönergesindeki `restore` komutuyla
geri yükleyin. Arşiv içeriği yerel hostta yoksa rapordaki hash/indeks bilgisi
tek başına ham veriyi getirmez.

## 25 Ekim belge sadeleştirmesi

Genel bakım kuralları [dokümantasyon standardında](DOCUMENTATION_STANDARD.md)
tanımlanır; aşağıdaki liste bu tarihteki temizlik kaydıdır.

25 Ekim 2026'da giriş belgeleri kısaltıldı; ayrıntılı protokoller ve kanıt
raporları korunur. Aşağıdaki eski plan özetlerinin güncel durumları ve sıradaki
önerileri [ROADMAP](ROADMAP.md) ile [TODO](../TODO.md) içine taşındı; ölçüm
kanıtları bağlantılı raporlardadır:

- `COEXISTENCE_PLAN.md`
- `ECOSYSTEM_LEARNING_PLAN.md`
- `PARALLEL_CAPABILITY_PLAN.md`
- `PARALLEL_WATER_DESIGN.md`
- `PARALLEL_REFUGE_DESIGN.md`
- `PARALLEL_ECOLOGY_DESIGN.md`

Silinen `SRC.md`, üretilmiş tam kaynak kodu dökümüydü; güncel kaynak için
`src/` ağacı yetkilidir. Mimari yönlendirme ana README, VISION, CONTROL ve
LEARNING_NEXT içindedir.
Bu özetlerin kaldırılması, onlarda adı geçen bütün işleri tamamlanmış veya
geçersiz saymaz. Eski metinler, temizlik öncesi Git revizyonu
`fd3d01e04fa1f2a06a3e8180370644adcfea368a` üzerinden geri alınabilir.
