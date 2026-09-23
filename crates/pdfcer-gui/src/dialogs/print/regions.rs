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

/// The **Properties…** button's published region.
///
/// ★ Published for `ui-verify`, which is the only oracle this project trusts
/// for a layout claim. See `tools/ui-verify/src/checks/print_paper.rs` for
/// what it asserts, and for why a driven check reads this rect without ever
/// clicking it.
pub(super) const REGION_PROPERTIES: &str = "print.properties";

/// The paper selector's published region — the combo itself, closed.
pub(super) const REGION_PAPER: &str = "print.paper";

/// One published region per entry in the OPEN paper list, indexed from zero
/// with the "from the printer's own settings" entry as index 0.
///
/// # Why the ENTRIES are published and not only the combo
///
/// Because a check that can open a list but not choose from it can only
/// assert that a control exists — and "the control exists" is exactly the
/// claim that was true of the tray checkbox for four months while it did
/// nothing. The property worth asserting is that **choosing a sheet changes
/// the plan**, and that needs a click on a specific entry.
///
/// An egui combo popup is an `Area` laid out at paint time; its entries have
/// no position anything outside the process could compute. Publishing them is
/// the only route, and it costs nothing when `PDFCER_DIAG` is unset.
pub(super) const REGION_PAPER_ITEM_PREFIX: &str = "print.paper.item.";

/// The **Match the pages in this document** entry's own published region —
/// operator request O167, 2026-09-10.
///
/// # ★ Why it is NOT `print.paper.item.1`
///
/// It sits second in the list on screen, so the obvious thing would have been
/// to give it index 1 and push the driver's forms up by one. That would have
/// been wrong in a way no gate would catch.
///
/// `REGION_PAPER_ITEM_PREFIX`'s numbering is a **contract with the driver's
/// own form list**: index 0 is "say nothing", and index *n* is `forms[n - 1]`.
/// A driven check reads those numbers to click a specific enumerated sheet and
/// then asserts that the planned sheet moved. Inserting a policy entry into
/// that namespace would leave the existing check clicking a different thing
/// from the one it names, still green, still reporting a sentence about a
/// form — the class of defect this project has now written down four times.
///
/// So auto gets a name of its own, outside the numbered namespace. Better for
/// the check that needs it, too: `print.paper.auto` cannot silently become a
/// different entry when the driver's list changes length.
pub(super) const REGION_PAPER_AUTO: &str = "print.paper.auto";

/// One published region per scale mode, suffixed with the mode's own WORD.
///
/// # Why the scale radios are published at all
///
/// The dialog opens on **Fit**, which scales a page down to the printable area
/// and therefore clips nothing. Every claim about what gets cropped — the
/// hatch, the ink verdict, the whole Position group — is unreachable from that
/// state, so a driven check that cannot choose **Actual size** cannot assert
/// any of it. `tools/ui-verify/src/checks/print_clip_claim.rs` skipped on this
/// machine for exactly that reason and said so in its own header.
///
/// A word and not an index, unlike [`REGION_PAPER_ITEM_PREFIX`]: an index here
/// would be a contract with the order of a `for` loop rather than with anything
/// outside the process, and this list has gained a mode once already.
pub(super) const REGION_SCALE_PREFIX: &str = "print.scale.";

/// The Position group's five buttons, one region each.
///
/// Published individually rather than as a group union because the group's
/// rectangle cannot answer *which* button was pressed, and the four placements
/// differ only in the number they write — see `check-region-names.py`'s third
/// failure shape.
pub(super) const REGION_POSITION_RESET: &str = "print.position.reset";
/// See [`REGION_POSITION_RESET`].
pub(super) const REGION_POSITION_CENTRE: &str = "print.position.centre";
/// See [`REGION_POSITION_RESET`].
pub(super) const REGION_POSITION_CENTRE_H: &str = "print.position.centre-h";
/// See [`REGION_POSITION_RESET`].
pub(super) const REGION_POSITION_CENTRE_V: &str = "print.position.centre-v";
/// See [`REGION_POSITION_RESET`].
pub(super) const REGION_POSITION_RESET_ALL: &str = "print.position.reset-all";

/// One published region per tab, suffixed with the tab's own WORD.
///
/// Every claim the dialog makes about *what gets cropped* now lives behind the
/// **Position** tab, so a driver that cannot press a tab cannot reach any of
/// it. A word rather than an index, for the reason
/// [`REGION_SCALE_PREFIX`] gives: an index is a contract with the order of a
/// `for` loop, and this list has just gained a fourth entry.
pub(super) const REGION_TAB_PREFIX: &str = "print.tab.";

/// The highest-resolution field on the Pages tab. Drawn on every open, capped
/// or not.
pub(super) const REGION_RESOLUTION: &str = "print.resolution";
