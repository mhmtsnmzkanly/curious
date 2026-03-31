//! # Curious - Authoritative Simulation Core
//!
//! Bu kütüphane, çok oyunculu (multiplayer) veya karmaşık yapay zeka (AI) senaryolarına uygun,
//! deterministik ve modüler bir simülasyon çekirdeği sunar.
//!
//! ## Mimari Prensipler
//! 1. **Authoritative Logic**: Tüm kurallar (hareket maliyeti, beslenme enerjisi vb.) sunucu tarafındaki `systems` ve `rules` altında merkezi olarak yönetilir.
//! 2. **Event-Driven**: Simülasyonun sonuçları `SimulationEvent` nesneleri üzerinden dışarıya aktarılır.
//! 3. **Modular Systems**: Karar verme, planlama ve uygulama aşamaları birbirinden ayrılmıştır.
//!
//! ## Ana Modüller
//! * `world`: Veri yapıları ve dünya durumunu barındırır.
//! * `systems`: Simülasyonun lojik kurallarını (hareket, metabolizma, etkileşim vb.) içerir.
//! * `simulation`: Dış dünya ile iletişim kuran motor (Engine) katmanı.
//! * `rules`: Merkezi biyolojik sabitler ve kanunlar.

pub mod creatures;
pub mod entity;
pub mod logger;
pub mod map;
pub mod world;
pub mod simulation;
pub mod systems;
pub mod rules;

use std::sync::atomic::{AtomicU64, Ordering};

/// Simülasyonda ki chunk büyüklüğü
pub const CHUNK_SIZE: usize = 16;

/// Rastgele sayı üretmek için tohum
static RNG_STATE: AtomicU64 = AtomicU64::new(12345);

/// Tohumu günceller
pub fn set_global_seed(seed: u64) {
    RNG_STATE.store(seed, Ordering::Relaxed);
}

/// Tohumu zaman damgası ile günceller
pub fn set_global_seed_with_time() {
    set_global_seed(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    );
}

/// Bir sonraki rastgele sayıyı atomik olarak üretir
pub fn next_rand() -> u64 {
    // fetch_update: Mevcut değeri güvenli bir şekilde okur,
    // hesaplamayı yapar ve kimse araya girmeden yeni değeri yazar.
    RNG_STATE
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |state| {
            Some(state.wrapping_mul(6364136223846793005).wrapping_add(1))
        })
        .unwrap_or(0)
}

/// [min, max] aralığında sayı üretir
pub fn gen_range(min: isize, max: isize) -> isize {
    let range = (max - min).abs() as u64;
    if range == 0 {
        return min;
    }
    let rand_val = next_rand() % (range + 1);
    min + rand_val as isize
}

pub fn print_with_color(val: usize) {
    // ANSI TrueColor formatı: \x1b[38;2;R;G;Bm
    // \x1b[0m kodu ise rengi sıfırlamak içindir
    print!(
        "\x1b[38;2;{};{};{}m@ \x1b[0m",
        (val & 0xFF) as u8,
        ((val >> 8) & 0xFF) as u8,
        ((val >> 16) & 0xFF) as u8
    );
}
