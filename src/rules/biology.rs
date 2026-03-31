//! Biyolojik kurallar ve sabitler.

/// Bir öğünde yenilebilecek maksimum enerji miktarı
pub const EAT_ENERGY_AMOUNT: usize = 5;

/// Bir içimde içilebilecek maksimum su miktarı
pub const DRINK_WATER_AMOUNT: usize = 5;

/// Saldırı sırasında saldırganın tükettiği enerji
pub const ATTACK_ENERGY_COST: usize = 3;

/// Saldırı sırasında hedefe verilen hasar
pub const ATTACK_DAMAGE_AMOUNT: usize = 6;

/// Cesetlerin haritada kalma süresi (tick)
pub const CORPSE_DURATION_TICKS: usize = 5;

/// Bir cesedin sağladığı gıda miktarının çarpanı (max_health / x)
pub const CORPSE_FOOD_DIVISOR: usize = 4;

/// Minimum gıda miktarı (cesetlerden çıkan)
pub const MIN_FOOD_FROM_CORPSE: usize = 5;

// --- Metabolizma Kuralları ---

/// Her tick birimi başına yaşlanma miktarı
pub const AGING_RATE: usize = 1;

/// Her tick birimi başına bazal enerji tüketimi
pub const BMR_ENERGY_COST: usize = 1;

/// Her tick birimi başına bazal su tüketimi
pub const BMR_WATER_COST: usize = 1;

/// Enerji doluysa yenilenecek can miktarı
pub const PASSIVE_HEAL_AMOUNT: usize = 1;

/// Pasif iyileşme için gereken enerji maliyeti
pub const PASSIVE_HEAL_ENERGY_COST: usize = 2;

/// Enerji bittiğinde candan harcanan miktar (açlıktan ölme başlangıcı)
pub const STARVATION_HEALTH_LOSS: usize = 3;

/// Enerjiyi yenilemek için candan harcanan karşılığında kazanılan miktar
pub const STARVATION_ENERGY_RESTORE: usize = 9;

/// Su bittiğinde candan harcanan miktar (susuzluktan ölme)
pub const DEHYDRATION_HEALTH_LOSS: usize = 2;
