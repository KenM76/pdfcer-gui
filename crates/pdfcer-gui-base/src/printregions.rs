//! The names this dialog publishes for the driving harness.
//!
//! One file because a published region name is a **contract with
//! `tools/ui-verify`** rather than an implementation detail: the driver has no
//! other way to find a control inside a child OS viewport laid out at paint
//! time, and a rename here silently retargets or blinds a driven check. Keeping
//! them together means the contract can be read in one screen, and means
//! `check-region-names.py` has one file to reason about.
//!
//! Every constant is `pub(super)` or narrower and every one carries the
//! argument for its own shape — indexed versus worded, group versus
//! per-control — because those arguments are what a future addition has to fit
//! into.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printregions.md`.

/// The **Properties…** button's published region.
pub const REGION_PROPERTIES: &str = "print.properties";

/// The paper selector's published region — the combo itself, closed.
pub const REGION_PAPER: &str = "print.paper";

/// One published region per entry in the OPEN paper list, indexed from zero
/// with the "from the printer's own settings" entry as index 0.
pub const REGION_PAPER_ITEM_PREFIX: &str = "print.paper.item.";

/// The **Match the pages in this document** entry's own published region —
/// operator request O167, 2026-09-10.
pub const REGION_PAPER_AUTO: &str = "print.paper.auto";

/// One published region per scale mode, suffixed with the mode's own WORD.
pub const REGION_SCALE_PREFIX: &str = "print.scale.";

/// The Position group's five buttons, one region each.
pub const REGION_POSITION_RESET: &str = "print.position.reset";
/// See [`REGION_POSITION_RESET`].
pub const REGION_POSITION_CENTRE: &str = "print.position.centre";
/// See [`REGION_POSITION_RESET`].
pub const REGION_POSITION_CENTRE_H: &str = "print.position.centre-h";
/// See [`REGION_POSITION_RESET`].
pub const REGION_POSITION_CENTRE_V: &str = "print.position.centre-v";
/// See [`REGION_POSITION_RESET`].
pub const REGION_POSITION_RESET_ALL: &str = "print.position.reset-all";

/// One published region per tab, suffixed with the tab's own WORD.
pub const REGION_TAB_PREFIX: &str = "print.tab.";

/// The highest-resolution field on the Pages tab. Drawn on every open, capped
/// or not.
pub const REGION_RESOLUTION: &str = "print.resolution";
