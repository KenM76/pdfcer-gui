#![cfg(test)]
//! # `dialogs::textannot_tests` — the sticky-note/text-box/stamp window, proved headlessly
//!
//! Split out of [`super::textannot`] on 2026-09-10, when operator request O172
//! — *"add our own custom stamps and use them, preferrably exactly the same way
//! acrobat does"* — took that file past R2's 1500-line ceiling. The seam is the
//! one `dialogs::print::preview_tests` and `egui-shell`'s `dock/width_tests`
//! already use in this workspace: **the module keeps the code, the sibling
//! keeps the proof.**
//!
//! ## Why the tests were the right half to move
//!
//! Because the alternative was worse. The other candidate seam was lifting the
//! stamp gallery into its own module, and it would have split a *decision* that
//! belongs together: the gallery's two radio groups hold the invariant *"exactly
//! one of `stamp` and `custom` is the selection"* between them, and putting the
//! two halves of that invariant in two files is precisely the arrangement
//! `DEFECTS.md` records as the cause of the old GUI's worst pair of bugs — two
//! lines thousands of lines apart that no reviewer could be expected to see
//! together.
//!
//! The tests, by contrast, have no invariant with the code beyond `use super::*`
//! and the private fields they read. `mod tests` is already a child module, so
//! this is a change of file and nothing else.
//!
//! ## What is proved here
//!
//! Four subjects, in the order the window presents them:
//!
//! 1. **Layout and geometry** — that the window sits where it was asked to,
//!    survives a small screen, and reports its regions.
//! 2. **The three kinds** — that a sticky note, a text box and a stamp each
//!    draw their own controls and none of the others'.
//! 3. **What reaches the commit action** — the text, the colour, the icon, the
//!    stamp, the size, and (O172) the operator's own stamp.
//! 4. **The custom half** — that it exists only when he has stamps, and that
//!    choosing one clears the standard selection rather than shadowing it.
//!
//! `#![cfg(test)]` is the FIRST line of the file, so nothing here reaches a
//! release build and the module costs the shipped binary nothing.

use super::*;
use crate::stamps::library::Category;

fn rect() -> Rect {
    Rect {
        llx: 0.0,
        lly: 0.0,
        urx: 100.0,
        ury: 40.0,
    }
}

/// **A `RawInput` describing a real screen at a deterministic time.**
///
/// Two fields of `RawInput::default()` are wrong for driving this dialog,
/// and each cost a debugging round when it was left alone.
///
/// **`screen_rect` is `None`.** This dialog sizes itself from
/// `content_rect` — `420.min(width - 40)` — so a default input hands
/// `egui::Window` a degenerate size and the field inside it a width nothing
/// can be focused in. A test that lays out differently from the application
/// is measuring a different program.
///
/// **`time` is `None`, and egui then fills it from the wall clock.** That
/// makes frame timing depend on how loaded the machine is, so a test that
/// drives several frames is reproducible when run alone and intermittent
/// when run beside a thousand others — which is precisely the flake that
/// gets re-run until it is green and then believed. Time is supplied here,
/// one 60 Hz tick per frame, so the sequence is the same every time.
fn on_screen(frame: u32) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 800.0),
        )),
        time: Some(f64::from(frame) / 60.0),
        predicted_dt: 1.0 / 60.0,
        ..Default::default()
    }
}

/// The application window this dialog's geometry is computed against.
fn screen() -> egui::Rect {
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0))
}

/// ★★★ **The note window does not open in the corner** — review finding
/// A16c.
///
/// The whole of the defect in one assertion. `dialogs/textannot.rs`
/// computed this position and then wrote `let _ = pos;`, so the dialog was
/// placed by `Host`'s corner inset instead — on every open, dozens of times
/// in a markup session, because a dialog dismissed that often almost never
/// has a remembered position to restore.
///
/// The position is asserted as a **relationship** rather than as two
/// numbers: centred across the window and between a fifth and half of the
/// way down it. Pinning the exact pixels would fail the next time the
/// window's size changed for an unrelated reason, which is how a test stops
/// being read and starts being edited.
#[test]
fn the_note_window_does_not_open_in_the_corner() {
    let screen = screen();
    let size = window_size(screen, TextAnnotKind::TextBox, 0.0);
    let at = opening_position(screen, size);

    assert!(
        at.x > 0.0 && at.y > 0.0,
        "the note dialog opened at {at:?} — the top-left corner of the window is \
         precisely what A16c reported"
    );
    let centre_gap = (at.x + size.x / 2.0) - screen.center().x;
    assert!(
        centre_gap.abs() < 1.0,
        "the window must be centred across the application window; its centre is \
         {centre_gap} pt off"
    );
    let down = at.y / screen.height();
    assert!(
        (0.2..0.5).contains(&down),
        "a third of the way down, not half and not the top: got {down}"
    );
}

/// **The window is squeezed to fit a narrow application window, and never
/// below the size it refuses to be dragged to.**
///
/// The floor is the half that was missing: the expression used to be
/// `420.min(width - 40)` with no `max`, which is **negative** for an
/// application window under 40 pt wide. Unreachable today and free to
/// close.
#[test]
fn the_note_window_is_squeezed_but_never_below_its_own_floor() {
    let roomy = window_size(screen(), TextAnnotKind::TextBox, 0.0);
    assert_eq!(roomy, WINDOW_PTS, "a wide window gets the size asked for");

    let narrow = window_size(
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(380.0, 800.0)),
        TextAnnotKind::TextBox,
        0.0,
    );
    assert!(narrow.x < WINDOW_PTS.x, "a narrow window squeezes it");
    assert!(narrow.x >= MIN_WINDOW_PTS.x);

    let absurd = window_size(
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(10.0, 10.0)),
        TextAnnotKind::TextBox,
        0.0,
    );
    assert!(
        absurd.x >= MIN_WINDOW_PTS.x,
        "a window narrower than the margin must not produce a size of {absurd:?}"
    );
    assert!(
        absurd.y >= MIN_WINDOW_PTS.y,
        "the height floor is the width floor's twin and arrived with it: {absurd:?}"
    );
}

/// ★★★ **Each kind's window is as tall as its own body needs, and the one
/// kind with nothing added did not move.**
///
/// Two chooser have been added under two of the three bodies: the sticky's
/// icon radios on 2026-09-06, and the stamp's label-size combo on
/// 2026-09-10 (engine `Pass 287.0`). Without the extra height the Accept
/// button sits below the window's own bottom edge — a dialog the operator
/// cannot finish, which is a worse failure than any it replaces.
///
/// ★★ **The text-box assertion is the positive control** and it is what
/// makes this a test at all. Asserting only *"the sticky and the stamp are
/// taller"* passes on a `window_size` that had gone taller for **every**
/// kind — the change would be invisible, the text box would grow a strip of
/// empty window, and nothing here would say so.
///
/// ⚠ **This test was renamed on 2026-09-10, and the old name is the
/// lesson.** It was `only_the_sticky_notes_window_grew_for_its_chooser`,
/// and it asserted `stamp.y == WINDOW_PTS.y` with the message *"the stamp
/// has no chooser and must not have grown"*. That sentence was true when it
/// was written and became false the moment the stamp got one. A test whose
/// **name and message state a property the program no longer has** is worse
/// than no test: it reads as a measurement, and the next person to grep for
/// *"which kinds have choosers?"* finds an answer rather than a question.
#[test]
fn each_kinds_window_is_as_tall_as_its_body_needs() {
    let screen = screen();
    let sticky = window_size(screen, TextAnnotKind::Sticky, 0.0);
    let boxed = window_size(screen, TextAnnotKind::TextBox, 0.0);
    let stamp = window_size(screen, TextAnnotKind::Stamp, 0.0);

    assert!(
        sticky.y > boxed.y,
        "the icon chooser needs room the text box does not: {sticky:?} vs {boxed:?}"
    );
    assert!(
        stamp.y > boxed.y,
        "the size chooser needs room the text box does not: {stamp:?} vs {boxed:?}"
    );
    // ★ The ordering, not merely the inequality. Seven radio rows and a
    // disclosure is a taller addition than a heading, one combo row and a
    // disclosure, and if that ever inverts it is because somebody changed
    // one of the two constants without reading the other's argument.
    assert!(
        sticky.y > stamp.y,
        "seven radio rows must ask more room than one combo: {sticky:?} vs {stamp:?}"
    );
    assert_eq!(
        boxed.y, WINDOW_PTS.y,
        "the text box has nothing added and must not have grown"
    );
    assert_eq!(
        sticky.x, boxed.x,
        "only the height is per-kind; a second varying number would have no reason"
    );
    assert_eq!(stamp.x, boxed.x, "the width is per-kind for nobody");
}

/// ★★★ **The icon the operator picked reaches the action — and the two
/// kinds that have no icon still carry the default rather than a
/// contradiction.**
///
/// The whole placement half of `Pass 253.2` in one assertion: before this,
/// every sticky note pdfcer ever authored carried `/Note` because the field
/// did not exist.
///
/// ★★ The `stamp` assertion beside it is the **positive control for the
/// route**, not decoration. `StampName` already travelled this exact path,
/// so asserting the two together is what says the icon was added *to* a
/// working carrier rather than replacing one — and if a later edit dropped
/// either field out of the `Action::CommitTextAnnot` literal, the surviving
/// assertion would still be about a live route.
#[test]
fn the_chosen_icon_reaches_the_commit_action() {
    let mut d = TextAnnotDialog::open(3, TextAnnotKind::Sticky, rect());
    assert_eq!(
        d.icon, DEFAULT_STICKY_ICON,
        "a fresh chooser opens on Acrobat's default, not the engine's"
    );
    d.icon = StickyIcon::Key;
    d.stamp = StampName::Final;
    d.stamp_size = StampSize::Points(24);
    d.text = "note".to_owned();
    d.accept_requested = true;

    let ctx = egui::Context::default();
    let mut actions = Vec::new();
    let _ = ctx.run_ui(on_screen(0), |ui| {
        d.show(ui.ctx(), &mut actions);
    });

    let Some(Action::CommitTextAnnot {
        icon,
        stamp,
        stamp_size,
        ..
    }) = actions.first()
    else {
        panic!("Accept must raise a commit, got {actions:?}");
    };
    assert_eq!(*icon, StickyIcon::Key, "the operator's icon did not travel");
    assert_eq!(
        *stamp,
        StampName::Final,
        "the field the icon was modelled on must still travel too"
    );
    // ☑ The third operand, added 2026-09-10 with the size chooser, and it
    // is asserted at a NON-default value for the reason the two above are:
    // a field left at its default travels identically whether it is carried
    // or silently reconstructed at the far end.
    assert_eq!(
        *stamp_size,
        StampSize::Points(24),
        "the operator's label size did not travel"
    );
}

/// A fresh dialog carries no words and every chooser's stated default.
///
/// ★★ **The size assertion is the load-bearing one**, and it is not merely
/// completeness. `StampSize`'s own header argues that a fresh gallery must
/// offer the size DERIVED from the drawn box — the behaviour of every build
/// before engine `Pass 287.0` — rather than the engine's flat 12 pt,
/// because adopting the engine default would have shrunk every stamp on the
/// operator's drawings as a side effect of a fix he asked for. That
/// argument is only enforced if something asserts the value.
#[test]
fn a_fresh_dialog_is_empty_and_defaulted() {
    let d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
    assert!(d.text.is_empty(), "no words are invented for the operator");
    assert_eq!(d.stamp, DEFAULT_STAMP);
    assert_eq!(d.icon, DEFAULT_STICKY_ICON);
    assert_eq!(
        d.stamp_size,
        StampSize::FitTheBox,
        "a fresh gallery must offer the size the drawn box implies, \
         not the engine's flat 12 pt"
    );
    assert!(!d.accept_requested);
}

/// ★ The page and the rect are captured, not re-read.
///
/// The property that stops a page change under an open window redirecting
/// the annotation. Asserted on the stored values because there is nothing
/// else to assert it on — the whole point is that nothing re-reads them.
#[test]
fn the_page_and_rect_are_captured_at_open() {
    let d = TextAnnotDialog::open(7, TextAnnotKind::Sticky, rect());
    assert_eq!(d.page, 7);
    assert!((d.rect.urx - 100.0).abs() < f64::EPSILON);
}

/// ★ Accept is live for a stamp with no typed text, and dead for the
/// others.
///
/// The readiness rule, which is the gallery exception stated once more at
/// the control that depends on it. A stamp whose Accept required typing
/// could never be authored; a callout whose Accept did not would author an
/// empty box.
#[test]
fn readiness_follows_the_gallery_rule() {
    let ready = |d: &TextAnnotDialog| d.kind.uses_gallery() || !d.text.trim().is_empty();

    let stamp = TextAnnotDialog::open(0, TextAnnotKind::Stamp, rect());
    assert!(ready(&stamp), "a stamp needs no typed words");

    let mut box_ = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
    assert!(!ready(&box_), "an empty callout must not be authorable");
    box_.text = "   ".to_owned();
    assert!(!ready(&box_), "whitespace is not words");
    box_.text = "note".to_owned();
    assert!(ready(&box_));
}

/// **The oracle for *"it doesn't type anything in the box when I type"*.**
///
/// Every test above asserts on the struct's fields, which is exactly the
/// blind spot `DEFECTS.md` D1 was: they all pass on a build whose window
/// accepts no keystrokes, because none of them ever draws one. This drives
/// a real `egui::Context` through two frames — one to build the field and
/// take its one-shot focus, one carrying a real `Event::Text` — and asserts
/// the words arrived.
#[test]
fn typing_into_the_open_window_reaches_the_draft() {
    let ctx = egui::Context::default();
    let mut d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
    let mut actions = Vec::new();

    // Frame 0: the field is created and requests focus.
    let _ = ctx.run_ui(on_screen(0), |ui| {
        d.show(ui.ctx(), &mut actions);
    });

    // Frame 1: a real keystroke, the way a keyboard delivers one.
    let mut input = on_screen(1);
    input.events.push(egui::Event::Text("h".to_owned()));
    let _ = ctx.run_ui(input, |ui| {
        d.show(ui.ctx(), &mut actions);
    });

    assert_eq!(d.text, "h", "the window took the keystroke");
}

/// ★★ **The regression test: focus LOST on the opening frame is re-taken.**
///
/// The defect this replaced latched on having *asked* for focus rather than
/// on holding it, so a request that lost its frame was never retried and
/// the field sat there looking typeable while every keystroke went
/// elsewhere. That is unreachable in a bare `egui::Context` — the request
/// always wins when nothing competes — which is why the test above passed
/// on the broken build and why this one takes the focus away by hand.
///
/// The theft models what the real frame does: the dialog's first draw is
/// the frame AFTER the gesture that opened it, so the pointer release that
/// finished the drag is still being resolved around the request.
#[test]
fn focus_stolen_on_the_opening_frame_is_taken_back() {
    let ctx = egui::Context::default();
    let mut d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
    let mut actions = Vec::new();
    let thief = egui::Id::new("whatever-won-the-release");

    // Frame 0: the dialog draws and asks for focus...
    let _ = ctx.run_ui(on_screen(0), |ui| {
        d.show(ui.ctx(), &mut actions);
    });
    // ...and loses it, the way a release being resolved would take it.
    ctx.memory_mut(|m| m.request_focus(thief));

    // The retry frames. Bounded by the budget rather than assuming one
    // frame is enough: when two widgets ask for focus in the same pass egui
    // keeps the earlier request, so the field can need a second attempt to
    // win it back. The claim under test is *"within the budget"*, which is
    // what the production code promises - not *"on the very next frame"*.
    for frame in 1..=u32::from(FOCUS_ATTEMPT_FRAMES) {
        let _ = ctx.run_ui(on_screen(frame), |ui| {
            d.show(ui.ctx(), &mut actions);
        });
    }

    // The keystroke, which is the assertion that matters -- "focus was
    // requested" is the very claim that shipped broken.
    let mut input = on_screen(u32::from(FOCUS_ATTEMPT_FRAMES) + 1);
    input.events.push(egui::Event::Text("h".to_owned()));
    let _ = ctx.run_ui(input, |ui| {
        d.show(ui.ctx(), &mut actions);
    });

    assert_eq!(
        d.text, "h",
        "the field lost focus on its opening frame and never took it back, so the operator \
         types into a window that is ignoring them"
    );
}

/// ★ ...and the retry is BOUNDED, so Cancel stays clickable.
///
/// The objection the original one-shot latch was written to answer, and it
/// is still correct: a field that asks for focus every frame takes it back
/// from whatever the operator clicked, and a window that cannot be
/// dismissed is worse than one that cannot be typed into.
///
/// The competitor is a **real drawn button**, not a bare `Id`. egui drops
/// focus for an id no widget registered that frame, so focusing an invented
/// id proves nothing about who won — it only proves egui tidied up.
#[test]
fn the_focus_retry_gives_up_so_another_control_can_hold_it() {
    let ctx = egui::Context::default();
    let mut d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
    let mut actions = Vec::new();
    let mut other = None;

    // A real button, drawn every frame beside the dialog, taking focus the
    // way a control the operator clicked would. Its id is read back from
    // the `Response` rather than invented, so the assertion names the
    // widget egui actually registered.
    let mut n = 0;
    let mut frame = |steal: bool, d: &mut TextAnnotDialog, other: &mut Option<egui::Id>| {
        n += 1;
        let _ = ctx.run_ui(on_screen(n), |ui| {
            d.show(ui.ctx(), &mut actions);
            let r = ui.button("Cancel");
            *other = Some(r.id);
            if steal {
                r.request_focus();
            }
        });
    };

    // Outlast the budget, taking focus back every single frame.
    for _ in 0..(FOCUS_ATTEMPT_FRAMES as usize + 2) {
        frame(true, &mut d, &mut other);
    }
    // One more frame with nobody competing: the field must NOT grab it.
    frame(false, &mut d, &mut other);

    assert_eq!(
        ctx.memory(|m| m.focused()),
        other,
        "the field kept grabbing focus back, so nothing else in the window can be used"
    );
}

// ─────────────────────────────────────────────────────────────────────
// O172 — the operator's own stamps in the gallery
// ─────────────────────────────────────────────────────────────────────

/// One of his stamps, as the library would hand it over.
///
/// The values are his: `Signatures` is the category his own collection
/// declares and `Ken` is one of the two stamps in it. Using the real shape
/// rather than `foo`/`bar` costs nothing and means a failure message names
/// something a reader can go and look at.
fn a_custom_stamp() -> CustomStamp {
    CustomStamp {
        label: "Ken".to_owned(),
        category: "Signatures".to_owned(),
        file: std::path::PathBuf::from("Signatures.pdf"),
        page_index: 0,
        dynamic: false,
    }
}

/// **The invariant the whole gallery rests on: exactly one selection.**
///
/// ★★★ This is the assertion that stands between the operator and the
/// worst bug this feature could have had — picking `Approved` and getting
/// his signature, silently, on the second click of a session. `radio_value`
/// would have shipped it: it writes one variable and knows nothing about
/// the other, and the commit path reads `custom` first.
///
/// Both directions, deliberately. A one-way test passes on a
/// [`TextAnnotDialog::select_standard`] that forgets to clear, or on a
/// [`TextAnnotDialog::select_custom`] that forgets to set — and the
/// project's own standing lesson is that a suite trying one sign is not
/// testing the value.
#[test]
fn exactly_one_of_the_two_galleries_holds_the_selection() {
    let mut d = TextAnnotDialog::open(0, TextAnnotKind::Stamp, rect());
    assert!(
        d.custom.is_none(),
        "a fresh gallery opens on a standard stamp"
    );

    d.select_custom(a_custom_stamp());
    assert_eq!(
        d.custom.as_ref().map(|c| c.label.as_str()),
        Some("Ken"),
        "his stamp did not become the selection"
    );

    d.select_standard(StampName::Final);
    assert!(
        d.custom.is_none(),
        "picking a standard stamp must CLEAR his — otherwise he chooses \
         `Final` and gets his signature"
    );
    assert_eq!(d.stamp, StampName::Final);

    // …and back again, so neither direction is the only one exercised.
    d.select_custom(a_custom_stamp());
    assert!(d.custom.is_some(), "the selection did not return");
}

/// **His stamp reaches the commit action**, which is the only thing that
/// makes the gallery more than a picture.
///
/// ★ Asserted alongside `stamp` still carrying a NON-default value, for
/// the positive-control reason `the_chosen_icon_reaches_the_commit_action`
/// gives: if a later edit dropped `custom` out of the action literal, the
/// surviving assertion is still about a live route.
#[test]
fn the_operators_own_stamp_reaches_the_commit_action() {
    let mut d = TextAnnotDialog::open(2, TextAnnotKind::Stamp, rect());
    d.stamp = StampName::Final;
    d.select_custom(a_custom_stamp());
    d.accept_requested = true;

    let ctx = egui::Context::default();
    let mut actions = Vec::new();
    let _ = ctx.run_ui(on_screen(0), |ui| {
        d.show(ui.ctx(), &mut actions);
    });

    let Some(Action::CommitTextAnnot { custom, stamp, .. }) = actions.first() else {
        panic!("Accept must raise a commit, got {actions:?}");
    };
    let carried = custom.as_ref().expect("his stamp did not travel");
    assert_eq!(carried.label, "Ken");
    assert_eq!(carried.page_index, 0);
    assert_eq!(
        carried.file,
        std::path::PathBuf::from("Signatures.pdf"),
        "the PATH must travel — the far end has to reopen the collection"
    );
    assert_eq!(
        *stamp,
        StampName::Final,
        "the standard field must still travel: this route reads `custom` \
         first, and an assertion on a dead field proves nothing"
    );
}

/// **Only the stamp kind pays for a filesystem scan.**
///
/// A sticky note and a text box cannot reach a gallery, so opening one
/// must not walk Acrobat's stamps folder. This asserts the *observable*
/// consequence — an empty library — rather than counting `Document::load`
/// calls, because the count is the mechanism and the emptiness is the
/// contract.
///
/// ⚠ It is NOT an assertion that the machine has no stamps. On a machine
/// with none, all three kinds produce an empty library and this test is
/// vacuous — a real limitation, written down rather than papered over. The
/// falsifying case is the operator's own machine, where the stamp kind
/// finds two and this test would go red if the guard were removed.
#[test]
fn only_the_stamp_kind_scans_the_stamps_folder() {
    for kind in [TextAnnotKind::Sticky, TextAnnotKind::TextBox] {
        let d = TextAnnotDialog::open(0, kind, rect());
        assert!(
            d.library.is_empty(),
            "{kind:?} has no gallery and must not walk the disk"
        );
        assert!(
            d.library.folder.is_none(),
            "{kind:?} must not even resolve the folder"
        );
    }
}

/// **R9, both ways: an empty library draws NOTHING, a full one draws.**
///
/// An unavailable capability renders nothing — no empty group, no heading
/// over an empty list, no *"you can add your own"* invitation on a machine
/// that has never made a stamp.
///
/// ★★ The second half is what stops this being vacuous. An
/// absence-assertion alone passes on a `custom_stamps` that returns early
/// unconditionally — which is to say, on a build where the whole feature
/// is missing. The populated case is the control: it fails if the function
/// never draws, and the empty case fails if it always does.
#[test]
fn the_custom_half_appears_only_when_there_is_something_in_it() {
    let ctx = egui::Context::default();

    let mut empty = TextAnnotDialog::open(0, TextAnnotKind::Stamp, rect());
    empty.library = Library::default();
    let mut full = TextAnnotDialog::open(0, TextAnnotKind::Stamp, rect());
    full.library = Library {
        categories: vec![Category {
            name: "Signatures".to_owned(),
            named_from_file: false,
            stamps: vec![a_custom_stamp()],
        }],
        folder: None,
        unreadable: 0,
        unplaceable: 0,
    };

    let (mut drew_empty, mut drew_full) = (true, false);
    let _ = ctx.run_ui(on_screen(0), |ui| {
        drew_empty = empty.custom_stamps(ui);
        drew_full = full.custom_stamps(ui);
    });

    assert!(
        !drew_empty,
        "with no stamps the custom half must not exist at all"
    );
    assert!(
        drew_full,
        "with stamps it must — otherwise O172 shipped dead"
    );
}

/// ★★★ **The stamp window opens tall enough to show the operator's own
/// stamps** — the defect the first driven run of O172 found, 2026-09-10.
///
/// `custom_stamp_reaches_the_page` armed Markup ▸ Stamp, dragged a box, and
/// captured a dialog that showed the seven standard stamps and the Add/Cancel
/// row with **none** of his three stamps on it. They were laid out below the
/// scrolled body's fold, at content y 298, 326 and 354, inside a body that
/// ended at 270 — published as rectangles, invisible as controls, and the
/// harness clicked the first of them into the dialog's own drop shadow.
///
/// Nothing was clipped wrongly and nothing was laid out wrongly.
/// [`window_size`] added a per-kind constant written for a body that did not
/// yet contain this section, and **a guessed size is a claim about the content
/// that the content can outgrow**.
///
/// ★★ The two negative controls are what make this a test rather than an
/// observation. Asserting only *"the stamp window got taller"* passes on a
/// `window_size` that ignored its new argument and grew unconditionally, and
/// passes on one that applied the growth to every kind — the sticky note and
/// the text box would each gain a strip of empty window, and no assertion here
/// would say so.
#[test]
fn the_stamp_window_grows_for_the_operators_own_stamps() {
    let screen = screen();
    let extra = custom_extra_pts(&a_library());
    assert!(
        extra > 0.0,
        "the fixture library must ask for height, or the rest of this proves nothing"
    );

    let without = window_size(screen, TextAnnotKind::Stamp, 0.0);
    let with = window_size(screen, TextAnnotKind::Stamp, extra);
    assert!(
        with.y > without.y,
        "a stamp dialog on a machine WITH custom stamps must be taller than one \
         without: {with:?} vs {without:?}. This is the O172 defect — his stamps \
         were drawn below the fold of a window sized before they existed."
    );

    // ★ Negative control 1: the other two kinds have no gallery to put a
    // custom half in, so the same argument must move neither.
    for kind in [TextAnnotKind::Sticky, TextAnnotKind::TextBox] {
        assert_eq!(
            window_size(screen, kind, extra).y,
            window_size(screen, kind, 0.0).y,
            "{kind:?} has no custom gallery and must not grow for one"
        );
    }
    // ★ Negative control 2: a machine with no stamps gets the window it always
    // had. R9's height twin — an unavailable capability costs nothing, not even
    // empty space.
    assert_eq!(
        custom_extra_pts(&Library::default()),
        0.0,
        "an empty library must add no height at all"
    );
}

/// **The height asked for grows with the collection, and stops.**
///
/// Both halves matter and neither is obvious from the other. Growth is the
/// feature; the cap is what stops a forty-stamp collection opening a dialog
/// the full height of the application window, standing over the drawing being
/// annotated. Past the cap the body scrolls, which is what a scroll area is
/// for.
#[test]
fn the_custom_half_asks_for_more_room_up_to_a_stated_limit() {
    let small = custom_extra_pts(&a_library());
    let big = custom_extra_pts(&a_library_of(40));
    assert!(
        big > small,
        "more stamps must ask for more room: {big} is not more than {small}"
    );
    assert!(
        big <= CUSTOM_EXTRA_MAX_PTS,
        "forty stamps asked for {big} pt, past the {CUSTOM_EXTRA_MAX_PTS} pt cap — \
         the dialog would stand over the whole sheet he is annotating"
    );
}

/// A library shaped like the check's planted fixture: one category, three
/// stamps, one of them dynamic.
fn a_library() -> Library {
    Library {
        categories: vec![Category {
            name: "Site Review".to_owned(),
            named_from_file: false,
            stamps: vec![
                CustomStamp {
                    label: "Issued".to_owned(),
                    category: "Site Review".to_owned(),
                    file: std::path::PathBuf::from("collection-under-test.pdf"),
                    page_index: 2,
                    dynamic: true,
                },
                CustomStamp {
                    label: "For Review".to_owned(),
                    category: "Site Review".to_owned(),
                    file: std::path::PathBuf::from("collection-under-test.pdf"),
                    page_index: 1,
                    dynamic: false,
                },
                CustomStamp {
                    label: "Approved".to_owned(),
                    category: "Site Review".to_owned(),
                    file: std::path::PathBuf::from("collection-under-test.pdf"),
                    page_index: 0,
                    dynamic: false,
                },
            ],
        }],
        folder: None,
        unreadable: 0,
        unplaceable: 0,
    }
}

/// A library of `n` stamps in one category, for the cap.
fn a_library_of(n: usize) -> Library {
    let mut lib = a_library();
    lib.categories[0].stamps = (0..n)
        .map(|i| CustomStamp {
            label: format!("Stamp {i}"),
            category: "Site Review".to_owned(),
            file: std::path::PathBuf::from("collection-under-test.pdf"),
            page_index: i,
            dynamic: false,
        })
        .collect();
    lib
}
