//! # `prefs::families` — the keyed groups a preferences line is offered to
//!
//! Each group owns its keys and its parser; a line nobody's fixed key matched is
//! offered to each in turn, and the first to claim it answers. No two groups
//! share a key spelling, so the order decides nothing but cost.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/families.md`.

use super::Prefs;
use super::printing::KeyOutcome;

type Parse = fn(&mut Prefs, &str, &str) -> KeyOutcome;

/// Every group, in the order a line is offered to them.
const FAMILIES: [Parse; 6] = [
    |p, k, v| super::offpage::parse_key(&mut p.off_page, k, v),
    |p, k, v| super::printing::parse_key(&mut p.print, k, v),
    |p, k, v| super::exporting::parse_key(&mut p.export, k, v),
    |p, k, v| super::shortcuts::parse_key(&mut p.shortcuts, k, v),
    |p, k, v| super::snapshot::parse_key(&mut p.snapshot, k, v),
    |p, k, v| super::ocrmodels::parse_key(&mut p.ocr_models, k, v),
];

/// Offer `key = value` to every group; `NotMine` when none claims it.
pub(super) fn parse_key(prefs: &mut Prefs, key: &str, value: &str) -> KeyOutcome {
    for parse in FAMILIES {
        match parse(prefs, key, value) {
            KeyOutcome::NotMine => {}
            mine => return mine,
        }
    }
    KeyOutcome::NotMine
}
