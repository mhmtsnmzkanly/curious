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
            .find(|entity| entity.species.hunts(&own_species) && entity.steps.len() <= THREAT_RANGE)
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

        // A locally available mate is an opportunity, not a reason to postpone
        // feeding indefinitely. Physiological reproduction limits still apply.
        if life.can_reproduce()
            && perception
                .entities
                .iter()
                .any(|e| e.species == own_species && e.reproductive && e.steps.len() <= 1)
        {
            return InstinctDecision {
                instinct: Instinct::Mating,
                threat: None,
            };
        }

        // Refill reserves before seeking reproduction; quarter-full thresholds
        // otherwise trap adults permanently below the reproduction requirement.
        // Injured animals need a reserve above the healing threshold. Stopping
        // feeding at that same threshold lets basal metabolism switch healing
        // off again, leaving damaged juveniles below reproductive health.
        let mut food_target = if life.health <= life.max_health / 2 {
            life.max_energy.saturating_mul(9) / 10
        } else {
            life.max_energy.saturating_mul(3) / 4
        };
        // Complete a reproductive reserve from a profitable visible meal, not
        // by starting extra hunts above the normal hunger threshold. A blanket
        // higher appetite overconsumes prey in the coexistence fixture. Only
        // prepare for an actual adjacent ready partner, not a hypothetical one.
        let available_meal = perception.foods.iter().any(|food| {
            food.amount > 0
                && if food.is_corpse {
                    own_species.eats_meat()
                } else {
                    own_species.eats_plants()
                }
                && super::foraging::value(food.amount, food.steps.len(), life, false) > 0
        });
        if life.is_mature()
            && life.reproduction_cooldown == 0
            && life.health >= life.max_health / 2
            && available_meal
            && perception.entities.iter().any(|mate| {
                mate.species == own_species && mate.reproductive && mate.steps.len() <= 1
            })
        {
            food_target = food_target.max(life.reproduction_energy_required());
        }
        let needs_food = life.energy < food_target;
        let needs_water = life.water < life.max_water.saturating_mul(3) / 4;
        if needs_water
            && (!needs_food
                || life.water.saturating_mul(life.max_energy)
                    <= life.energy.saturating_mul(life.max_water))
        {
            return InstinctDecision {
                instinct: Instinct::Thirst,
                threat: None,
            };
        }
        if life.energy == 0 {
            return InstinctDecision {
                instinct: Instinct::Survival,
                threat: None,
            };
        }
        if needs_food {
            return InstinctDecision {
                instinct: Instinct::Hunger,
                threat: None,
            };
        }
        if life.can_reproduce() {
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
