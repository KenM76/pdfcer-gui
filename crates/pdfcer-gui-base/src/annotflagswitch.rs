//! # `annotflagswitch` — the three annotation-flag switches the Properties
//! panel offers, and what each does to the whole `/F` word
//!
//! `EditSession::set_annotation_flags` takes the complete word because Table
//! 165's bits interact. The switches are resolved against the word as the
//! document holds it **when the action is applied**, never when it was raised,
//! so a switch pressed behind another queued edit cannot write a stale word.
//!
//! Hidden (bit 2) means neither on screen nor printed, so the screen and print
//! switches each keep the other's state when they clear it: un-hiding for
//! print leaves the mark off screen (`NoView`), and un-hiding for the screen
//! leaves it unprinted.

use pdfcer_core::annot::AnnotFlags;

/// One switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagSwitch {
    /// Drawn on screen: neither Hidden nor NoView.
    OnScreen,
    /// Printed: Print set and not Hidden.
    Prints,
    /// Locked (bit 8): its properties may not be changed and it may not be
    /// moved, resized or deleted.
    Locked,
}

/// Whether `switch` is on in `word`.
#[must_use]
pub const fn is_on(word: u32, switch: FlagSwitch) -> bool {
    match switch {
        FlagSwitch::OnScreen => word & (AnnotFlags::HIDDEN | AnnotFlags::NO_VIEW) == 0,
        FlagSwitch::Prints => word & AnnotFlags::PRINT != 0 && word & AnnotFlags::HIDDEN == 0,
        FlagSwitch::Locked => word & AnnotFlags::LOCKED != 0,
    }
}

/// `word` with `switch` turned `on`, every other switch keeping its state and
/// every bit pdfcer has no switch for left as the file wrote it.
#[must_use]
pub const fn switched(word: u32, switch: FlagSwitch, on: bool) -> u32 {
    let hidden = word & AnnotFlags::HIDDEN != 0;
    match (switch, on) {
        (FlagSwitch::OnScreen, true) => {
            let shown = word & !(AnnotFlags::HIDDEN | AnnotFlags::NO_VIEW);
            if hidden {
                shown & !AnnotFlags::PRINT
            } else {
                shown
            }
        }
        (FlagSwitch::OnScreen, false) => {
            if hidden {
                word
            } else {
                word | AnnotFlags::NO_VIEW
            }
        }
        (FlagSwitch::Prints, true) => {
            let printed = (word & !AnnotFlags::HIDDEN) | AnnotFlags::PRINT;
            if hidden {
                printed | AnnotFlags::NO_VIEW
            } else {
                printed
            }
        }
        (FlagSwitch::Prints, false) => word & !AnnotFlags::PRINT,
        (FlagSwitch::Locked, true) => word | AnnotFlags::LOCKED,
        (FlagSwitch::Locked, false) => word & !AnnotFlags::LOCKED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [FlagSwitch; 3] = [FlagSwitch::OnScreen, FlagSwitch::Prints, FlagSwitch::Locked];

    /// Every switch lands where it was asked and moves no other switch, from
    /// every combination of the four bits they read.
    #[test]
    fn a_switch_moves_itself_and_nothing_else() {
        let bits = [
            AnnotFlags::HIDDEN,
            AnnotFlags::PRINT,
            AnnotFlags::NO_VIEW,
            AnnotFlags::LOCKED,
        ];
        for mask in 0..16u32 {
            let word = (0..4)
                .filter(|i| mask & (1 << i) != 0)
                .fold(AnnotFlags::NO_ZOOM, |w, i| w | bits[i as usize]);
            for switch in ALL {
                for on in [true, false] {
                    let after = switched(word, switch, on);
                    assert_eq!(is_on(after, switch), on, "{word:#x} {switch:?} {on}");
                    for other in ALL.into_iter().filter(|o| *o != switch) {
                        assert_eq!(
                            is_on(after, other),
                            is_on(word, other),
                            "{word:#x} {switch:?}"
                        );
                    }
                    assert_ne!(after & AnnotFlags::NO_ZOOM, 0, "an unswitched bit survives");
                }
            }
        }
    }
}
