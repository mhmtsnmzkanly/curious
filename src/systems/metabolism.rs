use crate::{rules::biology, world::EntitySlot};

/// Canlıların içsel durumlarını (enerji tüketimi, yaşlanma, iyileşme) yöneten sistem.
pub struct MetabolismSystem;

impl MetabolismSystem {
    pub fn update(entities: &mut [EntitySlot]) {
        for slot in entities {
            if matches!(
                slot.phase,
                crate::entity::phase::EntityPhase::Corpse { .. }
                    | crate::entity::phase::EntityPhase::Removed
            ) {
                continue;
            }
            let life = slot.base.life_mut();
            life.stamina = life
                .stamina
                .saturating_add(if life.exertion == 0 { 4 } else { 1 })
                .min(life.max_stamina);
            life.exertion = 0;
            life.moves_used = 0;
            if !life.is_alive() {
                continue;
            }
            life.age = life.age.saturating_add(biology::AGING_RATE);
            if life.age > life.max_age {
                life.health = 0;
                life.death_cause = Some(crate::simulation::event::DeathCause::OldAge);
                continue;
            }
            life.reproduction_cooldown = life.reproduction_cooldown.saturating_sub(1);
            life.consume_energy(biology::BMR_ENERGY_COST);
            life.consume_water(biology::BMR_WATER_COST);
            if life.energy == 0 {
                life.damage_from(
                    biology::STARVATION_HEALTH_LOSS,
                    crate::simulation::event::DeathCause::Starvation,
                );
            }
            if life.water == 0 {
                life.damage_from(
                    biology::DEHYDRATION_HEALTH_LOSS,
                    crate::simulation::event::DeathCause::Dehydration,
                );
            }
            if life.is_alive()
                && life.energy >= life.max_energy.saturating_mul(3) / 4
                && !life.is_water_low()
                && life.health < life.max_health
            {
                life.consume_energy(biology::PASSIVE_HEAL_ENERGY_COST);
                life.heal(biology::PASSIVE_HEAL_AMOUNT);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::herbivore::HerbivoreEntity;
    use crate::entity::{Entity, controller::EntityController, phase::EntityPhase};
    use crate::map::movement::Position;

    #[test]
    fn test_aging_and_death() {
        let mut entity = HerbivoreEntity::default();
        entity.life_mut().age = 999;
        entity.life_mut().max_age = 1000;

        let slot = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(entity),
        );
        let mut slots = [slot];

        MetabolismSystem::update(&mut slots);
        assert!(slots[0].base.life().age > 999);

        // Bir sonraki tick'te ölecek mi?
        MetabolismSystem::update(&mut slots);
        assert!(!slots[0].base.life().is_alive());
    }

    #[test]
    fn test_starvation_logic() {
        let mut entity = HerbivoreEntity::default();
        entity.life_mut().energy = 0;
        entity.life_mut().health = 100;

        let slot = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(entity),
        );
        let mut slots = [slot];

        MetabolismSystem::update(&mut slots);

        // Enerji 0 olduğu için candan yemeli ve bir miktar "starvation energy" gelmeli
        assert!(slots[0].base.life().health < 100);
        assert_eq!(slots[0].base.life().energy, 0);
    }

    #[test]
    fn test_passive_healing() {
        let mut entity = HerbivoreEntity::default();
        entity.life_mut().health = 50;
        entity.life_mut().energy = 100; // Enerji var, iyileşmeli

        let slot = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(entity),
        );
        let mut slots = [slot];

        MetabolismSystem::update(&mut slots);
        assert!(slots[0].base.life().health > 50);
        assert!(slots[0].base.life().energy < 100);
    }
}
