use crate::{
    world::{EntitySlot, SleepPlan},
    map::Map,
    entity::phase::EntityPhase,
    rules::biology,
    simulation::SimulationEvent,
};

/// Canlıların yaşam döngüsü (yaşlanma, uyku, ölüm) aşamalarını yöneten sistem.
pub struct LifeCycleSystem;

impl LifeCycleSystem {
    pub fn finalize_tick(
        entities: &mut [EntitySlot],
        map: &mut Map,
        events: &mut Vec<SimulationEvent>,
    ) {
        // Authoritative Metabolism: Her şey merkezi kurallara göre işler
        crate::systems::metabolism::MetabolismSystem::update(entities);

        for slot in entities {
            slot.phase.tick();

            // Ölüm Kontrolü
            if slot.phase == EntityPhase::Active && !slot.entity().life().is_alive() {
                let species = slot.entity().species();
                slot.phase = EntityPhase::Corpse { remaining: biology::CORPSE_DURATION_TICKS };
                let amount = (slot.entity().life().max_health / biology::CORPSE_FOOD_DIVISOR).max(biology::MIN_FOOD_FROM_CORPSE);
                map.add_food(slot.pos, amount);
                
                events.push(SimulationEvent::EntityDied { id: slot.id, pos: slot.pos, species });
            }
        }
    }

    pub fn resolve_sleep(
        entities: &mut [EntitySlot],
        plans: &[SleepPlan],
        events: &mut Vec<SimulationEvent>,
    ) {
        for plan in plans {
            if let Some(slot) = entities.iter_mut().find(|s| s.id == plan.id) {
                if slot.phase.is_active() {
                    slot.phase = EntityPhase::Sleeping { remaining: plan.duration };
                    events.push(SimulationEvent::EntitySlept { id: plan.id, duration: plan.duration });
                }
            }
        }
    }
}
