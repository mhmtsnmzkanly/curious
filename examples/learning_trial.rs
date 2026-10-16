//! Episodic water-choice laboratory. Explicitly resets bodies and water each trial.
//! Both variants use the same resolver and competing drinker; this is not a live ecology.
//! Reversal moves the competitor to the formerly preferred source after 400 trials.
use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{controller::EntityController, intent::Intent, phase::EntityPhase},
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    world::{EntitySlot, World},
};
fn run(seed: u64, enabled: bool) -> serde_json::Value {
    experiment(seed, enabled, false)
}
fn experiment(seed: u64, enabled: bool, reverse: bool) -> serde_json::Value {
    let actors = vec![
        EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            Position::new(2, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(HerbivoreEntity::default()),
        ),
    ];
    let mut w = World::with_seed(0, 4, 0, 4, actors, seed);
    w.logger = curious::logger::Logger::disabled();
    w.environment = Default::default();
    w.entities[0].learner.enabled = enabled;
    let mut received = [0usize; 4];
    let mut attempts = [0usize; 4];
    for trial in 0..if reverse { 800 } else { 400 } {
        w.map = Map::new(0, 4, 0, 4);
        w.map
            .set_cell(Position::new(1, 0), Cell::Water { amount: 10 });
        w.map
            .set_cell(Position::new(0, 3), Cell::Water { amount: 10 });
        w.entities[0].pos = Position::new(0, 0);
        let changed = reverse && trial >= 400;
        w.entities[1].pos = if changed {
            Position::new(0, 4)
        } else {
            Position::new(2, 0)
        };
        for s in &mut w.entities {
            let l = s.base.life_mut();
            l.age = 0;
            l.energy = l.max_energy;
            l.water = 10;
            l.health = l.max_health;
            l.stamina = l.max_stamina;
        }
        w.push_command(
            2,
            Intent::Drink {
                at: Steps(vec![if changed {
                    Direction::Up
                } else {
                    Direction::Left
                }]),
            },
        );
        let events = w.tick();
        let phase = match trial {
            0..=99 => 0,
            300..=399 => 1,
            400..=499 => 2,
            700..=799 => 3,
            _ => continue,
        };
        attempts[phase] += 1;
        for e in events {
            if let curious::simulation::SimulationEvent::EntityDrank { id: 1, amount, .. } = e {
                received[phase] += amount;
            }
        }
    }
    serde_json::json!({"seed":seed,"learning":enabled,"first_100_water":received[0],"last_100_water":received[1],"environment_reversal":reverse,"after_change_first_100_water":received[2],"after_change_last_100_water":received[3],"choices":attempts,"values":w.entities[0].learner.values})
}
fn main() {
    for seed in [1, 2, 3, 4, 5, 42, 12345, 2026] {
        for enabled in [false, true] {
            println!("{}", run(seed, enabled));
            println!("{}", experiment(seed, enabled, true));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn experience_improves_real_drinking_under_competition() {
        for seed in [1, 2, 3, 42] {
            let static_policy = run(seed, false);
            let learned = run(seed, true);
            assert_eq!(static_policy["last_100_water"], 500);
            assert!(learned["last_100_water"].as_u64().unwrap() > 900);
        }
    }
    #[test]
    fn changed_competition_does_not_lock_in_the_old_preference() {
        for seed in [1, 2, 3, 4, 5, 42, 12345, 2026] {
            let fixed = experiment(seed, false, true);
            let learned = experiment(seed, true, true);
            assert_eq!(fixed["last_100_water"], 500);
            assert_eq!(fixed["after_change_last_100_water"], 1000);
            assert!(learned["last_100_water"].as_u64().unwrap() > 900);
            assert!(learned["after_change_last_100_water"].as_u64().unwrap() >= 900);
        }
    }
}
