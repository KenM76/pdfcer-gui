//! # `panels::comments::tests` — the Comments panel's own assertions
//!
//! Split out of [`super`] on 2026-09-05, when that file reached **1,670
//! lines** against R2's ceiling of 1,500. The seam is the cheapest honest one
//! available and it is the same one `redact`, `save` and `text::redact` took
//! the day before: **the module is the panel, and this is what is asserted
//! about it.**
//!
//! ★ Chosen over splitting the drawing code because the drawing genuinely is
//! one surface — a row is a header, a note, a byline and two controls laid out
//! together, and cutting between them would put the reasoning about a single
//! row in two files. The tests have no such coupling: each one names its own
//! subject. R2's rule is *find the seam*, and this is where the file actually
//! comes apart.
//!
//! ⚠ **`#![cfg(test)]` on the first line**, not `#[cfg(test)] mod tests`
//! around the contents. `tools/gates/check-ui-strings.sh` recognises a
//! test-only file by that inner attribute; without it every assertion message
//! in here would be read as an operator-visible literal outside the catalog
//! and the gate would fail on a file that shows the operator nothing.
#![cfg(test)]

/// ★★★ **A READING STANCE OFFERS NO CONTROL THAT WRITES.**
///
/// # The defect, and how it was actually found
///
/// On 2026-09-05 the Delete button and the note editor were both drawn,
/// **live and effective, in Read** — the mode whose entire stated posture
/// is *the document is not yours to alter*. `deletable` asked
/// `EditSession::annotation_deletion_refusal`, which answers *"would the
/// engine refuse this document?"* (encrypted, certified) and says nothing
/// whatever about the operator's stance. Nothing else asked either.
///
/// It was found by launching the release binary **off screen** on the
/// comment fixture and reading its trace:
///
/// ```text
/// mode-changed to=read panels=4
/// comments-panel listed=3 with_note=3 authors=3 replies=1
/// ui-rect name=comments.note_edit rect=[[1086.0 347.0] - [1146.9 365.0]]
/// ui-rect name=comments.delete    rect=[[1133.7 368.0] - [1239.0 386.0]]
/// ```
///
/// At that moment forty-six tests over this panel passed, all twenty-nine
/// gates were green, and the ribbon comparison exited 0. **R1 is the rule
/// this illustrates**: a green suite is not a report of working software.
///
/// # Why the older tests could not have caught it, and why this one can
///
/// None of them enters a *mode*. They call the panel with an `OpenDoc` and
/// no stance at all, and `canvas::tool::capabilities` falls back to
/// `Capabilities::FULL` for an unset `Context` — deliberately, and
/// correctly, since a build with no validated manifest must not silently
/// withhold everything. So every existing test ran as though it were in
/// Edit and could not have seen this.
///
/// ★★ And a predicate test would have been worse than none: this project's
/// standing lesson is that **a unit test which calls the verb cannot see
/// the chain in front of it** — eight green tests once passed while the
/// feature did one of fourteen things. A test of a
/// `should_offer_delete(caps)` helper would have passed on the exact build
/// that never called it. So this drives the real `body` through
/// `Context::run_ui` and counts what was actually **drawn**.
///
/// # Both stances, deliberately
///
/// An absence assertion alone is vacuous — it passes on a panel that draws
/// nothing at all, on a fixture with no comments, or on a build where the
/// count is never incremented. The Review half is the positive control: it
/// proves the fixture has rows, that the count rises, and therefore that
/// the Read half is measuring a real absence rather than an empty room.
#[test]
fn a_reading_stance_offers_no_control_that_writes_to_the_document() {
    use crate::app::modes::Capabilities;

    fn writing_controls_in(caps: Capabilities) -> u32 {
        let ctx = egui::Context::default();
        crate::canvas::tool::store_capabilities(&ctx, caps);
        // The engine's own threaded-comment fixture — real annotations
        // with real authors, which is what makes the positive control
        // below a control rather than a formality.
        let doc = crate::app::state::open_fixture("annot/thread.pdf");
        let mut state = crate::panels::PanelsState::default();
        let mut actions = Vec::new();
        let mut drawn = 0;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(360.0, 900.0),
            )),
            ..Default::default()
        };
        // Two frames, for `dimension_groups`' reason: an immediate-mode
        // layout's first pass is a guess, and the scroll area's own size
        // settles on the second.
        for _ in 0..2 {
            let _ = ctx.run_ui(input.clone(), |ui| {
                body(ui, &doc, &mut state, &mut actions);
                drawn = state.comments_mut().writing_controls_drawn;
            });
        }
        drawn
    }

    let review = writing_controls_in(Capabilities::FULL);
    assert!(
        review > 0,
        "the positive control drew nothing, so the Read assertion below \
         would pass on an empty panel and prove nothing"
    );

    // Read's stance, spelled as the capability rather than the mode name:
    // Capabilities::for_mode derives this from the mode's tab list,
    // and a test naming "read" would be asserting against a string the
    // manifest owns rather than against the property that matters.
    let read = writing_controls_in(Capabilities::NONE);
    assert_eq!(
        read, 0,
        "a reading stance drew {read} control(s) that write to the \
         document — Read may READ a comment somebody else wrote, and may \
         not delete it or retype it"
    );
}

use super::*;
use crate::panels::Panel;

/// A row with the author this test is about and nothing else that matters.
///
/// Built by hand rather than by `model::collect`, because the subject is a
/// decision about **one field** and collecting a row would make the test
/// depend on a document, an annotation and a walk — three things that can
/// fail for reasons this assertion is not about.
fn row_by(author: Option<&str>) -> CommentRow {
    CommentRow {
        page_index: 0,
        id: Some(pdfcer_core::object::ObjId {
            num: 7,
            generation: 0,
        }),
        subtype: "Square".to_owned(),
        is_ce_dimension: false,
        note: Note::Absent,
        author: author.map(str::to_owned),
        modified: None,
        suppressed: false,
        appearance_unresolved: false,
        relation: None,
    }
}

/// ★★★ **Correcting somebody else's typo must not re-attribute their
/// comment.**
///
/// The mistake `pdfcer-core` warned about by name when it shipped
/// `set_markup_note`: writing all three keys unconditionally *"would
/// silently strip the author and date on every correction, leaving a review
/// comment from nobody, dated never, looking exactly like a note somebody
/// else had mangled."*
///
/// `true` here means the action sends **no `/T`**, which is what leaves the
/// existing one alone.
#[test]
fn a_note_with_an_author_keeps_it() {
    assert!(keeps_author(&row_by(Some("Ken Mantle"))));
}

/// The other half, and it is the half that makes the first one mean
/// something: a shape this shell drew has no byline, so a note written onto
/// it is **ours to sign**.
///
/// Asserting only the preservation case would pass on an implementation
/// that never writes `/T` at all — every comment anonymous, which is the
/// same defect wearing the other value.
#[test]
fn a_note_with_no_author_is_ours_to_sign() {
    assert!(!keeps_author(&row_by(None)));
}

/// ★ Whitespace is absent. A producer writing `/T ()` or `/T ( )` leaves a
/// byline nobody wrote, and preserving it would credit the comment to a
/// space — while the row's own byline, which trims the same way, would show
/// nothing at all. Two surfaces, one rule.
#[test]
fn a_blank_author_is_no_author() {
    assert!(!keeps_author(&row_by(Some(""))));
    assert!(!keeps_author(&row_by(Some("   "))));
}

use crate::shell::{commands, manifest};
use egui_shell::CommandRegistry;
use std::collections::BTreeSet;

/// **★ The command that opens this panel exists and is on the ribbon.**
///
/// The check three panels in the old shell shipped without: they had a
/// body, a rail entry and a diagnostic step, and *"no control an operator
/// could click"*, so every verification passed while they were unreachable
/// in a real build.
///
/// Two assertions, and both are needed. A command **the manifest
/// references** is one the ribbon draws a control for; a command **the
/// registry holds** is one that has a label, a tooltip and an enable
/// predicate. Either alone is half a control.
///
/// `crate::panels::tests::every_panel_is_reachable_from_the_ribbon` sweeps
/// the same property across every panel; this one names *this* panel in
/// its failure message, which is what a reader who has just added it
/// wants to see.
#[test]
fn the_comments_command_is_reachable_from_the_ribbon() {
    let shell = manifest::built_in();
    let mut registry = CommandRegistry::new();
    commands::register(&mut registry);
    let referenced: BTreeSet<String> = shell
        .command_references()
        .into_iter()
        .map(|(_, id)| id)
        .collect();

    assert!(
        referenced.contains(COMMAND_ID),
        "no tab, QAT slot or key binding references `{COMMAND_ID}`, so an \
         operator cannot open the Comments panel. `RIBBON_IA.md` §7 puts it \
         on Markup ▸ Comments."
    );
    assert!(
        registry.get(COMMAND_ID).is_some(),
        "`{COMMAND_ID}` is not registered, so the ribbon has an id with no \
         label, no tooltip and no enable predicate, and draws nothing for it."
    );
}

/// **The panel and this module name the same command.**
///
/// Two spellings of one id is two things to keep in step, and the failure
/// when they drift is a panel that opens from the ribbon and draws nothing
/// in the dock — which looks like a rendering bug and is not.
#[test]
fn the_panel_enum_and_this_module_agree() {
    assert_eq!(Panel::Comments.command_id(), COMMAND_ID);
}

/// **★ The page index travels 0-based and prints 1-based.**
///
/// The off-by-one that would otherwise be invisible.
/// [`crate::app::actions::Action::GoToPage`] takes a 0-based index — the
/// same convention `crate::panels::bookmarks` pins from its own side — and
/// every string a human reads takes the number one higher. Getting it
/// backwards produces a panel that navigates one page past every comment,
/// which looks like a document defect.
///
/// Asserted against a real fixture rather than a constructed row, so the
/// indices are ones the collector actually produced.
#[test]
fn the_page_index_travels_zero_based_and_prints_one_based() {
    use crate::panels::objects::test_support::engine_fixture;

    let path = engine_fixture("annot/thread.pdf");
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    let session = pdfcer_core::edit::EditSession::new(doc);
    let listing = model::collect(
        &session.view(),
        &pages,
        &model::ce_dimension_annots(&session),
    );
    assert!(
        !listing.rows.is_empty(),
        "the fixture must carry annotations, or this test proves nothing"
    );

    for comment in &listing.rows {
        // What the row would push …
        let action = Action::GoToPage(comment.page_index);
        assert_eq!(action, Action::GoToPage(comment.page_index));
        // … and what it prints, which is one higher, in both the heading
        // and the button's tooltip.
        let human = comment.page_index + 1;
        let heading = t::comment_row_heading(&comment.subtype, human);
        assert!(heading.contains(&human.to_string()), "{heading}");
        let tip = t::comment_row_goto_tooltip(human);
        assert!(tip.contains(&human.to_string()), "{tip}");
    }
}

/// ★★★ **The Delete this panel now offers actually reaches the engine.**
///
/// # Why this is a test and not a paragraph
///
/// Because the paragraph it replaces was **wrong for three weeks** and
/// nothing could tell. This module's header said *"there is no Delete,
/// because `Action` has no variant that could carry the intent"* while the
/// variant, the dispatch arm and the engine verb all existed. Prose cannot
/// go red.
///
/// `RESUME.md` states the rule this is an instance of — *"a sentence about
/// what the engine cannot do is a dated citation with a shelf life
/// measured in hours … where the claim can be an assertion, make it one"* —
/// and it names the day a unit test asserting such a claim went red the
/// moment the engine shipped, *"which is the behaviour a paragraph cannot
/// have."*
///
/// # What it asserts, in the order the panel does it
///
/// 1. the fixture carries an addressable annotation — otherwise the rest
///    proves nothing;
/// 2. `annotation_deletion_refusal` says the document permits deletion,
///    which is the predicate [`delete_control`] gates the button on;
/// 3. `delete_annotation` **succeeds** on it;
/// 4. and the annotation is **gone from the session's own view** — not
///    merely that the call returned `Ok`. A verb that reported success and
///    changed nothing is the exact failure a return-value check cannot
///    see, and this project has been bitten by *"the verb did nothing"*
///    twice.
#[test]
fn the_delete_control_reaches_the_engine() {
    use crate::panels::objects::test_support::engine_fixture;

    let path = engine_fixture("annot/thread.pdf");
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    let mut session = pdfcer_core::edit::EditSession::new(doc);

    let listing = model::collect(
        &session.view(),
        &pages,
        &model::ce_dimension_annots(&session),
    );
    let target = listing
        .rows
        .iter()
        .find_map(|row| row.id)
        .expect("the fixture must carry an addressable annotation");

    assert!(
        session.annotation_deletion_refusal().is_none(),
        "the fixture refuses deletion document-wide, so this test would \
         pass on a build with no Delete at all"
    );

    session
        .delete_annotation(target)
        .expect("`delete_annotation` refused the annotation the panel offers Delete on");

    let after = model::collect(
        &session.view(),
        &pages,
        &model::ce_dimension_annots(&session),
    );
    assert!(
        !after.rows.iter().any(|row| row.id == Some(target)),
        "the engine reported success and the annotation is still listed — \
         the panel's Delete would leave the row on screen"
    );
}

/// **A ce dimension's heading names it as one and keeps the subtype.**
///
/// Rule 15 at the point of use. The bracketed `/Line` is not decoration:
/// the exclusion argument in this module's header turns on ce dimensions
/// *being* `/Line` annotations, and a heading that hid that would quietly
/// contradict the argument that put the row in the list.
#[test]
fn a_ce_dimension_row_says_ce_dimension_and_still_says_line() {
    let heading = t::comment_row_ce_dimension_heading("Line", 3);
    assert!(heading.contains("ce dimension"), "{heading}");
    assert!(heading.contains("Line"), "{heading}");
    // …and an ordinary `/Line` markup is not relabelled.
    let plain = t::comment_row_heading("Line", 3);
    assert!(!plain.contains("dimension"), "{plain}");
}
