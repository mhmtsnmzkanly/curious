# Curious Simulation

`Curious` bir canlı-harita simülasyonu çerçevesidir. Bu proje, **basit bir ekosistem simülasyonu** yaratmayı hedefler ve Rust dilinde geliştirilmiştir. Canlılar, enerji ve sağlık gibi yaşam durumlarına sahiptir; harita üzerinde hareket eder, yiyecek arar, saldırır veya çiftleşir.

---

## Temel Kavramlar

### World, Map ve Entity

* **Map**: Statik çevresel yapı. Hücrelerin içeriğini tutar (Empty, Food, Water). Map **sadece okunur ve sınırlı şekilde değiştirilir**. Entity çakışmaları Map içinde çözülmez.
* **World**: Hakem rolündedir. Tüm entity'leri ve onların pozisyonlarını tutar. İki aşamalı tick mekanizması burada çalışır: önce entity kendi durumunu günceller (`tick()`), sonra aksiyonlar toplanır ve uygulanır (`apply()` ve resolve_conflict).
* **Entity**: Canlıları temsil eder. Her entity’nin yaşam durumu (`LifeState`), faz durumu (`EntityPhase`), algısı (`Perception`) ve aksiyonları (`Action`) vardır. Entity kendi iç durumunu yönetir, dış dünya ile etkileşimler World tarafından kontrol edilir.

### WorldView

* Entity’lerin dünyayı **okuma amaçlı** kullanabileceği bir görünüm sağlar
* Entity’ler:

  * Haritayı bilmez
  * Diğer entity pozisyonlarını ve faz durumlarını doğrudan değiştiremez
* Sağladığı bilgiler:

  * Hücre durumu (`Cell`)
  * Pozisyondaki entity’ler ve sayısı
  * Canlı mı, ceset mi
  * Yakın çevredeki entity’ler ve yiyecekler

### Perception

* Entity’lerin WorldView’den aldığı verileri işleyerek anlamlı bir algı oluşturur
* Algı öğeleri:

  * Foods: Yiyecekler ve cesetler
  * Enemies: Tehdit olarak algılanan canlılar
  * Mates: Çiftleşme için uygun canlılar
* Algılama menzili, Manhattan mesafesi ile sınırlıdır

---

### Action

Entity’ler aşağıdaki aksiyonları seçebilir:

* `Move(Direction)` – Pozisyon değiştirme (World tarafından uygulanır)
* `Eat` – Enerji kazanımı, hücredeki yiyecek World tarafından azaltılır
* `Attack { target_id }` – Saldırı
* `Flee(Direction)` – Kaçış
* `Idle` – Boş aksiyon

---
### World.tick Fazları
World::tick fonksiyonun ortaya koymasını istediğim davranışı:
`
[ START: DÜNYA TICK SİNYALİ ]
│
├─► FAZ 1: BİYOLOJİK SAAT VE FAZ GÜNCELLEME
│   │   "Zaman herkes için akıyor..."
│   ├─ [CORPSE / REMOVED]: İşlemi sonlandır (return).
│   ├─ [SLEEPING]: 'remaining' süresini -1 azalt. 
│   │   └─ Süre bittiyse: Phase = ACTIVE olur.
│   └─ [YAŞLANMA]: Aktif veya Uyuyan herkes için Yaş (Age) +1.
│
├─► FAZ 2: ALGI VE KARAR (Sadece Aktifler)
│   │   "Zihni açık olanlar karar verir."
│   ├─ Sadece Active canlılar "think" metodunu çalıştırır.
│   └─ Karar Matrisi: Kaç, Saldır, Ye, Çiftleş veya IDLE (Bilinçli Durma).
│
├─► FAZ 3: FİZİKSEL UYGULAMA VE ANLIK BEDEL
│   │   "Her eylemin bir maliyeti vardır."
│   ├─ [MOVE]: Konum değişir (E: -2).
│   ├─ [EAT]: Yemek tüketilir (Enerji artar).
│   ├─ [MATE/ATTACK]: Etkileşim gerçekleşir (E: -15 / -5).
│   │   └─ ÖNEMLİ: Çiftleşen ebeveynler otomatik SLEEPING moduna alınır.
│   └─ [IDLE]: Konum sabit kalır (E: -1).
│
├─► FAZ 4: METABOLİK DÖNÜŞÜM (Enerji-Can Terazisi)
│   │   "Vücut bütçeyi dengeliyor."
│   ├─ 1. AÇLIK: Enerji == 0 ise ──► [Can -1 | Enerji +2].
│   ├─ 2. İYİLEŞME (Rejenerasyon):
│   │   └─ ŞART: (Phase == Sleeping VEYA Karar == Idle) 
│   │           VE (Enerji Doluluk > %60) VE (Can < Max)
│   │      └─ [Enerji -2] ──► [Can +1] (Hücre onarımı).
│   └─ 3. BAZAL TÜKETİM:
│       └─ Uyuyanlar daha az yakıt harcar (E: -0.5).
│
└─► FAZ 5: HASAT VE DOĞUM
    ├─ Can <= 0 veya Age >= Max olanları haritadan temizle.
    └─ Yeni doğanları ebeveyn yanına ekle (Bebeklik uykusuyla başlat).
`
