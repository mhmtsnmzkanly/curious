# Kaynak güvenilirliği: öğrenme hedefi sözleşmesi

**Tarih:** 26 Ekim 2026
**Durum:** Önerildi; uygulanmadı ve ampirik olarak kabul edilmedi.
**Ana alan:** K3 bireysel öğrenme; K1 ResourceSchedule ve K2 beden/çözücü
bağımlılıkları.

## Neden ayrı bir hedef gerekiyor?

V19 water-context artığı, su seçimi anında zaten görünen miktar, bedenin su
açığı, hareket maliyeti ve rota süresini kullanan
[`foraging::value`](../src/entity/foraging.rs) skoruna ekleniyor. 25 Ekim V2
formal kayıtlarında 147 kabul edilmiş bireysel bağlam gözleminin 146'sı sıfır;
tek `+67` düzeltme, ölçülen taze su seçimi sıralamasını değiştirmedi. Ayrıntı:
[Water signal diagnosis](reports/WATER_SIGNAL_DIAGNOSIS_20261026.md).

Bu sonuç, V2 fixture'ında çözücü sonucunun mevcut görünen miktar ve rota
hesabıyla büyük ölçüde uyuştuğunu gösteriyor. Daha iyi bir hedef, aynı görünür
bilgiyi yeniden öğrenmek yerine **kaynağa varılana kadar gerçekleşen miktar
değişimini** tahmin etmelidir. Bu öngörü yalnızca bireyin önceki kendi
ziyaretlerinden öğrenilir. Böylece bugün görünen miktar temel skorun girdisi
olarak kalır; öğrenici, bu miktarın tahmini varış zamanında ne kadar değiştiğine
ilişkin sınırlı bir düzeltme üretir.

## Hedef ve gözlem sözleşmesi

Bir karar anında `A0`, algının seçilebilir su kaynağı için verdiği mevcut miktar
olsun. O bireyin mevcut bedeni ve su açığıyla görünen tahmini alınacak miktar:

```text
forecast = min(A0, max_water - current_water, DRINK_WATER_AMOUNT)
```

Seçilen kaynak için doğrulanmış resolver ziyaretinde gerçekten alınan su
`actual` olsun. Öğrenme etiketi `delta = actual - forecast` olur; birimi su
miktarıdır. Bu etiket **saf kaynak arzı veya kaynağın nedensel güvenilirliği
ölçümü değildir**. Varış anındaki beden su açığı, aradaki fizyolojik değişim,
rota ve kaynak arzı birlikte sonucu belirleyebilir. Örneğin varışta artmış beden
açığı pozitif `delta` üretebilir; bu tek başına kaynağın yenilendiğini kanıtlamaz.
Gelecekteki analiz `delta`yı bireyin **görünen hizmet tahmini hatası** olarak
adlandırmalı; bireysel servis etkileşimi dışına kaynak arızası/yenilenme
nedenselliği çıkarmamalıdır. Negatif değer görünür tahminden az hizmeti, pozitif değer ise
varıştan önce yenilenme gibi nedenlerle daha çok hizmeti belirtir. Etiket
`[-10, +10]` aralığına kırpılır. İşaret yalnız forecast-relative farkı gösterir;
tek başına kaynakta eksilme, yenilenme veya arıza etiketi değildir. Bu, tam
rotanın yaşam faydası veya güvenlik olasılığı da değildir; yalnızca başlatılmış
bireysel ziyaretin kısa ufuklu miktar hatasıdır.

Etiket yalnızca aşağıdaki kanıtla yazılır:

- Resolver aynı bireyin başlattığı aynı hedefe başarılı `Drink` çözümlemişse,
  `actual` çözücüde kaydedilen kendi su kazanımıdır.
- Yol tamamlama, hedef konum ve eylem kimliği doğrulanmışken `Drink` eylemi
  yalnız `MissingResource` nedeniyle reddedilmişse `actual = 0` kaydedilebilir.
- Ölüm, başka hedefe geçiş, dış komut, kesinti, başka eylem/red nedeni, hedef
  veya aktör atıf uyuşmazlığı, taşma ve eksik kayıt sansürlüdür. Sansürlü
  deneme `delta = 0` ya da başarısızlık etiketi olmaz.
- Diğer canlıların gözlemi, dünya kaynak tablosu, `ResourceSchedule` alanları,
  sonraki RNG veya gelecekteki miktar etikete ya da özelliğe girmez. Bireyin
  `tick` değeri, o anki algı, bedeni ve kendi önceki tamamlanmış ziyaretleri
  kullanılabilir.

Bu hedef yalnız gözlenen seçilmiş kaynak için etiket üretir. Seçilmemiş
alternatifin karşıolgusal miktarını veya uzun dönemli rota güvenliğini tahmin
ettiğini iddia etmez.

## Önerilen küçük, saf aday

Aday, mevcut `foraging::value` temel skorunu tüm kollarda aynen korur. Öğrenici
yalnız algıdaki mevcut su adaylarına ek bir sıralama düzeltmesi önerir; hareket,
resolver maliyeti, kaynak tüketimi ve yaşam sonucuna karar vermez.

Önerilen tablo bireyin kendi belleğinde en çok **32 bağlam satırı** tutar; her
satır son **4 doğrulanmış `delta` örneğini** ve en son kendi gözlem tick'ini
sayar. Anahtar, yalnız karar anında bilinen şu girdilerden oluşur:

- algılanan kaynak konumu;
- mevcut görünür miktarın önceden sabitlenmiş kaba aralığı;
- mevcut `foraging::value` ile uyumlu tahmini varış süresi aralığı;
- karar bağlamıyla açıkça verilen bireyin karar-zamanı tick'inden türetilen
  tahmini varış fazı.

Bir *öneri* olan sabit faz özelliklendirmesi
`((decision_tick + estimated_travel_ticks) mod 32) div 4` biçimindedir. 32 tick
ve 4-tick kovası henüz doğrulanmamış genel önbilgidir; gerçek kaynak periyodunu
aramaz veya `ResourceSchedule` içinden okumaz. Adayın karar arayüzü bu tick'i
bireyin kendi karar-zamanı girdisi olarak açıkça sağlamalıdır; öğrenici saat
veya dünya durumunu ayrıca sorgulamaz. Bu saat varsayımı uygun görülmezse
alternatif anahtar, bireyin aynı kaynağı en son kendi gözlemlediği andan beri
geçen süredir. Bu alternatif periyodik fazı temsil etmeyebilir ve protokolde
ayrı raporlanmalıdır. Faz eşleşmeyen veya desteklenmeyen bağlamda düzeltme
sıfırdır. Tablo 128 tickten eski satırı kullanmaz. Yeni anahtar eklenirken 32
satır doluysa en eski kendi gözlemi deterministik bağ kırımıyla çıkarılır.
Zaten bozuk ya da 32 satırdan büyük tablo ve aralık dışı/taşan tekil güncelleme
atomik reddedilir. Herhangi bir bağlamın kabul eşiği olarak en az iki kendi
örneği gerekir. Ortalama hata, `[-10, +10]` su biriminde tutulur.

Önerilen skor düzeltmesi mevcut su skoruyla aynı birime çevrilir:

```text
estimated_ticks = max(1, estimated_travel_ticks)
correction = clamp(trunc_toward_zero(200 * predicted_delta / estimated_ticks), -1000, 1000)
candidate_score = foraging::value(current_visible_amount, route, body, water=true)
                  + correction
```

Payda daima en az 1'dir; öğreniciye geçersiz/boş rota veya bilinmeyen
varış-süresi verilirse düzeltme üretilmez. Tamsayı bölmesi sıfıra doğru yuvarlar.

Öneri yalnız mevcut geçerli su adaylarının sıralamasında kullanılır. Eşit skor
bağı ve mevcut keşif davranışı değişmez. Henüz iki örneği olmayan bağlamda
skor mevcut baseline ile aynıdır.

Özellik deneysel bayrakla **varsayılan kapalı** olur. Kapalıyken karar, RNG,
State ve bellek davranışı mevcut kaynakla aynı kalmalıdır. Frozen kol aday
seçimini ve RNG keşfini sürdürebilir, fakat bu tablonun satır, değer ve ziyaret
sayıları değişmez. Her yeni birey tablosu boş başlar; ebeveynden veya popülasyon
ortalamasından öğrenilmiş satır kopyalanmaz. Kapasite dolu iken yeni satırın
deterministik eviction'ı normal bounded davranıştır; bozuk bellek veya geçersiz
güncelleme atomik reddedilir ve sessiz kısmi yazım olmaz.

## Küçük gelecek protokol taslağı

Bu, çalıştırma izni veya tamamlanmış protokol değildir. Yeni kod ya da World
çalışması başlamadan önce ayrı protokol ve kaynak özdeşliği gözden geçirilir.

1. **Fixture:** Var olan `ResourceSite` ve `ResourceSchedule`
   ([`environment.rs`](../src/environment.rs)) kullanılabilir. İki aynı tür su
   kaynağı aynı başlangıç miktarı ve eşit algılanabilir rota uzaklığında kurulur.
   `capacity`, `replenish` ve `interval` sabit kalır. Schedule dönemi önceden
   ilan edilmiş 13, 16 veya 19 tick değerlerinden biri; `active_ticks` dönemin
   yarısı, `geographic_phase_step = 0`, `phase_offset` ise iki site için yarım
   dönem aralıklıdır. Kaynak değişimi yalnız mevcut `ResourceSchedule`
   uygulamasının doğal replenishment kuralıyla olur. Canlıya veya başka
   servislere dokunulmaz.
2. **Bilgi sınırı:** Adaya yalnız aynı `DecisionView`, beden, `tick`, kendi
   WaterEpisode/öğrenilmiş tablosu ve kimliği doğrulanmış resolver sonucu verilir.
   Schedule yapılandırması yalnız fixture kurucusundadır; politika veya
   öğreniciye aktarılmaz. Baseline su miktarı/rota hesabı hiçbir kolda
   düşürülmez veya gizlenmez.
3. **Zaman ve eşleme:** Her yeni tohumda aynı birey, başlangıç bedeni, harita,
   algı, komutlar ve RNG fork'u kullanılır. 256 ticklik online-own-experience
   prefix'i sonrası 256 tick evaluation suffix'i; beden veya kaynak sıfırlaması
   yapılmaz. Aynı fork'tan trained-online, aynı tabloyla frozen, boş tablolı
   naive-online ve değişmemiş rule-based kolları yürütülür. Tohumlar, program,
   fixture, protokol hash'i ve durdurma koşulu koşudan önce sabitlenir.
4. **Karşılaştırma ve ölçüm:** Birincil mekanizma ölçümü, yalnız doğrulanmış
   seçilmiş ziyaretlerde `delta` tahmininin held-out MAE'sidir; sıfır düzeltme
   baseline'ı ve frozen/naive kollarla eşleştirilir. Her kol için destek sayısı,
   kaynak, görünür miktar kovası, tahmini varış süresi, faz kovası ve sansür
   nedeni raporlanır. Seçim sıralamasının gerçekten değiştiği taze karar sayısı
   ayrıca verilir. Su kazanımı, hareket enerjisi, resolver tick'i, hasar,
   kesinti, ölüm ve terminal beden değerleri ayrı fiziksel birimlerle raporlanır.
   Tahmin hatasının düşmesi tek başına tercih faydası veya ekolojik üstünlük
   sayılmaz.
5. **Geçerlilik ve durma:** En az 8 yeni, önceden listelenmiş seed; her fork'ta
   kollar arası başlangıç State/RNG/beden/algı eşitliği, frozen tablo
   değişmezliği, kapalı yol State+RNG denkliği, checkpoint devamı, kaynak
   hash'i ve etiket/çözücü mutabakatı gerekir. Önceden kaydedilmiş en az destek koşulunu karşılamayan
   seed sonuçtan sonra değiştirilmez; destek yetersizliği olarak raporlanır.
   Kaynak/hash, resolver atfı veya eş başlangıç ihlali çalışmayı geçersiz kılar.
   Eşik/kovalar sonuç görüldükten sonra ayarlanmaz; yeni tuning yeni protokol
   ve görülmemiş evaluation seed'i gerektirir.

`ResourceSchedule` yalnız deterministik zaman-pencere sınaması için fixture
aracıdır. Öğreniciye schedule oracle'ı verilmesi veya aynı miktar/rota
baseline'ının zayıflatılması bu hedefin parçası değildir. Üstünlük bulunmasa
bile bu deney yalnız belirtilen iki kaynaklı kısa ufukta sonuç verir; karma
ekosisteme, doğal hayvan davranışına veya aylar süren dünya faydasına
genellenmez.

## Kaynak dayanağı ve açık karar

- [`src/entity/foraging.rs`](../src/entity/foraging.rs): mevcut görünür miktar,
  su açığı, hareket enerjisi ve rota süresi fiziksel baseline'ı.
- [`src/entity/learning.rs`](../src/entity/learning.rs): mevcut
  `choice_baseline`, doğrulanmış fiziksel residual, 4 örnekli context kaydı ve
  48 tick TTL uygulaması; bu belge aynı algoritmanın yeni kabulü değildir.
- [`src/systems/water_outcomes.rs`](../src/systems/water_outcomes.rs): resolver
  kapanışı, kendi hedefi doğrulanan başarısızlık kontrolü ve context güncelleme
  sınırı.
- [`src/environment.rs`](../src/environment.rs): `ResourceSchedule` period,
  active window, geographic phase ve phase offset uygulaması; schedule yalnız
  kaynak yenileme fixture'ını tanımlar.
- [Water signal diagnosis](reports/WATER_SIGNAL_DIAGNOSIS_20261026.md): altı
  formal seedde sıfıra yakın mevcut residual sinyalinin yeniden analizi.

Karar: Bu sözleşme incelemeye hazır, **öneri** statüsündedir. Bir adayın
uygulanması, yeni seedlerle dondurulmuş protokol çalışması veya davranış
faydası bu belgeyle kabul edilmiş sayılmaz.
