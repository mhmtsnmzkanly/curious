/// Canlının kimin tarafından yönetildiğini belirler
pub enum EntityController {
    /// Otonom AI tarafından yönetiliyor
    AI,
    /// Bir oyuncu tarafından yönetiliyor (ID bazlı)
    Player(u32),
}
