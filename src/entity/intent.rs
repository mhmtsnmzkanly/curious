use crate::map::direction::Direction;

/// World -> Perception -> Intent şeklinde yola koyulur.
/// World, canlının bulunduğu konumu baz alarak Perception oluşturur.
/// Entity, bu Perception ile kendi içerisinde ki özel mekanizma ile karar alır
/// BU KARAR KESİNLİK DEĞİLDİR, WORLD SON SÖZÜ SÖYLER
/// ÇAKIŞAN NİYETLER İÇİN WORLD İNSİYATİF ALABİLİR
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// Gidilmek istenilen nokta
    Move { steps: Vec<Direction> },
    /// Yenilmek istenilen yemeğin konumu,
    /// Not: Yemek aynı hücrede ise at okunmaz,
    /// miktar canlının yiyebiliceği ve World izin verdiği miktarda olur
    Eat { at: Vec<Direction>, corpse_id: Option<usize> },
    /// Çiftleşmek istenilen canlı
    Mate { target_id: usize },
    /// Saldırılmak istenilen canlı
    Attack { target_id: usize },
    /// Kaçınılmak istenilen canlı
    Flee { target_id: usize },
    /// Bekleme niyeti, iyileşme için (yavaş)
    Idle { duration: usize },
    /// Keyfi olarak uyuma eylemi, iyileşme için (hızlı)
    Sleep { duration: usize },
}
