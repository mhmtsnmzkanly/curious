// Modülü dahil et
pub mod creatures;
pub mod entity;
pub mod map;
pub mod world;

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
