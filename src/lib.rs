// Modülü dahil et
pub mod creatures;
pub mod entity;
pub mod map;
pub mod world;

pub fn generate_random_id() -> usize {
    // Geçici bir değişken oluşturup onun bellek adresini alıyoruz
    let variable = 0;
    let address = &variable as *const i32 as usize;

    // Adresi, işlemcinin zaman damgasıyla (TSC) harmanlayarak
    // rastgeleliği artırıyoruz
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as usize;

    // XOR ve bit kaydırma (bit-mixing) ile benzersiz bir sayı üretiyoruz
    let mut x = address ^ timestamp;
    x = x.wrapping_mul(0x517cc1b727220a95);
    x ^= x >> 31;

    x
}
