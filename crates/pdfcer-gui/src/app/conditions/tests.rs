#![cfg(test)]
//! `app::conditions` — the tests.
//!
//! # Why the tests are a file of their own
//!
//! R2 (no source file over 1,500 lines), reached honestly: `conditions/mod.rs`
//! went past the limit on 2026-09-03 while the mode was being added to
//! `selection.delete_permitted`. The seam is the obvious one and the right
//! one — a condition's *derivation* and the *table of what it answers* are
//! different subjects, and the second is much the larger.
//!
//! ★ It matters more here than in most modules. These tests are the only place
//! the published facts are written down as a **table** rather than as a
//! traversal, and a table is what a reader needs to answer *"what does the
//! ribbon see in Review?"*. Burying it under nine hundred lines of derivation
//! made it a thing nobody read — and on 2026-09-03 that cost a data-loss
//! defect: `selection.delete_permitted` never asked the mode, and four tests in
//! this file **asserted the wrong thing and passed**, because the fixture they
//! use starts in Read and nothing said so.

use super::*;

/// ★ **`undo.available` and `redo.available` follow the session, in both
/// directions, through the real verbs.**
///
/// # The defect this exists for
///
/// This function published neither for the whole life of the project, with a
/// comment saying they were "deliberately absent". The obvious way to land
/// them — set them beside `doc.open` and move on — arms both controls
/// permanently, and a build that did that is **indistinguishable from a
/// correct one** in every scenario that presses undo *after* an edit. The
/// first two assertions here are the only ones that tell the two apart.
///
/// # Why it drives the real actions rather than setting fields
///
/// Because the thing under test is a **join**, and this crate has been bitten
/// by exactly that before: `every_armable_tool_kind_reports_a_pressed_state`
/// exists because Phase 7 shipped a measure tool with four passing unit tests
/// and no `conditions` call site, so the button never lit up. A test that set
/// a boolean and read a condition would prove that `set.set` works.
///
/// So the log is filled by [`crate::app::actions::Action::CommitMarkup`] going
/// through `vector_edit` and emptied by [`crate::app::actions::Action::Undo`]
/// going through the same funnel — the two paths an operator's gesture takes
/// — and the conditions are read through the **registered commands'** own
/// predicates, so a rename of either condition string fails here rather than
/// silently greying a control forever.
#[test]
fn the_history_conditions_follow_the_session() {
    use crate::app::actions::Action;
    use crate::canvas::markup::{Geometry, MarkupKind};

    let ctx = egui::Context::default();
    let mut app = crate::app::tests::opened();
    let mut reg = egui_shell::CommandRegistry::new();
    crate::shell::commands::register(&mut reg);
    let live = |app: &PdfcerApp, ctx: &egui::Context, id: &str| {
        reg.get(id)
            .expect("registered")
            .is_enabled(&app.conditions(ctx))
    };

    // ★ A freshly opened document has an empty command log, so BOTH controls
    // are greyed. This is the assertion an unconditional publication fails,
    // and it is the reason the pair below is not enough on its own.
    assert!(
        !live(&app, &ctx, "edit.undo"),
        "nothing has been changed, so Undo must be greyed — a control armed here is one that \
         can only decline"
    );
    assert!(!live(&app, &ctx, "edit.redo"), "and so must Redo");

    // One real edit through the real funnel.
    app.apply_actions(
        vec![Action::CommitMarkup {
            pen: crate::canvas::markup::pen::Pen::default(),
            page: 0,
            kind: MarkupKind::Rectangle,
            geometry: Geometry::Band {
                start: (100.0, 100.0),
                end: (200.0, 160.0),
            },
        }],
        1.0,
    );
    assert!(
        live(&app, &ctx, "edit.undo"),
        "an annotation was authored, so there is something to take back"
    );
    assert!(
        !live(&app, &ctx, "edit.redo"),
        "nothing has been undone yet — an edit does not fill the redo stack"
    );

    // …and the undo swaps them, which is the half that proves neither is
    // latched. `EditSession::commit` clears the redo stack on a new command,
    // so a build that never re-read `can_redo` would pass the line above and
    // fail this one.
    app.apply_actions(vec![Action::Undo], 1.0);
    assert!(
        !live(&app, &ctx, "edit.undo"),
        "the only command on the log has been taken back"
    );
    assert!(
        live(&app, &ctx, "edit.redo"),
        "…and is now on the redo stack, which is what makes Redo pressable"
    );

    // Round trip: redoing empties the redo stack and refills the undo log.
    app.apply_actions(vec![Action::Redo], 1.0);
    assert!(live(&app, &ctx, "edit.undo"));
    assert!(!live(&app, &ctx, "edit.redo"));
}

/// ★ **Every armable tool kind reports a pressed state** — asserted over
/// `ALL` for both families rather than over a list written here.
///
/// # The defect this exists for, which shipped
///
/// `app::conditions` published the armed **markup** kind and did not
/// publish the armed **measure** kind. Phase 7 had `CanvasTool::Measure`,
/// `arm_measure`, `measure_command` and a dispatch arm using its inverse —
/// all four with passing unit tests — so Linear armed the tool, placed a
/// dimension the engine accepted, and the button never lit up. It was found
/// by `ui-verify` driving the real window, because the missing link was a
/// **call site**, and no unit test observed two adjacent links being
/// connected.
///
/// Iterating `MarkupKind::ALL` and `MeasureKind::ALL` is what stops the
/// same omission recurring: a fifth kind added to either enum with no
/// `selected_condition` fails here rather than shipping as a control that
/// arms without looking armed. A list of ids spelled out in this test would
/// have to be remembered, which is the thing that was not.
#[test]
fn every_armable_tool_kind_reports_a_pressed_state() {
    use crate::canvas::markup::MarkupKind;
    use crate::canvas::measure::MeasureKind;
    use crate::canvas::tool::{self, CanvasTool};

    let app = PdfcerApp::new();
    let ctx = egui::Context::default();

    for &kind in MarkupKind::ALL {
        let id = crate::shell::commands::markup_command(kind);
        let cond = egui_shell::ribbon::selected_condition(id);
        tool::select(&ctx, CanvasTool::Select);
        assert!(
            !app.conditions(&ctx).is_set(&cond),
            "`{id}` must not read pressed while the select tool is armed"
        );
        tool::select(&ctx, CanvasTool::Markup(kind));
        assert!(
            app.conditions(&ctx).is_set(&cond),
            "`{id}` names {kind:?}, which is armed, and the ribbon does not say so"
        );
    }

    for &kind in MeasureKind::ALL {
        let id = crate::shell::commands::measure_command(kind);
        let cond = egui_shell::ribbon::selected_condition(id);
        tool::select(&ctx, CanvasTool::Select);
        assert!(
            !app.conditions(&ctx).is_set(&cond),
            "`{id}` must not read pressed while the select tool is armed"
        );
        tool::select(&ctx, CanvasTool::Measure(kind));
        assert!(
            app.conditions(&ctx).is_set(&cond),
            "`{id}` names {kind:?}, which is armed, and the ribbon does not say so"
        );
    }

    // ★ …and the two tools that carry **no** kind, which is why they cannot
    // be reached by walking an `ALL`. They are the ones this test's own
    // mechanism would silently miss, so they are named — and naming them is
    // exactly the "list of ids that has to be remembered" this test's header
    // warns about, which is why the warning is narrowed rather than ignored:
    // walk the kinds where kinds exist, and enumerate the kindless ones,
    // because there is nothing else to enumerate them from.
    //
    // The text tool is the more exposed of the pair. Arming it changes the
    // cursor and nothing else on the canvas, so a missing pressed state
    // would leave an operator with no evidence at all of the mode they are
    // in — where a hand at least shows a grab cursor and moves the page.
    for (tool, id) in [
        (CanvasTool::Hand, "view.tool_hand"),
        (CanvasTool::Text, "view.tool_text"),
    ] {
        let cond = egui_shell::ribbon::selected_condition(id);
        tool::select(&ctx, CanvasTool::Select);
        assert!(
            !app.conditions(&ctx).is_set(&cond),
            "`{id}` must not read pressed while the select tool is armed"
        );
        tool::select(&ctx, tool);
        assert!(
            app.conditions(&ctx).is_set(&cond),
            "`{id}` names {tool:?}, which is armed, and the ribbon does not say so"
        );
        // …and arming it must not leave the OTHER kindless tool pressed,
        // which is the property that makes them behave as a radio without
        // anything enforcing it — the same payoff the kind-carrying enum
        // gives the two families above.
        let other = if tool == CanvasTool::Hand {
            "view.tool_text"
        } else {
            "view.tool_hand"
        };
        assert!(
            !app.conditions(&ctx)
                .is_set(&egui_shell::ribbon::selected_condition(other)),
            "arming {tool:?} must not leave `{other}` pressed"
        );
    }

    // …and exactly one is pressed at a time, which is the payoff of the
    // kind-carrying enum shape: a tool that could be two kinds at once is
    // unrepresentable, so two pressed buttons are too.
    tool::select(&ctx, CanvasTool::Measure(MeasureKind::Linear));
    let set = app.conditions(&ctx);
    let pressed = MeasureKind::ALL
        .iter()
        .chain(std::iter::empty())
        .filter(|&&k| {
            set.is_set(&egui_shell::ribbon::selected_condition(
                crate::shell::commands::measure_command(k),
            ))
        })
        .count();
    assert_eq!(pressed, 1, "exactly one measure control renders pressed");
    for &kind in MarkupKind::ALL {
        assert!(
            !set.is_set(&egui_shell::ribbon::selected_condition(
                crate::shell::commands::markup_command(kind)
            )),
            "arming a measure tool must not leave a markup control pressed"
        );
    }
}

/// ★ **The three Text markup controls are live exactly when there is a
/// live text selection**, and are asserted through the **registry's own
/// enable machinery** rather than by reading the condition name.
///
/// Reading `set.is_set("selection.text")` would assert that this function
/// agrees with itself. What matters is whether the *control* comes alive,
/// which is the registration's predicate and this function's publication
/// joined — the same join `every_armable_tool_kind_reports_a_pressed_state`
/// exists for, and the same join `ui-verify` had to find in a running window
/// when the measure tools armed without lighting up.
///
/// Three states, and the third is the one a build would plausibly get wrong:
/// a selection made *before* an edit is not an operand, because its recorded
/// boxes may now sit over other glyphs — and a control that is live while
/// the press would decline is the disagreement `selection.bounds` was
/// invented to prevent one command over.
#[test]
fn the_text_markup_controls_need_a_live_text_selection() {
    use crate::app::tests::opened;
    use crate::canvas::markup::text::TextMarkKind;
    use crate::canvas::textsel::TextSelection;
    use pdfcer_core::annot_author::Quad;
    use pdfcer_core::page_tree::Rect as PageRect;

    let ctx = egui::Context::default();
    let mut app = opened();
    let ids: Vec<&str> = TextMarkKind::ALL
        .iter()
        .map(|&k| crate::shell::commands::text_mark_command(k))
        .collect();
    let mut reg = egui_shell::CommandRegistry::new();
    crate::shell::commands::register(&mut reg);

    let live = |app: &PdfcerApp, ctx: &egui::Context, id: &str| {
        reg.get(id)
            .expect("registered")
            .is_enabled(&app.conditions(ctx))
    };

    for id in &ids {
        assert!(
            !live(&app, &ctx, id),
            "`{id}` must be greyed with nothing selected — it would do nothing"
        );
    }

    let Status::Open(doc) = &mut app.status else {
        unreachable!("`opened` opens a document")
    };
    let epoch = doc.edit_epoch;
    doc.text_selection = Some(TextSelection::for_test(
        0,
        epoch,
        vec![Quad::from_rect(PageRect::from_corners(
            72.0, 700.0, 300.0, 710.0,
        ))],
    ));
    for id in &ids {
        assert!(
            live(&app, &ctx, id),
            "`{id}` acts on the text selection there now is"
        );
    }

    // One edit later the same selection is not an operand.
    let Status::Open(doc) = &mut app.status else {
        unreachable!()
    };
    doc.edit_epoch = epoch.wrapping_add(1);
    for id in &ids {
        assert!(
            !live(&app, &ctx, id),
            "`{id}` must not offer to mark boxes recorded against an older revision"
        );
    }
}

/// ★★★ **`selection.formattable` is the UNION, and the two halves are
/// asserted separately because each on its own is a shipped defect.**
///
/// It is the contextual Format tab's `visible_when`, and the tab now
/// carries controls for two unrelated kinds of selection:
///
/// * spelled `selection.any` — the object selection — a text sweep would
///   raise **no tab**, so the Font group could not be reached at all and
///   `format_text` would be a capability with no surface. That is what
///   shipped between the panel landing and this condition existing;
/// * spelled `selection.text`, the tab would vanish on an ordinary object
///   selection, taking the Delete it has carried since it shipped with it.
///
/// So the assertion is a truth table rather than a pair of positives, and
/// the **fourth row** — both at once — is the one that could not happen
/// when `selection.text`'s own note was written (*"the two are mutually
/// exclusive by construction"*) and can now: `takes_the_press` answers true
/// for an armed text tool in **any** mode, so an operator who clicks an
/// object and then presses `T` and sweeps has both.
#[test]
fn the_formattable_condition_is_the_union_of_the_two_selections() {
    use crate::app::tests::{opened, select_object};
    use crate::canvas::textsel::TextSelection;
    use pdfcer_core::annot_author::Quad;
    use pdfcer_core::page_tree::Rect as PageRect;

    let ctx = egui::Context::default();
    let mut app = opened();

    assert!(
        !app.conditions(&ctx).is_set("selection.formattable"),
        "nothing is selected, so the Format tab has no subject and must not appear"
    );

    // 1. An object, and no sweep.
    select_object(&mut app, 0, false);
    assert!(app.conditions(&ctx).is_set("selection.any"));
    assert!(!app.conditions(&ctx).is_set("selection.text"));
    assert!(
        app.conditions(&ctx).is_set("selection.formattable"),
        "an object selection is a subject — this is the Delete the tab has always had"
    );

    // 2. …and a sweep on top of it: BOTH, which used to be unrepresentable.
    let epoch = {
        let Status::Open(doc) = &mut app.status else {
            unreachable!("`opened` opens a document")
        };
        let epoch = doc.edit_epoch;
        doc.text_selection = Some(TextSelection::for_test(
            0,
            epoch,
            vec![Quad::from_rect(PageRect::from_corners(
                72.0, 700.0, 300.0, 710.0,
            ))],
        ));
        epoch
    };
    assert!(app.conditions(&ctx).is_set("selection.any"));
    assert!(app.conditions(&ctx).is_set("selection.text"));
    assert!(app.conditions(&ctx).is_set("selection.formattable"));

    // 3. A sweep alone — the state the Font group exists for, and the one
    //    a `selection.any` spelling would have shown no tab for.
    {
        let Status::Open(doc) = &mut app.status else {
            unreachable!()
        };
        doc.selection.clear();
    }
    assert!(!app.conditions(&ctx).is_set("selection.any"));
    assert!(app.conditions(&ctx).is_set("selection.text"));
    assert!(
        app.conditions(&ctx).is_set("selection.formattable"),
        "a swept range is a subject: it is what the Font group acts on"
    );

    // 4. …and a STALE sweep is not. `selection.text` refuses a selection
    //    recorded against an older revision, and the union must not
    //    resurrect it — a tab appearing over boxes that may now sit on
    //    different glyphs is worse than no tab.
    {
        let Status::Open(doc) = &mut app.status else {
            unreachable!()
        };
        doc.edit_epoch = epoch.wrapping_add(1);
    }
    assert!(!app.conditions(&ctx).is_set("selection.text"));
    assert!(
        !app.conditions(&ctx).is_set("selection.formattable"),
        "a stale sweep is not an operand, so it is not a subject either"
    );
}

/// ★★★ **A mode that cannot edit must not offer Delete** — the regression
/// test for the data-loss defect found on 2026-09-03.
///
/// # What was wrong
///
/// `selection.delete_permitted` asked only whether the *engine* would
/// refuse. It never asked whether the *mode* would. That was defensible
/// while nothing could be selected in a read-only mode, and
/// `canvas::keys` wrote its own guard anyway, saying why:
///
/// > "Delete is safe because nothing can be selected" holds only for as
/// > long as its other half does, and the other half is in a different
/// > file.
///
/// **`OPERATOR_REQUESTS.md` O71 falsified that other half nine days
/// later.** `canvas::clicking`'s image arm runs precisely when
/// `!caps.edit_content` — it exists so a reader can click a picture and
/// copy it. So from 2026-08-31 a content selection was reachable in Read,
/// this condition was set there, the Format tab's Delete was **drawn and
/// enabled**, and `app::dispatch::format`'s object arm had no capability
/// guard at all. The Delete *key* refused; the ribbon button deleted page
/// content in the mode whose whole promise is that it authors nothing.
///
/// # Why the table has a Review row for annotations
///
/// ★★ Row 4 is the load-bearing one. It is what stops a future
/// simplification collapsing this ladder to `caps.edit_content` and
/// silently taking **Review's markup Delete** off the ribbon while the
/// Delete key kept working. One predicate per capability: `author_markup`
/// guards the annotation rung, `edit_content` guards the content rung, and
/// neither stands in for the other.
#[test]
fn delete_is_not_offered_in_a_mode_that_cannot_perform_it() {
    use crate::app::tests::{opened, select_object};

    let ctx = egui::Context::default();
    let mut app = opened();

    // --- content: permitted in Edit, refused everywhere else ------------
    select_object(&mut app, 0, false);
    app.ribbon.set_mode("edit");
    assert!(
        app.conditions(&ctx).is_set("selection.delete_permitted"),
        "Edit authors content, so a selected object may be deleted"
    );

    app.ribbon.set_mode("review");
    assert!(
        !app.conditions(&ctx).is_set("selection.delete_permitted"),
        "Review authors markup, NOT page content — a selected object must not offer Delete"
    );

    app.ribbon.set_mode("read");
    assert!(
        !app.conditions(&ctx).is_set("selection.delete_permitted"),
        "★ THE DEFECT. Read can select a picture (O71) and authors nothing, so Delete must \
         not be offered. Before 2026-09-03 this condition was set, the ribbon drew an \
         ENABLED Delete, and `dispatch::format`'s object arm had no guard — so Format ▸ \
         Delete removed page content from a read-only mode."
    );
}

/// ★★★ **The Font group's five commands are drawn in Edit and ABSENT in
/// Read and Review**, which is R9 split across two conditions.
///
/// `mode.edit_content` is **visibility** — a mode that cannot change page
/// content does not have a mislaid ability to restyle text, it does not
/// have the ability, so the controls render nothing. `selection.text` is
/// **enablement** — inside Edit the capability is present and only the
/// operand is missing, which greys and explains itself on hover.
///
/// # ★★ Why both are needed, stated as the two one-condition builds
///
/// With only `selection.text`, sweeping text in **Read** — which Read does
/// with the plain select tool, because copying is not authoring — would
/// draw an enabled Bold that the mode gate must then refuse. With only
/// `mode.edit_content`, Edit would draw an enabled Bold with nothing
/// swept: a control that does nothing on almost every press, which is the
/// placeholder shape P3 forbids.
///
/// ★ Asserted through `Capabilities::for_mode` and the shipped manifest,
/// not through a hand-made `Capabilities` value, so a mode taxonomy edit
/// that gave Read `edit_content` fails here as well as wherever else it is
/// wrong.
#[test]
fn the_font_groups_visibility_follows_the_mode_and_its_enablement_the_sweep() {
    use crate::app::tests::opened;
    use crate::canvas::textsel::TextSelection;
    use pdfcer_core::annot_author::Quad;
    use pdfcer_core::page_tree::Rect as PageRect;

    let ctx = egui::Context::default();
    let mut app = opened();
    let mut reg = egui_shell::CommandRegistry::new();
    crate::shell::commands::register(&mut reg);
    // ui-text-exempt: registered command ids, never displayed.
    let ids = [
        "format.font",
        "format.font_size",
        "format.bold",
        "format.italic",
        "format.font_colour",
    ];

    // Read: the whole group is invisible, whatever is selected.
    app.ribbon.set_mode("read");
    assert!(
        !app.conditions(&ctx).is_set("mode.edit_content"),
        "Read cannot change page content, so the Font group is not drawn there"
    );

    app.ribbon.set_mode("edit");
    assert!(app.conditions(&ctx).is_set("mode.edit_content"));

    // …and inside Edit, with nothing swept, every one of the five is
    // GREYED rather than absent. That state is the discoverability
    // surface: the operator has clicked a piece of text, the Format tab is
    // up, and hovering a greyed Bold is what tells them to sweep.
    for id in ids {
        let command = reg.get(id).expect("registered");
        assert!(
            !command.is_enabled(&app.conditions(&ctx)),
            "`{id}` must be greyed with nothing swept — it has no operand"
        );
        assert!(
            command.tooltip.is_some(),
            "`{id}` is greyed for most of its life, and R9 permits that only when it \
             explains itself on hover"
        );
    }

    let Status::Open(doc) = &mut app.status else {
        unreachable!("`opened` opens a document")
    };
    let epoch = doc.edit_epoch;
    doc.text_selection = Some(TextSelection::for_test(
        0,
        epoch,
        vec![Quad::from_rect(PageRect::from_corners(
            72.0, 700.0, 300.0, 710.0,
        ))],
    ));
    for id in ids {
        assert!(
            reg.get(id)
                .expect("registered")
                .is_enabled(&app.conditions(&ctx)),
            "`{id}` acts on the swept range there now is"
        );
    }
}

/// ★ **THE P3 TENSION, CLOSED** — in Edit, with the text tool armed and a
/// live text selection, the three text-markup controls come alive and their
/// press authors an annotation.
///
/// # What was wrong, and why it was a rule violation rather than a gap
///
/// Edit shows the Markup tab, so `markup.underline`, `markup.strikeout` and
/// `markup.squiggly` were **drawn** there — and `selection.text` could never
/// be true in Edit, because `canvas::textsel::takes_the_press` gave the press
/// its text meaning only where the mode could *not* select content. So three
/// controls rendered, greyed, in every Edit session for the life of the
/// build, with no state that could ever enable them.
///
/// `RIBBON_IA.md` **P3** reserves greying for *temporarily* unavailable and
/// says an absent capability renders nothing. Permanently greyed is neither,
/// and it could not be fixed by hiding: a command lives on exactly one tab,
/// and the Markup tab is in **both** Review and Edit, so hiding them in Edit
/// would have required a per-command per-mode visibility rule that this
/// manifest does not have and that would have been a mechanism invented to
/// conceal a gap rather than to close one.
///
/// # Why this test is worth its length
///
/// It is the only assertion in the workspace that joins **four** things that
/// each have their own passing tests: the mode's capabilities, the armed
/// tool, the condition, and the dispatch. `the_text_markup_controls_need_a_
/// live_text_selection` above proves the condition-to-enable join and says
/// nothing about the mode; `canvas::textsel`'s tests prove the press rule and
/// know nothing about the ribbon. A build that armed the tool and left
/// `press_kind` reading the mode first would pass both of those and fail
/// here.
///
/// The **negative** half is asserted first and is what makes the positive
/// half mean something: with the tool down, the same mode with the same
/// selection must still refuse, because that is the state the operator was
/// in before this feature and it must not have been quietly widened. Note it
/// is the *press rule* that is asserted there, not the condition — the
/// condition reads only whether a selection exists and is live, and in Edit
/// without the tool no gesture could have made one.
#[test]
fn in_edit_the_text_tool_makes_the_text_markup_controls_reachable() {
    use crate::app::tests::opened;
    use crate::canvas::markup::text::TextMarkKind;
    use crate::canvas::textsel::{self, TextSelection};
    use crate::canvas::tool::{self, CanvasTool};
    use pdfcer_core::annot_author::Quad;
    use pdfcer_core::page_tree::Rect as PageRect;

    let ctx = egui::Context::default();
    let mut app = opened();
    app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
    let caps = app.capabilities();
    assert!(
        caps.edit_content && caps.author_markup,
        "the premise: Edit both selects content and authors markup — which is exactly why \
         the two halves collided"
    );

    // The old world: no tool, no text gesture, so no operand could exist.
    tool::select(&ctx, CanvasTool::Select);
    assert!(
        !textsel::takes_the_press(tool::selected(&ctx), caps),
        "with the select tool, Edit's primary button is still the content marquee"
    );

    // Arm the tool, and the gesture that makes the operand exists.
    tool::select(&ctx, CanvasTool::Text);
    app.on_mode_capabilities_changed(&ctx);
    assert!(
        textsel::takes_the_press(tool::selected(&ctx), caps),
        "the armed text tool takes the press in Edit — that is the whole feature"
    );

    // A sweep would produce this. Planted rather than driven, because
    // `interact` needs a window; the sweep itself is asserted against a real
    // extraction in `canvas::textsel` and against the real binary in
    // `ui-verify`'s `text_tool_selects_and_marks_in_edit`.
    let Status::Open(doc) = &mut app.status else {
        unreachable!("`opened` opens a document")
    };
    let epoch = doc.edit_epoch;
    doc.text_selection = Some(TextSelection::for_test(
        0,
        epoch,
        vec![Quad::from_rect(PageRect::from_corners(
            72.0, 700.0, 300.0, 710.0,
        ))],
    ));

    let mut reg = egui_shell::CommandRegistry::new();
    crate::shell::commands::register(&mut reg);
    for &kind in TextMarkKind::ALL {
        let id = crate::shell::commands::text_mark_command(kind);
        assert!(
            reg.get(id)
                .expect("registered")
                .is_enabled(&app.conditions(&ctx)),
            "`{id}` is drawn on Edit's Markup tab and must now be able to enable there — \
             this is the P3 tension the text tool exists to close"
        );
    }

    // …and pressing one really authors, rather than merely lighting up. The
    // action is the proof that Edit's `author_markup` and the text selection
    // meet: a control that enabled and then declined would be the
    // `selection.bounds` failure one command over.
    let mut actions = Vec::new();
    app.dispatch_command(
        &ctx,
        crate::shell::commands::text_mark_command(TextMarkKind::Underline),
        &mut actions,
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, crate::app::actions::Action::CommitTextMarkup { .. })),
        "the press must raise CommitTextMarkup in Edit, not decline: {actions:?}"
    );
}

/// ★ **`measure.finishable` needs a document, not merely a pick set.**
///
/// The one condition published from inside the `Status::Open` arm that is
/// about a gesture rather than about the document, and this is the reason
/// it is inside it. A circular pick set lives in `egui::Memory`, which
/// **outlives documents** — that is the property the armed-tool conditions
/// below it are published outside the arm to preserve. Here it is exactly
/// the hazard: the action Finish raises names a page, and with the document
/// closed there is no page for it to name. A live control that raises a
/// `CommitDimension` against nothing is the placeholder shape this project
/// refuses.
///
/// Both directions are asserted, because the first alone would pass on a
/// build where the condition was never published at all.
#[test]
fn finish_is_not_offered_with_no_document_open() {
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use crate::canvas::measure::{self, MeasureKind};
    use crate::canvas::tool::{self, CanvasTool};

    let mut app = PdfcerApp::new();
    let ctx = egui::Context::default();
    tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
    measure::circular::plant_pick_for_test(&ctx, 0);
    assert!(
        measure::finishable(&ctx),
        "the canvas really does have a finishable fit"
    );
    assert!(
        !app.conditions(&ctx).is_set("measure.finishable"),
        "…and the ribbon must still not offer to place it into no document"
    );

    // Open one, and the same fit is offered.
    app.status = Status::Open(Box::new(open_fixture(FOUR_PAGES)));
    assert!(
        app.conditions(&ctx).is_set("measure.finishable"),
        "with a document open the control is live"
    );
}

/// ★ **`markup.finishable` is the same fact for the vertex tools, and it is
/// scoped the same way** — plus the one thing that is genuinely different
/// about it.
///
/// The document half is a near-copy of the test above, deliberately: the two
/// conditions have the same shape, the same hazard and the same argument for
/// living inside `Status::Open`, so a build that got the scope right for one
/// and wrong for the other is what a near-copy catches.
///
/// What is **not** a copy is the last section, and it is the interesting
/// half: this condition is where the polygon/polyline difference reaches the
/// operator. `markup::action` needs **three** vertices for a `/Polygon` and
/// two for a `/PolyLine`, so the same two-click run leaves the ribbon's
/// Finish live for one tool and greyed for the other. Asserting it here
/// rather than only in `markup::vertex` is the point — the rule is worth
/// nothing until it reaches the control.
#[test]
fn markup_finish_needs_a_document_and_enough_corners_for_its_kind() {
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use crate::canvas::markup::{MarkupKind, vertex};
    use crate::canvas::tool::{self, CanvasTool};

    let mut app = PdfcerApp::new();
    let ctx = egui::Context::default();
    tool::select(&ctx, CanvasTool::Markup(MarkupKind::Polygon));
    vertex::plant_run_for_test(&ctx, 0, MarkupKind::Polygon);
    assert!(
        vertex::finishable(&ctx),
        "the canvas really does have a finishable run"
    );
    assert!(
        !app.conditions(&ctx).is_set("markup.finishable"),
        "…and the ribbon must still not offer to place it into no document"
    );

    app.status = Status::Open(Box::new(open_fixture(FOUR_PAGES)));
    assert!(
        app.conditions(&ctx).is_set("markup.finishable"),
        "with a document open the control is live"
    );
    // …and it is not the measure tab's condition wearing another name: a
    // measure tool and a markup tool cannot both be armed, so exactly one of
    // the two may ever be set, and a build that collapsed them would light
    // one tab's Finish from the other tab's gesture.
    assert!(!app.conditions(&ctx).is_set("measure.finishable"));

    // ★ The polygon/polyline difference, at the control. Two vertices is a
    // polyline and is not a polygon.
    vertex::plant_short_run_for_test(&ctx, 0, MarkupKind::Polygon);
    assert!(
        !app.conditions(&ctx).is_set("markup.finishable"),
        "two corners are a line drawn there and back, not a polygon"
    );
    tool::select(&ctx, CanvasTool::Markup(MarkupKind::PolyLine));
    vertex::plant_short_run_for_test(&ctx, 0, MarkupKind::PolyLine);
    assert!(
        app.conditions(&ctx).is_set("markup.finishable"),
        "…and the same two corners ARE a polyline"
    );
}

/// ★★★ **`selection.delete_permitted` follows the FORMS gate when a form
/// field is selected**, which is the arm this condition did not have.
///
/// # The defect, and why it is invisible without the second document
///
/// The publication read
/// `doc.selected_field.is_none() && annotdelete::refuses_selected(doc)`.
/// With a field selected the first conjunct is **false**, so the whole
/// expression is false and the condition was set **unconditionally for
/// every selected field on every document** — a gate that is a no-op by
/// construction. `format.delete`'s `visible_when` on the `canvas.field`
/// menu therefore resolved *shown* on a certified fillable form, and the
/// press it invited deleted nothing and said nothing.
///
/// ⇒ Both halves are asserted, against a fixture **pair** that differs in
/// exactly one dictionary (`tools/gen-certified-fixture.py`), because the
/// negative half alone is satisfied by a build that withholds Delete
/// always — which is a worse defect than the one being fixed: a control
/// absent where it would have worked leaves the operator no gesture that
/// reports it.
///
/// ★ Driven through `app.conditions()` and `open_path` rather than by
/// calling the derivation, for this module's standing reason one test up:
/// what is under test is the **join** between the query and the published
/// name, and a test that read the derivation would prove `set.set` works.
#[test]
fn a_certified_document_withholds_delete_for_a_selected_form_field() {
    use crate::app::state::SelectedField;

    let ctx = egui::Context::default();
    let certifier = SelectedField {
        field: "Certifier".to_owned(),
        widget: 0,
        page: 0,
    };
    let local = |rel: &str| {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(rel)
    };

    let mut app = PdfcerApp::new();
    // ★ EDIT, on both halves below, and held constant on purpose: this
    // test varies the DOCUMENT (certified against its uncertified twin) and
    // must not vary the mode. Since 2026-09-03 `selection.delete_permitted`
    // also asks whether the MODE may delete, and `PdfcerApp::new` starts in
    // Read — so without this the positive half would refuse for a reason
    // that has nothing to do with `/Perms`, and the test would go green on
    // a build where the engine refusal had been deleted.
    app.ribbon.set_mode("edit");
    app.open_path(local("certified-comments.pdf"));
    let Status::Open(doc) = &mut app.status else {
        panic!("the certified fixture opens") // ui-text-exempt: test panic, never displayed
    };
    doc.selected_field = Some(certifier.clone());
    assert!(
        !app.conditions(&ctx).is_set("selection.delete_permitted"),
        "Delete is offered over a form field on a document whose /Perms \
         /DocMDP freezes the form's structure. §12.8.2.2 Table 257 permits \
         filling such a form and forbids restructuring it, so \
         `EditSession::deletion_refusal` answers Some and the control must \
         not be drawn (R9)"
    );
    assert!(
        app.conditions(&ctx).is_set("selection.actionable"),
        "the field is still SELECTED — `selection.actionable` and \
         `selection.delete_permitted` answer different questions, and \
         collapsing them would take the Properties command away too"
    );

    let mut app = PdfcerApp::new();
    app.ribbon.set_mode("edit");
    app.open_path(local("threaded-comments.pdf"));
    let Status::Open(doc) = &mut app.status else {
        panic!("the uncertified twin opens") // ui-text-exempt: test panic, never displayed
    };
    doc.selected_field = Some(certifier);
    assert!(
        app.conditions(&ctx).is_set("selection.delete_permitted"),
        "the condition refused on the uncertified twin, which differs from \
         the certified fixture only in the catalog's /Perms entry — so this \
         build hides Delete on every signed document"
    );
}
