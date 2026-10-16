# Sıradaki işler

Hedef [VISION](docs/VISION.md), kullanıcı sırası ve dürüst durum
[ROADMAP](docs/ROADMAP.md), ölçümler [sonuç raporu](docs/reports/ROADMAP_RESULTS.md).

- [ ] Kaynak şokundan toparlanmayı çöz: V9 sabit kurallı uzun doğrulama 30/30
  geçti; kaynak şoku 4/8 ile araştırmayı durdurdu. Teşhis ve sonraki sınırlı
  deney [RESOURCE_SHOCK_ANALYSIS](docs/reports/RESOURCE_SHOCK_ANALYSIS.md).
  202 beslenme hedefi / 238 üreme eşiği uyumsuzluğunu ve daralmada yerel eş
  bulmayı ayrı ölç. Otçul toparlanması avcı soyunun toparlanması anlamına gelmez.
  Öğrenme/mutasyon kapıları çalışmadı, yeni dünya açılmadı. Gizli takviye ekleme.
- [ ] Su öğrenmesinin ortam değişimi deneyini değerlendir (`learning_trial` reversal);
  V10 su denemeleri gecikmiş sonucu ve gerçek hareket maliyetini izliyor.
  [Öğrenme sözleşmesine](docs/LEARNING_NEXT.md) göre sıradaki iş sınırlı av
  tercihi; yeni öğreniciyi ayrıca karma ekosistemde doğrula.
- [ ] Gerçek 1/7/30 gün gözlemini değerlendirme; raporlayıcı otomatik Notify gönderir.
- [ ] Kalıtım deneyini av baskısında tekrarlama; dağılım değişimini sürüklenmeden ayırma.
- [ ] Soy ağacı ve ölmüş ataların arayüzden doğrudan seçilmesini geliştirme.
- [ ] Daha büyük nüfusta SQLite/HTTP maliyetini ölçme; motor-only hızını kayıt hızı sanma.
- [ ] Disk dolmadan uzun vadeli arşiv kapasitesi planı; geçmişi sessizce silme.
- [ ] Host dışından gözetim: aynı sunucudaki bütün izleyiciler birlikte durursa içeride
  çalışan bir timer bunu dışarı bildiremez.

Tamamlanan ilk uygulamalar: kaynak yenilenmesi/nesiller, deterministik RNG,
SQLite+arşiv+checkpoint, kontrol/gözlem, hareket sözleşmeleri, bireysel hafıza,
sınırlı öğrenme, bedelli kalıtım/mutasyon, responsive gözlem ekranı, nüfus ölçümü,
izleyici gözetimi, sürümlü ikili/yedekli laboratuvar dağıtımı. Test/commit/temizlik/
önemli Notify döngüsü devam eder.
