# Curious Simulation

`Curious` bir canlı-harita simülasyonu çerçevesidir. Bu proje, **basit bir ekosistem simülasyonu** yaratmayı hedefler ve Rust dilinde geliştirilmiştir. Canlılar, enerji ve sağlık gibi yaşam durumlarına sahiptir; harita üzerinde hareket eder, yiyecek arar, saldırır veya çiftleşir.

---

## Temel Kavramlar

### World, Map ve Entity

* **Map**: Statik çevresel yapı. Hücrelerin içeriğini tutar (Empty, Food, Water). Map **sadece okunur ve sınırlı şekilde değiştirilir**. Entity çakışmaları Map içinde çözülmez.
* **World**: Hakem rolündedir. Tüm entity'leri ve onların pozisyonlarını tutar. İki aşamalı tick mekanizması burada çalışır: önce entity kendi durumunu günceller (`tick()`), sonra aksiyonlar toplanır ve uygulanır (`apply()` ve resolve_conflict).
* **Entity**: Canlıları temsil eder. Her entity’nin yaşam durumu (`LifeState`), faz durumu (`EntityPhase`), algısı (`Perception`) ve aksiyonları (`Action`) vardır. Entity kendi iç durumunu yönetir, dış dünya ile etkileşimler World tarafından kontrol edilir.

---

### İki Fazlı Tick Mekanizması

1. **Internal Update (`tick`)**

   * Yaşlanma, enerji kaybı, üreme bekleme süresi
   * Moves_used sıfırlama
   * Faz kontrolü: Sleeping, Active, Corpse, Removed
   * Ölüm kontrolü: Sağlık ≤ 0 → Corpse

2. **Decision & Action (`think` + `apply`)**

   * Her entity, WorldView üzerinden çevresini algılar ve aksiyon seçer (`think`)
   * Seçilen aksiyon entity’nin iç durumuna uygulanır (`apply`)
   * Pozisyon değişiklikleri veya kaynak tüketimi gibi world’a bağlı kısımlar **World** tarafından yönetilir
   * Çakışmalar (aynı hücreye birden fazla entity girmesi) çözülür, örneğin:

     * Yemek yeme: Otçullar aynı kaynaktan yiyebilir
     * Saldırı: Etçiller savaşır veya terk eder

---

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

---

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

### Conflict Resolution

* Aynı hücreye birden fazla entity girebilir:

  * Çiftleşme, Saldırı gibi durumlarda entity kendi içinde karar verir
  * Kaynak çatışmalarında, örneğin az ot varsa otçullar kavga edebilir veya terk edebilir
* Bu mekanizma ileride `resolve_conflict` fonksiyonu ile World seviyesinde genişletilecek

---

### Dosya ve Modül Yapısı

```
curious/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── entity/
│   │   ├── mod.rs
│   │   ├── lifestate.rs
│   │   ├── phase.rs
│   │   ├── perception.rs
│   │   └── action.rs
│   ├── map/
│   │   ├── mod.rs
│   │   ├── cell.rs
│   │   ├── position.rs
│   │   └── direction.rs
│   └── world.rs
```

* `entity/`: Canlıların durumları, algıları, aksiyonları ve fazları
* `map/`: Hücreler, pozisyonlar ve yönler
* `world.rs`: Tüm entity’lerin yönetildiği, tick ve WorldView mekanizmasının bulunduğu dosya

---

### Örnek Kullanım

```rust
use curious::{entity::action::Action, map::{Map, cell::Cell, position::Position}, world::World};

fn main() {
    // 10x10 boş harita
    let map = Map { width: 10, height: 10, grid: vec![Cell::Empty; 100] };

    // Örnek entity listesi (kendi struct implementasyonu ile)
    let entities: Vec<Box<dyn Entity>> = Vec::new();

    let mut world = World::new(map, entities);

    // Tek tick çalıştır
    world.tick();
}
```

