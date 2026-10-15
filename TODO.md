# Sıradaki işler

Hedef ve karar gerekçeleri: [docs/VISION.md](docs/VISION.md).
Çalışma/işletim kuralları: [AGENTS.md](AGENTS.md).

- [x] Mevcut dünyayı farklı seed'lerle çalıştırıp ilk nesilde kopan yaşam döngüsünü ölç.
- [x] Ölüm nedenleri, üreme/saldırı retleri ve bedensel üreme engellerini kaydet.
- [ ] Temel tick/metabolizma hataları düzeltildi; hareket, diyet ve kaynak sözleşmelerini tamamla.
- [x] RNG'yi dünya başına ayır; iç içe dünyalar ve checkpoint devamını doğrula.
- [x] Kaynak yenilenmesi ve otçul çayırında torunların üremesi; 10.000 tickte 219–230. nesil.
- [x] Seed 2 üreme fırsatı darboğazını teşhis et ve düzelt; 8 seed × 2.000 tick doğrulandı.
- [ ] Farklı kurucu nüfusları ve kaynak kıtlıklarını karşılaştır.
- [x] Avlama, beslenme, dayanıklılık, sıkışınca savunma ve etçil doğumunu ayrı deneylerde doğrula.
- [ ] Etçil yavrunun besin/suya erişimini ve avcı soyunun devamını çöz; karma denge henüz yok.
- [x] Kayıpsız arşivle ölçülen boyutu 5,55 kat azalt; ayrı çayır gözlemi ve otomatik hata bildirimlerini başlat.
- [ ] Gerçek süreli çayır koşusunu günler/haftalar boyunca değerlendir; izleyicinin kendi sağlığına da gözetim ekle.
- [x] Bireyin algı/karar/sonuç akışını API ve kalıcı karar izleriyle incele.
- [ ] Bireysel hafıza ve tek bir karar alanında öğrenme deneyi kur.
- [ ] Kalıtsal özellikler, bedeller, mutasyon ve soy takibini ayrı bileşenlerde kur.
- [ ] Uzun koşu, nüfus ölçekleme, bellek ve disk büyümesini ölç; hedefleri ölçümlerle güncelle.

## Sonraki değerlendirmeler

- Kaynak yerleştirmeyi hücre başına olasılık yerine chunk içinde rastgele N hücre
  yaklaşımıyla karşılaştır; gerekçesiz biçimde dağılımı değiştirme.

Son aşama: [karar ve avcı–av raporu](docs/reports/DECISION_AND_PREDATION.md).
