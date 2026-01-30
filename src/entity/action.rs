use crate::map::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Move(Direction),
    Eat,
    Attack { target_id: usize },
    Flee(Direction),
    Idle,
    Mate { target_id: usize }, // Yeni: Belirli bir hedefle çiftleşme isteği
}
