## src/creatures/carnivore.rs
```
use crate::{
    entity::{
        Entity,
        instinct::{Instinct, InstinctEvaluator},
        intent::Intent,
        lifestate::LifeState,
        perception::Perception,
        species::Species,
    },
    map::movement::{DIRECTION_ARRAY, Steps},
};

pub struct CarnivoreEntity {
    pub life_state: LifeState,
}

impl CarnivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: LifeState {
                max_age: 120,
                max_health: 140,
                max_energy: 90,
                max_water: 70,
                maturity_age: 25,
                vision_range: 7,
                age: 0,
                health: 140,
                energy: 90,
                water: 70,
                reproduction_cooldown: 0,
                speed: 4,
                moves_used: 0,
            },
        }
    }
}

impl Entity for CarnivoreEntity {
    fn life(&self) -> &LifeState {
        &self.life_state
    }

    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life_state
    }

    fn species(&self) -> Species {
        Species::Carnivore
    }

    fn make_intent(&self, perception: Perception) -> Intent {
        let decision =
            InstinctEvaluator::evaluate(&self.life_state, &perception, Species::Carnivore);

        let best_prey = perception
            .entities
            .iter()
            .filter(|e| e.species != Species::Carnivore)
            .min_by_key(|e| (e.steps.len(), e.power));

        let best_water = perception
            .waters
            .iter()
            .min_by_key(|w| (w.steps.len(), usize::MAX - w.amount));

        match decision.instinct {
            Instinct::Threat => {
                if let Some(threat) = decision.threat {
                    if threat.can_win {
                        return Intent::Attack {
                            target_id: threat.target_id,
                        };
                    }
                    return Intent::Flee {
                        target_id: threat.target_id,
                    };
                }
                Intent::Idle { duration: 1 }
            }
            Instinct::Survival | Instinct::Hunger => {
                if let Some(prey) = best_prey {
                    if prey.steps.len() <= 1 {
                        return Intent::Attack { target_id: prey.id };
                    }
                    return Intent::Move {
                        steps: prey.steps.clone(),
                    };
                }

                let mut steps = Steps::empty();
                for _ in 0..self.life_state.speed {
                    steps
                        .0
                        .push(DIRECTION_ARRAY[crate::gen_range(0, 7isize) as usize])
                }
                Intent::Move { steps }
            }
            Instinct::Thirst => {
                if let Some(water) = best_water {
                    if !self.life_state.is_water_full() {
                        return Intent::Drink {
                            at: water.steps.clone(),
                        };
                    }
                }
                Intent::Idle { duration: 1 }
            }
            Instinct::Mating => {
                if let Some(target) = perception
                    .entities
                    .iter()
                    .find(|e| e.species == Species::Carnivore)
                {
                    if target.steps.len() <= 1 {
                        Intent::Mate {
                            target_id: target.id,
                        }
                    } else {
                        // Çiftleşmek için hedefe yaklaş (tek adım)
                        let mut one_step = Steps::empty();
                        if let Some(first) = target.steps.0.first() {
                            one_step.0.push(*first);
                        }
                        Intent::Move { steps: one_step }
                    }
                } else {
                    Intent::Idle { duration: 1 }
                }
            }
            Instinct::Idle => Intent::Idle { duration: 1 },
        }
    }

    fn tick(&mut self) {
        self.life_state.tick();
    }

    fn reproduce(&self) -> Box<dyn Entity> {
        let mut child_life = self.life_state.clone();
        child_life.age = 0;
        child_life.energy = child_life.max_energy / 2;
        child_life.water = child_life.max_water / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(CarnivoreEntity::new(child_life))
    }
}

```

## src/creatures/herbivore.rs
```
use crate::{
    entity::{
        Entity,
        instinct::{Instinct, InstinctEvaluator},
        intent::Intent,
        lifestate::LifeState,
        perception::Perception,
        species::Species,
    },
    map::movement::{DIRECTION_ARRAY, Steps},
};

pub struct HerbivoreEntity {
    pub life_state: LifeState,
}

impl HerbivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: LifeState {
                max_age: 105,
                max_health: 120,
                max_energy: 80,
                max_water: 60,
                maturity_age: 20,
                vision_range: 6,
                age: 0,
                health: 120,
                energy: 80,
                water: 60,
                reproduction_cooldown: 0,
                speed: 3,
                moves_used: 0,
            },
        }
    }
}

impl Entity for HerbivoreEntity {
    fn life(&self) -> &LifeState {
        &self.life_state
    }

    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life_state
    }

    fn species(&self) -> Species {
        Species::Herbivore
    }

    fn make_intent(&self, perception: Perception) -> Intent {
        let decision =
            InstinctEvaluator::evaluate(&self.life_state, &perception, Species::Herbivore);

        let best_food = perception
            .foods
            .iter()
            .min_by_key(|f| (f.steps.len(), usize::MAX - f.amount));
        let best_water = perception
            .waters
            .iter()
            .min_by_key(|w| (w.steps.len(), usize::MAX - w.amount));

        match decision.instinct {
            Instinct::Threat => {
                if let Some(threat) = decision.threat {
                    if threat.can_win {
                        return Intent::Attack {
                            target_id: threat.target_id,
                        };
                    }
                }
                let mut steps = Steps::empty();
                for _ in 0..self.life_state.speed {
                    steps
                        .0
                        .push(DIRECTION_ARRAY[crate::gen_range(0, 7isize) as usize])
                }
                Intent::Move { steps }
            }
            Instinct::Survival | Instinct::Hunger => {
                if let Some(food) = best_food {
                    if !self.life_state.is_energy_full() {
                        return Intent::Eat {
                            at: food.steps.clone(),
                            corpse_id: None,
                        };
                    }
                }

                let mut steps = Steps::empty();
                for _ in 0..self.life_state.speed {
                    steps
                        .0
                        .push(DIRECTION_ARRAY[crate::gen_range(0, 7isize) as usize])
                }
                Intent::Move { steps }
            }
            Instinct::Thirst => {
                if let Some(water) = best_water {
                    if !self.life_state.is_water_full() {
                        return Intent::Drink {
                            at: water.steps.clone(),
                        };
                    }
                }
                Intent::Idle { duration: 1 }
            }
            Instinct::Mating => {
                if let Some(target) = perception
                    .entities
                    .iter()
                    .find(|e| e.species == Species::Herbivore)
                {
                    if target.steps.len() <= 1 {
                        Intent::Mate {
                            target_id: target.id,
                        }
                    } else {
                        // Çiftleşmek için hedefe yaklaş (tek adım)
                        let mut one_step = Steps::empty();
                        if let Some(first) = target.steps.0.first() {
                            one_step.0.push(*first);
                        }
                        Intent::Move { steps: one_step }
                    }
                } else {
                    Intent::Idle { duration: 1 }
                }
            }
            Instinct::Idle => Intent::Idle { duration: 1 },
        }
    }

    fn tick(&mut self) {
        self.life_state.tick();
        //self.life_state.metabolic_cost();
    }

    fn reproduce(&self) -> Box<dyn Entity> {
        let mut child_life = self.life_state.clone();
        child_life.age = 0;
        child_life.energy = child_life.max_energy / 2;
        child_life.water = child_life.max_water / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(HerbivoreEntity::new(child_life))
    }
}

```

## src/creatures/mod.rs
```
pub mod herbivore;
pub mod carnivore;
pub mod omnivore;

```

## src/creatures/omnivore.rs
```
use crate::{
    entity::{
        Entity,
        instinct::{Instinct, InstinctEvaluator},
        intent::Intent,
        lifestate::LifeState,
        perception::Perception,
        species::Species,
    },
    map::movement::{DIRECTION_ARRAY, Steps},
};

pub struct OmnivoreEntity {
    pub life_state: LifeState,
}

impl OmnivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: LifeState {
                max_age: 110,
                max_health: 130,
                max_energy: 85,
                max_water: 65,
                maturity_age: 22,
                vision_range: 6,
                age: 0,
                health: 130,
                energy: 85,
                water: 65,
                reproduction_cooldown: 0,
                speed: 3,
                moves_used: 0,
            },
        }
    }
}

impl Entity for OmnivoreEntity {
    fn life(&self) -> &LifeState {
        &self.life_state
    }

    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life_state
    }

    fn species(&self) -> Species {
        Species::Omnivore
    }

    fn make_intent(&self, perception: Perception) -> Intent {
        let decision =
            InstinctEvaluator::evaluate(&self.life_state, &perception, Species::Omnivore);

        let best_food = perception
            .foods
            .iter()
            .min_by_key(|f| (f.steps.len(), usize::MAX - f.amount));
        let best_water = perception
            .waters
            .iter()
            .min_by_key(|w| (w.steps.len(), usize::MAX - w.amount));
        let best_prey = perception
            .entities
            .iter()
            .filter(|e| e.species != Species::Omnivore)
            .min_by_key(|e| (e.steps.len(), e.power));

        match decision.instinct {
            Instinct::Threat => {
                if let Some(threat) = decision.threat {
                    if threat.can_win {
                        return Intent::Attack {
                            target_id: threat.target_id,
                        };
                    }
                    return Intent::Flee {
                        target_id: threat.target_id,
                    };
                }
                Intent::Idle { duration: 1 }
            }
            Instinct::Survival | Instinct::Hunger => {
                if let Some(food) = best_food {
                    if !self.life_state.is_energy_full() {
                        return Intent::Eat {
                            at: food.steps.clone(),
                            corpse_id: None,
                        };
                    }
                }

                if let Some(prey) = best_prey {
                    if prey.steps.len() <= 1 {
                        return Intent::Attack { target_id: prey.id };
                    }
                    return Intent::Move {
                        steps: prey.steps.clone(),
                    };
                }

                let mut steps = Steps::empty();
                for _ in 0..self.life_state.speed {
                    steps
                        .0
                        .push(DIRECTION_ARRAY[crate::gen_range(0, 7isize) as usize])
                }
                Intent::Move { steps }
            }
            Instinct::Thirst => {
                if let Some(water) = best_water {
                    if !self.life_state.is_water_full() {
                        return Intent::Drink {
                            at: water.steps.clone(),
                        };
                    }
                }
                Intent::Idle { duration: 1 }
            }
            Instinct::Mating => {
                if let Some(target) = perception
                    .entities
                    .iter()
                    .find(|e| e.species == Species::Omnivore)
                {
                    if target.steps.len() <= 1 {
                        Intent::Mate {
                            target_id: target.id,
                        }
                    } else {
                        // Çiftleşmek için hedefe yaklaş (tek adım)
                        let mut one_step = Steps::empty();
                        if let Some(first) = target.steps.0.first() {
                            one_step.0.push(*first);
                        }
                        Intent::Move { steps: one_step }
                    }
                } else {
                    Intent::Idle { duration: 1 }
                }
            }
            Instinct::Idle => Intent::Idle { duration: 1 },
        }
    }

    fn tick(&mut self) {
        self.life_state.tick();
    }

    fn reproduce(&self) -> Box<dyn Entity> {
        let mut child_life = self.life_state.clone();
        child_life.age = 0;
        child_life.energy = child_life.max_energy / 2;
        child_life.water = child_life.max_water / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(OmnivoreEntity::new(child_life))
    }
}

```

## src/entity/instinct.rs
```
use crate::entity::{lifestate::LifeState, perception::Perception, species::Species};

/// İçgüdü seviyeleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instinct {
    /// Hayatta kalma (kritik sağlık/enerji).
    Survival,
    /// Tehlike algılandı.
    Threat,
    /// Açlık (enerji düşük).
    Hunger,
    /// Susuzluk (su düşük).
    Thirst,
    /// Çiftleşme (üreme mümkün).
    Mating,
    /// Özel bir dürtü yok.
    Idle,
}

/// İçgüdü değerlendirme aracı.
#[derive(Debug, Clone, Copy)]
pub struct InstinctEvaluator;

#[derive(Debug, Clone, Copy)]
pub struct ThreatAssessment {
    pub target_id: usize,
    pub can_win: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct InstinctDecision {
    pub instinct: Instinct,
    pub threat: Option<ThreatAssessment>,
}

impl InstinctEvaluator {
    /// Basit içgüdü sıralaması uygular.
    pub fn evaluate(
        life: &LifeState,
        perception: &Perception,
        own_species: Species,
    ) -> InstinctDecision {
        // Tehdit algısı için mesafe eşiği (adım sayısı)
        const THREAT_RANGE: usize = 2;

        let own_power = life.health + life.energy;
        let threat = perception
            .entities
            .iter()
            .find(|entity| entity.species != own_species && entity.steps.len() <= THREAT_RANGE)
            .map(|entity| ThreatAssessment {
                target_id: entity.id,
                can_win: own_power >= entity.power,
            });

        if threat.is_some() {
            return InstinctDecision {
                instinct: Instinct::Threat,
                threat,
            };
        }

        if life.is_health_low() || life.energy == 0 {
            return InstinctDecision {
                instinct: Instinct::Survival,
                threat: None,
            };
        }
        if life.is_energy_low() {
            return InstinctDecision {
                instinct: Instinct::Hunger,
                threat: None,
            };
        }
        if life.is_water_low() {
            return InstinctDecision {
                instinct: Instinct::Thirst,
                threat: None,
            };
        }
        if life.can_reproduce() && !perception.entities.is_empty() {
            return InstinctDecision {
                instinct: Instinct::Mating,
                threat: None,
            };
        }
        InstinctDecision {
            instinct: Instinct::Idle,
            threat: None,
        }
    }
}

```

## src/entity/intent.rs
```
use crate::map::movement::Steps;

/// Canlının görüş açısıyla yola çıkarak ortaya koyduğu niyet
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// Gidilmek istenilen nokta
    Move { steps: Steps },
    /// Yenilmek istenilen yemeğin konumu,
    /// Not: Yemek aynı hücrede ise at okunmaz,
    /// miktar canlının yiyebiliceği ve World izin verdiği miktarda olur
    Eat { at: Steps, corpse_id: Option<usize> },
    /// İçilmek istenilen suyun konumu
    Drink { at: Steps },
    /// Çiftleşmek istenilen canlı
    Mate { target_id: usize },
    /// Saldırılmak istenilen canlı
    Attack { target_id: usize },
    /// Kaçınılmak istenilen canlı
    Flee { target_id: usize },
    /// Bekleme niyeti, iyileşme için (yavaş)
    Idle { duration: usize },
    /// Keyfi olarak uyuma eylemi, iyileşme için (hızlı)
    Sleep { duration: usize },
}

```

## src/entity/lifestate.rs
```
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

    /// Maksimum su
    pub max_water: usize,

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

    /// Anlık su
    pub water: usize,

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
            self.health -= 3;
            self.restore_energy(9);
        }

        // Su yoksa can düşsün
        if self.water == 0 && !self.is_health_low() {
            self.health = self.health.saturating_sub(2);
        }

        self.consume_energy(1);
        self.consume_water(1);

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
    /// Su düşük kabul edilen eşik
    pub fn low_water_threshold(&self) -> usize {
        self.max_water / 4
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

    pub fn is_water_low(&self) -> bool {
        self.water <= self.low_water_threshold()
    }

    pub fn is_water_full(&self) -> bool {
        self.water >= self.max_water
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

    pub fn can_move_for(&self, need: usize) -> bool {
        self.enough_energy(need) && self.enough_moves(need)
    }

    /// Yeterli enerji var mı?
    pub fn enough_energy(&self, need: usize) -> bool {
        self.energy >= need
    }

    /// Yeterli hareket hakkı var mı?
    pub fn enough_moves(&self, need: usize) -> bool {
        self.moves_used.saturating_add(need) <= self.speed
    }
    // ===============================
    // DURUM DEĞİŞTİRİCİLER
    // ===============================

    /// Bir hareket kullanıldığında çağrılır
    pub fn on_move(&mut self, step: usize) {
        self.moves_used += step;
        self.consume_energy(step);
    }

    pub fn consume_energy(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount);
    }

    pub fn restore_energy(&mut self, amount: usize) {
        // Enerjiyi artır ama maksimum kapasiteyi aşma
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    pub fn consume_water(&mut self, amount: usize) {
        self.water = self.water.saturating_sub(amount);
    }

    pub fn restore_water(&mut self, amount: usize) {
        self.water = (self.water + amount).min(self.max_water);
    }

    pub fn heal(&mut self, amount: usize) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Hasar al
    pub fn take_damage(&mut self, amount: usize) {
        self.health = self.health.saturating_sub(amount);
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
    }*/
}

```

## src/entity/mod.rs
```
pub mod intent;
pub mod instinct;
pub mod lifestate;
pub mod perception;
pub mod phase;
pub mod species;

use crate::entity::{intent::Intent, lifestate::LifeState, perception::*, species::Species};

/// Canlının temel alacağı arayüz
pub trait Entity {
    /// Canlının yaşam durumu (genetik + dinamik)
    fn life(&self) -> &LifeState;
    fn life_mut(&mut self) -> &mut LifeState;

    /// Varlık türü
    fn species(&self) -> Species;

    /// Karar verme (sadece okuma yapmalı)
    fn make_intent(&self, view: Perception) -> Intent;

    /// Tek tick güncellemesi
    /// World'un işini kolaylaştırmak için var;
    fn tick(&mut self);

    /// Canlının kendi türünden yeni bir üye (yavru) oluşturmasını sağlar.
    /// World bu metodu çağırır ama dönen somut türü (Herbivore vs.) bilmez.
    fn reproduce(&self) -> Box<dyn Entity>;
}

```

## src/entity/perception.rs
```
use crate::{
    entity::species::Species,
    map::movement::{Direction, Steps},
};
use std::{
    collections::HashMap,
    ops::{Add, AddAssign},
};

/// Algılanan tekil hedef
#[derive(Debug, Clone)]
pub struct PerceivedEntity {
    /// Algılanan canlının kimliği (Kaldırılabilir, Emin değilim)
    pub id: usize,
    /// Algılanan canlının türü
    pub species: Species,
    /// Algılanan canlının güç tahmini
    pub power: usize,
    /// Algılanan canlının yön ve mesafe bilgisi
    pub steps: Steps,
}

/// Algılanan yiyecek
#[derive(Debug, Clone)]
pub struct PerceivedFood {
    /// Algılanan yemeğin miktarı
    pub amount: usize,
    /// Algılanan yemek ceset mi?
    pub is_corpse: bool,
    /// Algılanan yemeğin yön ve mesafe bilgisi
    pub steps: Steps,
}

/// Algılanan su
#[derive(Debug, Clone)]
pub struct PerceivedWater {
    /// Algılanan su miktarı
    pub amount: usize,
    /// Algılanan suyun yön ve mesafe bilgisi
    pub steps: Steps,
}

/// Canlının görüş açısında olan Yemekler, Diğer canlılar, Gidebiliceği Mesafe
/// - Bu pozisyonda canlı var mı ve kaç tane var?
/// - Canlı mı / ceset mi?
/// - Yakın çevrede kimler var?
#[derive(Debug, Clone)]
pub struct Perception {
    /// Algılanan yemekler
    pub foods: Vec<PerceivedFood>,
    /// Algılanan sular
    pub waters: Vec<PerceivedWater>,
    /// Algılanan canlılar
    pub entities: Vec<PerceivedEntity>,
    /// Gidilebilicek mesafe, u8 değeri hangi yöne kaç adımı gidebiliceği simgeler
    pub directions: HashMap<Direction, u8>,
}

impl Perception {
    /// Boş bir görüş açısı oluştur
    pub fn empty() -> Self {
        Self {
            foods: Vec::new(),
            waters: Vec::new(),
            entities: Vec::new(),
            directions: HashMap::new(),
        }
    }
    /// Algılanan yiyeceğe adım ekle
    pub fn add_food(&mut self, amount: usize, is_corpse: bool, steps: Steps) {
        self.foods.push(PerceivedFood {
            amount,
            is_corpse,
            steps,
        });
    }

    /// Algılanan suya adım ekle
    pub fn add_water(&mut self, amount: usize, steps: Steps) {
        self.waters.push(PerceivedWater { amount, steps });
    }

    /// Algılanan canlıya adım ekle
    pub fn add_entity(&mut self, id: usize, species: Species, power: usize, steps: Steps) {
        self.entities.push(PerceivedEntity {
            id,
            species,
            power,
            steps,
        });
    }

    /// Bir yöne adım ekle veya mevcut adımı güncelle
    pub fn add_direction(&mut self, dir: Direction, distance: u8) {
        self.directions
            .entry(dir)
            .and_modify(|d| *d = (*d).max(distance))
            .or_insert(distance);
    }
}

impl Add<Direction> for PerceivedEntity {
    type Output = Self;
    fn add(mut self, dir: Direction) -> Self {
        self.steps += dir;
        self
    }
}

impl Add<Steps> for PerceivedEntity {
    type Output = Self;
    fn add(mut self, steps: Steps) -> Self {
        self.steps += steps;
        self
    }
}

impl AddAssign<Direction> for PerceivedEntity {
    fn add_assign(&mut self, dir: Direction) {
        self.steps += dir;
    }
}

impl AddAssign<Steps> for PerceivedEntity {
    fn add_assign(&mut self, steps: Steps) {
        self.steps += steps;
    }
}

impl Add<Direction> for PerceivedFood {
    type Output = Self;
    fn add(mut self, dir: Direction) -> Self {
        self.steps += dir;
        self
    }
}

impl Add<Steps> for PerceivedFood {
    type Output = Self;
    fn add(mut self, steps: Steps) -> Self {
        self.steps += steps;
        self
    }
}

impl AddAssign<Direction> for PerceivedFood {
    fn add_assign(&mut self, dir: Direction) {
        self.steps += dir;
    }
}

impl AddAssign<Steps> for PerceivedFood {
    fn add_assign(&mut self, steps: Steps) {
        self.steps += steps;
    }
}

impl Add<Direction> for PerceivedWater {
    type Output = Self;
    fn add(mut self, dir: Direction) -> Self {
        self.steps += dir;
        self
    }
}

impl Add<Steps> for PerceivedWater {
    type Output = Self;
    fn add(mut self, steps: Steps) -> Self {
        self.steps += steps;
        self
    }
}

impl AddAssign<Direction> for PerceivedWater {
    fn add_assign(&mut self, dir: Direction) {
        self.steps += dir;
    }
}

impl AddAssign<Steps> for PerceivedWater {
    fn add_assign(&mut self, steps: Steps) {
        self.steps += steps;
    }
}

impl Add<PerceivedEntity> for Perception {
    type Output = Self;

    fn add(mut self, entity: PerceivedEntity) -> Self {
        self.entities.push(entity);
        self
    }
}

impl AddAssign<PerceivedEntity> for Perception {
    fn add_assign(&mut self, entity: PerceivedEntity) {
        self.entities.push(entity);
    }
}

impl Add<PerceivedFood> for Perception {
    type Output = Self;

    fn add(mut self, food: PerceivedFood) -> Self {
        self.foods.push(food);
        self
    }
}

impl AddAssign<PerceivedFood> for Perception {
    fn add_assign(&mut self, food: PerceivedFood) {
        self.foods.push(food);
    }
}

impl Add<PerceivedWater> for Perception {
    type Output = Self;

    fn add(mut self, water: PerceivedWater) -> Self {
        self.waters.push(water);
        self
    }
}

impl AddAssign<PerceivedWater> for Perception {
    fn add_assign(&mut self, water: PerceivedWater) {
        self.waters.push(water);
    }
}

```

## src/entity/phase.rs
```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityPhase {
    /// Aktif, karar alabilir
    Active,

    /// Uyuyor, "remaining" tick boyunca aksiyon yok
    Sleeping { remaining: usize },

    /// Ölü, "remaining" tick sonra kaldırılacak
    Corpse { remaining: usize },

    /// World tarafından kaldırılacak
    Removed,
}

impl EntityPhase {
    /// Canlı karar verebilir mi?
    pub fn is_active(&self) -> bool {
        matches!(self, EntityPhase::Active)
    }

    /// Yaşıyor mu? Ölü mü?
    pub fn is_corpse(&self) -> bool {
        matches!(self, EntityPhase::Corpse { .. })
    }

    /// Uyuyor mu?
    pub fn is_sleeping(&self) -> bool {
        matches!(self, EntityPhase::Sleeping { .. })
    }

    /// Kaldırılmasına gerek var mı?
    pub fn need_remove(&self) -> bool {
        matches!(self, EntityPhase::Removed)
    }

    /// World için tick kolaylığı ve otomatik durum güncellemesi
    pub fn tick(&mut self) {
        match self {
            // Uyuyorsa zamanı düşür, dolduysa sonra ki aşamaya geçir
            EntityPhase::Sleeping { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                } else {
                    *self = EntityPhase::Active;
                }
            }
            // Cesedin ortadan kalkması gereken süreyi düşür,
            // Bittiyse sisteme kaldırası gerektiğini bildir
            EntityPhase::Corpse { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                } else {
                    *self = EntityPhase::Removed;
                }
            }
            _ => {}
        }
    }
}

```

## src/entity/species.rs
```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Species {
    Herbivore,
    Carnivore,
    Omnivore,
}

```

## src/lib.rs
```
// Modülü dahil et
pub mod creatures;
pub mod entity;
pub mod logger;
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

```

## src/logger.rs
```
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Log seviyeleri (sade ama genişletilebilir)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Basit ama ayarlanabilir logger
pub struct Logger {
    min_level: LogLevel,
    to_stdout: bool,
    file: Option<File>,
}

impl Logger {
    /// Dosyaya log yazan logger üretir
    pub fn new(file_path: &str) -> Self {
        let file = Self::open_log_file(file_path);
        Self {
            min_level: LogLevel::Info,
            to_stdout: false,
            file,
        }
    }

    /// Seviye filtreleme (min seviyeden aşağısı yazılmaz)
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }

    /// Stdout yazımını aç/kapat
    pub fn set_stdout(&mut self, enabled: bool) {
        self.to_stdout = enabled;
    }

    /// Tek satır log yaz
    pub fn log(&mut self, level: LogLevel, message: &str) {
        if level < self.min_level {
            return;
        }

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let line = format!("[{}] {:?} | {}\n", ts, level, message);

        if self.to_stdout {
            print!("{}", line);
        }

        if let Some(file) = &mut self.file {
            let _ = file.write_all(line.as_bytes());
        }
    }

    /// Çoklu satır log yazmak için yardımcı
    pub fn log_many(&mut self, level: LogLevel, lines: &[String]) {
        for line in lines {
            self.log(level, line);
        }
    }

    fn open_log_file(file_path: &str) -> Option<File> {
        let path = Path::new(file_path);
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()
    }
}

```

## src/main.rs
```
use curious::{
    creatures::carnivore::CarnivoreEntity,
    creatures::herbivore::HerbivoreEntity,
    creatures::omnivore::OmnivoreEntity,
    entity::{perception::Perception, phase::EntityPhase},
    map::movement::Position,
    set_global_seed_with_time,
    world::{EntitySlot, World},
};
use std::{thread, time::Duration};

fn main() {
    // RNG için seed'i zaman damgası olarak günceller
    set_global_seed_with_time();
    let entities: Vec<EntitySlot> = vec![
        EntitySlot::new(
            1,
            (-15isize, -15isize).into(),
            EntityPhase::Active,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            (-14isize, -15isize).into(),
            EntityPhase::Active,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            3,
            (14isize, -15isize).into(),
            EntityPhase::Active,
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            4,
            (15isize, -15isize).into(),
            EntityPhase::Active,
            Box::new(OmnivoreEntity::default()),
        ),
        EntitySlot::new(
            5,
            (-15isize, 15isize).into(),
            EntityPhase::Active,
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            6,
            (-14isize, 15isize).into(),
            EntityPhase::Active,
            Box::new(OmnivoreEntity::default()),
        ),
    ];
    // İnteraktif dünya
    let mut world = World::new(-15, 14, -15, 14, entities);
    // İnteraktif dünya sayacı
    let mut tick_counter: usize = 0;
    loop {
        print!("\x1B[2J\x1B[1;1H\n");
        world.tick();
        tick_counter += 1;
        print_map(&world, tick_counter);
        thread::sleep(Duration::from_millis(300));
    }
}

pub fn print_map(world: &World, tick: usize) {
    let map_width = world.map.map_width();
    let map_height = world.map.map_height();

    println!(
        "=== SIMULATION | Map: ({}x{})  | Tick: {} ===",
        map_width, map_height, tick
    );
    println!("{:-<1$}", "", map_width * 5);

    for y in world.map.min_y..=world.map.max_y {
        // --- SOL KOLON: HARİTA ---
        for x in world.map.min_x..=world.map.max_x {
            let pos = (x, y).into();

            // Hücredeki varlığı kontrol et (Öncelik: Canlı > Ceset > Yemek)
            if let Some(slot) = world.entities.iter().find(|e| e.pos == pos) {
                match slot.phase {
                    // ANSI TrueColor formatı: \x1b[38;2;R;G;Bm
                    // \x1b[0m kodu ise rengi sıfırlamak içindir
                    EntityPhase::Active => {
                        // Türüne göre renk: Etçil kırmızı, Otçul yeşil, Hepçil mavi
                        let (r, g, b) = match slot.base.species() {
                            curious::entity::species::Species::Carnivore => (220, 40, 40),
                            curious::entity::species::Species::Herbivore => (40, 200, 40),
                            curious::entity::species::Species::Omnivore => (60, 120, 220),
                        };
                        print!("\x1b[38;2;{};{};{}m@ \x1b[0m", r, g, b);
                    } // Canlı
                    EntityPhase::Corpse { .. } => {
                        // Ceset turuncu
                        print!("\x1b[38;2;255;140;0mX \x1b[0m");
                    }
                    _ => print!("? "),
                }
            } else if let Some(curious::map::cell::Cell::Food { .. }) = world.map.cell(pos) {
                // Yemek sarı
                print!("\x1b[38;2;240;220;0mf \x1b[0m");
            } else if let Some(curious::map::cell::Cell::Water { .. }) = world.map.cell(pos) {
                // Su sarı
                print!("\x1b[38;2;240;220;0mw \x1b[0m");
            } else {
                print!(". "); // Boş hücre
            }
        }

        // --- SAĞ KOLON: CANLI DURUMLARI ---
        // Sadece haritanın ilk birkaç satırında canlı bilgilerini yazdır
        let entity_index = (y - world.map.min_y) as usize;
        if let Some(slot) = world.entities.get(entity_index) {
            let life = slot.entity().life();
            print!(
                "  {:?} | @{:<2} {:?} HP:{:<3} EN:{:<3} AGE:{:<3} Ph:{:?} ",
                slot.base.species(),
                slot.id,
                slot.pos,
                life.health,
                life.energy,
                life.age,
                slot.phase
            );
        }

        println!(); // Alt satıra geç
    }
    println!("{:-<1$}", "", map_width + 5);
    println!("@: Canlı | X: Ceset | f: Yemek | w: Su");
}

```

## src/map/cell.rs
```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Food { amount: usize },
    Water { amount: usize },
}

```

## src/map/mod.rs
```
pub mod cell;
pub mod movement;

use std::collections::{HashMap, VecDeque};

use crate::{
    CHUNK_SIZE, gen_range,
    map::{
        cell::Cell,
        movement::{DIRECTION_ARRAY, Direction, Position, Steps},
    },
    next_rand,
};

#[derive(Debug)]
struct Chunk {
    cells: Vec<Cell>,
}

impl Chunk {
    fn new() -> Self {
        Self {
            cells: vec![Cell::Empty; CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    /// Hücre indexi oluştur
    #[inline]
    fn idx(x: usize, y: usize) -> usize {
        y * CHUNK_SIZE + x
    }

    /// Hücreyi oku
    fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[Self::idx(x, y)]
    }

    /// Hücreyi değiştir
    fn cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[Self::idx(x, y)]
    }

    /// Hücre tamamen boşalmış mı?
    fn is_completely_empty(&self) -> bool {
        self.cells.iter().all(|c| matches!(c, Cell::Empty))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    cx: isize,
    cy: isize,
}

#[derive(Debug)]
pub struct Map {
    /// Yatay eksende sağ kısım
    pub min_x: isize,
    /// Yatay eksende sol kısım
    pub max_x: isize,
    /// Dikey eksende sağ kısım
    pub min_y: isize,
    /// Dikey eksende sol kısım
    pub max_y: isize,
    /// Parçalara ayrılmış harita.
    chunks: HashMap<ChunkCoord, Chunk>,
}

impl Map {
    /// Sınırları kontrol ederek güvenli bir dünya oluşturur
    pub fn new(x1: isize, x2: isize, y1: isize, y2: isize) -> Self {
        // Kullanıcı değerleri ters girse bile (min/max) doğru eşleştirilir
        let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };

        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            chunks: HashMap::new(),
        }
    }

    /// Bir dünya pozisyonunun hangi chunk koordinatına denk geldiğini döndürür
    pub fn chunk_coord(pos: Position) -> ChunkCoord {
        ChunkCoord {
            cx: pos.x.div_euclid(CHUNK_SIZE as isize),
            cy: pos.y.div_euclid(CHUNK_SIZE as isize),
        }
    }

    fn local_coord(pos: Position) -> (usize, usize) {
        (
            pos.x.rem_euclid(CHUNK_SIZE as isize) as usize,
            pos.y.rem_euclid(CHUNK_SIZE as isize) as usize,
        )
    }

    fn index_of(&self, pos: Position) -> (ChunkCoord, usize, usize) {
        let cc = Self::chunk_coord(pos);
        let (lx, ly) = Self::local_coord(pos);
        (cc, lx, ly)
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= self.min_x && pos.x <= self.max_x && pos.y >= self.min_y && pos.y <= self.max_y
    }

    pub fn cell(&self, pos: Position) -> Option<&Cell> {
        if !self.in_bounds(pos) {
            return None;
        }
        let (cc, lx, ly) = self.index_of(pos);
        self.chunks.get(&cc).map(|c| c.cell(lx, ly))
    }

    pub fn is_cell(&self, pos: Position, expected: &Cell) -> bool {
        self.cell(pos).map(|c| c == expected).unwrap_or(false)
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        matches!(
            self.cell(pos),
            Some(Cell::Empty | Cell::Food { .. } | Cell::Water { .. })
        )
    }

    pub fn set_cell(&mut self, pos: Position, cell: Cell) {
        // 1. Adım: Dünya sınırları kontrolü
        if !self.in_bounds(pos) {
            return;
        }

        let (cc, lx, ly) = self.index_of(pos);

        // 2. Adım: Eğer hücre boşsa ve chunk yoksa, boş bir hücre için yeni chunk yaratma.
        if cell == Cell::Empty && !self.chunks.contains_key(&cc) {
            return;
        }

        // 3. Adım: Chunk'ı al veya oluştur, ardından hücreyi yaz
        let chunk = self.chunks.entry(cc).or_insert_with(Chunk::new);
        *chunk.cell_mut(lx, ly) = cell;
    }

    pub fn reduce_cell_amount(&mut self, pos: Position, amount: usize) -> bool {
        if !self.in_bounds(pos) {
            return false;
        }

        let (cc, lx, ly) = self.index_of(pos);

        let should_remove = {
            let chunk = match self.chunks.get_mut(&cc) {
                Some(c) => c,
                None => return false,
            };

            match chunk.cell_mut(lx, ly) {
                Cell::Food { amount: a } | Cell::Water { amount: a } => {
                    *a = a.saturating_sub(amount);
                    if *a == 0 {
                        *chunk.cell_mut(lx, ly) = Cell::Empty;
                    }
                }
                _ => return false,
            }
            // Hücre boşaldıktan sonra chunk'ın durumunu kontrol et
            chunk.is_completely_empty()
        };

        if should_remove {
            self.chunks.remove(&cc);
        }
        true
    }

    pub fn clear_cell(&mut self, pos: Position) {
        self.set_cell(pos, Cell::Empty);
    }

    /// Hücreye yiyecek ekle (varsa miktarı artır)
    pub fn add_food(&mut self, pos: Position, amount: usize) {
        if !self.in_bounds(pos) {
            return;
        }

        let new_cell = match self.cell(pos) {
            Some(Cell::Food { amount: a }) => Cell::Food { amount: a + amount },
            _ => Cell::Food { amount },
        };
        self.set_cell(pos, new_cell);
    }

    /// Bir yönde engel gelene kadar kaç adım?
    pub fn walkable_distance(&self, from: Position, dir: Direction) -> u8 {
        let mut cur = from;
        let mut steps = 0u8;

        loop {
            let next = cur + dir;
            if !self.is_walkable(next) {
                break;
            }
            steps += 1;
            cur = next;
            if steps == u8::MAX {
                break;
            }
        }
        steps
    }

    pub fn walkable_distances(&self, from: Position) -> HashMap<Direction, u8> {
        let mut map = HashMap::new();
        for d in DIRECTION_ARRAY {
            map.insert(d, self.walkable_distance(from, d));
        }
        map
    }

    /// Radius ile sınırlı BFS
    pub fn bfs_steps_to(&self, start: Position, goal: Position, radius: usize) -> Option<Steps> {
        if !self.is_walkable(goal) {
            return None;
        }

        let mut queue = VecDeque::new();
        let mut came_from: HashMap<Position, (Position, Direction)> = HashMap::new();

        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            if current == goal {
                break;
            }

            let dist = (current.x - start.x).abs() + (current.y - start.y).abs();
            if dist as usize >= radius {
                continue;
            }
            for dir in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
                Direction::UpLeft,
                Direction::UpRight,
                Direction::DownLeft,
                Direction::DownRight,
            ] {
                let next = current + dir;
                if !self.is_walkable(next) {
                    continue;
                }
                if came_from.contains_key(&next) || next == start {
                    continue;
                }

                came_from.insert(next, (current, dir));
                queue.push_back(next);
            }
        }

        // Path reconstruction
        let mut steps = Vec::new();
        let mut cur = goal;

        while cur != start {
            let (prev, dir) = *came_from.get(&cur)?;
            steps.push(dir);
            cur = prev;
        }

        steps.reverse();
        Some(Steps(steps))
    }

    pub fn scan_foods_within(
        &self,
        center: Position,
        radius: usize,
    ) -> Vec<(Position, Steps, usize)> {
        let mut result = Vec::new();

        for y in (center.y - radius as isize)..=(center.y + radius as isize) {
            for x in (center.x - radius as isize)..=(center.x + radius as isize) {
                let pos = Position { x, y };

                if !self.in_bounds(pos) {
                    continue;
                }

                let manhattan = (center.x - x).abs() + (center.y - y).abs();
                if manhattan as usize > radius {
                    continue;
                }

                if let Some(Cell::Food { amount }) = self.cell(pos) {
                    if let Some(steps) = self.bfs_steps_to(center, pos, radius) {
                        result.push((pos, steps, *amount));
                    }
                }
            }
        }

        result
    }

    pub fn scan_waters_within(
        &self,
        center: Position,
        radius: usize,
    ) -> Vec<(Position, Steps, usize)> {
        let mut result = Vec::new();

        for y in (center.y - radius as isize)..=(center.y + radius as isize) {
            for x in (center.x - radius as isize)..=(center.x + radius as isize) {
                let pos = Position { x, y };

                if !self.in_bounds(pos) {
                    continue;
                }

                let manhattan = (center.x - x).abs() + (center.y - y).abs();
                if manhattan as usize > radius {
                    continue;
                }

                if let Some(Cell::Water { amount }) = self.cell(pos) {
                    if let Some(steps) = self.bfs_steps_to(center, pos, radius) {
                        result.push((pos, steps, *amount));
                    }
                }
            }
        }

        result
    }

    /// Tüm haritayı chunk chunk doldurur (Orkestra Şefi)
    pub fn populate_resources(&mut self, density: f32) {
        // Haritanın kapsadığı chunk sınırlarını hesapla
        // Negatif koordinatlar için div_euclid kullanılmalı,
        // aksi halde değerler 0'a doğru yuvarlandığı için yanlış chunklar seçilir.
        let min_cx = self.min_x.div_euclid(CHUNK_SIZE as isize);
        let max_cx = self.max_x.div_euclid(CHUNK_SIZE as isize);
        let min_cy = self.min_y.div_euclid(CHUNK_SIZE as isize);
        let max_cy = self.max_y.div_euclid(CHUNK_SIZE as isize);

        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                self.populate_chunk(ChunkCoord { cx, cy }, density);
            }
        }
    }

    /// Sadece belirli bir chunk içine odaklanır (Uzman)
    pub fn populate_chunk(&mut self, coord: ChunkCoord, density: f32) {
        let start_x = coord.cx * CHUNK_SIZE as isize;
        let start_y = coord.cy * CHUNK_SIZE as isize;

        let spawn_threshold = (density.clamp(0.0, 1.0) * 100.0).round() as u64;

        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let world_pos = Position::new(start_x + lx as isize, start_y + ly as isize);

                if !self.in_bounds(world_pos)
                    || !self
                        .cell(world_pos)
                        .map_or(true, |c| matches!(c, Cell::Empty))
                {
                    continue;
                }

                let roll = next_rand() % 100;
                if roll >= spawn_threshold {
                    continue;
                }

                let amount = (next_rand() % 7 + 5) as usize;
                let water_roll = next_rand() % 100;
                if water_roll < 20 {
                    self.set_cell(world_pos, Cell::Water { amount });
                } else {
                    self.set_cell(world_pos, Cell::Food { amount });
                }
            }
        }
    }

    pub fn map_width(&self) -> usize {
        (self.max_x - self.min_x + 1) as usize
    }

    pub fn map_height(&self) -> usize {
        (self.max_y - self.min_y + 1) as usize
    }
}

```

## src/map/movement.rs
```
use std::ops::{Add, AddAssign};

/// - Dünya koordinatıdır (chunk bağımsız)
/// - Negatif koordinatları destekler
/// - (0,0) merkezli dünya için uygundur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    /// Yeni bir pozisyon oluştur
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    /// Pozisyonu doğrudan güncelle
    pub fn set(&mut self, other: Position) {
        self.x = other.x;
        self.y = other.y;
    }

    /// Manhattan mesafesi
    /// Çapraz yönler olsa bile karar mekanizması için hâlâ en stabil metriktir
    pub fn distance_to(&self, other: Position) -> usize {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as usize
    }
}

impl From<(isize, isize)> for Position {
    fn from(t: (isize, isize)) -> Position {
        Position { x: t.0, y: t.1 }
    }
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

pub const DIRECTION_ARRAY: [Direction; 8] = [
    Direction::Down,
    Direction::Up,
    Direction::Left,
    Direction::Right,
    Direction::UpLeft,
    Direction::UpRight,
    Direction::DownLeft,
    Direction::DownRight,
];
/// Hareket etme talimat dizisi
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Steps(pub Vec<Direction>);

impl Steps {
    /// Boş bir hareket dizisi oluşturur
    pub fn empty() -> Steps {
        Steps(Vec::new())
    }

    /// Girdiyi Hareket talimatı yapısına ekler
    pub fn new(value: Vec<Direction>) -> Steps {
        Steps(value)
    }

    /// Listenin başından bir eleman al
    /// Eğer boşsa None döner
    pub fn pop_front(&mut self) -> Option<Direction> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.remove(0))
        }
    }

    /// Listenin başındaki elemanı gör ama silme
    pub fn peek_front(&self) -> Option<&Direction> {
        self.0.first()
    }

    /// Vec<Direction> ekle
    pub fn extend(&mut self, other: Steps) {
        self.0.extend(other.0);
    }

    /// Iterator ile erişim
    pub fn iter(&self) -> std::slice::Iter<'_, Direction> {
        self.0.iter()
    }

    /// Mutable iterator
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Direction> {
        self.0.iter_mut()
    }

    /// Adım sayısını döner
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Boş mu kontrol
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// IntoIterator implementasyonu (for x in steps)
impl IntoIterator for Steps {
    type Item = Direction;
    type IntoIter = std::vec::IntoIter<Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// &Steps için iterator
impl<'a> IntoIterator for &'a Steps {
    type Item = &'a Direction;
    type IntoIter = std::slice::Iter<'a, Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// &mut Steps için iterator
impl<'a> IntoIterator for &'a mut Steps {
    type Item = &'a mut Direction;
    type IntoIter = std::slice::IterMut<'a, Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// + Direction -> yeni Steps
impl Add<Direction> for Steps {
    type Output = Steps;

    fn add(mut self, rhs: Direction) -> Steps {
        self.0.push(rhs);
        self
    }
}

// + Vec<Direction> -> yeni Steps
impl Add<Vec<Direction>> for Steps {
    type Output = Steps;

    fn add(mut self, rhs: Vec<Direction>) -> Steps {
        self.0.extend(rhs);
        self
    }
}

/// `Steps += Direction` ile sonuna ekleme
impl AddAssign<Direction> for Steps {
    fn add_assign(&mut self, rhs: Direction) {
        self.0.push(rhs);
    }
}

/// `Steps += Vec<Direction>` ile birden fazla ekleme
impl AddAssign<Vec<Direction>> for Steps {
    fn add_assign(&mut self, rhs: Vec<Direction>) {
        self.0.extend(rhs);
    }
}

/// `Steps += Steps` ile birden fazla ekleme
impl AddAssign<Steps> for Steps {
    fn add_assign(&mut self, rhs: Steps) {
        self.0.extend(rhs);
    }
}

/// Vec<Direction> -> Steps
impl From<Vec<Direction>> for Steps {
    fn from(vec: Vec<Direction>) -> Steps {
        Steps(vec)
    }
}

/// Steps -> Vec<Direction>
impl From<Steps> for Vec<Direction> {
    fn from(steps: Steps) -> Vec<Direction> {
        steps.0
    }
}

/// Position + Direction → Position
///
/// Çapraz hareketler desteklenir.
/// World isterse çaprazı yasaklayabilir (Map / validation katmanı).
impl Add<Direction> for Position {
    type Output = Position;

    /// Yön bazlı yeni pozisyon (immutable)
    fn add(self, dir: Direction) -> Position {
        match dir {
            Direction::Up => Position {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Position {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Position {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Position {
                x: self.x + 1,
                y: self.y,
            },
            Direction::UpLeft => Position {
                x: self.x - 1,
                y: self.y - 1,
            },
            Direction::UpRight => Position {
                x: self.x + 1,
                y: self.y - 1,
            },
            Direction::DownLeft => Position {
                x: self.x - 1,
                y: self.y + 1,
            },
            Direction::DownRight => Position {
                x: self.x + 1,
                y: self.y + 1,
            },
        }
    }
}

```

## src/world.rs
```
use crate::{
    entity::{Entity, intent::Intent, perception::*, phase::EntityPhase},
    gen_range,
    logger::{LogLevel, Logger},
    map::{
        movement::{Direction, Position, DIRECTION_ARRAY},
        Map,
    },
};
use std::collections::HashMap;

/// Canlının yönetim biçimi
pub struct EntitySlot {
    /// Canlının benzerhsiz kimlik numarası
    pub id: usize,
    /// Canlının konumu
    pub pos: Position,
    /// Canlının bulunduğu durum (aktif, uykuda, ölü, silinecek)
    pub phase: EntityPhase,
    /// Canlının verisi
    pub base: Box<dyn Entity>,
}

impl EntitySlot {
    /// Yeni canlı oluştur
    pub fn new(id: usize, pos: Position, phase: EntityPhase, base: Box<dyn Entity>) -> EntitySlot {
        Self {
            id,
            pos,
            phase,
            base,
        }
    }

    /// Canlının bulunduğu konum
    pub fn position(&self) -> &Position {
        &self.pos
    }

    /// Canlının bulunduğu konumu (değiştirilebilir)
    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.pos
    }

    /// Canlıyı döndürür, okumak için
    pub fn entity(&self) -> &dyn Entity {
        self.base.as_ref()
    }
    /// Canlıyı döndürür, yazmak için
    pub fn entity_mut(&mut self) -> &mut dyn Entity {
        self.base.as_mut()
    }

    /// Canlı durumunu döndürür
    pub fn phase(&self) -> &EntityPhase {
        &self.phase
    }

    /// Canlının durumunu değiştirilebilir
    pub fn phase_mut(&mut self) -> &mut EntityPhase {
        &mut self.phase
    }
}

/// Entity'ler burada tutulur,
/// Intent verebilicek durumda ki Entity'lere
/// Perception verip, Intent alarak
/// Kendi içerisinde ki kurallar dahilinde
/// Son kararı verir.
pub struct World {
    /// Simülasyon haritası
    pub map: Map,

    /// Tüm Canlıların ID, Pos ve Entity listesi
    pub entities: Vec<EntitySlot>,

    /// Simülasyon tur sayacı
    pub tick_counter: usize,

    /// Gelişmiş loglama
    pub logger: Logger,
}

impl World {
    pub fn new(x1: isize, x2: isize, y1: isize, y2: isize, entities: Vec<EntitySlot>) -> World {
        // Haritayı oluştur
        let mut map = Map::new(x1, x2, y1, y2);
        // Haritanın % kısmına rastgele kaynak yerleştir.
        // Kaynak yoğunluğunu biraz düşür (aşırı doygunluk davranışları baskılamasın)
        map.populate_resources(0.05f32);
        // Döndür
        // Her çalıştırmada ayrı log dosyası oluştur (okunabilir tarih/saat)
        let now = time::OffsetDateTime::now_utc();
        let format = time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]-[second]")
            .unwrap_or_else(|_| time::format_description::parse("[year][month][day]_[hour][minute][second]").unwrap());
        let ts = now.format(&format).unwrap_or_else(|_| "unknown_time".to_string());
        let log_path = format!("logs/simulation_{}.log", ts);
        let mut logger = Logger::new(&log_path);
        logger.set_min_level(LogLevel::Info);
        World {
            map,
            entities,
            tick_counter: 0,
            logger,
        }
    }

    /// Tick, bir zaman birimidir
    /// Tick, canlının bulunduğu konumu baz alarak Perception oluşturur.
    /// Entity, verilen Perception ile karar alır.
    /// World, Perception -> Intent şeklinde yola koyulur.
    /// Son adımda sonuca Intent Resolver karar verir.
    /// BU KARAR KESİNLİK DEĞİLDİR, WORLD SON SÖZÜ SÖYLER
    /// ÇAKIŞAN NİYETLER İÇİN WORLD İNSİYATİF ALABİLİR
    pub fn tick(&mut self) {
        self.tick_counter += 1;

        // Removed aşamasındaki entityleri sil
        self.entities
            .retain(|slot| !matches!(slot.phase, EntityPhase::Removed));

        let mut log_lines: Vec<String> = Vec::new();
        log_lines.push(format!("=== Tick {} ===", self.tick_counter));

        // Çakışma çözümü ve hızlı erişim için dolu hücre haritası
        let mut occupied: HashMap<Position, usize> = self
            .entities
            .iter()
            .filter(|slot| !matches!(slot.phase, EntityPhase::Corpse { .. } | EntityPhase::Removed))
            .map(|slot| (slot.pos, slot.id))
            .collect();

        let mut intents: Vec<(usize, Intent)> = Vec::new();

        // Her entity için perception ve intent oluştur
        for slot in &self.entities {
            if !slot.phase.is_active() {
                continue; // Sadece aktif canlılar karar verir
            }
            let perception = self.build_perception(slot);
            let intent = slot.entity().make_intent(perception);
            intents.push((slot.id, intent));

            // Niyet logu (Idle ise her 5 tick'te bir yaz)
            let last_intent = intents.last().unwrap().1.clone();
            let should_log_intent = !matches!(last_intent, Intent::Idle { .. })
                || (self.tick_counter % 5 == 0);
            if should_log_intent {
                log_lines.push(format!(
                    "[Niyet] @{} {:?} Pos:{:?} => {:?}",
                    slot.id,
                    slot.base.species(),
                    slot.pos,
                    last_intent
                ));
            }
        }

        // Intentleri çöz
        //
        // 1. Move planları ve mate planlarını önceden topla
        let mut move_plans: Vec<(usize, Position, usize)> = Vec::new();
        let mut eat_plans: Vec<(usize, Position, usize)> = Vec::new();
        let mut drink_plans: Vec<(usize, Position, usize)> = Vec::new();
        let mut mate_plans: Vec<(usize, usize)> = Vec::new();
        let mut attack_plans: Vec<(usize, usize)> = Vec::new();
        let mut flee_plans: Vec<(usize, Position, usize)> = Vec::new();
        let mut sleep_plans: Vec<(usize, usize)> = Vec::new();

        for (id, intent) in intents {
            match intent {
                Intent::Move { steps } => {
                    if !steps.is_empty() {
                        if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                            let mut new_pos: Position = slot.pos;
                            let mut cost: usize = 0;

                            for dir in steps.0.iter() {
                                if !self.map.is_walkable(new_pos + *dir)
                                    || !slot.base.life().can_move_for(cost + 1)
                                {
                                    break;
                                }
                                cost += 1;
                                new_pos = new_pos + *dir;
                            }
                            move_plans.push((id, new_pos, cost));
                            log_lines.push(format!(
                                "[Plan] Move  @{} {:?} -> {:?} adim:{}",
                                slot.id,
                                slot.base.species(),
                                new_pos,
                                cost
                            ));
                        }
                    }
                }
                Intent::Eat { at, corpse_id: _ } => {
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        let mut new_pos: Position = slot.pos;
                        let mut cost: usize = 0;
                        for dir in at.0.iter() {
                            if !self.map.is_walkable(new_pos + *dir)
                                || !slot.base.life().can_move_for(cost + 1)
                            {
                                break;
                            }
                            cost += 1;
                            new_pos = new_pos + *dir;
                        }
                        // Aynı hücredeyse de yeme planı üret
                        eat_plans.push((id, new_pos, cost));
                        log_lines.push(format!(
                            "[Plan] Eat   @{} {:?} -> {:?} adim:{}",
                            slot.id,
                            slot.base.species(),
                            new_pos,
                            cost
                        ));
                    }
                }
                Intent::Drink { at } => {
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        let mut new_pos: Position = slot.pos;
                        let mut cost: usize = 0;
                        for dir in at.0.iter() {
                            if !self.map.is_walkable(new_pos + *dir)
                                || !slot.base.life().can_move_for(cost + 1)
                            {
                                break;
                            }
                            cost += 1;
                            new_pos = new_pos + *dir;
                        }
                        // Aynı hücredeyse de içme planı üret
                        drink_plans.push((id, new_pos, cost));
                        log_lines.push(format!(
                            "[Plan] Drink @{} {:?} -> {:?} adim:{}",
                            slot.id,
                            slot.base.species(),
                            new_pos,
                            cost
                        ));
                    }
                }
                Intent::Mate { target_id } => {
                    mate_plans.push((id, target_id));
                    log_lines.push(format!(
                        "[Plan] Mate  @{} -> @{}",
                        id, target_id
                    ));
                }
                Intent::Attack { target_id } => {
                    attack_plans.push((id, target_id));
                    log_lines.push(format!(
                        "[Plan] Attack @{} -> @{}",
                        id, target_id
                    ));
                }
                Intent::Flee { target_id } => {
                    let target_pos = match self.entities.iter().find(|s| s.id == target_id) {
                        Some(t) => t.pos,
                        None => continue,
                    };
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        let mut new_pos: Position = slot.pos;
                        let mut cost: usize = 0;

                        for _ in 0..slot.base.life().speed {
                            // Hedefe en çok uzaklaştıran yönü seç
                            let mut best_dir: Option<Direction> = None;
                            let mut best_dist: usize = new_pos.distance_to(target_pos);

                            for dir in DIRECTION_ARRAY {
                                let candidate = new_pos + dir;
                                if !self.map.is_walkable(candidate) {
                                    continue;
                                }
                                let dist = candidate.distance_to(target_pos);
                                if dist > best_dist {
                                    best_dist = dist;
                                    best_dir = Some(dir);
                                }
                            }

                            // Daha iyi bir yön yoksa, yürünebilir herhangi bir yönü seç
                            let dir = match best_dir {
                                Some(d) => d,
                                None => {
                                    let mut fallback: Option<Direction> = None;
                                    for d in DIRECTION_ARRAY {
                                        let candidate = new_pos + d;
                                        if self.map.is_walkable(candidate) {
                                            fallback = Some(d);
                                            break;
                                        }
                                    }
                                    let Some(d) = fallback else { break };
                                    d
                                }
                            };
                            if !slot.base.life().can_move_for(cost + 1) {
                                break;
                            }
                            cost += 1;
                            new_pos = new_pos + dir;
                        }

                        flee_plans.push((id, new_pos, cost));
                        log_lines.push(format!(
                            "[Plan] Flee  @{} -> {:?} (hedef @{}) adim:{}",
                            id, new_pos, target_id, cost
                        ));
                    }
                }
                Intent::Idle { duration: _ } => {
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        // Hafif gezinme: %30 ihtimalle 1 adım rastgele dene
                        const IDLE_MOVE_CHANCE: isize = 30;
                        let roll = gen_range(1, 100);
                        if roll <= IDLE_MOVE_CHANCE && slot.base.life().can_move_for(1) {
                            let mut chosen: Option<Position> = None;
                            for _ in 0..8 {
                                let dir = DIRECTION_ARRAY[gen_range(0, 7isize) as usize];
                                let candidate = slot.pos + dir;
                                if self.map.is_walkable(candidate) {
                                    chosen = Some(candidate);
                                    break;
                                }
                            }
                            if let Some(pos) = chosen {
                                move_plans.push((id, pos, 1));
                                log_lines.push(format!(
                                    "[Plan] Idle->Move @{} {:?} -> {:?} adim:1",
                                    slot.id,
                                    slot.base.species(),
                                    pos
                                ));
                            }
                        }
                    }
                }
                Intent::Sleep { duration } => {
                    sleep_plans.push((id, duration));
                    log_lines.push(format!(
                        "[Plan] Sleep @{} sure:{}",
                        id, duration
                    ));
                }
                _ => {}
            }
        }

        // ------------------------------
        // 2. Move planlarını uygula (çakışma çözümü ile)
        // ------------------------------
        let mut move_candidates: HashMap<Position, Vec<(usize, Position, usize)>> = HashMap::new();
        for plan in &move_plans {
            move_candidates.entry(plan.1).or_default().push(*plan);
        }

        let mut move_winners: Vec<(usize, Position, usize)> = move_candidates
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|(id, _, _)| *id);
                group[0]
            })
            .collect();

        move_winners.sort_by_key(|(id, _, _)| *id);

        for (id, new_pos, cost) in move_winners {
            // Başka biri orayı tutuyorsa hareketi engelle
            if let Some(other_id) = occupied.get(&new_pos) {
                if *other_id != id {
                    log_lines.push(format!(
                        "[Engel] Move  @{} -> {:?} (doluluk @{})",
                        id, new_pos, other_id
                    ));
                    continue;
                }
            }

            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                // Eski pozisyonu boşalt
                occupied.remove(&slot.pos);
                slot.base.life_mut().on_move(cost);
                slot.pos = new_pos;
                occupied.insert(new_pos, id);

                log_lines.push(format!(
                    "[Uygula] Move  @{} -> {:?} adim:{}",
                    id, new_pos, cost
                ));
            }
        }

        // ------------------------------
        // 3. Eat planlarını uygula (çakışma çözümü ile)
        // ------------------------------
        let mut eat_candidates: HashMap<Position, Vec<(usize, Position, usize)>> = HashMap::new();
        for plan in &eat_plans {
            eat_candidates.entry(plan.1).or_default().push(*plan);
        }

        let mut eat_winners: Vec<(usize, Position, usize)> = eat_candidates
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|(id, _, _)| *id);
                group[0]
            })
            .collect();

        eat_winners.sort_by_key(|(id, _, _)| *id);

        for (id, new_pos, cost) in eat_winners {
            if let Some(other_id) = occupied.get(&new_pos) {
                if *other_id != id {
                    log_lines.push(format!(
                        "[Engel] Eat   @{} -> {:?} (doluluk @{})",
                        id, new_pos, other_id
                    ));
                    continue;
                }
            }

            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                occupied.remove(&slot.pos);
                slot.pos = new_pos;
                slot.base.life_mut().on_move(cost);
                occupied.insert(new_pos, id);

                if let Some(cell) = self.map.cell(new_pos) {
                    if let crate::map::cell::Cell::Food { amount } = cell {
                        //println!("[@{}] Entity eating from {:?}", slot.id, slot.pos);
                        let eat_amount = *amount.min(&5);
                        slot.entity_mut().life_mut().restore_energy(eat_amount);
                        self.map.reduce_cell_amount(new_pos, eat_amount);

                        log_lines.push(format!(
                            "[Uygula] Eat   @{} -> {:?} miktar:{}",
                            id, new_pos, eat_amount
                        ));
                    }
                }
            }
        }

        // ------------------------------
        // 3.1 Drink planlarını uygula (çakışma çözümü ile)
        // ------------------------------
        let mut drink_candidates: HashMap<Position, Vec<(usize, Position, usize)>> = HashMap::new();
        for plan in &drink_plans {
            drink_candidates.entry(plan.1).or_default().push(*plan);
        }

        let mut drink_winners: Vec<(usize, Position, usize)> = drink_candidates
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|(id, _, _)| *id);
                group[0]
            })
            .collect();

        drink_winners.sort_by_key(|(id, _, _)| *id);

        for (id, new_pos, cost) in drink_winners {
            if let Some(other_id) = occupied.get(&new_pos) {
                if *other_id != id {
                    log_lines.push(format!(
                        "[Engel] Drink @{} -> {:?} (doluluk @{})",
                        id, new_pos, other_id
                    ));
                    continue;
                }
            }

            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                occupied.remove(&slot.pos);
                slot.pos = new_pos;
                slot.base.life_mut().on_move(cost);
                occupied.insert(new_pos, id);

                if let Some(cell) = self.map.cell(new_pos) {
                    if let crate::map::cell::Cell::Water { amount } = cell {
                        let drink_amount = *amount.min(&5);
                        slot.entity_mut().life_mut().restore_water(drink_amount);
                        self.map.reduce_cell_amount(new_pos, drink_amount);

                        log_lines.push(format!(
                            "[Uygula] Drink @{} -> {:?} miktar:{}",
                            id, new_pos, drink_amount
                        ));
                    }
                }
            }
        }

        // ------------------------------
        // 4. Mate planlarını uygula
        // ------------------------------

        let mut new_entities: Vec<crate::world::EntitySlot> = Vec::new();
        let id_to_index: HashMap<usize, usize> = self
            .entities
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id, i))
            .collect();

        for (self_id, target_id) in mate_plans {
            let self_index = id_to_index.get(&self_id).copied();
            let target_index = id_to_index.get(&target_id).copied();

            let (self_index, target_index) = match (self_index, target_index) {
                (Some(a), Some(t)) if a != t => (a, t),
                _ => continue,
            };

            // Aynı anda iki mutable borrow için split_at_mut kullanılır
            let (left, right) = if self_index < target_index {
                let (l, r) = self.entities.split_at_mut(target_index);
                (l, r)
            } else {
                let (l, r) = self.entities.split_at_mut(self_index);
                (r, l)
            };

            let (self_slot, target_slot) = if self_index < target_index {
                (&mut left[self_index], &mut right[0])
            } else {
                (&mut left[0], &mut right[target_index])
            };

            // İkisi de aktif olmalı
            if !self_slot.phase.is_active() || !target_slot.phase.is_active() {
                continue;
            }

            // Yakınlık kontrolü (çapraz dahil komşu)
            let dx = (self_slot.pos.x - target_slot.pos.x).abs();
            let dy = (self_slot.pos.y - target_slot.pos.y).abs();
            if dx > 1 || dy > 1 {
                log_lines.push(format!(
                    "[Engel] Mate  @{} + @{} (mesafe x:{} y:{})",
                    self_id, target_id, dx, dy
                ));
                continue;
            }

            // İki tarafın da üreme koşulları uygun olmalı
            if !self_slot.entity().life().can_reproduce()
                || !target_slot.entity().life().can_reproduce()
            {
                continue;
            }

            // Çocuğun doğacağı boş bir komşu hücre bul
            let mut child_pos: Option<Position> = None;
            for dir in DIRECTION_ARRAY {
                let candidate = target_slot.pos + dir;
                if self.map.is_walkable(candidate) && !occupied.contains_key(&candidate) {
                    child_pos = Some(candidate);
                    break;
                }
            }
            let Some(child_pos) = child_pos else {
                log_lines.push(format!(
                    "[Engel] Mate  @{} + @{} (bos komsu yok)",
                    self_id, target_id
                ));
                continue;
            };

            // Üreme maliyetleri
            self_slot.entity_mut().life_mut().on_reproduce();
            target_slot.entity_mut().life_mut().on_reproduce();

            let child = target_slot.entity_mut().reproduce();
            let new_id = self.entities.iter().map(|s| s.id).max().unwrap_or(0) + 1;

            new_entities.push(crate::world::EntitySlot::new(
                new_id,
                child_pos,
                crate::entity::phase::EntityPhase::Active,
                child,
            ));

            // Yeni doğan pozisyonu işgal edildi
            occupied.insert(child_pos, new_id);

            log_lines.push(format!(
                "[Uygula] Mate  @{} + @{} => @{} {:?}",
                self_id, target_id, new_id, child_pos
            ));
        }
        self.entities.extend(new_entities);

        // ------------------------------
        // 5. Attack planlarını uygula
        // ------------------------------
        let id_to_index: HashMap<usize, usize> = self
            .entities
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id, i))
            .collect();

        for (attacker_id, target_id) in attack_plans {
            let attacker_index = id_to_index.get(&attacker_id).copied();
            let target_index = id_to_index.get(&target_id).copied();

            let (attacker_index, target_index) = match (attacker_index, target_index) {
                (Some(a), Some(t)) if a != t => (a, t),
                _ => continue,
            };

            // Aynı anda iki mutable borrow için split_at_mut kullanılır
            let (left, right) = if attacker_index < target_index {
                let (l, r) = self.entities.split_at_mut(target_index);
                (l, r)
            } else {
                let (l, r) = self.entities.split_at_mut(attacker_index);
                (r, l)
            };

            let (attacker, target) = if attacker_index < target_index {
                (&mut left[attacker_index], &mut right[0])
            } else {
                (&mut left[0], &mut right[target_index])
            };

            // Sadece aktif hedefe saldır
            if !target.phase.is_active() {
                log_lines.push(format!(
                    "[Engel] Attack @{} -> @{} (hedef aktif degil)",
                    attacker_id, target_id
                ));
                continue;
            }

            // Yakınlık kontrolü (çapraz dahil komşu)
            let dx = (attacker.pos.x - target.pos.x).abs();
            let dy = (attacker.pos.y - target.pos.y).abs();
            if dx <= 1 && dy <= 1 {
                // Basit hasar modeli
                attacker.entity_mut().life_mut().consume_energy(3);
                target.entity_mut().life_mut().take_damage(6);

                log_lines.push(format!(
                    "[Uygula] Attack @{} -> @{} hasar:{}",
                    attacker_id, target_id, 6
                ));
            } else {
                log_lines.push(format!(
                    "[Engel] Attack @{} -> @{} (mesafe x:{} y:{})",
                    attacker_id, target_id, dx, dy
                ));
            }
        }

        // ------------------------------
        // 6. Flee planlarını uygula (çakışma çözümü ile)
        // ------------------------------
        let mut flee_candidates: HashMap<Position, Vec<(usize, Position, usize)>> = HashMap::new();
        for plan in &flee_plans {
            flee_candidates.entry(plan.1).or_default().push(*plan);
        }

        let mut flee_winners: Vec<(usize, Position, usize)> = flee_candidates
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|(id, _, _)| *id);
                group[0]
            })
            .collect();

        flee_winners.sort_by_key(|(id, _, _)| *id);

        for (id, new_pos, cost) in flee_winners {
            if let Some(other_id) = occupied.get(&new_pos) {
                if *other_id != id {
                    log_lines.push(format!(
                        "[Engel] Flee  @{} -> {:?} (doluluk @{})",
                        id, new_pos, other_id
                    ));
                    continue;
                }
            }

            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                occupied.remove(&slot.pos);
                if new_pos != slot.pos {
                    slot.base.life_mut().on_move(cost);
                    slot.pos = new_pos;
                } else {
                    log_lines.push(format!(
                        "[Engel] Flee  @{} -> {:?} (yerinde kaldı)",
                        id, slot.pos
                    ));
                }
                occupied.insert(slot.pos, id);

                log_lines.push(format!(
                    "[Uygula] Flee  @{} -> {:?} adim:{}",
                    id, slot.pos, cost
                ));
            }
        }

        // ------------------------------
        // 7. Sleep planlarını uygula
        // ------------------------------
        for (id, duration) in sleep_plans {
            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                if slot.phase.is_active() {
                    slot.phase = EntityPhase::Sleeping { remaining: duration };
                    log_lines.push(format!(
                        "[Uygula] Sleep @{} sure:{}",
                        id, duration
                    ));
                }
            }
        }

        for slot in &mut self.entities {
            // Sadece canlı olanların tick güncellemelerini uygula (yaş, enerji, speed reset vb.)
            if slot.phase.is_active() {
                slot.entity_mut().tick();
            }
            // Fazları güncelle ve ölüleri işaretle
            slot.phase.tick();

            if slot.phase == EntityPhase::Active && !slot.entity().life().is_alive() {
                slot.phase = EntityPhase::Corpse { remaining: 5 }; // Ceset 5 tick kalacak
                // Cesedi yiyeceğe dönüştür
                let life = slot.entity().life();
                let amount = (life.max_health / 4).max(5);
                self.map.add_food(slot.pos, amount);

                log_lines.push(format!(
                    "[Durum] Ceset @{} -> Food miktar:{}",
                    slot.id, amount
                ));
            }
        }

        // Tick sonunda logları yaz
        self.logger.log_many(LogLevel::Info, &log_lines);
    }

    /// Intentleri çöz ve uygulama fonksiyonu
    //pub fn resolve_intent(&mut self, intents: Vec<(usize, Intent)>) {}

    /// Entity "Intent" üretebilmesi için "Perception" üretir
    pub fn build_perception(&self, current_slot: &EntitySlot) -> Perception {
        let mut perception = Perception::empty();
        let radius = current_slot.base.life().vision_range; // Görüş mesafesi (yarıçap)

        // 1. Yakındaki Yiyecekleri Algıla
        let found_foods = self.map.scan_foods_within(current_slot.pos, radius);
        for (_f_pos, steps, amount) in found_foods {
            perception.add_food(amount, false, steps);
        }

        // 1.1 Yakındaki Suları Algıla
        let found_waters = self.map.scan_waters_within(current_slot.pos, radius);
        for (_w_pos, steps, amount) in found_waters {
            perception.add_water(amount, steps);
        }

        // 2. Yakındaki Diğer Canlıları Algıla
        for other in &self.entities {
            // Kendisini algılamasın
            if other.id == current_slot.id {
                continue;
            }

            // Mesafe kontrolü (Manhattan mesafesi kullanılıyor)
            let dist = current_slot.pos.distance_to(other.pos);

            if dist <= radius {
                // Canlıya giden yolu (Steps) BFS ile hesapla
                if let Some(steps) = self.map.bfs_steps_to(current_slot.pos, other.pos, radius) {
                    // Algılanan canlıyı ekle (ID, Tür ve Adımlar)
                    let other_life = other.entity().life();
                    let power = other_life.health + other_life.energy;
                    perception.add_entity(other.id, other.entity().species(), power, steps);
                }
            }
        }

        // 3. Yürünebilir Yönleri ve Mesafeleri Algıla
        let walkable_map = self.map.walkable_distances(current_slot.pos);
        for (dir, dist) in walkable_map {
            perception.add_direction(dir, dist);
        }

        perception
    }

    // Bu pozisyonda entity var mı?
    //pub fn has_entity(&self, pos: Position) -> bool { self.entity_pos.contains_key(&pos) }
    // Bu pozisyondaki entity id'leri
    //pub fn entities_at(&self, pos: Position) -> &[usize] { self.entity_pos.get(&pos).map(|v| v.as_slice()).unwrap_or(&[]) }
    // Bu pozisyonda canlı entity var mı?
    //pub fn has_alive_entity(&self, pos: Position) -> bool { self.entities_at(pos).iter().any(|id| self.entity_phase.get(id).is_some_and(|p| p.is_active()))}
    // Bu pozisyonda ceset var mı?
    //pub fn has_corpse(&self, pos: Position) -> bool {self.entities_at(pos).iter().any(|id| self.entity_phase.get(id).is_some_and(|p| p.is_corpse()))}
    // Belirli bir merkez etrafında (Manhattan mesafe)
    // entity olan pozisyonları döner
    //pub fn nearby_entities(&self, center: Position, radius: usize) -> Vec<(Position, usize)> {      let mut result = Vec::new();        for (pos, ids) in self.entity_pos.iter() {            let dx = pos.x.abs_diff(center.x);            let dy = pos.y.abs_diff(center.y);            if dx + dy <= radius {                for id in ids {       result.push((*pos, *id));                }            }        }   result  }
}

```

