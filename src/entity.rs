use crate::{action::Action, position::Position, world::WorldView};

pub trait Entity {
    fn id(&self) -> usize; // Canlıya ait benzersiz kimlik
    fn position(&self) -> Position; // Canlının bulunduğu konum
    fn think(&self, ctx: &WorldView) -> Action; // Karar verme mekanizması
    fn apply(&mut self, action: Action); // Kararı işleme koy
}
