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
    pub fn evaluate(life: &LifeState, perception: &Perception) -> InstinctDecision {
        let own_power = life.health + life.energy;
        let threat = perception
            .entities
            .iter()
            .find(|entity| entity.species != Species::Herbivore)
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
