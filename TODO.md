# Sıradaki işler

Hedef ve karar gerekçeleri: [docs/VISION.md](docs/VISION.md).
Çalışma/işletim kuralları: [AGENTS.md](AGENTS.md).

- [x] Mevcut dünyayı farklı seed'lerle çalıştırıp ilk nesilde kopan yaşam döngüsünü ölç.
- [x] Ölüm nedenleri, üreme/saldırı retleri ve bedensel üreme engellerini kaydet.
- [ ] Temel tick/metabolizma hataları düzeltildi; hareket, diyet ve kaynak sözleşmelerini tamamla.
- [x] RNG'yi dünya başına ayır; iç içe dünyalar ve checkpoint devamını doğrula.
- [x] Kaynak yenilenmesi ve otçul çayırında torunların üremesi; 10.000 tickte 219–230. nesil.
- [ ] Çayırda yok olan seed 2'nin davranış darboğazını ve farklı başlangıç nüfuslarını incele.
- [ ] Avcı–av beslenmesi/kovalama/üreme döngüsünü ayrı senaryolarda doğrula.
- [ ] 3 tick/s için yaklaşık 3,8 GB/gün olan kayıt büyümesini kayıpsız azalt; sonra uzun sunucu koşusu başlat.
- [ ] Bireyin algı/karar/sonuç akışını incelemeyi kolaylaştır.
- [ ] Bireysel hafıza ve tek bir karar alanında öğrenme deneyi kur.
- [ ] Kalıtsal özellikler, bedeller, mutasyon ve soy takibini ayrı bileşenlerde kur.
- [ ] Uzun koşu, nüfus ölçekleme, bellek ve disk büyümesini ölç; hedefleri ölçümlerle güncelle.

## Sonraki değerlendirmeler

- Kaynak yerleştirmeyi hücre başına olasılık yerine chunk içinde rastgele N hücre
  yaklaşımıyla karşılaştır; gerekçesiz biçimde dağılımı değiştirme.
