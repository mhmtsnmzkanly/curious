# Üreme rezervi: yerel fırsatı değerlendirme

## Davranış düzeltmesi

Sağlıklı yetişkinin normal beslenme hedefi ile üreme enerji eşiği arasındaki
aralıkta bir çiftleşme fırsatı kaçabiliyordu. Yeni karar, ancak aşağıdakilerin
hepsi varsa üreme enerji eşiğine kadar beslenmeyi önceliklendirir:

- Birey yetişkin, üreme bekleme süresi bitmiş ve sağlık koşulu uygun.
- Bitişikte görülen aynı türden bir birey üremeye hazır.
- Görüşte diyetine uygun, pozitif net değerli mevcut besin var.

Bu ek amaç normal açlık eşiğinin üzerinde yeni bir av başlatmaz. Canlının
zaten var olan açlığı, susuzluğu ve tehdit öncelikleri korunur. Enerji eşiği
planlamada ve fiziksel üreme kontrolünde aynı LifeState metodundan alınır.
Gizli kaynak veya canlı takviyesi yoktur; beden profili değiştirilmez.

Bu, düşük yoğunlukta eş aramayı tamamen çözmez. İki birey de henüz hazır değilse
veya birbirlerini göremiyorsa yeni rezerv fırsatı tetiklenmez. Bir sonraki
çalışmada eş aramayı da aynı anda değiştirip etkileri karıştırmamak için bu sınır
bilerek korunur. 'Kaynak şoku çözüldü' iddiası henüz yoktur.

## Neden daha dar bir değişiklik?

İlk iki uygulama kısa karşılaştırmayla elendi; canlı servise dağıtılmadı:

1. Bütün uygun yetişkinlerde beslenme hedefini yükseltmek: seed 1'de otçullar
   1223. adımda, seed 3'te avcılar 1364. adımda tükendi.
2. Rezervi yalnız görünür mevcut besinle tamamlama, eş fırsatı aramadan:
   seed 1 2000 adımı geçti; seed 3'te otçullar 1603. adımda tükendi.

Bunlar geliştirme deneyleridir. Girdiler, probe ikilileri ve kaynak farkları
`data/experiments/reserve-goal-screen-v1`, `-v2`, `-v3` dizinlerinde korunur.
Bu sonuçlar beslenme eşiğini her bireyde yükseltmenin neden kabul edilmediğini
gösterir. Son değişikliğin amacı somut bir yerel fırsatı değerlendirmektir.

## Kısa testten sonra yapılacak karşılaştırma

Son aday kısa eleme deneyini geçerse iki bağımsız, sınırlı çalışma başlatılır:

- Aynı sekiz eğitim seedi × 20.000 adım, fixed, şoksuz kontrol.
- Aynı sekiz eğitim seedi × 20.000 adım, fixed; 10.000'de kaynak yenilenmesi
  %50 azalır ve 11.000'de geri gelir.

Her çalışma tek süreç kullanır; toplam iki CPU süreci. Deneyler sonlu, resumable
ve Notify sonuçlu olur; canlı dağıtım yapmaz. Koşu özeti ve tamamlama bilgisi
`data/experiments/reserve-opportunity/{normal,shock}` altında okunur.
Sürüm, değişmez probe ve senaryo hashleri çalışma manifestlerinde saklanır.
Servisler `curious-reserve-normal.service` ve `curious-reserve-shock.service`.
`study.json` çalışma kaynaklarını gösterir. Bu konumlar başlatılana kadar
çalışan deney anlamına gelmez.

Başarı ölçütü yalnız son canlı sayısı değildir: iki türün devamı ve son %10'da
her iki türden yavruların da üremesi aranır. Başlangıç hipotezi, şok sonucunun
eski 4/8'e göre iyileşmesi ve normal kontrolün bozulmamasıdır. 8 eğitim seedinde
başarı son kabul değildir; mevcut 30 seed görülmüş olduğundan sonraki uzun
kabul için yeni seedler ayrılmalıdır.

Probe örneklerine birikimli doğum/ölüm nedenleri, avcı yaş bantları, hazır birey
ve enerji nedeniyle üremeye kapalı birey sayısı eklendi. Böylece daralma dönemi
boyunca yaş ve üreme boşluğu önceki son-10-ölüm kayıtlarından daha iyi ayrılabilir.
Tanı bilgisi canlı kararına verilmez. Öğrenme kapalıdır; V10 su öğrenmesinin
etkisi bu deneye dahil edilmez. Eski V9 araştırmasının kaynakları ve sonuçları
korunur; bunlar yeni davranışın doğrulaması diye kullanılmaz.

Son adayın kısa elemesi: seed 1 ve 3, fixed, 2000 adım; 2/2 geçti.
Seed 1 sonunda 51 avcı/555 otçul, seed 3 sonunda 52 avcı/367 otçul var.
Bu sonuç kaynak şoku başarısı değildir. 56 Rust ve 21 Python testi, release
derleme ve HTTP/auth/yazıcı kilidi/SIGKILL kurtarma kontrolü geçti.
