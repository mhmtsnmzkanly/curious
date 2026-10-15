use crate::entity::species::Species;
use crate::map::movement::Position;

/// Simülasyonda gerçekleşen bir olayı temsil eder.
/// Bu olaylar ağ üzerinden istemcilere (clients) gönderilebilir.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum SimulationEvent {
    ResourceReplenished {
        pos: Position,
        kind: crate::environment::ResourceKind,
        amount: usize,
    },
    ActionRejected {
        id: usize,
        action: String,
        target: Option<usize>,
        reason: RejectionReason,
    },
    CorpseConsumed {
        id: usize,
        eater: usize,
        pos: Position,
    },
    EntityMoved {
        id: usize,
        from: Position,
        to: Position,
    },
    EntityAte {
        id: usize,
        pos: Position,
        amount: usize,
    },
    EntityDrank {
        id: usize,
        pos: Position,
        amount: usize,
    },
    EntityAttacked {
        attacker: usize,
        target: usize,
        damage: usize,
    },
    EntityMated {
        parent1: usize,
        parent2: usize,
        child: usize,
        pos: Position,
    },
    EntityDied {
        id: usize,
        pos: Position,
        species: Species,
        #[serde(default)]
        cause: Option<DeathCause>,
    },
    EntityBorn {
        id: usize,
        pos: Position,
        species: Species,
    },
    EntitySlept {
        id: usize,
        duration: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeathCause {
    OldAge,
    Starvation,
    Dehydration,
    Combat,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RejectionReason {
    MissingEntity,
    Fatigued,
    NoPath,
    MissingResource,
    InvalidDiet,
    InsufficientWater,
    PoorHealth,
    Inactive,
    OutOfRange,
    DifferentSpecies,
    Immature,
    Cooldown,
    InsufficientEnergy,
    Occupied,
    NoBirthSpace,
}

pub fn reject(
    events: &mut Vec<SimulationEvent>,
    id: usize,
    action: &str,
    target: Option<usize>,
    reason: RejectionReason,
) {
    events.push(SimulationEvent::ActionRejected {
        id,
        action: action.into(),
        target,
        reason,
    });
}

impl SimulationEvent {
    pub fn involves(&self,entity:usize)->bool {
        match self {
            Self::ResourceReplenished{..}=>false,
            Self::EntityMoved{id,..}|Self::EntityAte{id,..}|Self::EntityDrank{id,..}|Self::EntityDied{id,..}|Self::EntityBorn{id,..}|Self::EntitySlept{id,..}=>*id==entity,
            Self::CorpseConsumed{id,eater,..}=>*id==entity||*eater==entity,
            Self::EntityAttacked{attacker,target,..}=>*attacker==entity||*target==entity,
            Self::EntityMated{parent1,parent2,child,..}=>*parent1==entity||*parent2==entity||*child==entity,
            Self::ActionRejected{id,..}=>*id==entity,
        }
    }
}
