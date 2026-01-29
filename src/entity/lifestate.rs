/// ===============================
/// YAŞAM DURUMU
/// ===============================
///
/// Bu struct hem:
/// - genetik (sabit) bilgileri
/// - dinamik (tick ile değişen) bilgileri
/// birlikte tutar.
/// Ayrım yorumlar ve yardımcı fonksiyonlarla yapılır.
#[derive(Debug, Clone)]
pub struct LifeState {
    // -------- GENETİK (SABİT) --------
    /// Maksimum yaş (tick cinsinden)
    pub max_age: usize,

    /// Üreme için minimum yaş
    pub maturity_age: usize,

    /// Maksimum can
    pub max_health: usize,

    /// Maksimum enerji
    pub max_energy: usize,

    /// Enerji düşük kabul edilen eşik
    pub low_energy_threshold: usize,

    // -------- DİNAMİK (DEĞİŞEN) --------
    /// Şu ana kadar geçen tick sayısı
    pub age: usize,

    /// Anlık can
    pub health: usize,

    /// Anlık enerji
    pub energy: usize,

    /// Son çiftleşmeden sonra kalan bekleme süresi
    pub reproduction_cooldown: usize,

    /// Doğal hız (stat)
    pub speed: usize,

    /// Tick boyunca biriken hareket puanı
    pub points: usize,
}

impl LifeState {
    /// Her tick çağrılır
    pub fn tick(&mut self) {
        self.age += 1;

        // Pasif enerji kaybı
        self.energy = self.energy.saturating_sub(1);

        // Üreme bekleme süresi azalır
        if self.reproduction_cooldown > 0 {
            self.reproduction_cooldown -= 1;
        }

        // Yaşlılıktan ölüm
        if self.age >= self.max_age {
            self.health = 0;
        }

        self.points += self.speed;
    }

    // -------- DURUM SORGULARI --------

    /// Canlı yaşıyor mu?
    pub fn is_alive(&self) -> bool {
        self.health > 0
    } // ===============================
    /// YAŞAM DURUMU
    /// ===============================
    //

    /// Üreme olgunluğuna erişti mi?
    pub fn is_mature(&self) -> bool {
        self.age >= self.maturity_age
    }

    /// Enerji kritik seviyede mi?
    pub fn is_energy_low(&self) -> bool {
        self.energy <= self.low_energy_threshold
    }

    /// Enerji tam mı?
    pub fn is_energy_full(&self) -> bool {
        self.energy >= self.max_energy
    }

    /// Çiftleşmeye uygun mu?
    pub fn can_reproduce(&self) -> bool {
        self.is_alive()
            && self.is_mature()
            && self.reproduction_cooldown == 0
            && !self.is_energy_low()
    }

    // -------- DURUM DEĞİŞTİRİCİLER --------

    /// Enerji harcama
    pub fn consume_energy(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount);
    }

    /// Enerji kazanma
    pub fn restore_energy(&mut self, amount: usize) {
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    /// Can iyileştirme
    pub fn heal(&mut self, amount: usize) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Çiftleşme sonrası çağrılır
    pub fn on_reproduce(&mut self) {
        self.reproduction_cooldown = 100;
        self.consume_energy(10);
    }

    /// Yeterli puan var mı?
    pub fn can_move(&self, cost: usize) -> bool {
        self.points >= cost
    }

    /// Hareket puanı harca
    pub fn spend(&mut self, cost: usize) {
        self.points = self.points.saturating_sub(cost);
    }
}
