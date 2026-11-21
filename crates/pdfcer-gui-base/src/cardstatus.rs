//! # `cardstatus` — what each graphics-card path last did on this machine
//!
//! Each path that can draw on the graphics card or on the processor records
//! its latest [`Outcome`] here, overwriting the one before; the graphics-card
//! window (`pdfcer_gui::dialogs::cardswitches`) reads a [`snapshot`] each
//! frame. Writers call [`note`] at the point where the path decides, from any
//! thread. Only the latest outcome per path is kept: the window answers
//! "what is it doing now", not "what has it ever done".
//!
//! Trace: `card-status path=<Path::token> outcome=<Outcome::token>` whenever a
//! path's outcome changes.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The number of 3D viewer parts with their own switch
/// (`text::model3dlook::CardPart::ALL`).
pub const PARTS: usize = 7;

/// A path that draws on the graphics card or on the processor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Path {
    /// The canvas's pages.
    Pages,
    /// The 3D viewer's whole live picture.
    Picture,
    /// One part of it, by index in `CardPart::ALL`.
    Part(usize),
    /// The 3D viewer's bounce-light build.
    Bounce,
    /// Measure light's bounce volume.
    Meter,
    /// Colour by light level's map.
    Map,
}

/// Every path, in the window's order.
pub const ALL: [Path; 5 + PARTS] = [
    Path::Pages,
    Path::Picture,
    Path::Part(0),
    Path::Part(1),
    Path::Part(2),
    Path::Part(3),
    Path::Part(4),
    Path::Part(5),
    Path::Part(6),
    Path::Bounce,
    Path::Meter,
    Path::Map,
];

impl Path {
    /// Its place in [`ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Pages => 0,
            Self::Picture => 1,
            Self::Part(i) => 2 + if i < PARTS { i } else { PARTS - 1 },
            Self::Bounce => 2 + PARTS,
            Self::Meter => 3 + PARTS,
            Self::Map => 4 + PARTS,
        }
    }

    /// Its word in traces and region names.
    #[must_use]
    pub fn token(self) -> String {
        match self {
            Self::Pages => "pages".into(),       // ui-text-exempt: trace token
            Self::Picture => "picture".into(),   // ui-text-exempt: trace token
            Self::Part(i) => format!("part{i}"), // ui-text-exempt: trace token
            Self::Bounce => "bounce".into(),     // ui-text-exempt: trace token
            Self::Meter => "meter".into(),       // ui-text-exempt: trace token
            Self::Map => "map".into(),           // ui-text-exempt: trace token
        }
    }
}

/// What a path last did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Outcome {
    /// Not asked for this session.
    #[default]
    Untried,
    /// Drawn on the graphics card.
    InUse,
    /// Asked for, and the processor drew it instead; why.
    Processor(String),
    /// Drawn on the card within a limit of the card; which.
    Limit(String),
    /// Kept on the processor by an environment variable; its name.
    ForcedOff(&'static str),
    /// Switched off; the processor draws it.
    Off,
}

impl Outcome {
    /// Its word in traces.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Untried => "untried",         // ui-text-exempt: trace token
            Self::InUse => "in-use",            // ui-text-exempt: trace token
            Self::Processor(_) => "processor",  // ui-text-exempt: trace token
            Self::Limit(_) => "limit",          // ui-text-exempt: trace token
            Self::ForcedOff(_) => "forced-off", // ui-text-exempt: trace token
            Self::Off => "off",                 // ui-text-exempt: trace token
        }
    }
}

const UNTRIED: Outcome = Outcome::Untried;
static STATUS: Mutex<[Outcome; ALL.len()]> = Mutex::new([UNTRIED; ALL.len()]);
static OUT_OF_MEMORY: AtomicBool = AtomicBool::new(false);

/// Record `outcome` as `path`'s latest.
pub fn note(path: Path, outcome: Outcome) {
    let mut status = STATUS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let slot = &mut status[path.index()];
    if *slot == outcome {
        return;
    }
    let (p, o) = (path.token(), outcome.token());
    *slot = outcome;
    drop(status);
    crate::diag::trace(|| format!("card-status path={p} outcome={o}")); // ui-text-exempt: diagnostic trace, never displayed
}

/// Every path's latest outcome, in [`ALL`]'s order.
#[must_use]
pub fn snapshot() -> [Outcome; ALL.len()] {
    STATUS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Record that the GL error flag held out-of-memory.
pub fn saw_out_of_memory() {
    OUT_OF_MEMORY.store(true, Ordering::Relaxed);
}

/// Whether the GL error flag has held out-of-memory this session.
#[must_use]
pub fn out_of_memory_seen() -> bool {
    OUT_OF_MEMORY.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_path_has_its_own_slot_and_the_latest_wins() {
        for (i, path) in ALL.into_iter().enumerate() {
            assert_eq!(path.index(), i, "{path:?}");
        }
        note(Path::Part(3), Outcome::Off);
        note(Path::Part(3), Outcome::Processor("why".into()));
        let s = snapshot();
        assert_eq!(s[Path::Part(3).index()], Outcome::Processor("why".into()));
        assert_eq!(s[Path::Part(2).index()], Outcome::Untried);
    }
}
