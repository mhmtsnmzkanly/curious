---
description: Nasıl yeni bir kural veya sistem eklenir?
---

Bu döküman, `curious` authoritative simulation core üzerinde yeni bir lojik eklemek için izlenmesi gereken adımları içerir.

### 1. Yeni Bir Biyolojik Kural Eklemek
Eğer bir canlının hızı, enerji tüketimi gibi sabit bir kural değişecekse:
- `src/rules/biology.rs` dosyasını açın.
- Yeni sabiti ekleyin.
- İlgili sistemi (`MetabolismSystem` vb.) bu kuralı kullanacak şekilde güncelleyin.

### 2. Yeni Bir Sistem Eklemek
Eğer tamamen yeni bir mekanik (örneğin: Hava Durumu Sistemi) eklenecekse:
- `src/systems/weather.rs` gibi yeni bir dosya oluşturun.
- `src/systems/mod.rs` içinde bu modülü kaydedin.
- `src/world/mod.rs` içindeki `tick()` metoduna bu sistemi sırası geldiğinde çağrılacak şekilde ekleyin.

### 3. Yeni Bir Olay (Event) Eklemek
Ağ senkronizasyonu için yeni bir çıktı gerekiyorsa:
- `src/simulation/event.rs` içindeki `SimulationEvent` enum'una yeni varyantı ekleyin.
- İlgili sistemde (örneğin: `WeatherSystem`) bu olayı `events.push(...)` ile fırlatın.

### 4. Testleri Çalıştırmak
Sistemlerin doğruluğunu kontrol etmek için:
```bash
cargo test
```
komutunu kullanın.
