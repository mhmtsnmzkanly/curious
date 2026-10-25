# Seed başına kabul tanısı ve V12/V19 gölge köprüsü — 25 Ekim 2026

Birincil kategori K8; K3/K5/K6 ile kesişir. [Sabit tanı protokolü](../QUALIFICATION_DIAGNOSTIC_PROTOCOL_20261025.md), [ortak motor koşucusu](../SHADOW_ENGINE_BRIDGE.md). Kullanıcı bekleyen değişikliklerin kontrolünü ve önceki eksik işlerin sürdürülmesini istedi; başlangıçta çalışma ağacı temiz, HEAD `c5ca361` idi. Kaynak değişikliği üç Luna 6 göreviyle hazırlanıp root tarafından incelendi. Yeni özellik veya canlıya geçiş yapılmadı.

## Kod ve kanıt düzeltmesi

`18081b5`, seed başına sağlık/sağkalım vektörlerini ve nötr/iyileşme/kayıp ayrımını JSON'a ekler. Eski toplu `PassLiveCandidate` enum'u uyumluluk için korunur; bu çalışmanın kabul kararı değildir. Birleştirilmiş tanı, her seedde üç sağkalım bileşeninin kontrol altına düşmemesini ve toplam vektörün sıkı iyileşmesini ayrı değerlendirir. Eşitlik nötrdür. `total_damage_taken` adı eski koddadır; gerçekte atıfsız pozitif tick sağlık azalmasıdır, saldırı hasarı değildir.

Soak koşucusu artık olaylardan tür bazlı doğum/ölüm, yaşayan birey-tick, gerçek başlangıç ID kümesine göre yaşayan kurucu sayısı, 100-tick aralık özetleri ve doğrulanmış endpoint State saklar. Aktif ve uyuyan canlılar birlikte sayılır.

Yeni sentetik birim fixture'ında toplama hatası ilk çalışmada bir testin başarısız olmasına yol açtı; ilk log korundu. `389f24a` yalnız bu sentetik toplama örneğini düzeltir; deney, motor ve eşikler değişmez. Ayrı tekrar test geçti. Fiziksel ölçüm çıktısı ilk koşudan korunur; aynı seedler yeniden koşturulup bağımsız örneklem gibi sunulmadı.

## 10 seed ×150 tick sonuçlarının kapsamı

401..410 daha önce görülmüş seedlerdir; bu çalışma bağımsız held-out kabulü değildir. 120 seed-kol vektörü JSON'a yazıldı.

| Kol | Seed başına sonuç | Tanı |
|---|---|---|
| Yalnız bakım | 2 sıkı iyileşme, 8 nötr, 0 sağkalım kaybı | Bu fixture ve ufukta toplam vektör iyileşti |
| Yalnız ilk-aralık sığınak | 10 nötr | Kayıt var; fiziksel fayda gösterilmedi |
| Bakım+sığınak | 2 sıkı iyileşme, 8 nötr, 0 sağkalım kaybı | Bakımın ötesinde ek fayda bu tabloda gösterilmedi |
| Su bağlamı | 5 iyileşme, 5 sağkalım kaybı | Seed başına kapı geçilmedi |
| Merak | 2 iyileşme, 4 nötr, 4 sağkalım kaybı | Kapı geçilmedi |
| Yaralanma kapasitesi | 4 iyileşme, 6 sağkalım kaybı | Kapı geçilmedi |
| Pusu | 3 iyileşme, 6 sağkalım kaybı, 1 diğer ödünleşim | Kapı geçilmedi |
| Koordinasyon | 4 iyileşme, 6 sağkalım kaybı | Kapı geçilmedi |
| Gün/gece | 5 iyileşme, 5 sağkalım kaybı | Kapı geçilmedi |
| Mevsim+in | 3 iyileşme, 6 sağkalım kaybı, 1 diğer ödünleşim | Kapı geçilmedi |
| Tüm özellikler | 1 iyileşme, 9 sağkalım kaybı | Kapı geçilmedi |

Bu seçim ölçütü fizik kurallarının gerçekçiliğini ölçmez; örneğin kışın nüfusu azaltması tek başına motor hatası değildir. Mekanizma faydası belirli aynı-dünya karşılaştırmasına aittir.

## Eksik V12/V19 karşılaştırması

Aynı kapalı shard'ın SHA-256 ve SQLite integrity_check sonucu doğrulanarak record14/tick12639/V12/630slot ayrı kopyaya çıkarıldı. Input SHA: `357870bd9a3d414704ed6befbea050852028f54af304c27cdca482dc31abdad2`. Yerel ayrıca arşiv index'i bulunmadı; yeni manifest dosya kimliği ve çıkarım kaydını tutar, geçmişte bulunmayan index kanıtı varmış gibi sunulmaz.

Ortak Rust koşucusu iki farklı kütüphaneye derlendi: frozen V12 kaynak `8dc23de785dcfcd248611c17de0e9351b0116222` ve güncel V19. Bu, frozen kaynaktan yeniden derlenmiş V12'dir; üretimdeki release dosyasıyla bit-bit ikili eşdeğerliği iddia edilmez. Ortak başlangıç beden/genom alanları, konumlar, harita ve RNG korundu; V19'un yeni beden/genom alanları sıfır varsayılanla eklendi.

| Ölçü, 1.000 tick | V12 kaynak | V19 kapalı kontrol |
|---|---:|---:|
| Kümülatif yaşayan birey-tick | 593876 | 595736 |
| Son yaşayan otçul / etçil | 508 /94 | 482 /98 |
| Son toplam canlı | 602 | 580 |
| Doğum otçul /etçil | 5053 /91 | 5074 /87 |
| Ölüm otçul /etçil | 5078 /80 | 5125 /72 |
| 5-tick checkpoint resume State+RNG+olay eşitliği | geçti | geçti |

Her iki motor kendi içinde devam doğrulamasını geçti; motorlar arası yörünge aynı değildir. V19 sonunda 22 daha az canlı taşırken 1860 daha fazla birey-tick üretir. Bu ödünleşimdir; yalnız endpoint veya yalnız kümülatif metrikten genel üstünlük çıkarılamaz. İlk olay dizisi farkı tick12640'ta görülür; olay sıralamasının farklı olması tek başına belirli bir fizik kuralının neden olduğunu kanıtlamaz.

## Dört V19 kolunun tam dönem ölçümü

| V19 kolu | Yaşayan birey-tick | Kontrole fark | Son canlı | Doğum /ölüm | Son yaşayan kurucu |
|---|---:|---:|---:|---:|---:|
| Kapalı kontrol | 595736 | 0 | 580 | 5161 /5197 | 15 |
| Yalnız bakım | 591932 | -3804 | 575 | 5101 /5142 | 12 |
| Yalnız ilk-aralık sığınak | 593113 | -2623 | 584 | 5121 /5153 | 15 |
| Bakım+sığınak | 594308 | -1428 | 593 | 5146 /5169 | 15 |

Her kol için `616 + doğum - ölüm = son canlı` bilançosu doğrulandı. Ayrı köprü koşucusunun V19 kapalı endpoint State'i, dört kol koşusunun kapalı endpoint'iyle birebir aynı çıktı. Dört endpoint de State::restore doğrulamasını geçti.

Birleşik paket sonunda +13 canlı taşır ama toplam yaşayan birey-tick %0,240 daha düşüktür. Bu sonuçla tam dönem sağkalım-kaybı-yok veya Pareto üstünlüğü kabul edilemez. Yalnız bakım küçük fixture'daki kazancını bu gerçek-dünya başlangıcında genel üstünlük olarak korumaz. Bunlar aynı başlangıçtan deterministik kollar; bağımsız popülasyon örneklemleri veya evrensel zararlılık/fayda kanıtı değildir.

**Kurucu iddiasının düzeltmesi:** Paket tick12839'da hâlâ 66 başlangıç bireyi, tick13639'da 15 başlangıç bireyi taşır. Yeni nesillerin FI tablolarının tamamına sahip olması, bütün kurucuların yerini bıraktığını göstermez. Max generation üzerinden tüm soyların34 ardışık nesil ürediği de çıkarılamaz; farklı soyların başlangıç nesil etiketleri farklıdır.

Birey başı bellek satırı/pencere sınırları uygulanmıştır; bundan dünya nüfusu veya toplam checkpoint boyutunun gelecekte sabit kalacağı çıkarılamaz. Bu çalışma bir 5.000 tick koşusuna uzatılmadı; mevcut uzun-ufuk sonucu kabul iddiasını zaten sınırlıyor.

## Doğrulama, ham kanıt ve kalan karar

Odaklı ilk paket: 3 test geçti, 1 yeni sentetik-fixture testi başarısız, 1 soak testi ignored. Fixture düzeltildikten sonra yalnız ilgili test tekrar çalıştırılıp geçti. Ayrıca ignored dört-kol1.000 tick testi açıkça çalıştırıldı ve geçti. Tüm depo regresyon paketi bu görevde yeniden çalıştırılmadı.

Toplam ana tanı 24.000 World tick (18.000 küçük fixture +6.000 büyük gölge); ayrıca iki 5 tick resume çatalı, odaklı 20+20 tick gölge kontrolü ve bir doğum testi tick'i olmak üzere 51 doğrulama tick'i yürütüldü. Protokoldeki ana çalışma sınırı değişmedi; ek 40 tick odaklı kısa-gölge test maliyeti burada ayrıca açıklanır. İlk başarısız test logu ve yeni kaynak kimliği korunur; fiziksel parametre veya seed değiştirilmedi.

Kanıt kökü yerel ve Git-ignore edilen `data/experiments/qualification-diagnostic-20261025/` dizinidir: manifest, `per-seed.json`, `analysis.json`, köprü özetleri ve tick olay JSONL'leri, stdout/stderr, koşucu/ikili hash'leri ve dört kol endpointleri. Git raporundaki yapılandırılmış özet, ham verinin yerine geçmez. Canlı V12 7882 ve replay7883 servisleri değiştirilmedi; kapalı arşive veya canlı veriye yazılmadı.

İşletim incelemesinde eski bir frozen ikilinin desteklemediği yardım/sürüm argümanları yanlışlıkla çalıştırıldı; ilgili süreç durduruldu. Canlı PID119175 ve replay1137 etkilenmedi; yeni run dizini veya arşiv değişikliği saptanmadı. Sonraki ölçüm yalnız dosya tabanlı, ağsız koşucuyla yapıldı.

Canlı geçiş bu görevde yetkilendirilmiş işlem değildir ve başlatılmadı. Eski rapordaki `/control/pause` komutu mevcut API ile uyumlu değildir; gerçek rota POST `/control` ve JSON `{"action":"pause"}` biçimidir. Ayrıca bakım/sığınak bayraklarını açan genel bir control komutu yoktur; uygulanabilir offline göç/ayar adımı ve geri dönüş doğrulaması hazırlanmadan önceki runbook yürütülmemelidir. Bu rapor tamamlanmış bir deployment protokolü değildir.
