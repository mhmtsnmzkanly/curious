use crate::map::movement::Steps;

/// Canlının görüş açısıyla yola çıkarak ortaya koyduğu niyet
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// Gidilmek istenilen nokta
    Move { steps: Steps },
    /// Yenilmek istenilen yemeğin konumu,
    /// Not: Yemek aynı hücrede ise at okunmaz,
    /// miktar canlının yiyebiliceği ve World izin verdiği miktarda olur
    Eat { at: Steps, corpse_id: Option<usize> },
    /// İçilmek istenilen suyun konumu
    Drink { at: Steps },
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
