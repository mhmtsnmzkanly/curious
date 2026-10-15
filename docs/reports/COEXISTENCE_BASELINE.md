# Avcı soyunun kesilmesi: V8 başlangıç teşhisi

929d5bb kaynakları, c06ed79 dağıtım ikilisi (aynı V8 motor), müdahalesiz
mixed-refuge.json, seed 12345 ve 1, 1.100 tick. Ayrı HTTP süreçleri ve tam SQLite
geçmişi: data/experiments/coexistence-baseline-v8. Öğrenme ve mutasyon önceki
sonucu aynen üretmek için açık. Kontrollü kapalı koşullar ayrıca çalıştırılacak.

## Aynı yok oluşun iki farklı zinciri

12345: 30 etçil doğumu, hepsi yetişkin; 26 yavru da üredi, 20. nesil. 22 yaşlılık,
9 açlık, 1 susuzluk ölümü. Son avcı tick907'de hayatta, 908'de ölü. Tick500'de
32 otçul/2 etçil; 700'de 18/8; 800'de 3/6. Son avcı id307, 31 tick üremeye
hazır fakat hiç hazır eş görmedi. Soyun sonundaki eş yokluğu önceki besin
daralmasından ayrı bir rastlantısal eş arama hatası diye yorumlanmamalı.

Seed1: yalnız bir etçil doğdu (id55, tick25). Yetişkin oldu, 300 enerji yedi,
120 tick yaşadı; **hiçbir tick üremeye hazır olmadı**. Son gözlenen canlı
sağlığı 53, üreme için gereken 70'in altında. Diğer kurucu 87'de açlıktan,
ikinci kurucu 121'de yaşlılıktan öldü; yavru146'da yaşlılıktan öldü.

Karar beslenmeyi %75 rezervde bırakıyor; iyileşme de %75 rezerv istiyor ve
metabolizma önce çalışıyor. Bu, yaralanmış bireyin iyileşme rezervi biriktirmesini
zorlaştıran somut politika çatışması. İyileşmenin bedelini kaldırmadan daha yüksek
beslenme hedefi küçük kontrollü senaryoda sınanacak; denge çözümü diye sunulmayacak.

## Ölçülen besin bütçesi ve sonraki hipotez

Seed1 üç etçili toplam490 enerji yedi,603 enerji harcadı (251 hareket,93 saldırı;
geri kalanı bazal tüketim/iyileşme),44 enerjiyi yavruya aktardı. Başlangıç ve
yavru rezervleri ayrı; bunlar yalnız başına ömür boyu kazanç değildir.

Seed1'de otçul cesetlerinde23.280, seed12345'te8.740 enerji tüketilmeden süresi
doldu. Bu dünya toplamıdır; avcıların hepsine erişebildiği anlamına gelmez.
Mevcut ceset ömrü5 tick, öğün10, yetişkin otçul cesedi60 enerji. Tek yiyicinin
tek cesedi boşaltma süresi bile ömründen uzun. Ceset saklanma süresinin beslenme
verimine etkisi, erişilebilirlik ve rekabetle birlikte kontrollü karşılaştırılacak.
Bu rakamlar eti artırmak gerektiğini kanıtlamaz.

## Yeniden üretim

scripts/measure_ecosystem.py --binary bin/releases/curious-c06ed79 --output YENI_DIZIN
--scenario scenarios/mixed-refuge.json --seeds 12345,1 --ticks 1100

scripts/diagnose_coexistence.py YENI_DIZIN/seed-12345 --output SONUC.json

Analiz salt okunur raw/gz shardları işler. Süresi dolan et, Corpse→Removed geçişinde
bir kez sayılır. Enerji harcaması ardışık kayıtlarda gerçek rezerv farkı+tüketim
üzerinden hesaplanır; ebeveynin yavruya verdiği enerji ayrı tutulur. Hareket
maliyeti net konum farkından değil gerçekleşmiş moves_used'dan alınır.

Tam özet ve etçil bireyler COEXISTENCE_BASELINE.json içinde. Kalıcı denge açık.
