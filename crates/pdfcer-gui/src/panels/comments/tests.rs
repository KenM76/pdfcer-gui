//! # `panels::comments::tests` — the Comments panel's own assertions
//!
//!
//! Chosen over splitting the drawing code because the drawing genuinely is
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/comments/tests.md`.
#![cfg(test)]

/// **A READING STANCE OFFERS NO CONTROL THAT WRITES.**
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
/// And a predicate test would have been worse than none: this project's
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
        in_reply_to: None,
        rich: None,
    }
}

/// **Correcting somebody else's typo must not re-attribute their
/// comment.**
#[test]
fn a_note_with_an_author_keeps_it() {
    assert!(keeps_author(&row_by(Some("Ken Mantle"))));
}

/// The other half, and it is the half that makes the first one mean
/// something: a shape this shell drew has no byline, so a note written onto
/// it is **ours to sign**.
#[test]
fn a_note_with_no_author_is_ours_to_sign() {
    assert!(!keeps_author(&row_by(None)));
}

/// Whitespace is absent. A producer writing `/T ()` or `/T ( )` leaves a
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

/// **The command that opens this panel exists and is on the ribbon.**
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
#[test]
fn the_panel_enum_and_this_module_agree() {
    assert_eq!(Panel::Comments.command_id(), COMMAND_ID);
}

/// **The page index travels 0-based and prints 1-based.**
#[test]
fn the_page_index_travels_zero_based_and_prints_one_based() {
    use crate::panels::objects::test_support::engine_fixture;

    let path = engine_fixture("annot/thread.pdf");
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    let session = pdfcer_core::edit::EditSession::new(doc);
    let listing = model::collect(
        &session.view(),
        session.view().source(),
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

/// **The Delete this panel now offers actually reaches the engine.**
#[test]
fn the_delete_control_reaches_the_engine() {
    use crate::panels::objects::test_support::engine_fixture;

    let path = engine_fixture("annot/thread.pdf");
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    let mut session = pdfcer_core::edit::EditSession::new(doc);

    let listing = model::collect(
        &session.view(),
        session.view().source(),
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
        session.view().source(),
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
#[test]
fn a_ce_dimension_row_says_ce_dimension_and_still_says_line() {
    let heading = t::comment_row_ce_dimension_heading("Line", 3);
    assert!(heading.contains("ce dimension"), "{heading}");
    assert!(heading.contains("Line"), "{heading}");
    // …and an ordinary `/Line` markup is not relabelled.
    let plain = t::comment_row_heading("Line", 3);
    assert!(!plain.contains("dimension"), "{plain}");
}

// ===========================================================================
// ANSWERING A COMMENT — `EditSession::add_reply`, `Pass 253.0`
// ===========================================================================

/// **A blank reply is not offered, and a written one is.**
#[test]
fn a_reply_is_postable_only_when_it_says_something() {
    assert!(reply_is_postable("Done - rev C issued 5 Sep"));
    // The positive case has to include one with surrounding space, or an
    // implementation that trimmed the text it POSTS as well would pass while
    // silently editing the operator's words.
    assert!(reply_is_postable("  agreed  "));

    assert!(!reply_is_postable(""));
    assert!(!reply_is_postable("   "));
    assert!(!reply_is_postable("\n\t "));
}

/// **A reply row's *Go to* opens the ROOT's window, not its own.**
#[test]
fn a_reply_resolves_to_the_comment_at_the_head_of_its_thread() {
    use pdfcer_core::object::ObjId;
    let id = |num: u32| ObjId { num, generation: 0 };
    let reply_to = |num: u32, parent: Option<u32>| CommentRow {
        id: Some(id(num)),
        in_reply_to: parent.map(id),
        rich: None,
        ..row_by(None)
    };

    // 7 ← 8 ← 9: a comment, an answer, and an answer to the answer.
    let rows = vec![
        reply_to(7, None),
        reply_to(8, Some(7)),
        reply_to(9, Some(8)),
    ];
    assert_eq!(model::thread_root(&rows, id(9)), id(7));
    assert_eq!(model::thread_root(&rows, id(8)), id(7));
    // The control: an ordinary comment resolves to itself.
    assert_eq!(model::thread_root(&rows, id(7)), id(7));
}

/// **A cyclic `/IRT` terminates.**
#[test]
fn a_malformed_thread_resolves_to_something_real_rather_than_hanging() {
    use pdfcer_core::object::ObjId;
    let id = |num: u32| ObjId { num, generation: 0 };
    let reply_to = |num: u32, parent: Option<u32>| CommentRow {
        id: Some(id(num)),
        in_reply_to: parent.map(id),
        rich: None,
        ..row_by(None)
    };

    // A two-annotation cycle. Whichever end it stops at, it must stop, and it
    // must name one of the two rows that actually exist.
    let cycle = vec![reply_to(7, Some(8)), reply_to(8, Some(7))];
    let root = model::thread_root(&cycle, id(7));
    assert!(
        root == id(7) || root == id(8),
        "a cycle resolved to {root:?}, which is not a row in the document"
    );

    // A reply to itself — refused by `add_reply`, legal to write by hand.
    let selfish = vec![reply_to(7, Some(7))];
    assert_eq!(model::thread_root(&selfish, id(7)), id(7));

    // A `/IRT` pointing at something this list does not contain — a `/Widget`,
    // a `/Popup`, or an object that is not there. The nearest real place is the
    // honest answer for a destination resolver.
    let dangling = vec![reply_to(7, Some(99))];
    assert_eq!(model::thread_root(&dangling, id(7)), id(7));
}

/// **A READING STANCE OFFERS NO REPLY EDITOR EITHER — including when one
/// is already open.**
#[test]
fn a_reading_stance_draws_no_reply_editor_even_with_a_reply_draft_open() {
    use crate::app::modes::Capabilities;

    fn writing_controls_with_a_reply_open(caps: Capabilities) -> u32 {
        let ctx = egui::Context::default();
        crate::canvas::tool::store_capabilities(&ctx, caps);
        let doc = crate::app::state::open_fixture("annot/thread.pdf");
        let listing = model::collect(
            &doc.session.view(),
            doc.session.view().source(),
            &doc.pages,
            &model::ce_dimension_annots(&doc.session),
        );
        let target = listing.rows.iter().find_map(|row| row.id).expect(
            "the fixture must carry an addressable annotation, or this test \
             seeds nothing and proves nothing",
        );

        let mut state = crate::panels::PanelsState::default();
        // Seeded at the document's CURRENT epoch, or `NoteDraft::sync` would
        // drop it on the first frame and the test would assert about a panel
        // with no editor open — passing for the wrong reason.
        state
            .comments_mut()
            .draft
            .begin_reply(target, doc.edit_epoch);
        state.comments_mut().draft.text_mut().push_str("an answer");

        let mut actions = Vec::new();
        let mut drawn = 0;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(360.0, 900.0),
            )),
            ..Default::default()
        };
        for _ in 0..2 {
            let _ = ctx.run_ui(input.clone(), |ui| {
                body(ui, &doc, &mut state, &mut actions);
                drawn = state.comments_mut().writing_controls_drawn;
            });
        }
        drawn
    }

    let review = writing_controls_with_a_reply_open(Capabilities::FULL);
    assert!(
        review > 0,
        "the positive control drew nothing, so the Read assertion below would \
         pass on an empty panel and prove nothing"
    );

    let read = writing_controls_with_a_reply_open(Capabilities::NONE);
    assert_eq!(
        read, 0,
        "a reading stance drew {read} writing control(s) with a reply draft \
         open — Read may read a conversation and may not join one, and the \
         stance check must come before the draft branch rather than after it"
    );
}

// ===========================================================================
// ESCAPE WRITES THE DRAFT — `editor::escape_commits`
// ===========================================================================

/// **Escape writes what was typed.**
#[test]
fn escape_on_an_edited_note_writes_it() {
    let comment = row_by(Some("Ken Mantle"));
    let id = comment.id.expect("the fixture row carries an id");
    let mut draft = super::note::NoteDraft::default();
    draft.begin(id, 0, "");
    draft.text_mut().push_str("check this weld");

    let verb = super::editor::escape_commits(&comment, id, &draft);
    assert!(
        matches!(
            &verb,
            Some(AnnotAction::SetNote { text, .. }) if text == "check this weld"
        ),
        "Escape raised {verb:?} on a note with words in it — it must write \
         them, because nothing else can and the draft is about to be closed"
    );
}

/// The negative that keeps the positive honest: an editor opened and left
/// alone raises **nothing**.
#[test]
fn escape_on_an_untouched_note_writes_nothing() {
    let mut comment = row_by(Some("Ken Mantle"));
    comment.note = Note::Text("check this weld".to_owned());
    let id = comment.id.expect("the fixture row carries an id");
    let mut draft = super::note::NoteDraft::default();
    draft.begin(id, 0, "check this weld");

    assert!(
        super::editor::escape_commits(&comment, id, &draft).is_none(),
        "an untouched editor raised a verb, which spends an undo entry on a \
         document that has not changed"
    );
}

/// A `Note::Description` is compared the same way a `Note::Text` is.
#[test]
fn escape_over_an_untouched_description_writes_nothing() {
    let mut comment = row_by(None);
    comment.note = Note::Description("Opens the drawing index".to_owned());
    let id = comment.id.expect("the fixture row carries an id");
    let mut draft = super::note::NoteDraft::default();
    draft.begin(id, 0, "Opens the drawing index");

    assert!(
        super::editor::escape_commits(&comment, id, &draft).is_none(),
        "a description seeded the editor and then read as an edit, which is \
         the seed and the comparison having come to disagree"
    );
}

/// **Escape posts a reply**, and posts it as a reply.
#[test]
fn escape_on_a_reply_posts_it() {
    let comment = row_by(Some("Jo Smith"));
    let id = comment.id.expect("the fixture row carries an id");
    let mut draft = super::note::NoteDraft::default();
    draft.begin_reply(id, 0);
    draft.text_mut().push_str("agreed");

    let verb = super::editor::escape_commits(&comment, id, &draft);
    assert!(
        matches!(
            &verb,
            Some(AnnotAction::Reply { parent, text }) if *parent == id && text == "agreed"
        ),
        "Escape raised {verb:?} on a reply draft — a reply must reach \
         `add_reply`, and writing `/Contents` instead would silently overwrite \
         the comment it answers"
    );
}

/// A blank reply is the one draft where closing loses nothing, so it raises
/// nothing — the same answer *Post reply* gives by not being drawn at all.
///
/// Whitespace counts as blank, per `reply_is_postable`: a reply containing one
/// space is an annotation nobody can read and nobody meant to author.
#[test]
fn escape_on_a_blank_reply_posts_nothing() {
    let comment = row_by(Some("Jo Smith"));
    let id = comment.id.expect("the fixture row carries an id");
    let mut draft = super::note::NoteDraft::default();
    draft.begin_reply(id, 0);
    draft.text_mut().push_str("   ");

    assert!(
        super::editor::escape_commits(&comment, id, &draft).is_none(),
        "a blank reply was posted — the control that would author it is not \
         even drawn, and the key must not be a second route to what the button \
         refuses"
    );
}
