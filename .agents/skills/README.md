# Curious ajan becerileri

Bu beş beceri herhangi bir ajanın okuyabileceği Markdown ve `name`/`description`
YAML başlığı kullanır. Model, sağlayıcı, özel araç API'si veya global ajan
kurulumu gerektirmez. Talimatlar İngilizce, kullanıcıya yanıt dili kullanıcının dilidir.

| Beceri | Ne zaman kullanılır? | Yaygın roadmap alanları |
|---|---|---|
| [curious-development](curious-development/SKILL.md) | Motor, öğrenici, kalıtım, kayıt veya arayüz geliştirme | [K2](../../docs/ROADMAP.md#canli-beden), [K3](../../docs/ROADMAP.md#bireysel-ogrenme), [K8](../../docs/ROADMAP.md#motor-isletim); ana değişikliğe göre diğerleri |
| [curious-documentation](curious-documentation/SKILL.md) | Proje belgelerini oluşturma, güncelleme, düzenleme veya budama | Belgenin konusu ve K1–K8 ana alanı |
| [curious-experiment](curious-experiment/SKILL.md) | Hipotez, kontrollü deney, karşılaştırma ve sonuç analizi | Ölçülen hipoteze göre sekiz alanın tümü |
| [curious-operations](curious-operations/SKILL.md) | Servis, dünya, disk, kayıt ve uyarı kontrolü | [K8](../../docs/ROADMAP.md#motor-isletim), arşiv görüntüleyici için [K7](../../docs/ROADMAP.md#gozlem-arayuz) |
| [curious-review](curious-review/SKILL.md) | Mimari, davranış ve bilimsel kanıt denetimi | İncelenen değişiklik ve iddianın ilgili alanı; gerektiğinde alanlar arası |

Otomatik beceri keşfi destekleyen ajan kendi yükleme yöntemini kullanabilir.
Otomatik keşif desteklemeyen ajana ilgili `SKILL.md` dosyasını okumasını söylemek
ve depo erişimi vermek yeterlidir. Her ajanın bu dizini kendiliğinden keşfedeceği
veya bütün araçlara erişebileceği garanti edilmez. Depo giriş noktası
[AGENTS.md](../../AGENTS.md) bu kataloğa bağlantı verir. Başka bir ajanın proje
başlangıç talimatına bu dosyanın yolu eklenebilir; talimatları kopyalayıp ayrı
sürümler üretmeyin.

Örnek görev: “`.agents/skills/curious-review/SKILL.md` dosyasını kullanarak son
av öğrenmesi değişikliğini incele; canlı dünyayı değiştirme.”

Belge işi için `.agents/skills/curious-documentation/SKILL.md` dosyasını kullanın;
ayrıntılı [yazım standardı](../../docs/DOCUMENTATION_STANDARD.md) depo içinde
tutulur.

Yollar depo köküne göredir; becerileri tek başına başka yere kopyalamak yerine
depoyla birlikte kullanın. Araç yoksa mevcut eşdeğerini kullanın, çalıştırılmayan
kontrolü açıkça belirtin. Bu dosyalar görevlendirme, üretime dağıtım, dışarıya
mesaj veya veri silme için ek yetki vermez. Geçerli kullanıcı talimatı ve yürütme
ortamının izinleri önceliklidir. Alt ajan desteği veya belirli bir model gerekmez.
Mevcut oturumda verilmiş kullanıcı yetkisini izleyin; roadmap sınıflandırması bu
kapsamı genişletmez veya aynı yetkiyi gereksiz yere yeniden istemeyi gerektirmez.

Kategori, iş sırası değildir. Her özellik için bir ana sahip alan seçin; diğer
alanlardaki gerekli arayüz bağımlılıklarını belirtin. Alan tanımları ve güncel
plan [roadmap'tedir](../../docs/ROADMAP.md); beceri kullanımı tek başına kod,
deney veya dağıtım yetkisi vermez.
