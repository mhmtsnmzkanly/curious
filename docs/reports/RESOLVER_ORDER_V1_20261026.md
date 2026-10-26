# Resolver sırası V1: tek-tik tanı sonucu

**Tarih:** 26 Ekim 2026. **Durum:** Beş sabit vakada tamamlanmış, betimsel mekanizma ölçümü. Genel fayda, öğrenme/ekoloji kabulü, varsayılan değişikliği veya canlı kullanım kararı değildir.

## Sonuç

Her hücrede bitki enerjisi / alınan su / ödenen hareket enerjisi yer alır. Üç kol aynı hedef öncesi durum ve ortak hazırlanmış karar girdilerinden başlatıldı.

| Vaka (hedef tik) | A: temel karar, doğal sıra | B: aday planlar, sabit yerleşim | C: aday planlar, doğal sıra |
|---|---:|---:|---:|
| meadow 62502 (94) | 16 / 8 / 17 | 22 / 8 / 17 | 22 / 12 / 18 |
| meadow 62503 (92) | 18 / 22 / 21 | 24 / 22 / 21 | 24 / 12 / 19 |
| meadow 62505 (72) | 38 / 16 / 16 | 46 / 16 / 16 | 46 / 16 / 15 |
| sparse_resource 62503 (55) | 10 / 10 / 9 | 10 / 10 / 9 | 10 / 10 / 9 |
| sparse_resource 62505 (65) | 8 / 0 / 0 | 8 / 0 / 0 | 8 / 0 / 0 |

Ölçü birimleri: bitki enerjisi, su miktarı ve hareket için harcanan enerji. 15 kol sonucunun hiçbirinde ölüm olmadı. Sonuçlar, bu tek resolver tikinde kaynakların aktörlere ve eylem sırasına göre nasıl tahsis edildiğini gösterir; sonraki tikleri ya da yaşam boyu sonucu ölçmez.

B ve C aynı aday plan üyeliğini kullanır. B–C farkı yalnız `Eat` sırası değildir: sabit B yerleşimi, C'deki doğal döndürülmüş `Eat`, `Drink` ve diğer plan sıralarıyla karşılaştırılır. Beş vakanın tümünde toplam bitki enerjisi B ve C'de aynı kaldı. Bununla birlikte meadow 62503'te su toplamı B'de 22, C'de 12 oldu; toplam bitki enerjisi her ikisinde 24'tü. meadow 62502 ve 62505 ile sparse_resource 62505'te bazı öğün kazananları da değişti. Bu, ortak sıra düzeninin kısa vadeli tahsisi etkileyebildiğine dair bu vakalara özgü bir mekanizma gözlemidir.

A–B, aday override plan üyeliğinin yanı sıra yeni `Eat` aktörlerini belirli sıraya ekleme müdahalesini de içerir; saf bir karar etkisi olarak yorumlanamaz. sparse_resource vakalarında toplam fiziksel vektörler kollarda aynı kaldı. Reddedilen Eat/Drink sayıları, aktör kazananları, olay listeleri ve beden değişimleri [JSON ekinde](RESOLVER_ORDER_V1_20261026.json) vaka ve kol bazında kayıtlıdır.

## Koşu ve doğrulama

Önceden kaydedilmiş protokol beş seçilmiş olumsuz seed vakasını belirledi: meadow 62502/62503/62505 ve sparse_resource 62503/62505. Her vaka için eski `control` başlığındaki `initial_state`'ten hedef öncesine sıradan tiklerle replay yapıldı; her replay tikinin ham kayıtta bulunan olay ve metrik izdüşümleri karşılaştırıldı. Ham dosyada geçmiş tiklere ait tam State/RNG checkpoint'i bulunmadığından tarihsel State/RNG eşitliği iddiası yoktur. Hedef State/RNG restore-roundtrip'i, A/B/C kol kapıları, A'nın hedef ham satırla karşılaştırması ve A/C'nin olağan off/on referans eşitlikleri geçti.

Tamamlanan koşu kimliği `resolver-order-v1-runnerfix-20261026`; 398 çözücü-tik çağrısı ve beş ortak preamble içerir. Kayıt manifesti SHA-256 `e10fae2ea6c4221d0813d183f7a8aae5cadfe10f3d26afee45f738f072df0a11`. Kaynak commit `93cc4b2445fa44f9f7bd2e16fe418be1bd5d76c6`, izlenen dosyalar ağacı SHA-256 `3a01eaf3a481c4b6698a0879628847f1152dd2ecc29825df363c0e3b2f88e6e0`; kullanılan ikili SHA-256 `ff32c4fc7cad6e86ad97872e88f9c849ae282814a121c19b54b0193a27c9462b`. Dondurulmuş protokol (`e8357e8d2d0795ae110579befb416585f50161355f140ff5d80252dc4402f6ff`), kayıt eki (`7a271bbc907cb940b4190f1a734896b304f202a92c11282772c3719b4158a65b`) ve runner düzeltme notu (`dfa985b508e999634793b07741c81aa8b8f2b90e75a1be616c5813c4a57dde76`) değişmeden kaldı.

İlk kayıtlı deneme runner'ın prefix alan adı uyuşmazlığı nedeniyle başarısız oldu ve `failed` olarak korundu (manifest SHA-256 `9b8c43fcef5e4eb3025375f6cf7e40483d2a52c39c04fb753a2c48192b17fb28`). İlk vakada 98 çözücü çağrısı ve bir preamble çalışmıştı; bu bir sonuç koşusu sayılmaz. Dar şema düzeltmesi sonrası farklı kimlik ve dizine yapılan tamamlanmış koşu beş aynı vakayı kullandı. İki denemenin toplam iş yükü 496 çözücü çağrısı ve altı preamble'dır; örneklem büyüklüğü hâlâ beş vakadır.

Çevrimdışı analiz, ham girdi/çıktı hash'lerini, prefix ve hedef karşılaştırmalarını, fiziksel vektörleri, aktör denetimlerini ve sıra kurallarını doğruladı (final analysis SHA-256 `7a8c361b624278a1e9159a73ac8e1d8bed8886a72b8e79768c2901978baca608`). Bağımsız fiziksel ve bütünlük denetimleri de geçti. Bu kontroller ölçüm bütünlüğünü destekler; biyolojik yarar ölçütü değildir.

## Sınırlar ve yeniden üretim

Bu vakalar rastgele/genel bir popülasyon örneği değildir. Müdahale tek tikle sınırlıdır; sağkalım, üreme, yaşam boyu performans, öğrenme veya ekosistem etkisi bilinmiyor. Beş hedef ve bu resolver kesiti dışındaki sıra etkisi **Unknown** kalır. Bu rapor varsayılanı açma veya dağıtım gerekçesi oluşturmaz.

Dondurulmuş koşu ve analiz dosyaları yerel `/root/curious/data/experiments/resolver-order-v1-runnerfix-20261026/` ve `/root/curious/data/operations/resolver-order-20261026/` altındadır; repoya dahil değildir. Tekrar için sabit protokol, runner ve önceden hash'lenmiş girdiler gerekir; bu rapor yeni çalışma başlatma talimatı değildir.
