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

- [Qualified coexistence ve canlı gözlem](reports/QUALIFIED_OBSERVATION.md)
- [Canlı V12 arşiv bütçesi backportu](reports/ARCHIVE_BUDGET_DEPLOYMENT_20261025.md)
- [V17 su artığı sonucu](reports/WATER_RESIDUAL_V1.md)
- [Karışık habitat vision sonucu](reports/VISION_HABITAT_V1.md)
- [İlk resolver aralığı hasar ölçümü](reports/REFUGE_FIRST_RESOLUTION_V1.md)
- [Tam kaçış hasar baseline'ı](reports/REFUGE_RISK_BASELINE_V1.md)
- [Sınırlı ecology fixture sonucu](reports/ECOLOGY_FIXTURE_V1.md)

## Tasarım ve protokoller

Yürürlükteki ya da dondurulmuş ölçüm sözleşmeleri `docs/` içindedir; her rapor
kendi kaynak, seed, süre ve yorum sınırlarını belirtir. Öne çıkanlar:
[REFUGE_FIRST_RESOLUTION_PROTOCOL](REFUGE_FIRST_RESOLUTION_PROTOCOL.md),
[ECOLOGY_FIXTURE_PROTOCOL](ECOLOGY_FIXTURE_PROTOCOL.md),
[ECOLOGY_TRAITS_PROTOCOL](ECOLOGY_TRAITS_PROTOCOL.md) ve
[WATER_RESIDUAL_PROTOCOL](WATER_RESIDUAL_PROTOCOL.md). Protokol ya da ham veri
kimliği sonuç çıktıktan sonra sessizce değiştirilmez.

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
