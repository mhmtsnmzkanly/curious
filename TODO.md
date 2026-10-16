# Sıradaki işler

Hedef [VISION](docs/VISION.md), kullanıcı sırası ve dürüst durum
[ROADMAP](docs/ROADMAP.md), ölçümler [sonuç raporu](docs/reports/ROADMAP_RESULTS.md).

- [ ] Kalıcı avcı soyunu çöz: V8 soy/rezerv teşhisi tamamlandı; V9 koşullu
  araştırmanın `data/coexistence-research/state.json` ve aşama raporlarını incele.
  Uzun test ve duyarlılık geçmeden kalıcı denge deme. Kod kararı için son avcı
  ölümleri ve karar kuyrukları probe sonuçlarında vardır. Gizli takviye ekleme.
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
