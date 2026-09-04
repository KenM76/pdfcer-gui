#![cfg(test)]
//! # `panels::tests` — the reachability sweep and the width arithmetic
//!
//! Split out of [`super`] on 2026-09-04, when `elide_to_width` took that file
//! past R2's 1,500-line cap (`tools/gates/check-file-size.sh`). **A move, not a
//! rewrite**: every test below is byte-identical to the one that was inside
//! `mod tests` in `panels/mod.rs`, de-indented one level, plus the new
//! ellipsis tests at the end.
//!
//! ★ `#![cfg(test)]` is the FIRST line, an inner attribute on the file rather
//! than an outer `#[cfg(test)]` on the `mod` line. Both work; this one is the
//! shape the project uses, because it keeps the condition in the file it
//! conditions — a `mod tests;` in the parent with no attribute would compile
//! the file into release builds the day somebody forgets the attribute at the
//! declaration.

use super::*;

use crate::shell::{commands, manifest};
use egui_shell::CommandRegistry;
use std::collections::BTreeSet;

/// **★ Every panel is reachable from the ribbon.**
///
/// The check three panels shipped without. The old shell's
/// `panels_structure.rs` header records what that cost:
///
/// > All three shipped with a `PaneSubject`, a panel body, a rail entry
/// > and a diagnostic step — and no control an operator could click.
/// > Their only callers were the harness step handlers, so every
/// > verification passed while the panels were unreachable in a real
/// > build.
///
/// Two assertions per panel, and both are needed. A command **the
/// manifest references** is one the ribbon draws a control for; a
/// command **the registry holds** is one that has a label, a tooltip and
/// an enable predicate. Either alone is half a control: an id on a tab
/// with no registration renders nothing, and a registration nothing
/// references is an orphan (`crate::shell`'s own
/// `no_registered_command_is_orphaned` catches the second direction from
/// the other side).
///
/// This is deliberately stronger than the gate it replaces, which read
/// `main.rs` as a **string** and looked for a `show_pane_subject(…)`
/// substring outside the harness function. A substring search cannot
/// tell a live call from one inside a `#[cfg(test)]` block, and it
/// silently stops working the day the call is spelled differently. This
/// one asks the same data the ribbon draws itself from.
#[test]
fn every_panel_is_reachable_from_the_ribbon() {
    let shell = manifest::built_in();
    let mut registry = CommandRegistry::new();
    commands::register(&mut registry);
    let referenced: BTreeSet<String> = shell
        .command_references()
        .into_iter()
        .map(|(_, id)| id)
        .collect();

    for panel in Panel::ALL {
        let id = panel.command_id();
        assert!(
            referenced.contains(id),
            "{panel:?} names the command `{id}`, and no tab, QAT slot or key \
             binding references it. An operator cannot open this panel. \
             Give it a control in `shell::manifest`, or remove the panel."
        );
        assert!(
            registry.get(id).is_some(),
            "{panel:?} names the command `{id}`, which is not registered — so \
             the ribbon has an id with no label, no tooltip and no enable \
             predicate, and draws nothing for it."
        );
    }
}

/// **No two panels claim the same command.**
///
/// A shared id would make the reachability test above pass for both
/// while only one of them could ever be opened — the failure hiding
/// inside the fix. It is a live hazard rather than a hypothetical: two
/// of the nine panels are commissioned by one `RIBBON_IA.md` sentence
/// (`file.properties` names both the document's metadata and the
/// selection's properties), and the temptation to hang both off that one
/// id is exactly what this refuses.
#[test]
fn no_two_panels_share_a_command() {
    let mut seen: Vec<&str> = Vec::new();
    for panel in Panel::ALL {
        let id = panel.command_id();
        assert!(
            !seen.contains(&id),
            "{panel:?} claims `{id}`, which another panel already claims. \
             One command opens one panel."
        );
        seen.push(id);
    }
}

/// **The hand-written catalog is exhaustive.**
///
/// [`Panel::ALL`] is an array, and an array cannot notice a new variant.
/// The `match` below can: it has no catch-all arm, so adding a variant
/// to [`Panel`] fails to compile until it is listed here, and the length
/// assertion then fails until it is added to `ALL`. That chain is what
/// makes a hand-written enumeration self-defending, and it matters
/// because every sweep in this module — reachability included — is only
/// as complete as `ALL`.
#[test]
fn the_panel_catalog_is_complete() {
    // Exhaustive by construction: no `_` arm.
    const fn ordinal(p: Panel) -> usize {
        match p {
            Panel::Bookmarks => 0,
            Panel::Layers => 1,
            Panel::Signatures => 2,
            Panel::Fonts => 3,
            Panel::Objects => 4,
            Panel::Properties => 5,
            Panel::Forms => 6,
            Panel::Pages => 7,
            Panel::Comments => 8,
            Panel::Redact => 9,
            Panel::DimensionGroups => 10,
            Panel::Attachments => 11,
        }
    }
    let mut ordinals: Vec<usize> = Panel::ALL.iter().copied().map(ordinal).collect();
    ordinals.sort_unstable();
    ordinals.dedup();
    assert_eq!(
        ordinals,
        (0..Panel::ALL.len()).collect::<Vec<_>>(),
        "Panel::ALL is missing a variant, or lists one twice"
    );
}

/// **★ The container width exceeds the viewport when a row is wider —
/// which is the whole of the no-clipping fix.**
///
/// If this returned the viewport width, `ScrollArea` would compare
/// content against viewport, find them equal, draw no bar, and the row
/// would be cut off at the panel's edge with nothing to say so. That is
/// the exact defect the Objects panel had, and it is why this is a pure
/// function rather than three lines inside a closure.
#[test]
fn a_row_wider_than_the_viewport_widens_the_container() {
    // The measured case: a 600 pt row in a 370 pt dock pane.
    assert!((content_width([600.0], 370.0) - 600.0).abs() < f32::EPSILON);
    // The widest row wins, not the last or the first.
    assert!((content_width([120.0, 600.0, 90.0], 370.0) - 600.0).abs() < f32::EPSILON);
}

/// …and it fills the viewport when every row is narrower, rather than
/// leaving a dead strip.
#[test]
fn narrow_rows_still_fill_the_panel() {
    assert!((content_width([120.0, 90.0], 370.0) - 370.0).abs() < f32::EPSILON);
    // No rows at all — an empty page — is the viewport, not zero.
    assert!((content_width(std::iter::empty(), 370.0) - 370.0).abs() < f32::EPSILON);
}

/// A non-finite measurement is ignored rather than poisoning the width.
///
/// `f32::max` propagates `NaN` in one direction and swallows it in the
/// other depending on argument order, and a `NaN` container width makes
/// egui lay nothing out at all — a blank panel, which reads as a crash.
/// Filtering is cheaper than reasoning about which way round it went.
#[test]
fn a_non_finite_row_width_cannot_blank_the_panel() {
    let w = content_width([f32::NAN, 500.0, f32::INFINITY], 370.0);
    assert!(w.is_finite(), "container width went non-finite: {w}");
    assert!((w - 500.0).abs() < f32::EPSILON);
}

/// The focus is a toggle, so a row click is its own undo.
///
/// With no selection model there is no Escape ladder and no other route
/// back to "nothing focused". A panel an operator cannot get out of is
/// worse than one they cannot get into.
#[test]
fn clicking_the_focused_row_again_clears_the_focus() {
    let mut state = PanelsState::default();
    assert_eq!(state.focus(), None);
    state.set_focus(7);
    assert_eq!(state.focus(), Some(7));
    state.set_focus(9);
    assert_eq!(state.focus(), Some(9), "a different row moves the focus");
    state.set_focus(9);
    assert_eq!(state.focus(), None, "the same row clears it");
}

/// **★ The panel focus has not quietly become a selection.**
///
/// [`ObjectTreeUi::focus`]'s own docs say the field is **deleted** when
/// the real selection model lands, not extended — because two selections
/// that have to be kept in step will drift, and the drift is invisible
/// until an edit acts on the wrong object.
///
/// The danger is not that someone renames it in one commit. It is that it
/// grows into one an attribute at a time — a second index here, surviving
/// a page change there — until deleting it is a refactor nobody wants to
/// start. So the four properties that make it *not* a selection are
/// asserted directly, and the canvas's real selection model landing in
/// this same stage is exactly why they are asserted now:
///
/// 1. **Single-valued.** A selection is a set; this is one `Option`.
/// 2. **Does not survive a page change.** A selection is document-scoped;
///    a paint-order index is a position on one page.
/// 3. **Does not survive an edit.** Deleting one object renumbers every
///    object after it, so a retained index describes a different object.
/// 4. **Read by one panel, and drives nothing else.** No enable
///    predicate reads it. `crate::app::PdfcerApp::conditions` **does**
///    set `selection.any` as of S4 — but from `OpenDoc::selection`, the
///    canvas's real selection, and never from this focus. That is the
///    distinction this test defends: the two look alike, and the day
///    someone wires the condition to whichever one is nearest, a row
///    highlighted in a panel starts arming a destructive command.
#[test]
fn the_panel_focus_has_not_quietly_become_a_selection() {
    use crate::app::state::OpenDoc;
    use crate::panels::objects::test_support::engine_fixture;

    let mut state = PanelsState::default();
    let path = engine_fixture("pageops/four-pages.pdf");
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    let mut open = OpenDoc::new(path, pdfcer_core::edit::EditSession::new(doc), pages);

    // Property 1 is structural: `focus()` returns `Option<usize>`, so a
    // second focused row is not representable. Asserted by use — a set
    // would not compile on the left of this binding.
    state.sync(&open);
    state.set_focus(2);
    state.tree_mut().toggle_object(2);
    let focused: Option<usize> = state.focus();
    assert_eq!(focused, Some(2), "state within one page survives `sync`");

    // 2. A page change forgets it.
    open.view.page_index = 1;
    state.sync(&open);
    assert_eq!(
        state.focus(),
        None,
        "a paint-order index is a position on ONE page; carrying it \
         across a page step would describe a different object"
    );
    assert!(state.tree_mut().objects_expanded.is_empty());

    // 3. An edit forgets it, without moving page.
    state.set_focus(2);
    open.edit_epoch = 1;
    state.sync(&open);
    assert_eq!(
        state.focus(),
        None,
        "an edit renumbers every object after the one it touched"
    );

    // 4. Setting the focus arms nothing. A real selection now exists —
    //    `format.delete` is gated on `selection.any` — which makes this
    //    the live hazard rather than a hypothetical one: if the focus
    //    ever came to satisfy that predicate, clicking a row in a REPORT
    //    panel would enable a destructive command, and the operator
    //    would have no way to tell which of the two "selections" it was
    //    about to act on.
    //
    //    Asserted through the enable machinery itself: a `ConditionSet`
    //    that has seen nothing but a focus must not satisfy the
    //    predicate that gates those commands.
    let mut registry = CommandRegistry::new();
    commands::register(&mut registry);
    let mut conditions = egui_shell::commands::ConditionSet::new();
    conditions.set("doc.open");
    conditions.set("doc.pages");
    state.set_focus(2);
    let armed: Vec<&str> = registry
        .iter()
        .filter(|c| format!("{:?}", c.enable).contains("selection"))
        .filter(|c| c.is_enabled(&conditions))
        .map(|c| c.id.as_str())
        .collect();
    assert!(
        armed.is_empty(),
        "a selection-gated command is enabled by a document being open \
         and a panel row being focused: {armed:?}. The Objects panel's \
         focus is not a selection and must not arm one."
    );
}

// ===========================================================================
// The ellipsis decision — `OPERATOR_REQUESTS.md` O123
// ===========================================================================

/// A measurer with one point per character, so a test can state widths in
/// characters and read like the thing it is asserting.
///
/// ★ Deliberately NOT a real font. The property under test is the *decision*
/// — does this row need shortening, and to what — and a real font would make
/// every expected value a measurement nobody could check by reading.
fn per_char(text: &str) -> f32 {
    text.chars().count() as f32
}

/// **A row that fits is not touched.**
///
/// The commonest case by far, and the one an over-eager implementation gets
/// wrong by always appending an ellipsis.
#[test]
fn a_row_that_fits_is_returned_unchanged() {
    assert_eq!(elide_to_width("#12 Path", 100.0, per_char), None);
    // And exactly at the boundary: eight characters into eight points fits.
    assert_eq!(elide_to_width("#12 Path", 8.0, per_char), None);
}

/// ★★★ **A row that does not fit comes back shortened, ending in the ellipsis,
/// and no wider than the space it was given.**
///
/// The three conditions together are the whole contract. Asserting only the
/// first would pass on an implementation that returned the label untouched;
/// asserting only the last would pass on one that returned an empty string.
#[test]
fn a_row_that_overflows_is_shortened_to_fit() {
    let long = "#1382 Path filled and stroked #1A73E8, 0.50 pt wide";
    let out = elide_to_width(long, 20.0, per_char).expect("the row overflows 20 pt");
    assert!(
        out.ends_with(ELLIPSIS),
        "{out:?} does not end in an ellipsis"
    );
    assert!(
        per_char(&out) <= 20.0,
        "{out:?} is {} pt wide in a 20 pt pane",
        per_char(&out)
    );
    assert!(
        long.starts_with(&out[..out.len() - ELLIPSIS.len_utf8()]),
        "{out:?} is not a prefix of the row it shortened"
    );
}

/// ★★ **It keeps as much as it can**, which is what makes the shortened row
/// worth reading.
///
/// A correct-but-useless implementation returns the bare ellipsis every time
/// and satisfies every assertion above. This one pins the search: at 20 pt with
/// one point per character, nineteen characters plus the ellipsis is the answer,
/// and twenty would not fit.
#[test]
fn it_keeps_every_character_that_fits() {
    let long = "abcdefghijklmnopqrstuvwxyz0123456789";
    let out = elide_to_width(long, 20.0, per_char).expect("36 characters overflow 20 pt");
    assert_eq!(out.chars().count(), 20, "{out:?} left room unused");
    assert_eq!(out, "abcdefghijklmnopqrs\u{2026}");
}

/// ★ **Multi-byte characters are never split.**
///
/// Object rows carry the middle dot, the em dash and the multiplication sign,
/// and font names carry accents. A byte-offset slice would panic here rather
/// than misbehave, which is the good news; this test is what stops a future
/// "optimisation" to `&label[..n]` from reaching the operator.
#[test]
fn a_multi_byte_row_is_never_split_mid_character() {
    let row = "#25 Path \u{b7} filled \u{d7} stroked \u{2014} 3 nodes";
    let out = elide_to_width(row, 12.0, per_char).expect("the row overflows");
    assert!(row.starts_with(&out[..out.len() - ELLIPSIS.len_utf8()]));
    assert!(out.chars().count() <= 12);
}

/// **A pane with no width makes no decision.**
///
/// One frame of a zero-width pane is not a layout decision worth taking, and
/// returning `Some("\u{2026}")` for it would put an ellipsis in every row of a
/// panel that is merely mid-animation.
#[test]
fn a_pane_with_no_width_leaves_the_row_alone() {
    assert_eq!(elide_to_width("#12 Path", 0.0, per_char), None);
    assert_eq!(elide_to_width("#12 Path", -5.0, per_char), None);
    assert_eq!(elide_to_width("#12 Path", f32::NAN, per_char), None);
}

/// A pane too narrow for even one character still gets a row, and the row still
/// says there is something in it.
#[test]
fn an_impossibly_narrow_pane_still_gets_an_ellipsis() {
    assert_eq!(
        elide_to_width("#12 Path", 1.0, per_char),
        Some("\u{2026}".to_owned())
    );
}
