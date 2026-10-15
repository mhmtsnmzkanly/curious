# Av–avcı sürekliliği: uygulama sözleşmesi

Başlangıç: 929d5bb / checkpoint V8. Kullanıcının 11 aşamalı planı uygulama
talimatı üzerine başlatıldı. Çalışan 7879 ve 7881 dünyaları deney değildir.

1. [x] Başlangıç sürümünü ve eş seedli karşılaştırmaları sabitle.
2. [ ] Seed 12345 ve erken yok oluş için tam kayıtlı soy/karar teşhisi.
3. [ ] Avcının besin, bakım, avlanma, üreme ve ceset kaybı bütçesi.
4. [ ] Kanıtlanan karar hatalarını küçük senaryolarla düzelt.
5. [ ] Yavru → yetişkin → üreyen yetişkin dönüşümünü ve eş fırsatlarını ölç.
6. [ ] Sığınak kullanımı ve erişilebilir av üretimini karşılaştır.
7. [ ] Tek ayar yerine dayanıklı bir koşul aralığı bul.
8. [ ] Görülmemiş 30 seed × 100.000 tick, en az 27 birlikte süreklilik.
9. [ ] Geçici kaynak kaybından toparlanmayı ayrı müdahale deneylerinde ölç.
10. [ ] Öğrenme, mutasyon ve ikisini birlikte eş koşullarda doğrula.
11. [ ] Ayrı karma gözlem servisi, gerçek 1/7/30 gün ve işletim ölçümü.

Kabulde son bölümde doğum ve yavruların üremesi de gerekir. Sonuçtan sonra
eşik veya seed listesi değiştirilmez. Hızlandırılmış deney gerçek gün sayılmaz.
Tek başarılı koşu kalıcı denge değildir. Başarısız koşular raporda korunur.

Temel teşhiste öğrenme ve mutasyon kapalı kontrollere ek olarak V8 davranışını
aynen yeniden üreten koşular tutulur. Yeni mekanizmalar tek tek karşılaştırılır.
Yeni canlı, bedelsiz enerji veya nüfusa bağlı gizli dengeleyici eklenmez.
Gerekçeli biyolojik parametre değişikliği mümkündür; etki ve bedel raporlanır.

Her anlamlı aşama: deney/rapor → ilgili testler → commit → gerektiğinde build
temizliği → önemli bulgu veya tamamlamada kullanıcıya Notify. Push istenmedi.
