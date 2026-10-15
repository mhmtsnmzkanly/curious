# 4 — Hareket ve kaynak sözleşmeleri

Hareket, kaçış, yemek ve suya yaklaşma ortak `systems/locomotion.rs` üzerinden
çözülür. Plan anından sonra ara hücreye taşınmış rakip artık içinden geçilecek
bir nesne değildir: her adım çözümleme anında doluluk, beden bütçesi ve araziyle
yeniden doğrulanır. Kısmi yolun gerçek maliyeti ödenir. Gerçek konuma göre tüketim
mesafesi denetlenir; ulaşılamayan kaynak için ret olayı çıkar.

`Cell::Wall`, HTTP set_cell ve senaryo resources içinde kullanılabilir. Duvar
checkpointte korunur; iki duvar arasından çapraz geçiş yoktur. Duvar içine canlı
yerleştirme reddedilir. Çevre kaynağı duvarın üstüne yenilenmez.

Checkpoint V4, önceki V1–V3 okunur. Hareket sözleşmesi değiştiğinden eski sürümle
aynı gelecek garantisi yoktur. Bağımsız hareket/duvar testleri ve HTTP/crash
entegrasyonu geçti. Diyet, sonlu tüketim, paylaşım ve dönen öncelik mevcut
testlerle korunur. Gelecekte eklenen yetenekler de bu sözleşmelerden geçmelidir.
