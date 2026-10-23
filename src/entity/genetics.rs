//! Heritable variation is separate from individual experience.
use crate::{random::Random, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Genome {
    pub mobility: u8,
    #[serde(default)]
    pub vision: u8,
    #[serde(default)]
    pub stamina: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genetics {
    pub enabled: bool,
    pub mutation_per_mille: u16,
    #[serde(default)]
    pub extra_traits_enabled: bool,
}
impl Default for Genetics {
    fn default() -> Self {
        Self {
            enabled: true,
            mutation_per_mille: 20,
            extra_traits_enabled: false,
        }
    }
}
impl Genetics {
    pub fn validate(&self) -> Result<()> {
        if self.mutation_per_mille > 1000 {
            return Err("mutation rate must be 0..1000 per mille".into());
        }
        Ok(())
    }
}
impl Genome {
    pub fn validate(&self) -> Result<()> {
        if self.mobility > 2 || self.vision > 2 || self.stamina > 2 {
            return Err("genome alleles must be 0..2".into());
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
        // Keep the legacy mobility RNG path byte-for-byte unchanged unless the
        // new opt-in is active.
        if settings.extra_traits_enabled {
            for allele in [&mut child.vision, &mut child.stamina] {
                if rng.range(0, 999) < settings.mutation_per_mille as isize {
                    *allele = match *allele {
                        0 => 1,
                        2 => 1,
                        _ if rng.range(0, 1) == 0 => 0,
                        _ => 2,
                    };
                }
            }
        }
        child
    }
    pub fn express_from(&self, previous: &Self, life: &mut super::lifestate::LifeState) {
        life.speed = (life.speed as isize + self.mobility as isize - previous.mobility as isize)
            .clamp(1, 32) as usize;
        life.move_energy_cost = 1 + self.mobility as usize;
    }

    /// Express opt-in physiology separately from the established mobility
    /// allele path. Costs become body maintenance terms paid by metabolism.
    pub fn express_extra_from(&self, previous: &Self, life: &mut super::lifestate::LifeState) {
        let vision_delta = self.vision as isize - previous.vision as isize;
        life.vision_range = (life.vision_range as isize + vision_delta).clamp(1, 32) as usize;
        let stamina_delta = (self.stamina as isize - previous.stamina as isize) * 4;
        life.max_stamina = (life.max_stamina as isize + stamina_delta).clamp(1, 1000) as usize;
        life.stamina =
            (life.stamina as isize + stamina_delta).clamp(0, life.max_stamina as isize) as usize;
        life.vision_metabolic_cost = self.vision as usize;
        life.stamina_metabolic_cost = self.stamina as usize;
    }
}
