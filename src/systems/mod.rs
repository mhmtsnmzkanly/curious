//! World içindeki yükü bölmek için kullanılan sistemler katmanı.
//! İleride World::tick() içindeki resolve_* metodları buraya taşınacaktır.

pub mod movement;
pub mod interaction;
pub mod life_cycle;
pub mod metabolism;
pub mod perception;
pub mod intent_collection;
pub mod planning;

/// Herhangi bir sistemi temsil eden temel yapı (İleride trait'e dönüşebilir)
pub struct System;
