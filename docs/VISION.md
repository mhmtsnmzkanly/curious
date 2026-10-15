# Curious: amaç ve tasarım yönü

Bu notlar 15 Ekim 2026 tarihli kullanıcı görüşmesinden alınmıştır. Uygulamanın
mevcut durumuyla gelecekteki hedefler birbirine karıştırılmamalıdır.

## İstenen deneyim

Basitleştirilmiş bir dünyada canlıların hayatını belgesel izler gibi gözlemlemek.
Dünya gerçek Dünya'nın birebir kopyası olmayacak; canlıların da insan düzeyinde
zekâsı gerekmiyor. Buna karşılık çevrenin, ihtiyaçların ve geçmiş deneyimlerin
kararlar üzerinde anlaşılır etkisi olmalı.

Kendi kendine süren basit ekosistem bir ara hedeftir; projenin durma noktası değildir.
Sadece nüfusun sabit kalması veya canlıların rastgele dolaşması başarı sayılmaz.

Örnek hedef davranışlar:

- Açlık ve tehlikenin birlikte değerlendirilmesi; ihtiyaç arttıkça risk tercihlerinin değişmesi.
- Yorgun avcının başarısız kovalamacadan vazgeçmesi.
- Bireyin daha önce gördüğü kaynağı araması; kaynak kuruyunca bilgisini güncellemesi.
- Farklı deneyimler yaşayan aynı türden bireylerin farklı kararlar vermesi.

Bunlar tamamlanmış özellikler değil, geliştirilecek ve gözlemle doğrulanacak davranışlardır.

## Öğrenme ve evrim

Canlıların tecrübe kazanması isteniyor. Yaşa otomatik zekâ/güç bonusu vermek bu
hedefi karşılamaz. Yaşanan olaylar bireyin sonraki kararlarında iz bırakmalı.

Pekiştirmeli öğrenme, eylem sonuçlarından tercih güncellemek için bir adaydır;
her canlının ayrı büyük bir sinir ağı olması gerekmiyor. Temel davranışlar ile
bireysel öğrenmenin birleşimi ilk yaklaşım olabilir. Kesin yöntem küçük deneylerle
seçilecek; ödül puanını yükseltmek inandırıcı davranışın kanıtı sayılmayacak.

Öğrenme bireyin yaşamında, evrim nesiller arasında gerçekleşir. Beden/kalıtsal
özellikler ile öğrenilmiş hafıza ayrı tutulmalı. Kalıtım, mutasyon ve üreme başarısı
özellik dağılımını değiştirebilir. Hız gibi avantajların enerji gibi bedelleri
olmalı. Öğrenilmiş her bilginin otomatik kalıtılması varsayılmamalı.

Kullanıcı dünyayı haftalarca/aylarca sunucuda çalıştırabilmek, uzun vadede yıllara
uzanan bir yaşam ve evrim süreci gözlemlemek istiyor. Teknik süreklilik tasarlanabilir;
evrimin sürekli karmaşıklık veya yenilik üretmesi garanti edilemez.

## Mimari sözleşmeler

- Dünya gerçek durumu ve fiziksel/ekolojik kuralları tutar.
- Algı, canlının erişebildiği bilgidir; dünya bilgisiyle aynı şey değildir.
- Hafıza bireyin geçmişten bildiğidir; eski veya eksik olabilir.
- Karar bileşeni algı, beden ve hafızadan niyet üretir.
- Motor eylemi doğrular, çakışmaları çözer, maliyeti uygular ve sonucunu bildirir.
- Öğrenme sonuçları değerlendirir; ödül tasarımı dünya kurallarına gömülmez.
- Kalıtım doğumlar sırasında işler; bireysel öğrenmeden ayrı geliştirilir.
- Gözlem araçları simülasyon çekirdeğinden bağımsızdır.

Katkı yapan biri dünyayı değiştirebilmeli, biri yeni canlı ekleyebilmeli, biri
mevcut canlıya bir yetenek kazandırabilmeli. Her ihtimale yönelik dev bir eklenti
sistemi kurmak yerine bu genişletmelerle ayrımların işe yaradığı kanıtlanmalı.

## Geliştirme sırası ve kabul ölçütleri

1. Mevcut davranışı farklı seed'lerle ölç: yaşam nerede kesiliyor, üreme neden olmuyor?
2. Deterministik dünya ve kurallı kararları sağlamlaştır: aynı başlangıç, seed ve
   komutlar aynı sonuçları versin. Rastlantı kullanılabilir; kontrol altında olmalı.
3. Kaynak döngüsü ve nesillerin devamı: dışarıdan canlı eklemeden yavrular da üreyebilsin.
4. Birey bazında gözlem: algı, karar, maliyet, sonuç ve geçmiş incelenebilsin.
5. Hafıza ve sınırlı öğrenme: deneyim öncesi/sonrası davranış farkını göster.
6. Kalıtım, bedelli özellikler ve evrim: soy ve özellik dağılımlarını ölç.

MVP, küçük bir dünyada ihtiyaçlara göre davranan, nesilleri devam eden ve en az
bir karar alanında deneyimden öğrenen bireylerin izlenebilir örneğidir. Çok tür,
sosyal sürüler, karmaşık iklim ve gelişmiş görsellik ilk teslimin zorunluluğu değil.

## 15 Ekim 2026 itibarıyla gerçek durum

- Rust motorunda harita, üç tür, algı/niyet/plan/çözümleme, metabolizma ve yaşam döngüsü var.
- SQLite kaydı, dosya boyutuna göre shard geçişi, checkpointten devam ve HTTP kontrolü eklendi.
- Kararlar, olaylar, tam dünya durumu, metrikler ve kabul edilen müdahaleler kaydediliyor.
- Dört Rust testi ve HTTP/kapanma-kurtarma testi geçti; systemd restartında dünya
  durumunun korunması kontrol edildi. Ayrıntılar `CONTROL.md` içinde.
- Kaynak yenilenmesi, sürekli nesiller, öğrenme ve evrim henüz tamamlanmadı.
- Aylar süren dayanıklılık ve büyük nüfus performansı henüz doğrulanmadı.
- Sıradaki iş ilk nesilde yaşamın kesilmesine ilişkin ölçümlü başlangıç raporu.
