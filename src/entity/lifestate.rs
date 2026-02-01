//use crate::entity::{intent::Intent, phase::EntityPhase};

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

    /// Maksimum can
    pub max_health: usize,

    /// Maksimum enerji
    pub max_energy: usize,

    /// Üreme için minimum yaş
    pub maturity_age: usize,

    /// Canlının görüş açısı
    pub vision_range: usize, // Örn: 6

    // -------- DİNAMİK (DEĞİŞEN) --------
    /// Şu ana kadar geçen tick sayısı
    pub age: usize,

    /// Anlık can
    pub health: usize,

    /// Anlık enerji
    pub energy: usize,

    /// Son çiftleşmeden sonra kalan bekleme süresi
    pub reproduction_cooldown: usize,

    /// Tick başına maksimum hareket hakkı
    pub speed: usize,

    /// Bu tick içinde kullanılan hareket sayısı
    pub moves_used: usize,
}

impl LifeState {
    /// Her tick başında çağrılır.
    /// Hareket hakkı resetlenir.
    pub fn tick(&mut self) {
        // Yaşlanma
        self.age += 1;

        // Yaşlılıktan ölüm
        if self.age > self.max_age {
            self.health = 0;
            // Kendine not: Yaşlılıktan ölmek yerine her turda 5 can alacak şekilde değiştirilebilir.
            return; // Yaşlandığı için ekstra bir hesaplamaya gerek yok
        }

        // Üreme bekleme süresi
        if self.reproduction_cooldown > 0 {
            self.reproduction_cooldown -= 1;
        }

        // Pasif iyileşme süreci
        // 2 enerji'ye 1 can düşer; değerler değişebilir şimdilik bu
        if !self.is_energy_low() && self.health < self.max_health {
            self.consume_energy(2);
            self.heal(1);
        }

        // Can karşılığında Enerji kazanma
        // Enerji 0 ise, Can yakarak Enerji kazanma
        if self.energy == 0 && !self.is_health_low() {
            self.health -= 1;
            self.restore_energy(2);
        }

        // Bu tick için hareket sayacı sıfırlanır
        self.moves_used = 0;
    }

    // ===============================
    // DURUM SORGULARI
    // ===============================
    /// Enerji düşük kabul edilen eşik
    pub fn low_energy_threshold(&self) -> usize {
        self.max_energy / 4
    }
    /// Can düşük kabul edilen eşik
    pub fn low_health_threshold(&self) -> usize {
        self.max_health / 4
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn is_mature(&self) -> bool {
        self.age >= self.maturity_age
    }

    pub fn is_energy_low(&self) -> bool {
        self.energy <= self.low_energy_threshold()
    }

    pub fn is_energy_full(&self) -> bool {
        self.energy >= self.max_energy
    }

    pub fn is_health_low(&self) -> bool {
        self.health <= self.low_health_threshold()
    }

    pub fn is_health_full(&self) -> bool {
        self.health >= self.max_health
    }

    // LifeState içinde
    pub fn can_reproduce(&self) -> bool {
        (self.age >= self.maturity_age) && (self.reproduction_cooldown == 0 && self.energy > 15)
        // Çok düşük tut ki ölmeden hemen önce bile deneyebilsinler
    }

    /// Bu tick içinde hareket edebilir mi?
    pub fn can_move(&self) -> bool {
        self.moves_used < self.speed
    }

    // ===============================
    // DURUM DEĞİŞTİRİCİLER
    // ===============================

    /// Bir hareket kullanıldığında çağrılır
    pub fn on_move(&mut self) {
        self.moves_used += 1;
        self.consume_energy(1);
    }

    pub fn consume_energy(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount);
    }

    pub fn restore_energy(&mut self, amount: usize) {
        // Enerjiyi artır ama maksimum kapasiteyi aşma
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    pub fn heal(&mut self, amount: usize) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn on_reproduce(&mut self) {
        println!("Entity is reproducing...");
        self.reproduction_cooldown = 100;
        self.consume_energy(10);
    }

    /*
    pub fn metabolic_cost(&self, phase: &EntityPhase, intent: Option<&Intent>) -> usize {
        // 1. Bazal Metabolizma Hızı (BMR): Sadece hayatta kalmak için gereken min. enerji
        let bmr = 1;

        match phase {
            // Ölüler enerji harcamaz
            EntityPhase::Corpse { .. } | EntityPhase::Removed => 0,

            // Uyku Modu: En düşük maliyet. Görüş kapalı, hareket yok.
            EntityPhase::Sleeping { .. } => bmr,

            // Aktif Mod: Canlı uyanık ve çevresini işliyor.
            EntityPhase::Active => {
                let mut cost = bmr;

                // Algı Maliyeti: Geniş bir alanı taramak (vision_range) beyin/göz yorar.
                cost += self.vision_range / 5; // Örn: Her 5 birim görüş +1 maliyet

                // Niyet (Aksiyon) Maliyeti:
                if let Some(action) = intent {
                    match *action {
                        Intent::Move { steps } | Intent::Flee { target_id: steps } => {
                            // Hareket maliyeti: Hız ve atılan adım sayısı ile orantılı
                            cost += self.speed + (steps.len() / 2);
                        }
                        Intent::Mate { .. } => {
                            cost += 5; // Üreme çok yüksek enerji gerektirir
                        }
                        Intent::Eat { .. } => {
                            cost += 1; // Sindirim ve çiğneme eforu
                        }
                        Intent::Idle { .. } => {
                            // Idle (Bekleme): Ekstra maliyet yok, sadece BMR + Algı.
                        }
                    }
                }
                cost
            }
        }
    }
    */
}
