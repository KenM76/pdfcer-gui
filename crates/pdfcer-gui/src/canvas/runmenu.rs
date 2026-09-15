//! # `canvas::runmenu` — **the right-click route to ONE LINE of a text block**
//!
//! ## The operator's report, and the half of it this file is
//!
//! `OPERATOR_REQUESTS.md` O188, verbatim:
//!
//! > *"In text that is grouped together or whatever it is called, such as in
//! > my title blocks, I would like a way to move the individual text blocks
//! > within it around, and have the ability to delete them…"*
//!
//! That request has three halves and this is the third.
//!
//! * **(B)** delete one line — shipped: `EditSession::delete_text_run`,
//!   reachable at the Part rung.
//! * **(C)** move one line — **the engine has no verb**, request `G017`; the
//!   drag declines with a worded sentence rather than silently.
//! * **(A)** *find* either of them — **this file**.
//!
//! The note filed with (B) and (C) stated (A) in one line, and it is the line
//! this module exists to discharge:
//!
//! > ⚠ **(A) is still open.** The delete is reachable only through the Points
//! > tool (`A`, then click); a double-click on text opens the caret instead,
//! > which is O70's ruling and correct. The new sentence tells him Delete
//! > works, but only after he has tried to drag. **A route he can find
//! > *before* failing is owed.**
//!
//! ## ★★★ One route, announced nowhere
//!
//! Measured across every gesture this canvas has: without this module the Part
//! rung on a text object — the rung at which *one line* is the operand — is
//! reachable by **exactly one** sequence:
//!
//! > be in Edit mode, arm the **Points** tool (chord `A`, `view.tool_node`)
//! > *before* clicking, then single-left-click the line.
//!
//! Everything else lands at the Object rung and stays there:
//!
//! * **plain click, Select tool** — Object: `SelectionState::click` →
//!   `click_at_object_rung`, which discards the hit's part outright.
//! * **marquee** — Object, hard-set and correctly: a band names a region, and
//!   a region contains objects.
//! * **double-click** — the **caret**. O70's ruling, and correct: text under a
//!   double-click means *type here*.
//! * **right-click** — Object: `menus::right_clicked_object` asks `hit_test`,
//!   not `probe`.
//!
//! ⇒ So the one working route required knowing a chord that no surface names,
//! *and* arming it in advance. An operator who clicks the line first and then
//! goes looking has already lost the rung. That is not a discoverability
//! weakness; it is a capability that ships unreachable to anybody who was not
//! told.
//!
//! ## ★★ Why a MENU ROW and not a new gesture
//!
//! The standing rule is *use the conventional interaction, never invent one*,
//! and it bars a menu row that **performs an edit**: the conventional way to
//! move a piece of something is to drag it, not to pick a row called *Move*.
//! That is the *move* half, and it is not this file.
//!
//! A row that **changes what is selected** is the other kind of row entirely.
//! This menu already carries two — `format.select_form` and
//! `format.unshare_form` both re-aim the selection from the canvas, for the
//! reason this row exists too: the operator's hand is on the thing, and the
//! ribbon is three inches away. `canvas::annotnodes::menu`'s two shipped rows
//! are the same shape one surface along — a Points-tool chord route nothing
//! announced, given a row somebody can read.
//!
//! ★ And `canvas::menus`' own rule that *"a right-click never descends"* is
//! not breached. The right-click still selects the **whole object**, exactly
//! as it always has; the descent happens only when the operator reads a row
//! that says so and presses it. The rule is about what a click does silently.
//!
//! ## ★★★ The operand problem, and where it is parked
//!
//! A menu row carries **a command id and nothing else**
//! (`egui_shell::manifest::Item::Command`). *"Select this line of text"* needs
//! to know **which line** — a fact that exists only at the instant of the
//! secondary click, on a surface the dispatcher never sees.
//!
//! So the pick is parked in `egui::Memory` for the life of the popup, beside
//! the two things already parked there for the identical reason: which canvas
//! menu is open (`canvas::menus`' `MENU_MEMORY_KEY`) and which corner a markup
//! right-click landed on (`canvas::annotnodes::menu`'s `PICK_MEMORY_KEY`).
//! `egui` opens a popup ON the secondary click and redraws it every frame until
//! it is dismissed, and the pointer moves during those frames — onto the menu
//! itself, which is not over the text any more — so recomputing from the live
//! pointer would swap the row's meaning out from under the operator's hand
//! while they were reading it.
//!
//! ⇒ [`park`] is called once, on the click. [`parked`] is read on every frame
//! the menu is drawn (to decide the row's condition) **and** again when the
//! command is dispatched (to build the selection). One value, three readers,
//! no second derivation of *"which line did they mean"*.
//!
//! ## R9: offered or absent, never greyed
//!
//! There is no *temporarily unavailable* state here. Either the pointer is on
//! a line of a multi-line text object — in which case the row works — or it is
//! not, and nothing the operator can do while the menu is open changes that.
//! R9 reserves greying for the recoverable case, so this row has one condition
//! ([`crate::shell::menus::RUN_SELECT_OFFERED`]) carried on **both** the
//! item's `shown_when` and the command's `enabled_when`: shown implies
//! pressable, and a stale frame cannot press a dead row.
//!
//! ★★ **The `of > 1` gate is part of that decision and is not an
//! optimisation.** On a text object with a single run, *"select this line"*
//! and *"select this object"* name the same ink, and `delete_text_run` on the
//! only run is `delete_objects` spelled at a deeper rung. Offering the row
//! there would be offering a descent into a place identical to where the
//! operator already is — a row that appears to do nothing. See [`pick_at`].
//!
//! ## Rule 4 / R8b
//!
//! Nothing here paints, and nothing here marks the document. This module
//! answers two questions — *which line is under the pointer*, and *is that a
//! thing worth offering* — and both are about the **cursor**. What the row
//! does when pressed is change a selection, which is an affordance and not
//! content. The one disclosure this feature owes — *which* line, of how many,
//! is now selected — lives off-canvas in the status row, where
//! `app::toolstatus` puts it.
//!
//! ## Rule 15
//!
//! The word used throughout is **line**, never *dimension*. The CAD-exported
//! labels these runs usually are on the operator's drawings are **pdf
//! dimensions** — page content pdfcer reads and must not silently alter — and
//! this module neither reads them as measurements nor alters them. It moves a
//! selection.

use crate::canvas::mapping::PageMapping;
use crate::canvas::target::{CanvasTargetProvider, TargetId};
use crate::panels::objects::provider::{ObjectModelProvider, PartKind};

/// `egui::Memory` key for the text run the last right-click landed on.
///
/// One `Id`, per `egui::Context`, for the module header's reason: the pick is
/// taken at the click and read on every frame the popup is drawn, so it has to
/// outlive the click and must not outlive the session. `Memory` rather than
/// `PdfcerApp` state because this is frame-local interaction state with no
/// meaning across a document — closing one starts the next frame with no pick
/// and no popup in flight.
const PICK_MEMORY_KEY: &str = "pdfcer-text-run-pick"; // ui-text-exempt: internal memory id, never displayed

/// `text-run-menu pick=… offered=…` — what a right-click on a text object
/// resolved to, and whether the row will be drawn.
///
/// ★ Emitted on the frame of the click and on that frame only, so a driven
/// check can read *which line the menu thinks it is about* without pressing
/// anything. Without it, a row that failed to appear and a row that appeared
/// about the wrong line are the same observation.
pub const TRACE_MENU: &str = "text-run-menu"; // ui-text-exempt: diagnostic trace name

/// `text-run-command pick=… outcome=…` — the row was pressed, and this is the
/// operand it was carrying.
///
/// ★ It carries the **pick**, not only the outcome, because the whole class of
/// defect this design can produce is *the right verb on the wrong line*. A
/// trace line has to carry the number a wrong build would get wrong.
pub const TRACE_COMMAND: &str = "text-run-command"; // ui-text-exempt: diagnostic trace name

/// **What the right-click landed on.**
///
/// Two answers and no `Option`, for [`crate::canvas::annotnodes::menu::NodePick`]'s
/// stated reason: *"the pointer was nowhere near a line"* is a real state the
/// menu has to render (the row absent, the rest of the object menu intact)
/// rather than an error, and an `Option<RunPick>` would let a caller
/// `unwrap_or_default` its way into treating it as run 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunPick {
    /// On a text run, by index into `TextObject::runs` in content order.
    ///
    /// ★★ **The same numbering `delete_text_run` and `hit_test_text_runs`
    /// use**, so the number this menu picks is the number the engine acts on
    /// and the shell needs no second index space. That is the fact request
    /// `G017` relies on for its `move_text_run` signature too.
    Run {
        /// The page the click was on. Carried rather than re-derived at
        /// dispatch: a pick is only meaningful on the page it was taken on,
        /// and the dispatcher re-checks rather than assuming.
        page: usize,
        /// The text object, in whichever index space it lives in.
        object: TargetId,
        /// Which run, content order.
        run: usize,
        /// How many runs the object has — the *of* in *line 3 of 14*.
        ///
        /// Carried so the disclosure the status row owes can be written
        /// without a second walk of the object, and so a reader of the trace
        /// can tell a wrong pick from a short object.
        of: usize,
    },
    /// Nowhere near a line of a multi-line text object.
    #[default]
    Elsewhere,
}

impl RunPick {
    /// **Whether the row is drawn at all** — R9's one boolean for this
    /// feature.
    ///
    /// See the module header: there is no greyed state, so *shown* and
    /// *enabled* are one question, and this is it.
    #[must_use]
    pub const fn offered(self) -> bool {
        matches!(self, Self::Run { .. })
    }

    /// A short word for the trace. Never displayed.
    fn word(self) -> String {
        match self {
            // ui-text-exempt: diagnostic trace fragments, never displayed in the UI.
            Self::Run { run, of, .. } => format!("run:{run}/{of}"),
            Self::Elsewhere => "elsewhere".to_owned(),
        }
    }
}

/// **Which line of text a right-click at `screen` landed on.**
///
/// # The three gates, in the order they are cheapest to fail
///
/// 1. **The object under the pointer is a text object.** `part_kind_of`
///    answers `Some(PartKind::Run)` for `VectorObject::Text` and nothing else,
///    so a path, an image or an annotation leaves here immediately.
/// 2. **It has more than one run.** The module header carries why: on a
///    single-run object this row would descend to a place indistinguishable
///    from where the operator already stands.
/// 3. **The pointer is actually on a run.** `part_hits_of` is the SAME query
///    `canvas::input::probe` asks to enter the Part rung, so the line this
///    menu offers and the line a Points-tool click would enter cannot
///    disagree.
///
/// # ★★ Why `part_hits_of` and not a hit test of this module's own
///
/// Because the alternative is two spellings of *"which run is under this
/// point"*, and the failure mode of that shape is silent: a change to one
/// spelling's index handling leaves the other answering a different line, with
/// every unit test of both still green. One query, two callers, no drift.
///
/// # ★ The coordinates
///
/// `map.to_page(screen)` and `map.tolerance()` — the canonical pair, exactly
/// as `menus::right_clicked_object` and `canvas::input::probe` both use. The
/// tolerance is divided by zoom inside `PageMapping`, which is the one place
/// in `canvas/` that divides by zoom, and keeping that true is what stops a
/// second conversion drifting from this one.
///
/// Every argument is an `Option` because the caller holds them that way: there
/// may be no decomposed model, no object under the pointer, and no pointer at
/// all (an off-window frame). Each of those is [`RunPick::Elsewhere`], which
/// is the honest answer and not an error.
#[must_use]
pub fn pick_at(
    targets: Option<&ObjectModelProvider>,
    page: usize,
    object: Option<TargetId>,
    map: &PageMapping,
    screen: Option<egui::Pos2>,
) -> RunPick {
    let (Some(targets), Some(object), Some(at)) = (targets, object, screen) else {
        return RunPick::Elsewhere;
    };
    // 1.
    if targets.part_kind_of(object) != Some(PartKind::Run) {
        return RunPick::Elsewhere;
    }
    // 2. ★ `page_object_index` is `None` for a leaf — a text object painted
    // from inside a form XObject. `part_hits_of` already answers empty there
    // (`canvas::target`'s `(Some(PartKind::Run), None)` arm, which is an
    // acknowledged hole rather than an oversight), so the row would not be
    // offered anyway; failing here as well makes the reason legible instead of
    // arriving as a mysterious empty hit list two lines down.
    let Some(index) = object.page_object_index() else {
        return RunPick::Elsewhere;
    };
    let of = targets.text_run_count(index);
    if of <= 1 {
        return RunPick::Elsewhere;
    }
    // 3.
    let Some(&run) = targets
        .part_hits_of(page, object, map.to_page(at), map.tolerance())
        .first()
    else {
        return RunPick::Elsewhere;
    };
    RunPick::Run {
        page,
        object,
        run,
        of,
    }
}

/// **Park the pick for the life of the popup.** Called once, on the click.
pub fn park(ctx: &egui::Context, pick: RunPick) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(PICK_MEMORY_KEY), pick));
}

/// Read the parked pick. [`RunPick::Elsewhere`] before any right-click.
#[must_use]
pub fn parked(ctx: &egui::Context) -> RunPick {
    ctx.data_mut(|d| {
        d.get_temp::<RunPick>(egui::Id::new(PICK_MEMORY_KEY))
            .unwrap_or_default()
    })
}

/// Record what the menu resolved to, once per click.
///
/// Called by [`crate::canvas::menus`] on the frame of the secondary click. See
/// [`TRACE_MENU`] for why the *pick* is in it and not only the verdict.
pub fn trace(pick: RunPick) {
    crate::diag::trace(move || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!(
            "{TRACE_MENU} pick={} offered={}",
            pick.word(),
            pick.offered()
        )
    });
}

/// **The `(object, run)` a pressed row should select**, or `None` if it cannot.
///
/// # ★★ Why the pick is re-validated here and not trusted from the menu
///
/// The row was drawn from [`pick_at`] on some earlier frame, and everything
/// between then and now is a frame in which the document could have changed —
/// an undo, a background reflow, another surface's edit, a page turn.
/// Re-asking the provider costs one lookup on a press and removes the whole
/// class of *"the menu was right when it was drawn"*.
///
/// ⇒ A pick that has stopped being valid raises **nothing**, and the trace
/// says so. It does not raise a refusal sentence: the operator pressed a row
/// that has quietly stopped applying, which is a first-order rarity, and a
/// status line arriving after a menu closed would be a second explanation for
/// an event with no first one.
///
/// `page` is the page the canvas is showing **now**. A pick taken on another
/// page is refused rather than applied to the same index on this one — which
/// is *"the right verb on the wrong thing"*, one axis over.
#[must_use]
pub fn resolve(
    ctx: &egui::Context,
    targets: Option<&ObjectModelProvider>,
    page: usize,
) -> Option<(TargetId, usize)> {
    let pick = parked(ctx);
    let outcome = resolved(pick, targets, page);
    crate::diag::trace(move || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!(
            "{TRACE_COMMAND} pick={} outcome={}",
            pick.word(),
            if outcome.is_some() {
                "raised"
            } else {
                "declined"
            }
        )
    });
    outcome
}

/// The decision behind [`resolve`], with the trace lifted out so the function
/// above has one exit and one trace line rather than four of each.
fn resolved(
    pick: RunPick,
    targets: Option<&ObjectModelProvider>,
    page: usize,
) -> Option<(TargetId, usize)> {
    let RunPick::Run {
        page: picked_page,
        object,
        run,
        ..
    } = pick
    else {
        return None;
    };
    if picked_page != page {
        return None;
    }
    let targets = targets?;
    if targets.part_kind_of(object) != Some(PartKind::Run) {
        return None;
    }
    // ★ Bounds-checked against the CURRENT run count, not against the `of`
    // parked with the pick. A reflow between the click and the press can
    // shorten the object, and an index past the end is exactly the operand
    // `delete_text_run` would refuse — after the selection outline had already
    // moved somewhere the operator did not point.
    let index = object.page_object_index()?;
    (run < targets.text_run_count(index)).then_some((object, run))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **R9's one boolean.** A pick on a run is offered; nothing else is.
    ///
    /// ★ Falsified: defining `offered` as `true` makes the second assertion
    /// fail, which is the regression that would draw the row over paths,
    /// images and blank paper.
    #[test]
    fn only_a_run_pick_is_offered() {
        assert!(
            RunPick::Run {
                page: 0,
                object: TargetId::Object(3),
                run: 1,
                of: 14,
            }
            .offered()
        );
        assert!(!RunPick::Elsewhere.offered());
    }

    /// The pick's default is *nowhere near a line*, so a frame before any
    /// right-click cannot be read as "run 0 of object 0".
    ///
    /// ★ Falsified: making a `Run` variant the default makes this fail, and
    /// would have offered the row on every right-click anywhere on the canvas
    /// before the operator had pointed at anything.
    #[test]
    fn the_default_pick_names_no_line() {
        assert_eq!(RunPick::default(), RunPick::Elsewhere);
    }

    /// **A pick taken on another page does not resolve.**
    ///
    /// The provider is `None` here, which is a second reason to decline — so
    /// the page guard is asserted where it is the FIRST reason, above the
    /// `targets?`. That ordering is load-bearing, which is why the second
    /// assertion exists: it shows the two refusals are independent rather than
    /// one of them covering for the other.
    ///
    /// ★ Falsified: moving the page comparison below `let targets = targets?`
    /// leaves the first assertion passing for the wrong reason.
    #[test]
    fn a_pick_from_another_page_is_refused() {
        let pick = RunPick::Run {
            page: 7,
            object: TargetId::Object(3),
            run: 1,
            of: 14,
        };
        assert_eq!(resolved(pick, None, 2), None);
        assert_eq!(resolved(pick, None, 7), None);
    }

    /// `Elsewhere` resolves to nothing even with a matching page.
    ///
    /// ★ Falsified: an `_ =>` arm in [`resolved`] falling through to
    /// `Some((TargetId::Object(0), 0))` makes this fail — the shape of the
    /// *"unwrap_or_default into run 0"* defect the enum's two variants exist
    /// to prevent.
    #[test]
    fn an_elsewhere_pick_resolves_to_nothing() {
        assert_eq!(resolved(RunPick::Elsewhere, None, 0), None);
    }

    /// The trace word carries **both** numbers, because *"the right verb on
    /// the wrong line"* is the defect class this design can produce, and a
    /// check that could only see `run` could not tell a wrong pick from a
    /// short object.
    ///
    /// ★ Falsified: dropping `of` from the format string makes this fail.
    #[test]
    fn the_trace_word_names_the_line_and_the_total() {
        assert_eq!(
            RunPick::Run {
                page: 0,
                object: TargetId::Object(3),
                run: 2,
                of: 14,
            }
            .word(),
            "run:2/14"
        );
        assert_eq!(RunPick::Elsewhere.word(), "elsewhere");
    }
}
