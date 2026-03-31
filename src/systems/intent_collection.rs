use crate::{
    world::World,
    entity::{intent::Intent, controller::EntityController},
    systems::perception::PerceptionSystem,
};

/// Varlıkların niyetlerini (AI veya Dış Komut) toplayan sistem.
pub struct IntentCollectionSystem;

impl IntentCollectionSystem {
    /// Mevcut tüm aktif varlıklardan niyetlerini toplar.
    pub fn collect(world: &World, log_lines: &mut Vec<String>) -> Vec<(usize, Intent)> {
        let mut intents: Vec<(usize, Intent)> = Vec::new();

        for slot in &world.entities {
            if !slot.phase.is_active() {
                continue;
            }

            // 1. Algı Verisini Oluştur
            let perception = PerceptionSystem::build(world, slot);
            
            // 2. Niyet Belirle (Öncelik: Dış Komut > İçsel AI)
            let intent = if let Some(external_intent) = world.command_inputs.get(&slot.id) {
                log_lines.push(format!("[Input] @{} Dış Komut: {:?}", slot.id, external_intent));
                external_intent.clone()
            } else {
                match slot.controller {
                    EntityController::AI => slot.entity().make_intent(perception),
                    EntityController::Player(_) => {
                        // Oyuncuya ait bir birime komut gelmemişse beklemede kalsın.
                        Intent::Idle { duration: 1 }
                    }
                }
            };
            
            // 3. Loglama (Seyrek loglama stratejisi)
            let should_log = !matches!(intent, Intent::Idle { .. }) || (world.tick_counter % 5 == 0);
            if should_log {
                log_lines.push(format!(
                    "[Niyet] @{} {:?} @{:?} => {:?}",
                    slot.id, slot.base.species(), slot.pos, intent
                ));
            }
            
            intents.push((slot.id, intent));
        }
        intents
    }
}
