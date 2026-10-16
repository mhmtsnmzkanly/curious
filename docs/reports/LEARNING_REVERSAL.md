# Su öğrenmesi: ortam değişimi deneyi

Bu kontrollü laboratuvar, sabit ortamda öğrenmenin yanı sıra eski tercihin
çevre değişince güncellenmesini ölçer. 400. denemede rakip, önceden tercih edilen
uzak su kaynağına taşınır. Kaynaklar ve bedenler her denemede açıkça sıfırlanır;
ekolojik süreklilik, yaş veya genel öğrenme kanıtı değildir. Motor değiştirilmedi.

Sekiz seed: 1, 2, 3, 4, 5, 42, 12345, 2026. Her 100 denemede edinilebilecek
azami su 1000 birimdir. Ham çıktı `data/experiments/learning-reversal.jsonl`.

| Seed | Öğrenilmiş: değişim öncesi son 100 | Değişim sonrası ilk 100 | Son 100 |
|---|---:|---:|---:|
| 1 | 980 | 960 | 960 |
| 2 | 980 | 930 | 990 |
| 3 | 980 | 880 | 960 |
| 4 | 980 | 800 | 960 |
| 5 | 990 | 950 | 960 |
| 42 | 980 | 950 | 1000 |
| 12345 | 1000 | 920 | 980 |
| 2026 | 970 | 970 | 990 |

Öğrenme kapalı politika her seedde değişimden önce 500, değişimden sonra 1000
birim topladı. Öğrenen politika değişimden sonra toparlanıyor, fakat son 100
denemede bile bazı seedlerde keşif maliyeti var; kurallı politikayı her ortamda
yenmiyor. Son pencere değerleri uyumu gösterir; tam toparlanma zamanı ölçülmedi.
Bu seedler geliştirme içindir, ayrı tutulmuş değerlendirme seedleri değildir.

İki regression testi sabit rekabette öğrenme faydasını ve değişen rekabette
eski tercihe kilitlenmemeyi doğrular. Bu test motoru düzeltmez; yeni öğrenme
mekanizmasına geçerken korunacak davranış kanıtı sağlar. Sonraki uygulama
sözleşmesi `../LEARNING_NEXT.md`: gecikmiş sonuç, maliyet muhasebesi, sınırlı
av tercihi, bireysel deneyim ve kalıtım ayrımı.
