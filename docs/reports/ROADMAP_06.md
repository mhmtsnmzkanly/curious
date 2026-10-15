# 6 — Su seçimini sonuçtan öğrenme

Bağımsız `entity/learning.rs`, görünen su kaynaklarının gerçekleşen değerini
öğrenir. Kaynak başına sabit noktalı değer, deney sayısı; en fazla 32 kayıt.
%10 keşif, diğer adımlarda en iyi geçmiş değer. Gerçek içilen su ödülü artırır;
yolun enerji gideri, yaralanma ve başarısız tüketim azaltır. Öğrenme dünya
kurallarına veya canlıya ek su/enerji yazmaz. Açılıp kapanabilir (`learner.enabled`).
Bu, tek adımlı ödülden öğrenen bandit yaklaşımıdır; genel zekâ, sinir ağı veya
çok adımlı tam pekiştirmeli öğrenme sistemi değildir.

`learning_trial` örneği aynı eylem çözümleyicisiyle sekiz seed'i karşılaştırır.
Her denemede beden ve iki su kaynağı açıkça resetlenir: bu **kontrollü, tekrarlı
laboratuvar görevi**, sürekli ekosistem değildir. Yakın su için rakip içici vardır;
daha uzak su güvenilirdir. Fizik her iki grupta aynıdır.

400 denemenin son 100'ünde öğrenmeyen birey 500 su içer; öğrenen 970–1.000.
Uzak kaynağın değeri gerçek 2 enerji yol maliyeti sonrası 96, rakipli kaynağın
değeri daha düşük olur. Fark yalnız puanda değil gerçek içilen sudadır.
Sayılar ROADMAP_06.json. Genel hayatta kalma veya yaşlı avcının tehlikeliliği bu
sonuçla kanıtlanmaz. Sonraki görev/ortamlarda genelleme ayrıca ölçülmelidir.

Bireysel deneyim checkpoint V6'da saklanır; yavru boş deneyimle başlar. Karar
kayıtlarında LearnedWater gerekçesi ve gerçek sonuç görülebilir. RNG dünya
seed'inden gelir; aynı sürümde tekrar üretilebilir.
