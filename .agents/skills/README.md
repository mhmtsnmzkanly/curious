# Curious ajan becerileri

Bu dört beceri herhangi bir ajanın okuyabileceği Markdown ve `name`/`description`
YAML başlığı kullanır. Model, sağlayıcı, özel araç API'si veya global ajan
kurulumu gerektirmez. Talimatlar İngilizce, kullanıcıya yanıt dili kullanıcının dilidir.

| Beceri | Ne zaman kullanılır? |
|---|---|
| [curious-development](curious-development/SKILL.md) | Motor, öğrenici, kalıtım, kayıt veya arayüz geliştirme |
| [curious-experiment](curious-experiment/SKILL.md) | Hipotez, kontrollü deney, karşılaştırma ve sonuç analizi |
| [curious-operations](curious-operations/SKILL.md) | Servis, dünya, disk, kayıt ve uyarı kontrolü |
| [curious-review](curious-review/SKILL.md) | Mimari, davranış ve bilimsel kanıt denetimi |

Otomatik beceri keşfi destekleyen ajan kendi yükleme yöntemini kullanabilir.
Otomatik keşif desteklemeyen ajana ilgili `SKILL.md` dosyasını okumasını söylemek
ve depo erişimi vermek yeterlidir. Her ajanın bu dizini kendiliğinden keşfedeceği
veya bütün araçlara erişebileceği garanti edilmez. Depo giriş noktası
[AGENTS.md](../../AGENTS.md) bu kataloğa bağlantı verir. Başka bir ajanın proje
başlangıç talimatına bu dosyanın yolu eklenebilir; talimatları kopyalayıp ayrı
sürümler üretmeyin.

Örnek görev: “`.agents/skills/curious-review/SKILL.md` dosyasını kullanarak son
av öğrenmesi değişikliğini incele; canlı dünyayı değiştirme.”

Yollar depo köküne göredir; becerileri tek başına başka yere kopyalamak yerine
depoyla birlikte kullanın. Araç yoksa mevcut eşdeğerini kullanın, çalıştırılmayan
kontrolü açıkça belirtin. Bu dosyalar görevlendirme, üretime dağıtım, dışarıya
mesaj veya veri silme için ek yetki vermez. Geçerli kullanıcı talimatı ve yürütme
ortamının izinleri önceliklidir. Alt ajan desteği veya belirli bir model gerekmez.
