# Su uyum labı V1 — kalibrasyon geçersiz, formal koşu yapılmadı

Birincil kategori K3; K1/K7/K8 ile kesişir. Kaynak `17d0e47`, [önceden sabitlenen protokol](../WATER_ADAPTATION_V1_PROTOCOL_20261025.md). Kullanıcı tek bireyin su koşulları değişince öğrenen, dondurulmuş ve kurallı kollarını fiziksel sonuçlarla karşılaştırmayı başlattı. Yeni simülasyon özelliği eklenmedi; V19 bağlamsal su seçimi kullanıldı, V17 su artığı kapalı tutuldu. **Sonuç öğrenmenin faydalı/faydasız olduğu değildir; fixture karşılaştırma noktasına ulaşamadı.**

## Kurulan çalışma ve kaynak düzeltmesi

`examples/water_adaptation_lab.rs` tek ordinary AI otçulun 200 tick ortak geçmişini, ardından aynı checkpoint'ten dört 200-tick suffix kolunu kaydeder: trained_online, trained_frozen, naive_online, rule_based. Tekil kaynak yenilenme değişimi fork'ta yapılır; beden/konum sıfırlaması yoktur. Supervisor ham çıktı, karar/algı/çözücü olayları, tablo eşitlikleri, censored ilk B içmesi ve ağırlıksız fiziksel vektörleri saklar. Kaynak/binary/protokol SHA ve temiz Git ağacı koşudan önce doğrulanır.

İncelemede `update_enabled=false` olmasına rağmen context tablosunun choose aşamasında TTL ile silindiği bulundu. Küçük düzeltmeyle dondurulmuş tablolar saklanır; tahmin 48 tick'ten eski kaydı yine kullanmaz. Geçen World testi tablonun60tick boyunca değişmediğini ve eski tahminin sona erdiğini doğrular. Bu düzeltme bir öğrenme faydası değildir. Kaynak default-off ve canlı V12 servisleri değişmedi.

## Kalibrasyon 62199

Seed kaydı aday listesinin ön taramasında1375 metin dosyasında geçmiş kullanım bulunmadı. Tek kalibrasyon çalıştı; altı formal seed62101..62106 tüketilmedi. Kalibrasyon manifesti `failed`, `pilot_valid=false` ve `calibration validity gate failed; formal phase blocked` olarak kapandı. Panik veya sınıraşımı değil, önceden tanımlanan bilimsel geçerlilik kapısının reddidir.

| Gözlenen ölçü | Değer |
|---|---:|
| Fiziksel World tick | 106 |
| Fork hedefi | 200, ulaşılamadı |
| Taze iki-kaynak seçimi | 3; gereken en az5 |
| Resolver gıda kazanımı | 90 enerji,23 yeme olayı |
| Resolver su kazanımı | 30 su,3 içme olayı; hepsi A |
| Resolver hareket bedeli | 94 enerji,49 hareket olayı |
| Enerjinin ilk sıfır olduğu tick | 76 |
| Son konum | (2,1) |
| Son sağlık /enerji /su | 0 /0 /0 |
| Ölüm tick ve kayıtlı neden | 106, Starvation |
| Formal tick ve dört kol ölçümü | 0; yapılmadı |

Kaynak değişimi ve frozen müdahalesi henüz uygulanmadı. Bu tek prefix, trained_online ile frozen veya rule_based karşılaştırması sayılamaz. [Deney skill’i](../../.agents/skills/curious-experiment/SKILL.md) “Stop at registered rejection or infrastructure failure” kuralını koyar; bu çalışmada kalibrasyon reddi sonrasında formal aşamaya geçilmedi. Şartları gevşeterek formal kapısı açılmadı; aynı kimlikle yeniden kalibrasyon veya sonuca göre tuning yapılmadı.

## Sıfır tick ile ham kayıt teşhisi

Gıda kaynağı(0,-1), kapasite30, refill2/2tick. Bireyin enerjisi76.tick'te2'den0'a, (2,1)'deki hareket/metabolizma aralığında indi. Tick 76’daki kısmi yaklaşma reddi, enerji sıfıra düşen aralıkta oluştu; ardından tick 77–105 boyunca 29 sıfır-enerjili yaklaşma girişimi `Approach/OutOfRange` ile reddedildi. Birey gıdaya kalan hareket bedelini ödeyemedi. Bu aralıkta gıda yeniden birikerek ölüm anında30'a ulaştı. Sorun salt gıda yokluğu değil, **enerji tükenince kaynağa erişilememesi**.

Başlangıç enerji80 +gıda90 −hareket94 =76 diğer net enerji gideri; bu fark bütünüyle tek bir maliyet türüne nedensel olarak atanmadı. Sağlık kaybı 120; tick başındaki suya göre 44'ü su pozitifken, 76'sı su sıfırken görüldü. Saldırı yoktu; bu gözlemler sağlık kaybının tamamını yalnız açlığa veya susuzluğa paylaştırmaya yetmez. Son ölüm nedeninin Starvation olması diğer stresin yokluğu değildir.

Yeni protokol tasarımında gıdanın su koridorundaki olası konumlardan düşük enerjide erişilebilir olması sağlanabilir; besin miktarını keyfî artırmak veya bedeni her tick doldurmak çözüm olarak uygulanmadı. Böyle bir fixture ayrı kimlik ve kalibrasyon ister. Bu görevde V2 veya formal deney başlatılmadı.

## Replay ve doğrulama

`web/water-lab.html` yerel dosya seçicili, ağsız, salt okunur bir görüntüleyicidir. Dört gerçek kol varsa ortak suffix zaman çizgisini; bu başarısız kalibrasyonda ise açıkça **tek ortak prefix** panelini gösterir. Yeni sunum paketi `water-adaptation-prefix-replay-v1` şemasıyla mevcut106tick kaydından üretildi; ek World tick'i yok. Orijinal koşucunun ürettiği boş-kollu replay dosyası değiştirilmedi; sunum ayrı dizindedir.

Yerel paket: `data/experiments/water-adaptation-v1-presentation-20261025/water-lab-replay.zip`. ZIP'i çıkarıp water-lab.html dosyasını açın, dosya seçiciden replay.json seçin. Olaylardan çıkarılmayan başarı/tercih iddiası gösterilmez. Paket yeni canlı web servisi veya7883 dağıtımı değildir.

10 Rust öğrenme entegrasyon testi,6 Python analiz testi, yeni replay Node syntax/self-test ve mevcut experience/documentary kontrolleri geçti. Gerçek prefix bundle doğrulandı; tick106 sağlık0 kaydı projeksiyonda kontrol edildi. Gerçek tarayıcı screenshot/etkileşim testi yapılmadı. Türkçeleştirme sırasında bir Node testinde eski İngilizce hata metni beklentisi güncellenip tekrar geçti; bu düzeltme kalibrasyondan önceydi.

Ham kanıt: `data/experiments/water-adaptation-v1-calibration-20261025/` içinde manifest, raw JSON, boş stderr, geçerlilik özeti. Preflight seed taraması/build/test logları ve korunmuş probe ayrı `water-adaptation-v1-preflight-20261025/` dizininde; sunum hash'leri presentation dizinindedir. Bunlar yerel/ignore edilen artifaktlardır; Git'teki yapılandırılmış özet ham kaydın yerine geçmez. Kaynak protokol ve eski raporlar değiştirilmedi. Göreve ait inaktif target temizlenir; canlı7882/7883 ve frozen ikililer değişmez.
