//! Standalone, default-off local coordination & alarm call prototype (ROADMAP K4 / Trace D).
//!
//! Coordination uses strictly verified visible local information. A receiver
//! only reacts to an alarm call if the sender is within the receiver's local
//! perception radius (`chebyshev_distance <= receiver_vision_range`) and the
//! alarm was resolver-paid in the current or previous tick. Global threat
//! coordinates, hidden world state, and unverified signals are never leaked or used.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalCoord {
    pub x: i32,
    pub y: i32,
}

impl LocalCoord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn chebyshev_distance(&self, other: &Self) -> u32 {
        (self.x - other.x).abs().max((self.y - other.y).abs()) as u32
    }

    pub fn sector_to(&self, target: &Self) -> u8 {
        let dx = target.x - self.x;
        let dy = target.y - self.y;
        // 8 compass sectors (0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW):
        match (dx.signum(), dy.signum()) {
            (0, 1) => 0,
            (1, 1) => 1,
            (1, 0) => 2,
            (1, -1) => 3,
            (0, -1) => 4,
            (-1, -1) => 5,
            (-1, 0) => 6,
            (-1, 1) => 7,
            _ => 0,
        }
    }

    pub fn step_in_sector(&self, sector: u8) -> Self {
        let (dx, dy) = match sector % 8 {
            0 => (0, 1),
            1 => (1, 1),
            2 => (1, 0),
            3 => (1, -1),
            4 => (0, -1),
            5 => (-1, -1),
            6 => (-1, 0),
            7 => (-1, 1),
            _ => (0, 0),
        };
        Self::new(self.x + dx, self.y + dy)
    }
}

/// Finite local social actions as specified in ROADMAP K4 (Trace D).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LocalSocialAction {
    EmitAlarmCall {
        threat_sector: u8,
        energy_cost: u16,
        stamina_cost: u16,
    },
    AlignWithVisiblePeer {
        peer_id: usize,
        step_sector: u8,
    },
}

/// Resolver-verified paid alarm event. Contains only directional sector and sender position.
/// Never leaks hidden threat ID, threat health, or global threat coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedAlarmEvent {
    pub sender_id: usize,
    pub sender_pos: LocalCoord,
    pub threat_sector: u8,
    pub tick: u64,
    pub paid_energy: u16,
    pub paid_stamina: u16,
    pub is_paid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAgent {
    pub id: usize,
    pub pos: LocalCoord,
    pub energy: u16,
    pub stamina: u16,
    pub vision_range: u32,
    pub is_alive: bool,
    pub warned_sector: Option<u8>,
}

impl LocalAgent {
    pub fn new(id: usize, pos: LocalCoord, energy: u16, stamina: u16, vision_range: u32) -> Self {
        Self {
            id,
            pos,
            energy,
            stamina,
            vision_range,
            is_alive: true,
            warned_sector: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmResolution {
    Paid(VerifiedAlarmEvent),
    RejectedInsufficientEnergy { needed: u16, available: u16 },
    RejectedInsufficientStamina { needed: u16, available: u16 },
    RejectedInvalidSector(u8),
    RejectedZeroPayment,
}

/// Physical resolver for alarm calls: deducts exact energy and stamina costs.
pub fn resolve_alarm_call(
    sender: &mut LocalAgent,
    threat_sector: u8,
    energy_cost: u16,
    stamina_cost: u16,
    tick: u64,
) -> AlarmResolution {
    if threat_sector >= 8 {
        return AlarmResolution::RejectedInvalidSector(threat_sector);
    }
    if energy_cost == 0 && stamina_cost == 0 {
        return AlarmResolution::RejectedZeroPayment;
    }
    if sender.energy < energy_cost {
        return AlarmResolution::RejectedInsufficientEnergy {
            needed: energy_cost,
            available: sender.energy,
        };
    }
    if sender.stamina < stamina_cost {
        return AlarmResolution::RejectedInsufficientStamina {
            needed: stamina_cost,
            available: sender.stamina,
        };
    }
    sender.energy -= energy_cost;
    sender.stamina -= stamina_cost;
    AlarmResolution::Paid(VerifiedAlarmEvent {
        sender_id: sender.id,
        sender_pos: sender.pos,
        threat_sector,
        tick,
        paid_energy: energy_cost,
        paid_stamina: stamina_cost,
        is_paid: true,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmPerception {
    Perceived {
        sender_id: usize,
        warned_sector: u8,
    },
    IgnoredOutOfRange {
        distance: u32,
        vision_range: u32,
    },
    IgnoredUnpaidOrFraudulent,
    IgnoredStaleTick {
        alarm_tick: u64,
        current_tick: u64,
    },
    IgnoredSelf,
}

/// Receiver rule: a receiver can only perceive an alarm if the sender is within
/// the receiver's local vision range (`distance <= receiver_vision_range`) and the
/// alarm was resolver-paid in the current or previous tick.
pub fn process_local_alarm(
    receiver: &LocalAgent,
    alarm: &VerifiedAlarmEvent,
    current_tick: u64,
) -> AlarmPerception {
    if receiver.id == alarm.sender_id {
        return AlarmPerception::IgnoredSelf;
    }
    if !alarm.is_paid || (alarm.paid_energy == 0 && alarm.paid_stamina == 0) {
        return AlarmPerception::IgnoredUnpaidOrFraudulent;
    }
    if current_tick < alarm.tick || current_tick > alarm.tick + 1 {
        return AlarmPerception::IgnoredStaleTick {
            alarm_tick: alarm.tick,
            current_tick,
        };
    }
    let distance = receiver.pos.chebyshev_distance(&alarm.sender_pos);
    if distance > receiver.vision_range {
        return AlarmPerception::IgnoredOutOfRange {
            distance,
            vision_range: receiver.vision_range,
        };
    }
    AlarmPerception::Perceived {
        sender_id: alarm.sender_id,
        warned_sector: alarm.threat_sector,
    }
}

/// Coordinated step selection: shifts the agent's decision away from the warned sector.
pub fn choose_coordinated_step(
    agent: &mut LocalAgent,
    perceived_alarm: Option<&AlarmPerception>,
    default_step_sector: u8,
) -> u8 {
    if let Some(AlarmPerception::Perceived { warned_sector, .. }) = perceived_alarm {
        agent.warned_sector = Some(*warned_sector);
    }
    if let Some(warned) = agent.warned_sector {
        // Strictly opposite sector: moves away from threat bearing
        (warned + 4) % 8
    } else {
        default_step_sector
    }
}

/// Peer alignment: only possible if peer is visible to the agent (`distance <= agent.vision_range`).
pub fn choose_peer_alignment_step(
    agent: &LocalAgent,
    peer: &LocalAgent,
    peer_step_sector: u8,
) -> Option<u8> {
    let distance = agent.pos.chebyshev_distance(&peer.pos);
    if distance <= agent.vision_range {
        Some(peer_step_sector % 8)
    } else {
        None
    }
}

/// 3-arm comparison harness: ControlSolo vs. UnverifiedIgnored vs. VerifiedLocalCoordination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinationArm {
    ControlSolo,
    UnverifiedIgnored,
    VerifiedLocalCoordination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmMetrics {
    pub arm: CoordinationArm,
    pub focal_survived: bool,
    pub min_threat_distance_to_focal: u32,
    pub total_energy_spent_focal: u16,
    pub total_stamina_spent_focal: u16,
    pub total_energy_spent_sender: u16,
    pub total_stamina_spent_sender: u16,
    pub alarms_emitted: u32,
    pub alarms_verified_paid: u32,
    pub alarms_perceived_by_focal: u32,
    pub alarms_rejected_or_unperceived: u32,
}

pub fn run_coordination_harness(arm: CoordinationArm, max_ticks: u32) -> ArmMetrics {
    let mut threat_pos = LocalCoord::new(8, 0);

    // Focal agent starts at (0, 0), vision range 3
    let mut focal = LocalAgent::new(1, LocalCoord::new(0, 0), 100, 100, 3);

    // Sender conspecific at (3, 0), vision range 5 (can see threat at dist <= 5)
    let has_sender = arm != CoordinationArm::ControlSolo;
    let mut sender = LocalAgent::new(2, LocalCoord::new(3, 0), 100, 100, 5);

    let mut min_threat_dist = focal.pos.chebyshev_distance(&threat_pos);
    let mut focal_survived = true;

    let mut alarms_emitted = 0;
    let mut alarms_verified_paid = 0;
    let mut alarms_perceived_by_focal = 0;
    let mut alarms_rejected_or_unperceived = 0;

    let initial_energy_focal = focal.energy;
    let initial_stamina_focal = focal.stamina;
    let initial_energy_sender = sender.energy;
    let initial_stamina_sender = sender.stamina;

    let mut last_alarm: Option<VerifiedAlarmEvent> = None;

    for tick in 0..max_ticks as u64 {
        // 1. Sender evaluates threat if sender exists
        if has_sender && sender.is_alive {
            let dist_to_threat = sender.pos.chebyshev_distance(&threat_pos);
            if dist_to_threat <= sender.vision_range {
                alarms_emitted += 1;
                let threat_sec = sender.pos.sector_to(&threat_pos);
                match arm {
                    CoordinationArm::VerifiedLocalCoordination => {
                        // Sender pays cost (10 energy, 5 stamina)
                        if let AlarmResolution::Paid(evt) =
                            resolve_alarm_call(&mut sender, threat_sec, 10, 5, tick)
                        {
                            alarms_verified_paid += 1;
                            last_alarm = Some(evt);
                        }
                    }
                    CoordinationArm::UnverifiedIgnored => {
                        // Unpaid alarm (cost not paid, is_paid = false)
                        last_alarm = Some(VerifiedAlarmEvent {
                            sender_id: sender.id,
                            sender_pos: sender.pos,
                            threat_sector: threat_sec,
                            tick,
                            paid_energy: 0,
                            paid_stamina: 0,
                            is_paid: false,
                        });
                    }
                    CoordinationArm::ControlSolo => {}
                }
            }
        }

        // 2. Focal agent evaluates perception and decides step
        let perceived = if let Some(ref alarm) = last_alarm {
            let p = process_local_alarm(&focal, alarm, tick);
            match p {
                AlarmPerception::Perceived { .. } => {
                    alarms_perceived_by_focal += 1;
                    Some(p)
                }
                _ => {
                    alarms_rejected_or_unperceived += 1;
                    None
                }
            }
        } else {
            None
        };

        // Default foraging/wandering intent is East (sector 2) towards (8, 0)
        let default_sector = 2;
        let chosen_sector = choose_coordinated_step(&mut focal, perceived.as_ref(), default_sector);
        focal.pos = focal.pos.step_in_sector(chosen_sector);
        focal.energy = focal.energy.saturating_sub(1);

        // Threat moves 1 step West towards focal
        threat_pos.x -= 1;

        let current_dist = focal.pos.chebyshev_distance(&threat_pos);
        if current_dist < min_threat_dist {
            min_threat_dist = current_dist;
        }

        // Threat attack radius is 1
        if current_dist <= 1 {
            focal_survived = false;
            focal.is_alive = false;
            break;
        }
    }

    ArmMetrics {
        arm,
        focal_survived,
        min_threat_distance_to_focal: min_threat_dist,
        total_energy_spent_focal: initial_energy_focal.saturating_sub(focal.energy),
        total_stamina_spent_focal: initial_stamina_focal.saturating_sub(focal.stamina),
        total_energy_spent_sender: initial_energy_sender.saturating_sub(sender.energy),
        total_stamina_spent_sender: initial_stamina_sender.saturating_sub(sender.stamina),
        alarms_emitted,
        alarms_verified_paid,
        alarms_perceived_by_focal,
        alarms_rejected_or_unperceived,
    }
}
