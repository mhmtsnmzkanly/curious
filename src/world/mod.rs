use crate::{
    entity::{phase::EntityPhase, intent::Intent},
    map::{Map, movement::{Position, DIRECTION_ARRAY}},
    logger::{Logger, LogLevel},
};
use std::collections::HashMap;

pub mod entity_slot;
pub use entity_slot::EntitySlot;

/// Simülasyon dünyasını yöneten ana yapı.
/// Bu yapı, simülasyonun merkezi (authoritative simulation core) olarak görev yapar.
pub struct World {
    /// Simülasyon haritası ve kaynaklar
    pub map: Map,

    /// Tüm canlıların (EntitySlot) listesi
    pub entities: Vec<EntitySlot>,

    /// Simülasyon tur sayacı (kaç tick geçtiği)
    pub tick_counter: usize,

    /// Loglama mekanizması
    pub logger: Logger,

    /// Dışarıdan gelen (Player/Network) komutların biriktirildiği kuyruk.
    pub command_inputs: HashMap<usize, Intent>,
}

impl World {
    /// Yeni bir simülasyon dünyası oluşturur.
    pub fn new(x1: isize, x2: isize, y1: isize, y2: isize, entities: Vec<EntitySlot>) -> World {
        let mut map = Map::new(x1, x2, y1, y2);
        map.populate_resources(0.05f32);
        
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
            command_inputs: HashMap::new(),
        }
    }

    /// Dışarıdan bir birim için komut ekler.
    pub fn push_command(&mut self, entity_id: usize, intent: Intent) {
        self.command_inputs.insert(entity_id, intent);
    }

    /// Bir simülasyon adımı (Tick) gerçekleştirir ve oluşan olayları döner.
    pub fn tick(&mut self) -> Vec<crate::simulation::SimulationEvent> {
        self.tick_counter += 1;

        // 1. Temizlik: Silinmiş varlıkları tamamen kaldır
        self.entities.retain(|slot| !matches!(slot.phase, EntityPhase::Removed));

        let mut events = Vec::new();
        let mut log_lines: Vec<String> = Vec::new();
        log_lines.push(format!("=== Tick {} ===", self.tick_counter));

        // 2. Hazırlık: Çakışma kontrolü için meşgul olan hücrelerin tespiti
        let mut occupied: HashMap<Position, usize> = self
            .entities
            .iter()
            .filter(|slot| !matches!(slot.phase, EntityPhase::Corpse { .. } | EntityPhase::Removed))
            .map(|slot| (slot.pos, slot.id))
            .collect();

        // 3. Niyetleri Topla (Intent Collection System)
        let intents = crate::systems::intent_collection::IntentCollectionSystem::collect(self, &mut log_lines);

        // 4. Planları Oluştur (Planning System)
        let plans = crate::systems::planning::PlanningSystem::plan(self, intents, &mut log_lines);

        // 5. Çözümleme Aşamaları (Resolvers) - Sistemler üzerinden işletilir (Events toplanır)
        crate::systems::movement::MovementSystem::resolve_moves(&mut self.entities, &plans.moves, &mut occupied, &mut events);
        crate::systems::interaction::InteractionSystem::resolve_eats(&mut self.entities, &mut self.map, &plans.eats, &mut occupied, &mut events);
        crate::systems::interaction::InteractionSystem::resolve_drinks(&mut self.entities, &mut self.map, &plans.drinks, &mut occupied, &mut events);
        crate::systems::interaction::InteractionSystem::resolve_mates(&mut self.entities, &self.map, &plans.mates, &mut occupied, &mut events);
        crate::systems::interaction::InteractionSystem::resolve_attacks(&mut self.entities, &plans.attacks, &mut events);
        crate::systems::movement::MovementSystem::resolve_flee(&mut self.entities, &plans.flees, &mut occupied, &mut events);
        crate::systems::life_cycle::LifeCycleSystem::resolve_sleep(&mut self.entities, &plans.sleeps, &mut events);

        // 6. Birimlerin iç durumlarının ve ölümlerinin güncellenmesi
        crate::systems::life_cycle::LifeCycleSystem::finalize_tick(&mut self.entities, &mut self.map, &mut events);

        // 8. Logları yaz ve komut kuyruğunu temizle
        for event in &events {
            log_lines.push(format!("[Event] {:?}", event));
        }
        self.logger.log_many(LogLevel::Info, &log_lines);
        self.command_inputs.clear();

        events
    }

    /// Statik yardımcı metod.
    pub fn pick_empty_neighbor(map: &Map, pos: Position, occupied: &HashMap<Position, usize>) -> Option<Position> {
        DIRECTION_ARRAY.iter()
            .map(|&d| pos + d)
            .find(|&p| map.is_walkable(p) && !occupied.contains_key(&p))
    }
}

/// Planlanan eylemleri barındıran yardımcı yapılar
#[derive(Default)]
pub struct ActionPlans {
    pub moves: Vec<MovePlan>,
    pub eats: Vec<EatPlan>,
    pub drinks: Vec<DrinkPlan>,
    pub mates: Vec<MatePlan>,
    pub attacks: Vec<AttackPlan>,
    pub flees: Vec<FleePlan>,
    pub sleeps: Vec<SleepPlan>,
}

#[derive(Clone, Copy)]
pub struct MovePlan { pub id: usize, pub new_pos: Position, pub cost: usize }
#[derive(Clone, Copy)]
pub struct EatPlan { pub id: usize, pub new_pos: Position, pub cost: usize }
#[derive(Clone, Copy)]
pub struct DrinkPlan { pub id: usize, pub new_pos: Position, pub cost: usize }
#[derive(Clone, Copy)]
pub struct MatePlan { pub self_id: usize, pub target_id: usize }
#[derive(Clone, Copy)]
pub struct AttackPlan { pub attacker_id: usize, pub target_id: usize }
#[derive(Clone, Copy)]
pub struct FleePlan { pub id: usize, pub new_pos: Position, pub cost: usize }
#[derive(Clone, Copy)]
pub struct SleepPlan { pub id: usize, pub duration: usize }
