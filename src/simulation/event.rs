use crate::map::movement::Position;
use crate::entity::species::Species;

/// Simülasyonda gerçekleşen bir olayı temsil eder.
/// Bu olaylar ağ üzerinden istemcilere (clients) gönderilebilir.
#[derive(Debug, Clone)]
pub enum SimulationEvent {
    EntityMoved { id: usize, from: Position, to: Position },
    EntityAte { id: usize, pos: Position, amount: usize },
    EntityDrank { id: usize, pos: Position, amount: usize },
    EntityAttacked { attacker: usize, target: usize, damage: usize },
    EntityMated { parent1: usize, parent2: usize, child: usize, pos: Position },
    EntityDied { id: usize, pos: Position, species: Species },
    EntityBorn { id: usize, pos: Position, species: Species },
    EntitySlept { id: usize, duration: usize },
}
