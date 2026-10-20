# Av–avcı sürekliliği: uygulama sözleşmesi

Başlangıç: 929d5bb / checkpoint V8. Kullanıcının 11 aşamalı planı uygulama
talimatı üzerine başlatıldı. Çalışan 7879 ve 7881 dünyaları deney değildir.

1. [x] Başlangıç sürümünü ve eş seedli karşılaştırmaları sabitle.
2. [x] Seed 12345 ve erken yok oluş için tam kayıtlı soy/karar teşhisi.
3. [x] Avcının besin, bakım, avlanma, üreme ve ceset kaybı bütçesi.
4. [x] İlk düzeltmeler: ciddi yaralanma rezervi, bedelli kaynak seçimi, görülen eş hafızası.
5. [x] Yavru → yetişkin → üreyen yetişkin dönüşümünü ve eş fırsatlarını ölç.
6. [x] Sığınak kullanımı ve erişilebilir av üretimini karşılaştır.
7. [x] Tek ayar yerine dayanıklı bir koşul aralığı bul.
8. [x] Görülmemiş 30 seed × 100.000 tick, en az 27 birlikte süreklilik.
9. [x] Geçici kaynak kaybından toparlanmayı ayrı müdahale deneylerinde ölç.
10. [x] Öğrenme, mutasyon ve ikisini birlikte eş koşullarda doğrula.
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

Uygulama ve sınanacak kapılar: reports/COEXISTENCE_IMPLEMENTATION.md.
5–11 için ilk kod/deney altyapısı vardır; biyolojik kabul kapıları açık tutulur.
Görülmemiş doğrulama seedleri10001–10030 olarak önceden ayrıldı. Arka plan
programı kod üretmez; tanımlı deneyler ve koşullu ayrı7882 dağıtımıyla sınırlıdır.

## 20 Ekim kabul sonucu

32ddfb2 biyolojisiyle sabit 20001–20030 kümesi fixed ve both modlarında ayrı
30/30×100000 geçti. Dört ±%10 duyarlılık varyantı4/4; normal8/8, shock8/8,
learning/mutation/both kısa kapıları ayrı8/8. Ham sonuçlar yeniden hesaplandı;
yalnız status alanına güvenilmedi. Ayrıntı reports/QUALIFIED_OBSERVATION.md.
11'in gerçek süreli1/7/30gün kısmı açık kalır; yeni denge taraması gerekmez.
