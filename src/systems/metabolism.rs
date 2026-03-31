use crate::{
    world::EntitySlot,
    rules::biology,
};

/// Canlıların içsel durumlarını (enerji tüketimi, yaşlanma, iyileşme) yöneten sistem.
pub struct MetabolismSystem;

impl MetabolismSystem {
    pub fn update(entities: &mut [EntitySlot]) {
        for slot in entities {
            if !slot.phase.is_active() {
                continue;
            }

            let life = slot.base.life_mut();

            // 1. Yaşlanma
            life.age += biology::AGING_RATE;

            // Yaşlılıktan ölüm kontrolü
            if life.age > life.max_age {
                life.health = 0;
                continue;
            }

            // 2. Üreme bekleme süresi azalması
            if life.reproduction_cooldown > 0 {
                life.reproduction_cooldown -= 1;
            }

            // 3. Pasif İyileşme (Enerji varsa)
            if !life.is_energy_low() && life.health < life.max_health {
                life.consume_energy(biology::PASSIVE_HEAL_ENERGY_COST);
                life.heal(biology::PASSIVE_HEAL_AMOUNT);
            }

            // 4. Enerji Bittiğinde Candan Harcama (Starvation)
            if life.energy == 0 && !life.is_health_low() {
                life.health = life.health.saturating_sub(biology::STARVATION_HEALTH_LOSS);
                life.restore_energy(biology::STARVATION_ENERGY_RESTORE);
            }

            // 5. Su Bittiğinde Candan Harcama (Dehydration)
            if life.water == 0 && !life.is_health_low() {
                life.health = life.health.saturating_sub(biology::DEHYDRATION_HEALTH_LOSS);
            }

            // 6. Bazal Tüketim (BMR)
            life.consume_energy(biology::BMR_ENERGY_COST);
            life.consume_water(biology::BMR_WATER_COST);

            // 7. Tick sonu hazırlıkları
            life.moves_used = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::herbivore::HerbivoreEntity;
    use crate::entity::{phase::EntityPhase, controller::EntityController};
    use crate::map::movement::Position;

    #[test]
    fn test_aging_and_death() {
        let mut entity = HerbivoreEntity::default();
        entity.life_mut().age = 999; 
        entity.life_mut().max_age = 1000;
        
        let mut slot = EntitySlot::new(1, Position::default(), EntityPhase::Active, EntityController::AI, Box::new(entity));
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
        
        let mut slot = EntitySlot::new(1, Position::default(), EntityPhase::Active, EntityController::AI, Box::new(entity));
        let mut slots = [slot];

        MetabolismSystem::update(&mut slots);
        
        // Enerji 0 olduğu için candan yemeli ve bir miktar "starvation energy" gelmeli
        assert!(slots[0].base.life().health < 100);
        assert!(slots[0].base.life().energy > 0);
    }

    #[test]
    fn test_passive_healing() {
        let mut entity = HerbivoreEntity::default();
        entity.life_mut().health = 50;
        entity.life_mut().energy = 100; // Enerji var, iyileşmeli
        
        let mut slot = EntitySlot::new(1, Position::default(), EntityPhase::Active, EntityController::AI, Box::new(entity));
        let mut slots = [slot];

        MetabolismSystem::update(&mut slots);
        assert!(slots[0].base.life().health > 50);
        assert!(slots[0].base.life().energy < 100);
    }
}
