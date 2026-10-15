//! World içindeki yükü bölmek için kullanılan sistemler katmanı.
//! İleride World::tick() içindeki resolve_* metodları buraya taşınacaktır.

pub mod intent_collection;
pub mod interaction;
pub mod life_cycle;
pub mod locomotion;
pub mod metabolism;
pub mod movement;
pub mod perception;
pub mod planning;

/// Herhangi bir sistemi temsil eden temel yapı (İleride trait'e dönüşebilir)
pub struct System;
