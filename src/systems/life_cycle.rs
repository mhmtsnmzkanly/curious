use crate::{
    entity::phase::EntityPhase,
    map::Map,
    rules::biology,
    simulation::SimulationEvent,
    world::{EntitySlot, SleepPlan},
};

/// Canlıların yaşam döngüsü (yaşlanma, uyku, ölüm) aşamalarını yöneten sistem.
pub struct LifeCycleSystem;

impl LifeCycleSystem {
    pub fn begin_tick(
        entities: &mut [EntitySlot],
        map: &mut Map,
        events: &mut Vec<SimulationEvent>,
    ) {
        for slot in entities.iter_mut() {
            slot.phase.tick();
            if matches!(slot.phase, EntityPhase::Corpse { remaining } if remaining > 0 && remaining < 4)
                && slot.corpse_energy > biology::MIN_FOOD_FROM_CORPSE
            {
                slot.corpse_energy = slot
                    .corpse_energy
                    .saturating_sub(1)
                    .max(biology::MIN_FOOD_FROM_CORPSE);
            }
        }
        crate::systems::metabolism::MetabolismSystem::update(entities, events);
        Self::finalize_tick(entities, map, events);
    }

    pub fn finalize_tick(
        entities: &mut [EntitySlot],
        _map: &mut Map,
        events: &mut Vec<SimulationEvent>,
    ) {
        for slot in entities {
            // Ölüm Kontrolü
            if matches!(
                slot.phase,
                EntityPhase::Active | EntityPhase::Sleeping { .. }
            ) && !slot.entity().life().is_alive()
            {
                let species = slot.entity().species();
                slot.phase = EntityPhase::Corpse {
                    remaining: biology::CORPSE_DURATION_TICKS,
                };
                let amount = (slot.entity().life().max_health / biology::CORPSE_FOOD_DIVISOR)
                    .max(biology::MIN_FOOD_FROM_CORPSE);
                slot.corpse_energy = amount;

                events.push(SimulationEvent::EntityDied {
                    id: slot.id,
                    pos: slot.pos,
                    species,
                    cause: Some(
                        slot.base
                            .life()
                            .death_cause
                            .unwrap_or(crate::simulation::event::DeathCause::Unknown),
                    ),
                });
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
                    slot.phase = EntityPhase::Sleeping {
                        remaining: plan.duration,
                    };
                    events.push(SimulationEvent::EntitySlept {
                        id: plan.id,
                        duration: plan.duration,
                    });
                }
            }
        }
    }

    /// Helper to inspect corpse decay stage for a given entity slot.
    pub fn corpse_stage(slot: &EntitySlot) -> Option<crate::entity::phase::CorpseStage> {
        slot.phase.corpse_stage()
    }
}
