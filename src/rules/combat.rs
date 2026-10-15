//! Combat capabilities. Entity implementations may override their profile.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CombatProfile {
    pub damage: usize,
    pub energy_cost: usize,
    pub stamina_cost: usize,
}
