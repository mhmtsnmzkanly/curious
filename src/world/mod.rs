use crate::{
    entity::{intent::Intent, phase::EntityPhase},
    logger::{LogLevel, Logger},
    map::{
        Map,
        movement::{DIRECTION_ARRAY, Position},
    },
};
use std::collections::HashMap;

#[derive(Clone)]
struct PreparedTick {
    events: Vec<crate::simulation::SimulationEvent>,
    log_lines: Vec<String>,
    occupied: HashMap<Position, usize>,
    plans: ActionPlans,
}

pub mod entity_slot;
pub use entity_slot::EntitySlot;

#[cfg(feature = "resolver-lab")]
mod resolver_lab;
#[cfg(feature = "resolver-lab")]
pub use resolver_lab::ResolverOrderLabCase;

/// Simülasyon dünyasını yöneten ana yapı.
/// Bu yapı, simülasyonun merkezi (authoritative simulation core) olarak görev yapar.
pub struct World {
    pub policy: crate::policy::PolicySettings,
    pub genetics: crate::entity::genetics::Genetics,
    pub observation: crate::entity::decision::ObservationSettings,
    pub last_decisions: Vec<crate::entity::decision::DecisionTrace>,
    pub environment: crate::environment::Environment,
    pub rng: crate::random::Random,
    pub last_trace: Vec<String>,
    pub last_intents: Vec<(usize, Intent)>,
    pub next_entity_id: usize,
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
        Self::with_seed(x1, x2, y1, y2, entities, crate::rng_state())
    }

    pub fn with_seed(
        x1: isize,
        x2: isize,
        y1: isize,
        y2: isize,
        entities: Vec<EntitySlot>,
        seed: u64,
    ) -> World {
        let mut rng = crate::random::Random::new(seed);
        let mut map = Map::new(x1, x2, y1, y2);
        map.populate_resources(0.05f32, &mut rng);

        let next_entity_id = entities.iter().map(|s| s.id).max().unwrap_or(0) + 1;
        let environment = crate::environment::Environment::from_map(&map);
        World {
            policy: Default::default(),
            genetics: Default::default(),
            observation: Default::default(),
            last_decisions: Vec::new(),
            environment,
            rng,
            last_trace: Vec::new(),
            last_intents: Vec::new(),
            next_entity_id,
            map,
            entities,
            tick_counter: 0,
            logger: Logger::disabled(),
            command_inputs: HashMap::new(),
        }
    }

    /// Enables a timestamped file logger writing to `logs/simulation_<timestamp>.log`.
    pub fn enable_timestamped_file_logger(&mut self) {
        self.logger = Logger::new_timestamped("logs");
    }

    /// Dışarıdan bir birim için komut ekler.
    pub fn push_command(&mut self, entity_id: usize, intent: Intent) {
        self.command_inputs.insert(entity_id, intent);
    }

    /// Digs or refreshes a dynamic burrow on the world map and environment.
    pub fn dig_burrow(&mut self, pos: Position, ttl: usize) -> bool {
        self.environment
            .dig_burrow_on_map(&mut self.map, pos, self.tick_counter, ttl)
    }

    /// Bir simülasyon adımı (Tick) gerçekleştirir ve oluşan olayları döner.
    pub fn tick(&mut self) -> Vec<crate::simulation::SimulationEvent> {
        let mut prepared = self.prepare_tick(None);
        let mut plans = std::mem::take(&mut prepared.plans);
        plans.rotate_priority(self.tick_counter);
        self.resolve_prepared_tick(prepared, plans)
    }

    fn prepare_tick(
        &mut self,
        mut lab_perceptions: Option<&mut Vec<(usize, crate::entity::perception::Perception)>>,
    ) -> PreparedTick {
        self.tick_counter += 1;

        // 1. Temizlik: Silinmiş varlıkları tamamen kaldır
        self.entities
            .retain(|slot| !matches!(slot.phase, EntityPhase::Removed));

        let mut events = Vec::new();
        let mut log_lines: Vec<String> = Vec::new();
        log_lines.push(format!("=== Tick {} ===", self.tick_counter));

        self.environment
            .update(&mut self.map, self.tick_counter, &mut events);
        crate::systems::life_cycle::LifeCycleSystem::begin_tick(
            &mut self.entities,
            &mut self.map,
            &mut events,
        );
        if self.environment.season_cycle.is_some() {
            for slot in &self.entities {
                if slot.phase.is_active()
                    && self.map.cell(slot.pos) == Some(&crate::map::cell::Cell::Burrow)
                    && !self.environment.has_active_burrow(slot.pos, self.tick_counter)
                {
                    self.environment.dig_burrow(slot.pos, self.tick_counter, 1_000_000);
                }
            }
            crate::systems::metabolism::MetabolismSystem::apply_seasonal_thermal_costs(
                &mut self.entities,
                &self.environment,
                self.tick_counter,
            );
            crate::systems::life_cycle::LifeCycleSystem::finalize_tick(
                &mut self.entities,
                &mut self.map,
                &mut events,
            );
            if self
                .environment
                .season_cycle
                .unwrap()
                .current_season(self.tick_counter)
                == crate::environment::Season::Winter
            {
                for slot in self.entities.iter_mut() {
                    if slot.phase.is_active()
                        && slot.base.can_burrow()
                        && self.map.cell(slot.pos) == Some(&crate::map::cell::Cell::Empty)
                    {
                        let life = slot.base.life();
                        if life.energy > life.max_energy / 2 && life.stamina >= 4 {
                            let pos = slot.pos;
                            let has_nearby_burrow = (-1..=1).any(|dx| {
                                (-1..=1).any(|dy| {
                                    self.map.cell(Position::new(pos.x + dx, pos.y + dy))
                                        == Some(&crate::map::cell::Cell::Burrow)
                                })
                            });
                            if !has_nearby_burrow {
                                if self.environment.dig_burrow_on_map(
                                    &mut self.map,
                                    pos,
                                    self.tick_counter,
                                    20,
                                ) {
                                    slot.base.life_mut().stamina =
                                        slot.base.life().stamina.saturating_sub(2);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Hazırlık: Çakışma kontrolü için meşgul olan hücrelerin tespiti
        let mut occupied: HashMap<Position, usize> = self
            .entities
            .iter()
            .filter(|slot| {
                !matches!(
                    slot.phase,
                    EntityPhase::Corpse { .. } | EntityPhase::Removed
                )
            })
            .map(|slot| (slot.pos, slot.id))
            .collect();

        // 3. Niyetleri Topla (Intent Collection System)
        let intents = if let Some(perceptions) = lab_perceptions.as_mut() {
            crate::systems::intent_collection::IntentCollectionSystem::collect_with_perceptions(
                self,
                &mut log_lines,
                perceptions,
            )
        } else {
            crate::systems::intent_collection::IntentCollectionSystem::collect(self, &mut log_lines)
        };

        self.last_intents = intents.clone();

        if self.policy.local_coordination_enabled {
            self.environment.signals_enabled = true;
            let speaker_ids: Vec<usize> = self
                .entities
                .iter()
                .filter(|s| {
                    s.phase.is_active()
                        && s.base.life().is_alive()
                        && (self.last_decisions.iter().any(|d| {
                            d.id == s.id
                                && matches!(
                                    d.decision.reason,
                                    crate::entity::decision::Reason::EscapeThreat
                                        | crate::entity::decision::Reason::SeekShelter
                                        | crate::entity::decision::Reason::CorneredDefense
                                )
                        }) || self.entities.iter().any(|other| {
                            other.id != s.id
                                && other.base.life().is_alive()
                                && matches!(
                                    other.phase,
                                    EntityPhase::Active | EntityPhase::Sleeping { .. }
                                )
                                && other.base.species().hunts(&s.base.species())
                                && s.pos.distance_to(other.pos) <= s.base.life().vision_range
                        }))
                })
                .map(|s| s.id)
                .collect();

            for sid in speaker_ids {
                crate::systems::interaction::InteractionSystem::emit_local_alarm(
                    &mut self.entities,
                    &mut self.environment,
                    sid,
                    self.tick_counter,
                );
            }
        }

        // 4. Planları Oluştur (Planning System)
        let plans = crate::systems::planning::PlanningSystem::plan(self, intents, &mut log_lines);

        PreparedTick {
            events,
            log_lines,
            occupied,
            plans,
        }
    }

    fn resolve_prepared_tick(
        &mut self,
        prepared: PreparedTick,
        plans: ActionPlans,
    ) -> Vec<crate::simulation::SimulationEvent> {
        let PreparedTick {
            mut events,
            mut log_lines,
            mut occupied,
            ..
        } = prepared;

        // 5. Çözümleme Aşamaları (Resolvers) - Sistemler üzerinden işletilir (Events toplanır)
        crate::systems::movement::MovementSystem::resolve_moves(
            &mut self.entities,
            &plans.moves,
            &self.map,
            &mut occupied,
            &mut events,
            self.policy.injury_capacity_penalty,
        );
        crate::systems::interaction::InteractionSystem::resolve_eats(
            &mut self.entities,
            &mut self.map,
            &plans.eats,
            &mut occupied,
            &mut events,
            self.policy.injury_capacity_penalty,
        );
        crate::systems::interaction::InteractionSystem::resolve_drinks(
            &mut self.entities,
            &mut self.map,
            &plans.drinks,
            &mut occupied,
            &mut events,
            self.policy.injury_capacity_penalty,
        );
        crate::systems::interaction::InteractionSystem::resolve_mates(
            &mut self.entities,
            &self.map,
            &plans.mates,
            &mut occupied,
            &mut events,
            &mut self.next_entity_id,
            &mut self.rng,
            &self.genetics,
        );
        crate::systems::interaction::InteractionSystem::resolve_attacks_with_policy(
            &mut self.entities,
            &plans.attacks,
            &self.map,
            &mut events,
            &self.policy,
        );
        crate::systems::movement::MovementSystem::resolve_flee(
            &mut self.entities,
            &plans.flees,
            &self.map,
            &mut occupied,
            &mut events,
            self.policy.injury_capacity_penalty,
        );
        crate::systems::life_cycle::LifeCycleSystem::resolve_sleep(
            &mut self.entities,
            &plans.sleeps,
            &mut events,
        );
        if self.policy.juvenile_care_enabled {
            crate::systems::interaction::InteractionSystem::resolve_parental_care(
                &mut self.entities,
                &mut events,
            );
        }

        // 6. Birimlerin iç durumlarının ve ölümlerinin güncellenmesi
        crate::systems::life_cycle::LifeCycleSystem::finalize_tick(
            &mut self.entities,
            &mut self.map,
            &mut events,
        );

        crate::systems::water_outcomes::finish(
            &mut self.entities,
            &self.last_decisions,
            &self.last_intents,
            &events,
        );
        crate::systems::hunt_outcomes::finish(&mut self.entities, &events);
        let refuge_events = crate::systems::refuge_outcomes::finish(
            &mut self.entities,
            &self.map,
            &events,
            self.tick_counter,
        );
        events.extend(refuge_events);
        for slot in &mut self.entities {
            for result in slot.hunt_learner.take_results() {
                events.push(crate::simulation::SimulationEvent::HuntEpisodeEnded {
                    id: slot.id,
                    result,
                });
            }
        }
        for trace in &mut self.last_decisions {
            if let Some(slot) = self.entities.iter().find(|s| s.id == trace.id) {
                trace.outcome = Some(crate::entity::decision::Outcome {
                    pos: slot.pos,
                    life: slot.base.life().clone(),
                    phase: slot.phase,
                    events: events
                        .iter()
                        .filter(|e| e.involves(trace.id))
                        .cloned()
                        .collect(),
                });
            }
        }

        // 8. Logları yaz ve komut kuyruğunu temizle
        for event in &events {
            log_lines.push(format!("[Event] {:?}", event));
        }
        self.logger.log_many(LogLevel::Info, &log_lines);
        self.command_inputs.clear();
        self.last_trace = log_lines;

        events
    }

    /// Statik yardımcı metod.
    pub fn pick_empty_neighbor(
        map: &Map,
        pos: Position,
        occupied: &HashMap<Position, usize>,
    ) -> Option<Position> {
        Self::pick_empty_neighbor_for(map, pos, occupied, true)
    }
    pub fn pick_empty_neighbor_for(
        map: &Map,
        pos: Position,
        occupied: &HashMap<Position, usize>,
        can_burrow: bool,
    ) -> Option<Position> {
        DIRECTION_ARRAY
            .iter()
            .map(|&d| pos + d)
            .find(|&p| map.is_walkable_for(p, can_burrow) && !occupied.contains_key(&p))
    }
}

/// Planlanan eylemleri barındıran yardımcı yapılar
#[derive(Clone, Default)]
pub struct ActionPlans {
    pub moves: Vec<MovePlan>,
    pub eats: Vec<EatPlan>,
    pub drinks: Vec<DrinkPlan>,
    pub mates: Vec<MatePlan>,
    pub attacks: Vec<AttackPlan>,
    pub flees: Vec<FleePlan>,
    pub sleeps: Vec<SleepPlan>,
}

#[derive(Clone)]
pub struct MovePlan {
    pub path: Vec<Position>,
    pub id: usize,
    pub new_pos: Position,
    pub cost: usize,
}
#[derive(Clone)]
pub struct EatPlan {
    pub path: Vec<Position>,
    pub id: usize,
    pub new_pos: Position,
    pub cost: usize,
    pub resource_pos: Position,
    pub corpse_id: Option<usize>,
}
#[derive(Clone)]
pub struct DrinkPlan {
    pub path: Vec<Position>,
    pub id: usize,
    pub new_pos: Position,
    pub cost: usize,
    pub resource_pos: Position,
}
#[derive(Clone, Copy)]
pub struct MatePlan {
    pub self_id: usize,
    pub target_id: usize,
}
#[derive(Clone, Copy)]
pub struct AttackPlan {
    pub attacker_id: usize,
    pub target_id: usize,
}
#[derive(Clone)]
pub struct FleePlan {
    pub path: Vec<Position>,
    pub id: usize,
    pub new_pos: Position,
    pub cost: usize,
}
#[derive(Clone, Copy)]
pub struct SleepPlan {
    pub id: usize,
    pub duration: usize,
}

impl ActionPlans {
    /// Deterministic rotating priority avoids permanently favouring founder IDs.
    pub fn rotate_priority(&mut self, tick: usize) {
        fn rotate<T>(items: &mut [T], tick: usize) {
            if !items.is_empty() {
                let n = tick % items.len();
                items.rotate_left(n);
            }
        }
        rotate(&mut self.moves, tick);
        rotate(&mut self.eats, tick);
        rotate(&mut self.drinks, tick);
        rotate(&mut self.mates, tick);
        rotate(&mut self.attacks, tick);
        rotate(&mut self.flees, tick);
        rotate(&mut self.sleeps, tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_with_seed_initializes_disabled_logger_without_leaking_log_files() {
        let logs_path = std::path::Path::new("logs");
        let initial_count = if logs_path.exists() {
            std::fs::read_dir(logs_path).unwrap().count()
        } else {
            0
        };

        let mut world = World::with_seed(-5, 5, -5, 5, Vec::new(), 42);
        assert!(world.logger.is_disabled());
        assert!(!world.logger.is_file_enabled());

        let _ = world.tick();
        assert!(world.logger.is_disabled());

        let after_count = if logs_path.exists() {
            std::fs::read_dir(logs_path).unwrap().count()
        } else {
            0
        };

        assert_eq!(initial_count, after_count, "World::with_seed must not leak files to logs/");
    }

    #[test]
    fn world_new_initializes_disabled_logger_without_leaking_log_files() {
        let logs_path = std::path::Path::new("logs");
        let initial_count = if logs_path.exists() {
            std::fs::read_dir(logs_path).unwrap().count()
        } else {
            0
        };

        let mut world = World::new(-4, 4, -4, 4, Vec::new());
        assert!(world.logger.is_disabled());
        assert!(!world.logger.is_file_enabled());

        let _ = world.tick();

        let after_count = if logs_path.exists() {
            std::fs::read_dir(logs_path).unwrap().count()
        } else {
            0
        };

        assert_eq!(initial_count, after_count, "World::new must not leak files to logs/");
    }

    #[test]
    fn enable_timestamped_file_logger_creates_log_file() {
        let mut world = World::with_seed(-5, 5, -5, 5, Vec::new(), 42);
        assert!(world.logger.is_disabled());

        let temp_dir = tempfile::tempdir().expect("tempdir");
        world.logger = Logger::new_timestamped(temp_dir.path().to_str().unwrap());
        assert!(world.logger.is_file_enabled());
        assert!(!world.logger.is_disabled());

        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .expect("read_dir")
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1);
        let name = entries[0].file_name().to_string_lossy().into_owned();
        assert!(name.starts_with("simulation_"));
        assert!(name.ends_with(".log"));
    }
}
