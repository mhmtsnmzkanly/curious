# Sığınak: açık karma denge sorunu için habitat deneyi

Öğrenme/kalıtım sonrası küçük ve yoğun yerleşimler tekrar ölçüldü:
ROADMAP_FINAL_ECOLOGY.json. 16 başlangıcın hiçbirinde etçil 2.000'e ulaşmadı.
Bunu başarılı karma ekosistem diye sunmak doğru olmaz.

Yeni açık arazi kuralı `Cell::Burrow`: yalnız sığınağa girebilme yeteneği olan
bireyler yürüyebilir; yerleşik otçul bu yeteneğe sahip. Dar giriş, büyük etçilin
ulaşmasını ve içeri saldırmasını engeller. Sığınak enerji/su/sağlık üretmez,
yeni canlı eklemez ve yaşlanmayı durdurmaz. Dışarı çıkan birey normal biçimde
avlanabilir. Görülen sığınağa kaçış SeekShelter gerekçesiyle kaydedilir.

Yeteneğin sözleşmesi Entity::can_burrow, arazi geçişi Map::can_step_for; algı,
plan, gerçek hareket, doğum konumu ve saldırı aynı fiziksel sınırı kullanır.
HTTP ile uygun olmayan türü sığınağa spawn etme reddedilir; ID tüketmez.
Checkpoint V8 sığınakları saklar. Önceki V1–V7 okunur. Eski dünyada sığınak
kendiliğinden oluşmaz; sürekli V7 koşusu bu deney için değiştirilmedi.

`mixed-refuge.json`: dört küçük sığınak, 36 otçul + 2 etçil kurucu. Kaynak
konumları açık ve nüfustan bağımsız. Sığınaksız karşılaştırma
`mixed-refuge-open.json` aynı kurucu ve kaynakları kullanır; yalnız sığınak hücreleri
yoktur. Sekiz sığınaklı seed × 4.000 tick: otçullar 8/8 sürdü; hiçbirinde avcı
kalıcı olmadı. Seed 12345'te etçil soy 20. nesle/907. ticke ulaştı; sonra bitti.
Sığınaksız dört karşılaştırma seed'inden üçünde otçullar sürdü, biri tükendi;
avcılar yine devam etmedi. Bu dört seed sekiz seed ile oran karşılaştırması
olarak kullanılmamalı; eş seed'lerin sonuçları JSON'dadır.

Müdahalesiz karma süreklilik **hâlâ açık araştırma**. Doğru sonraki teşhis, bu
uzayan soyda çift bulma, avlanma ve ölüm zamanlarını ayırmaktır. Sırf ömrü
uzatmak için rastgele parametre değiştirme veya otomatik takviye eklenmedi.

Testler gerçek saldırı engeli, metabolizma bedeli, dışarı çıkınca avlanabilirlik,
checkpoint ve arayüzün bu hücreyi göstermesini doğrular. Ham özet REFUGE.json.
