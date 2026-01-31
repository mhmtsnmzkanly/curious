use crate::map::direction::Direction;

/// =======================================================
/// INTENT
/// =======================================================
///
/// Action yerine geçer.
/// Daha zengin, karşılaştırılabilir bir yapı.
/// World bunu VALIDATE eder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Move { to: Direction },
    Mate { target: usize },
    Eat { at: Direction },
    Attack { target: usize },
    Flee { from: usize },
    Idle,
}
