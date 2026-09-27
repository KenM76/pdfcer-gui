//! # `measure` — the measure tools' pick state machines and scale entry
//!
//! Pure state: which points a measure gesture has collected and what the
//! scale fields hold. `pdfcer_gui::canvas::measure` re-exports these and owns
//! the drawing and the document writes.

/// The radius/diameter tool's point set.
pub mod circpick;
/// Which dimensioning tool is armed.
pub mod kind;
/// The linear and two-line pick machines; re-exports [`circpick`]'s types.
pub mod pick;
/// Scale entry, and the dimension-group actions.
pub mod scale;
