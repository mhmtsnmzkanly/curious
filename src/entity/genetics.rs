//! Heritable variation is separate from individual experience.
use crate::{Result, random::Random};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Genome {
    pub mobility: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genetics {
    pub enabled: bool,
    pub mutation_per_mille: u16,
}
impl Default for Genetics {
    fn default() -> Self {
        Self {
            enabled: true,
            mutation_per_mille: 20,
        }
    }
}
impl Genome {
    pub fn validate(&self) -> Result<()> {
        if self.mobility > 2 {
            return Err("mobility allele must be 0..2".into());
        }
        Ok(())
    }
    pub fn child(&self, partner: &Self, rng: &mut Random, settings: &Genetics) -> Self {
        if !settings.enabled {
            return self.clone();
        }
        let mut child = if rng.range(0, 1) == 0 {
            self.clone()
        } else {
            partner.clone()
        };
        if rng.range(0, 999) < settings.mutation_per_mille as isize {
            child.mobility = match child.mobility {
                0 => 1,
                2 => 1,
                _ => {
                    if rng.range(0, 1) == 0 {
                        0
                    } else {
                        2
                    }
                }
            };
        }
        child
    }
    pub fn express_from(&self, previous: &Self, life: &mut super::lifestate::LifeState) {
        life.speed = (life.speed as isize + self.mobility as isize - previous.mobility as isize)
            .clamp(1, 32) as usize;
        life.move_energy_cost = 1 + self.mobility as usize;
    }
}
