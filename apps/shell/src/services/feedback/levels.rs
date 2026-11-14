//! Warning once at each level as a battery runs down: at 20 % and again at
//! 10 %, never twice at the same one, and anew once it has charged past
//! the reset. Shared by the laptop's own battery and its devices'.

/// The levels warned at, highest first.
pub const LEVELS: [u8; 2] = [20, 10];

/// Above this a battery counts as charged again, and is warned anew.
pub const RESET_ABOVE: u8 = 25;

/// From the level last warned at and the battery now: the level warned at
/// after this reading, and the one to warn about now, if any.
pub fn warning(warned: Option<u8>, percent: u8) -> (Option<u8>, Option<u8>) {
    if percent > RESET_ABOVE {
        return (None, None);
    }
    let due = LEVELS
        .iter()
        .copied()
        .filter(|level| percent <= *level)
        .min();
    match (warned, due) {
        (Some(warned), Some(due)) if due < warned => (Some(due), Some(due)),
        (None, Some(due)) => (Some(due), Some(due)),
        (warned, _) => (warned, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_once_at_each_level() {
        assert_eq!(warning(None, 50), (None, None));
        assert_eq!(warning(None, 20), (Some(20), Some(20)));
        assert_eq!(warning(Some(20), 15), (Some(20), None));
        assert_eq!(warning(Some(20), 9), (Some(10), Some(10)));
        assert_eq!(warning(Some(10), 5), (Some(10), None));
    }

    #[test]
    fn a_battery_found_very_low_warns_once_at_the_lowest() {
        assert_eq!(warning(None, 4), (Some(10), Some(10)));
    }

    #[test]
    fn charging_past_the_reset_warns_again() {
        assert_eq!(warning(Some(10), 22), (Some(10), None));
        assert_eq!(warning(Some(10), 30), (None, None));
        assert_eq!(warning(None, 19), (Some(20), Some(20)));
    }
}
