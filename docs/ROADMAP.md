# Kullanıcının onayladığı sıra ve gerçek durum

Her aşamada deney, test, not, commit, derleme temizliği ve önemli sonuçlarda
Notify uygulanır. Bir özelliğin ilk sürümü, bütün projenin tamamlandığı anlamına
gelmez. Hızlandırılmış tick gerçek gün/hafta gözleminin yerine geçmez.

1. [ ] Etçil yavru ve avcı soyunun devamı: enerji sorunu düzeldi, yavru olgunlaşıp üreyebiliyor; kalıcı avcı soyu açık.
2. [x] Av seçimi, tahmini maliyet, bırakma ve dinlenme için ilk kurallı politika.
3. [ ] Karma ekosistem: küçük/geniş/yoğun/sığınaklı karşılaştırmalar yapıldı; kalıcı birlikte yaşam sağlanmadı.
4. [x] Hareket, duvar, sığınak, diyet, paylaşım ve çakışma sözleşmeleri.
5. [x] Sınırlı bireysel kaynak/tehlike hafızası ve unutma.
6. [x] Su seçimi için sonuçtan öğrenme ve öğrenmeyen bireyle kontrollü deney.
7. [x] İlk hareket özelliğinde bedelli kalıtım, mutasyon ve nesil dağılımı ölçümü.
8. [x] Birey, karar, geçmiş, hafıza ve soy bilgisi için ilk görsel gözlem ekranı.
9. [ ] Gerçek gün/hafta dayanıklılığı: 10.000 tam kayıtlı tick geçti; gerçek süreli gözlem sürüyor.
10. [x] 10–1.000 kurucu ve farklı haritalarda ölçüm; algı indeksiyle hızlandırma.
11. [x] Ayrı izleyici gözetimi, yedekli dağıtım ve gerçek süreli bildirimler.

## Süren işler

### 23 Ekim — paralel capability planı

Altı capability izi A–F ile bounded performans/storage izi G'nin kapsamı ve
kanıt kapıları [PARALLEL_CAPABILITY_PLAN.md](PARALLEL_CAPABILITY_PLAN.md) içinde.
Root A/B/C izole tasarım/prototiplerini inceledi; A water prototype `769f683` ile
ana dala alındı. Bunlar üretim öğrenme politikası veya ekoloji kabulü değildir.
Ajan 7 A testi ve 11 B targeted kontrolü bildirdi; root `cargo test --all-targets`
sonucunu 181 passed / 0 failed, Python'ı 132 passed ve release build'i exit 0
bildirdi. Scoped `rustfmt --edition 2024 --check` geçti. C örneği yalnız derlendi;
runtime fixture/muhasebe audit'i yok. Üç geçici target dizini temizlendi. Yeni
registered ecology batch'i veya seed koşusu yapılmadı.

Sonraki işler: A'nın default-off production integration/baseline snapshot/residual
persistence'i; B compatibility calibration/schema tasarımı; C fixture accounting
ve single-world runtime audit. Ortak source/checkpoint değişiklikleri seri root
review ile yapılır; tasarım ve bağımsız örnekler paralel ilerleyebilir. Yeni
formal çalışma için protocol ve seed audit gerekir; kullanıcı onayı beklenmiyor.
Aylar/yıllar reliability soak'u hariçtir; kısa sabit ölçek/throughput testleri
kapsamdadır. Bu henüz kabul edilmiş özellik veya biyolojik sonuç değildir.
- Asıl açık biyolojik konu: avcı nesillerinin kalıcı olması. Sığınaklı seed
  12345'te 20. nesle ulaştı, 907'den sonra avcı kalmadı. Başarı diye etiketlenmez.
  Sıradaki teşhis bu soyda çift bulma, rezerv ve ölüm zamanlarını ayırmak.
- Gerçek süreli 9. aşama otomatik örnekleniyor. 1/7/30 gerçek gün ve yeterli
  örnek birikince Notify raporu gönderilecek; henüz gün/hafta geçti denemez.
- Öğrenme tek karar alanında; evrim tek özellikte. Genel öğrenme, kalıtsal yeni
  yeteneklerin kendiliğinden oluşması ve açık uçlu karmaşıklık henüz yok.

Ayrıntılı sonuçlar: [toplu rapor](reports/ROADMAP_RESULTS.md), aşama raporları
`reports/ROADMAP_01.md`–`ROADMAP_11.md`, son habitat deneyi `reports/REFUGE.md`.

Av–avcı için onaylanan yeni11 aşamalı çalışma `COEXISTENCE_PLAN.md` içindedir.
V8 teşhisi ve V9 düzeltmeleri uygulandı. Kalibrasyon/duyarlılık/uzun testler ve
koşullu ayrı dünya için çalıştırıcı hazırlandı; biyolojik kabul kapıları henüz
geçilmiş değildir. Ayrıntı `reports/COEXISTENCE_IMPLEMENTATION.md`.

## 20 Ekim 2026 — karma kabul

32ddfb2 rezerv/buluşma koşulu, sabitlenen yeni30seedde fixed ve both için
ayrı30/30×100000 adım ve son bölüm üreyen yavru soyu kapısını geçti.
Duyarlılık ve şok/öğrenme/mutasyon kapıları da tamamlandı. Bu, 1 ve3 için
sınanmış koşulda güçlü süreklilik kanıtıdır; sonsuz denge veya genel evrim
iddiası değildir. Gerçek gün/hafta9 hâlâ açık. Ayrı dünya ve işletim kanıtları
[QUALIFIED_OBSERVATION](reports/QUALIFIED_OBSERVATION.md) raporunda tutulur.

## 23 Ekim 2026 — ekoloji takvimi ve bedelli fizik adayları

İsteğe bağlı coğrafi kaynak takvimi ile kalıtsal görüş/dayanıklılık allelleri
uygulama ve mekanizma test aşamasında. Yeni fizik maliyeti enerjiyle ödenir;
varsayılan opt-in kapalıdır. Sabit karşılaştırma tasarımı
[ECOLOGY_TRAITS_PROTOCOL](ECOLOGY_TRAITS_PROTOCOL.md) içinde önkayıtlıdır;
henüz calibration veya held-out fitness değerlendirmesi çalıştırılmadı.
Kayıtlı çalışma temiz kaynak commiti ve incelemesinden sonra yürütülür.
