//! # `app::status::page_box` — page navigation, and the box you type into
//!
//! The status bar's right-hand cluster `⏴ ⟨n⟩ / ⟨N⟩ ⏵`, and the control this
//! whole stage exists to build. Split out of [`crate::app::status`] under
//! standing rule **R2** (no `.rs` file over 1,500 lines), and the seam is a
//! real one rather than a cut made to satisfy a gate:
//!
//! - Everything left in the parent answers *"how is the bar laid out, and
//!   what does each group show?"* — a fixed row, a demoted narrator, and two
//!   clusters of stateless buttons that read `doc.view` and push an
//!   [`Action`].
//! - Everything here answers *"what did the operator mean by what they
//!   typed?"* — which is a different question with its own state
//!   ([`PageBox`]), its own vocabulary ([`PageCommit`], [`Note`]), its own
//!   pure decision function ([`resolve`]) and its own hazard (defect D1's
//!   keyboard guard). None of that is shared with anything left behind.
//!
//! The parent's module docs carry the *why* for the surface as a whole,
//! including the four properties this control is judged on and the R128
//! fixed-height argument. What follows is the part that only concerns the
//! box.
//!
//! ## ★ Why an editable box at all
//!
//! `GUI_ROADMAP.md` 3.3: *"Reaching page 37 of 42 currently means the
//! thumbnail rail or 36 keystrokes."* Type `37`, press Enter, arrive.
//!
//! ## ★ The commit rule
//!
//! **Enter or focus loss. Never a keystroke.** Someone typing `42` passes
//! through `4`; a box that navigated per keystroke would take them to page
//! 4, rasterize a CAD sheet nobody asked for, and then take them to page 42.
//! [`Response::lost_focus`](egui::Response::lost_focus) covers both exits at
//! once, because egui's single-line `TextEdit` surrenders focus when its
//! return key is pressed (`egui-0.35.0/src/widgets/text_edit/builder.rs:1115`
//! — *"End input with enter"*). One commit path, so the two cannot disagree.
//!
//! `Escape` is the third exit and it **cancels**: the draft is dropped and
//! the box goes back to showing the current page. Without it, an operator
//! who started typing a page number has no way out but to retype the one
//! they were already on.
//!
//! ## ★ The three outcomes, and why none of them is silent
//!
//! | typed | outcome | what the operator sees |
//! |---|---|---|
//! | `37` in a 42-page document | [`PageCommit::Go`] | the page changes; the box reads `37` |
//! | `99` in a 42-page document | [`PageCommit::Clamped`] | the page changes to 42, **and a note names the number that does not exist** |
//! | `abc` | [`PageCommit::NotANumber`] | nothing moves, **`abc` stays in the box**, and a note says why |
//!
//! A silent clamp is indistinguishable from a box that ignored what was
//! typed, and an operator who cannot tell those apart stops trusting the
//! control. A refusal that *also* wiped the field would destroy the evidence
//! of what they meant — which is why the draft outlives a failed commit and
//! never outlives a successful one.
//!
//! ## ★ Defect D1, from the other end
//!
//! `crate::app::keyboard` guards the unmodified bindings — PageUp, PageDown,
//! Home, End, Delete — with `ctx.text_edit_focused()`, **not**
//! `egui_wants_keyboard_input()`. The latter means "any widget has focus",
//! the canvas takes focus on click, and the difference cost the operator
//! every one of those keys from the first click onward.
//!
//! `text_edit_focused()` resolves the focused id and asks whether a
//! `TextEditState` exists **for that id**. So this control has to be a real
//! [`egui::TextEdit`] with a stable, explicit id, or the guard cannot see it
//! — and `PageDown` would step the page while the operator was halfway
//! through typing `42`. A `DragValue` in display mode, a painted field, or a
//! label with a popup would each typecheck and each re-open D1 in reverse.
//!
//! [`tests::typing_a_digit_into_the_page_box_does_not_also_step_the_page`]
//! is the regression test, and it asserts the *failing condition is really
//! present* before asserting the fix — the shape the D1 post-mortem says the
//! original test was missing.

use egui::{Align, Id, Key, TextEdit};

use crate::app::actions::Action;
use crate::app::prefs::WheelPaging;
use crate::app::state::OpenDoc;
use crate::text::status as t;

/// How wide the editable page box is.
///
/// Four digits of a proportional face plus the field's own margins: enough
/// for `9999` without eliding, which covers every document anyone has opened
/// in this application. A box that grew with the page count would move the
/// two step buttons every time a different document was opened.
const PAGE_BOX_WIDTH_PTS: f32 = 44.0;

/// The most characters the page box will hold.
///
/// Not a validation rule — [`resolve`] is what decides whether the text
/// names a page — but a guard against an operator pasting a paragraph into a
/// 44-point-wide field and losing the control's shape. Nine digits is past
/// any real page count and past the point where the text is legible anyway.
const PAGE_BOX_MAX_CHARS: usize = 9;

/// Named region: `⏴ ⟨n⟩ / ⟨N⟩ ⏵`, plus the clamp/reject note when there is
/// one.
const REGION_PAGE: &str = "status-group:page"; // ui-text-exempt: trace region name, never displayed

/// Named region: the editable box alone.
///
/// Named separately from [`REGION_PAGE`] because it is the control this
/// stage exists to build: a legibility or hit-target check wants the field
/// itself, not the field plus two buttons and a possible note.
const REGION_PAGE_BOX: &str = "status-page-box"; // ui-text-exempt: trace region name, never displayed

/// The wheel-paging toggle, so a driven check can find it and assert that it
/// is absent under a continuous display mode as well as present under a
/// single-page one.
const REGION_WHEEL_PAGING: &str = "status-wheel-paging"; // ui-text-exempt: trace region name, never displayed

/// The page box's `egui` id.
///
/// **Fixed and explicit, not auto-generated.** Three things depend on it
/// being stable across frames: `TextEdit` finds its own cursor and selection
/// under it, `ctx.text_edit_focused()` finds the `TextEditState` under it
/// (defect D1's guard — see the module docs), and [`tests`] request focus on
/// it directly. An auto id derived from widget order would change the moment
/// a note appeared beside the box.
const PAGE_BOX_ID: &str = "pdfcer-status-page-box"; // ui-text-exempt: widget id, never displayed

/// Where the draft text and the last commit's verdict are kept.
const PAGE_STATE_ID: &str = "pdfcer-status-page-state"; // ui-text-exempt: widget id, never displayed

/// Draw `⏴ ⟨n⟩ / ⟨N⟩ ⏵`, plus the last commit's note when there is one.
///
/// Omitted entirely for a document with no pages (`/Count 0` is legal PDF):
/// every input to a page box over such a document is out of range, and a
/// control whose every answer is "no" is not a control.
pub(super) fn group(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    wheel_paging: &mut WheelPaging,
    actions: &mut Vec<Action>,
) {
    let page_count = doc.pages.len();
    if page_count == 0 {
        return;
    }
    let current = doc.view.page_index;
    let state_id = Id::new(PAGE_STATE_ID);
    let mut state = ui
        .ctx()
        .data_mut(|d| d.get_temp::<PageBox>(state_id))
        .unwrap_or_default();

    // ★ A clamp note describes where the LAST commit landed. The moment the
    // operator is looking at some other page it is describing history, and a
    // stale explanation beside a live control is worse than no explanation:
    // it attaches a reason to a page the operator reached by other means.
    //
    // Expressed as "the note is true while you are still where it put you"
    // rather than as a timer, which keeps it deterministic and testable.
    if let Some(Note::Clamped { landed, .. }) = state.note
        && landed != current
    {
        state.note = None;
    }

    let rect = ui
        .scope(|ui| {
            // The bar's right-hand cluster is laid out RIGHT-TO-LEFT (see
            // `super::show`), so the widget added first is drawn rightmost.
            // The screen reads `[note] ⏴ ⟨n⟩ / ⟨N⟩ ⏵`.
            if ui
                .button(t::next_page())
                .on_hover_text(t::next_page_tooltip())
                .clicked()
            {
                actions.push(Action::NextPage);
            }
            ui.label(t::page_of_total(page_count));
            field(ui, &mut state, current, page_count, actions);
            if ui
                .button(t::prev_page())
                .on_hover_text(t::prev_page_tooltip())
                .clicked()
            {
                actions.push(Action::PrevPage);
            }
            wheel_toggle(ui, doc, wheel_paging);
            // The note sits at the LEFT end of the group, next to ⏴, so that
            // appearing and disappearing pushes into the empty middle of the
            // bar rather than shifting the buttons the operator is clicking.
            if let Some(note) = state.note {
                let text = match note {
                    Note::Clamped { asked, landed } => {
                        t::page_clamped_note(asked, landed + 1, page_count)
                    }
                    Note::NotANumber => t::page_rejected_note().to_owned(),
                };
                ui.label(egui::RichText::new(text).small().weak());
            }
        })
        .response
        .rect;

    crate::diag::ui_rect(REGION_PAGE, rect);
    ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
}

/// **What the wheel does on a single page**, offered where the operator is
/// already thinking about pages — `OPERATOR_REQUESTS.md` O30.
///
/// > *"when in single page view there should be an option on screen near the
/// > button to scroll or flip through pages, or the current way it is now when
/// > the scroll wheel is used."*
///
/// # ★★★ It renders NOTHING under a continuous display mode
///
/// R9: an unavailable capability renders nothing, and greying is reserved for
/// *temporarily* unavailable. Under
/// [`crate::viewer::PageDisplay::Continuous`] the wheel scrolls the whole
/// document **by definition** — there is no second answer to offer, so there
/// is no control. A disabled toggle there would be a permanent apology for a
/// choice that does not exist.
///
/// ★ It sits immediately to the left of `⏴`, inside the page group's own
/// right-to-left scope, so it is adjacent to the two buttons it is an
/// alternative to. Placing it in the empty middle of the bar would put the
/// question a hand's width from its subject.
///
/// # A toggle, not a pair of labels
///
/// The two answers are not peers: one is what the build has always done and
/// the other is the departure from it. A pressed/unpressed control says that
/// — *flipping is on* — where two `selectable_label`s would present them as
/// equals and cost twice the width in a 24-point bar. The tooltip carries
/// both sentences, and the settings window carries the full argument for each.
fn wheel_toggle(ui: &mut egui::Ui, doc: &OpenDoc, wheel_paging: &mut WheelPaging) {
    if doc.view.display.is_continuous() {
        return;
    }
    let flipping = wheel_paging.flips();
    let response = ui
        .selectable_label(flipping, t::wheel_flip_pages())
        .on_hover_text(t::wheel_flip_pages_tooltip());
    if response.clicked() {
        *wheel_paging = if flipping {
            WheelPaging::Scroll
        } else {
            WheelPaging::FlipPages
        };
    }
    crate::diag::ui_rect(REGION_WHEEL_PAGING, response.rect);
}

/// The editable field itself: draw it, and commit it when it is committed.
///
/// See the module docs for the commit rule, the three outcomes, and why this
/// must be a real [`egui::TextEdit`].
///
/// While a draft exists the box shows the draft and **not** the page, which
/// is what makes rejection non-destructive: refusing `abc` leaves `abc` in
/// the field with a note beside it, rather than silently restoring the
/// current page and leaving the operator to wonder whether the field is
/// broken or they mistyped.
fn field(
    ui: &mut egui::Ui,
    state: &mut PageBox,
    current: usize,
    page_count: usize,
    actions: &mut Vec<Action>,
) {
    let id = Id::new(PAGE_BOX_ID);
    let mut text = state
        .draft
        .clone()
        .unwrap_or_else(|| t::page_number(current + 1));

    // ★ A real `egui::TextEdit`, with a stable explicit id — defect D1's
    // guard resolves the focused id and looks for a `TextEditState` under
    // it. See the module docs.
    let response = ui
        .add(
            TextEdit::singleline(&mut text)
                .id(id)
                .desired_width(PAGE_BOX_WIDTH_PTS)
                .horizontal_align(Align::Center)
                .char_limit(PAGE_BOX_MAX_CHARS),
        )
        .on_hover_text(t::page_box_tooltip());
    crate::diag::ui_rect(REGION_PAGE_BOX, response.rect);

    if response.changed() {
        // A keystroke starts a NEW attempt, so the previous attempt's
        // verdict stops applying. Clearing it here — rather than on commit —
        // is what stops "Not a page number" sitting beside text the operator
        // has already corrected.
        state.draft = Some(text.clone());
        state.note = None;
    }

    // ★ The commit gate. Everything above runs every frame; nothing below
    // runs until the operator has finished.
    if !response.lost_focus() {
        return;
    }

    if ui.input(|i| i.key_pressed(Key::Escape)) {
        state.draft = None;
        state.note = None;
        return;
    }

    match resolve(&text, page_count) {
        PageCommit::Go(index) => {
            state.draft = None;
            state.note = None;
            actions.push(Action::GoToPage(index));
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "status-page-commit outcome=go asked={} index={index}",
                    index + 1
                )
            });
        }
        PageCommit::Clamped { asked, landed } => {
            state.draft = None;
            state.note = Some(Note::Clamped { asked, landed });
            actions.push(Action::GoToPage(landed));
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "status-page-commit outcome=clamped asked={asked} index={landed} pages={page_count}"
                )
            });
        }
        PageCommit::NotANumber => {
            // ★ The text is KEPT. See the module docs.
            state.draft = Some(text);
            state.note = Some(Note::NotANumber);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "status-page-commit outcome=rejected".to_owned()
            });
        }
        PageCommit::Empty => {
            // An emptied box is not an error and not a request. Forgetting
            // the draft puts the current page back, which is the only
            // sentence a blank field could honestly be read as.
            state.draft = None;
            state.note = None;
        }
    }
}

// ---------------------------------------------------------------------------
// The box's state and its one decision
// ---------------------------------------------------------------------------

/// Everything the page box remembers between frames.
///
/// Kept in `egui`'s per-id store beside the `TextEditState` it belongs with.
/// [`crate::app::status`]'s module docs carry the argument for why that is
/// right for a text-editing draft and wrong for a canvas selection: this
/// value is discarded on focus loss, always, so it cannot outlive a document
/// and there is no identity to key on.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub(super) struct PageBox {
    /// What the operator has typed and not yet successfully committed.
    ///
    /// `None` means "show the current page". A draft outlives a **failed**
    /// commit on purpose, and never outlives a successful one.
    draft: Option<String>,
    /// The last commit's verdict, when it was worth saying out loud.
    note: Option<Note>,
}

/// What the last commit did, when it did something the operator should be
/// told about.
///
/// There is no `Ok` variant: a commit that went exactly where it was asked
/// needs no explanation, and a note beside every successful navigation would
/// train the operator to stop reading the ones that matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Note {
    /// The number was outside the document. `asked` is the 1-based number
    /// typed; `landed` is the **0-based** index it clamped to, so it can be
    /// compared directly against `view.page_index` to decide whether the
    /// note still describes where the operator is.
    Clamped { asked: usize, landed: usize },
    /// The text was not a page number at all.
    NotANumber,
}

/// What a committed string resolves to.
///
/// A separate type from [`Note`] because the two answer different questions:
/// this one says *what to do*, including the successful cases; `Note` says
/// *what to tell the operator*, which is only the surprising subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageCommit {
    /// Go to this 0-based page index.
    Go(usize),
    /// The number was out of range. Go to `landed` (0-based) and say so.
    Clamped { asked: usize, landed: usize },
    /// Not a page number. Change nothing, keep the text, say so.
    NotANumber,
    /// Nothing was typed. Change nothing, say nothing.
    Empty,
}

/// Decide what a committed page-box string means.
///
/// **Pure, and that is the point.** `crate::viewer`'s header states the
/// project's split — *"this module is unit-testable and the widget code is
/// not"* — and every property this control is judged on lives here rather
/// than inside an `egui` closure: what counts as a number, what happens at
/// the ends of the document, what happens to nonsense.
///
/// # The rules, and why each is what it is
///
/// - **Surrounding whitespace is ignored.** `" 37 "` is a pasted page
///   number, not a typo, and refusing it would be pedantry the operator has
///   to work around by hand.
/// - **A leading `+` is accepted, a leading `-` is not.** `+37` is a number;
///   `-37` is a *different* number, one no document has, and silently
///   reading it as 37 would be inventing an intent. It falls to
///   [`PageCommit::NotANumber`], which keeps the text so the operator can
///   see what they typed.
/// - **Only ASCII digits count.** A full-width `３` or an Arabic-Indic `٣`
///   is refused rather than guessed at; pdfcer has no locale model, and a
///   half-implemented one that worked for three scripts would be worse than
///   an honest refusal.
/// - **An absurdly long run of digits is a number, not nonsense.**
///   `99999999999999999999` overflows `usize`, and `parse` would report that
///   as an error indistinguishable from `abc`. It is saturated to
///   [`usize::MAX`] instead, so it clamps to the last page and *reports the
///   clamp* — which is the honest answer to "go to page ten quintillion".
/// - **Page 0 does not exist.** The box is 1-based (see
///   [`crate::text::status::page_number`]), so `0` is out of range at the
///   near end and clamps to page 1 with a note, exactly as `99` clamps to
///   the far end with one.
/// - **The clamp is `crate::viewer::clamp_page_index`**, not a second
///   spelling of the same arithmetic. There is one rule for "which page is
///   this really", it is already tested against the empty-document case, and
///   a private copy here is how two clamps drift apart.
///
/// `page_count == 0` yields [`PageCommit::Empty`]: there is nowhere to go.
/// The caller never asks — [`group`] draws nothing for a document with no
/// pages — but a decision function that panicked or invented an answer for
/// an input its caller happens not to produce is one refactor away from
/// being wrong.
#[must_use]
fn resolve(text: &str, page_count: usize) -> PageCommit {
    let trimmed = text.trim();
    if trimmed.is_empty() || page_count == 0 {
        return PageCommit::Empty;
    }
    let digits = trimmed.strip_prefix('+').unwrap_or(trimmed);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return PageCommit::NotANumber;
    }
    // Saturating rather than propagating the overflow: see the docs above.
    let asked: usize = digits.parse().unwrap_or(usize::MAX);
    if (1..=page_count).contains(&asked) {
        return PageCommit::Go(asked - 1);
    }
    PageCommit::Clamped {
        asked,
        landed: crate::viewer::clamp_page_index(asked.saturating_sub(1), page_count),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::status::show;
    use crate::app::status::test_support::{frame, key_press, opened};
    use egui::{Context, Modifiers, RawInput};

    // =======================================================================
    // The commit decision — pure, so every property is pinned without a window
    // =======================================================================

    /// An in-range number goes exactly where it says.
    #[test]
    fn an_in_range_number_goes_to_that_page() {
        assert_eq!(resolve("1", 42), PageCommit::Go(0));
        assert_eq!(resolve("37", 42), PageCommit::Go(36));
        assert_eq!(resolve("42", 42), PageCommit::Go(41));
    }

    /// ★ **The box is 1-based and the action is 0-based**, and the
    /// conversion happens exactly once.
    ///
    /// An off-by-one here is the most likely defect this control can carry
    /// and the least likely to be noticed in review: typing `37` and landing
    /// on 36 looks like a rendering delay until you check twice.
    #[test]
    fn the_displayed_number_and_the_action_index_differ_by_exactly_one() {
        for page in 1..=42usize {
            assert_eq!(resolve(&t::page_number(page), 42), PageCommit::Go(page - 1));
        }
    }

    /// Whitespace around a pasted number is ignored.
    #[test]
    fn surrounding_whitespace_is_ignored() {
        assert_eq!(resolve("  7  ", 42), PageCommit::Go(6));
        assert_eq!(resolve("\t7\n", 42), PageCommit::Go(6));
    }

    /// ★ **Out of range clamps into the document, at both ends, and is
    /// reported.**
    ///
    /// The verdict carries the number that was asked for as well as the one
    /// that was given, because that is what
    /// `crate::text::status::page_clamped_note` needs to distinguish "your
    /// number was out of range" from "the box ignored you".
    #[test]
    fn an_out_of_range_number_clamps_and_says_what_it_asked_for() {
        assert_eq!(
            resolve("99", 42),
            PageCommit::Clamped {
                asked: 99,
                landed: 41
            }
        );
        // Page zero does not exist: out of range at the NEAR end.
        assert_eq!(
            resolve("0", 42),
            PageCommit::Clamped {
                asked: 0,
                landed: 0
            }
        );
        // A one-page document clamps everything onto its one page.
        assert_eq!(
            resolve("5", 1),
            PageCommit::Clamped {
                asked: 5,
                landed: 0
            }
        );
    }

    /// A number too large for `usize` is still a number.
    ///
    /// `parse` reports an overflow the same way it reports `abc`, and
    /// treating "go to page ten quintillion" as a typing error would refuse
    /// an input whose meaning is perfectly clear. It clamps to the last page
    /// and reports the clamp.
    #[test]
    fn an_absurdly_large_number_clamps_rather_than_being_refused() {
        assert_eq!(
            resolve("99999999999999999999999999", 42),
            PageCommit::Clamped {
                asked: usize::MAX,
                landed: 41
            }
        );
    }

    /// ★ **Non-numeric input is refused.**
    ///
    /// The other half of the requirement — that the text survives the
    /// refusal — is a property of the widget and is asserted in
    /// [`a_refused_commit_keeps_what_the_operator_typed`].
    #[test]
    fn non_numeric_input_is_refused() {
        for text in ["abc", "3.5", "37a", "-1", "3 7", "٣", "３", "+"] {
            assert_eq!(
                resolve(text, 42),
                PageCommit::NotANumber,
                "{text:?} must not be read as a page number"
            );
        }
    }

    /// A leading `+` is a number; a leading `-` is not.
    #[test]
    fn a_leading_plus_is_accepted_and_a_leading_minus_is_not() {
        assert_eq!(resolve("+7", 42), PageCommit::Go(6));
        assert_eq!(resolve("-7", 42), PageCommit::NotANumber);
    }

    /// An empty box asks for nothing and is told nothing.
    #[test]
    fn an_empty_box_is_not_an_error() {
        assert_eq!(resolve("", 42), PageCommit::Empty);
        assert_eq!(resolve("   ", 42), PageCommit::Empty);
    }

    /// A document with no pages has nowhere to go.
    ///
    /// Unreachable through the widget — [`group`] draws nothing for
    /// `/Count 0` — and pinned anyway, because a decision function that
    /// invented an answer for an input its caller happens not to produce is
    /// one refactor away from being wrong.
    #[test]
    fn a_document_with_no_pages_resolves_to_nothing() {
        assert_eq!(resolve("1", 0), PageCommit::Empty);
    }

    // =======================================================================
    // The widget — driven through a real `egui::Context`
    // =======================================================================

    /// Take keyboard focus on the page box, the way a click on it would.
    ///
    /// Requested *after* a frame has drawn the box, so the `TextEditState`
    /// the D1 guard looks for already exists under the same id.
    fn focus_page_box(ctx: &Context) {
        ctx.memory_mut(|m| m.request_focus(Id::new(PAGE_BOX_ID)));
    }

    /// Read the page box's stored state back out.
    fn page_state(ctx: &Context) -> PageBox {
        ctx.data_mut(|d| d.get_temp::<PageBox>(Id::new(PAGE_STATE_ID)))
            .unwrap_or_default()
    }

    /// Seed the page box's draft, as though the operator had typed it.
    fn set_draft(ctx: &Context, draft: &str) {
        ctx.data_mut(|d| {
            d.insert_temp(
                Id::new(PAGE_STATE_ID),
                PageBox {
                    draft: Some(draft.to_owned()),
                    note: None,
                },
            );
        });
    }

    /// ★ **The D1 regression test, from the page box's end.**
    ///
    /// `crate::app::keyboard`'s guard is `ctx.text_edit_focused()`, and it
    /// only protects a control that egui recognises as a text edit. This
    /// asserts, in order:
    ///
    /// 1. `egui_wants_keyboard_input()` is genuinely `true` — so the test is
    ///    known to be exercising the condition rather than passing
    ///    vacuously. Its absence from the original D1 test is exactly why
    ///    that defect shipped.
    /// 2. `text_edit_focused()` is `true`, i.e. the box really is a
    ///    `TextEdit` under the id the guard resolves. Replace it with a
    ///    `DragValue` in display mode or a painted field and this fails.
    /// 3. With the box focused, `keyboard::collect` installs **no**
    ///    unmodified binding — so a digit typed on the way to `42` cannot
    ///    also page the document.
    /// 4. Typing does not commit. Nothing is raised until Enter or focus
    ///    loss.
    #[test]
    fn typing_a_digit_into_the_page_box_does_not_also_step_the_page() {
        let ctx = Context::default();
        let status = opened();

        // Frame 1: draw the bar so the box exists, then focus it.
        let _ = frame(&ctx, &status, RawInput::default());
        focus_page_box(&ctx);

        // Frame 2: type a digit, and press PageDown in the same frame.
        let input = RawInput {
            events: vec![
                egui::Event::Text("4".to_owned()),
                egui::Event::Key {
                    key: Key::PageDown,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        };
        let mut actions = Vec::new();
        let mut wants_keyboard = false;
        let mut text_focused = false;
        let mut keyboard_actions = Vec::new();
        let _ = ctx.run_ui(input, |ui| {
            let ctx = ui.ctx();
            wants_keyboard = ctx.egui_wants_keyboard_input();
            // typing-guard-exempt: a TEST asserting the harness actually reached
            // the focused state. Reading the raw egui answer is the point - a
            // test that asked `composing()` could not tell a focused widget from
            // a canvas draft, and the thing being proved is that the widget half
            // is reachable at all. D1 shipped because its test could not reach it.
            text_focused = ctx.text_edit_focused();
            keyboard_actions = crate::app::keyboard::collect(ctx, Some(4));
            show(
                ui,
                &status,
                &mut crate::find::FindState::default(),
                &mut crate::canvas::pick::PickFilter::default(),
                &mut crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT.to_owned(),
                &mut crate::app::prefs::WheelPaging::default(),
                &mut actions,
            );
        });

        assert!(
            wants_keyboard,
            "the test is vacuous unless a widget really holds focus — this is \
             the exact condition D1's guard mistook for typing"
        );
        assert!(
            text_focused,
            "the page box must be a real TextEdit under the id the D1 guard \
             resolves, or the unmodified bindings stay live while the \
             operator types a page number"
        );
        assert!(
            keyboard_actions.is_empty(),
            "PageDown must not reach the view while the page box has focus: \
             {keyboard_actions:?}"
        );
        assert!(
            actions.is_empty(),
            "a keystroke must not commit — someone typing 42 passes through \
             4, and a per-keystroke box would navigate there: {actions:?}"
        );
    }

    /// The guard is not permanent: with the box unfocused, `PageDown` works.
    ///
    /// The mirror of the test above, and the reason it matters is D1 itself
    /// — a guard that is always on is exactly as broken as one that is
    /// always off, and it is much harder to notice.
    #[test]
    fn the_page_keys_come_back_the_moment_the_box_loses_focus() {
        let ctx = Context::default();
        let status = opened();
        let _ = frame(&ctx, &status, RawInput::default());

        let mut keyboard_actions = Vec::new();
        let mut actions = Vec::new();
        let _ = ctx.run_ui(key_press(Key::PageDown, Modifiers::NONE), |ui| {
            keyboard_actions = crate::app::keyboard::collect(ui.ctx(), Some(4));
            show(
                ui,
                &status,
                &mut crate::find::FindState::default(),
                &mut crate::canvas::pick::PickFilter::default(),
                &mut crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT.to_owned(),
                &mut crate::app::prefs::WheelPaging::default(),
                &mut actions,
            );
        });
        assert_eq!(keyboard_actions, vec![Action::NextPage]);
    }

    /// ★ **Enter commits.**
    #[test]
    fn enter_commits_the_typed_page() {
        let ctx = Context::default();
        let status = opened();
        let _ = frame(&ctx, &status, RawInput::default());
        focus_page_box(&ctx);
        set_draft(&ctx, "3");

        let actions = frame(&ctx, &status, key_press(Key::Enter, Modifiers::NONE));
        assert_eq!(
            actions,
            vec![Action::GoToPage(2)],
            "page 3 of a four-page document is index 2"
        );
        assert_eq!(
            page_state(&ctx).draft,
            None,
            "a successful commit hands the box back to the document"
        );
    }

    /// ★ **Focus loss commits too**, with no Enter anywhere.
    ///
    /// Clicking away from a half-typed field and having it silently discard
    /// the number is the behaviour operators describe as "it didn't take".
    #[test]
    fn losing_focus_commits_the_typed_page() {
        let ctx = Context::default();
        let status = opened();
        let _ = frame(&ctx, &status, RawInput::default());
        focus_page_box(&ctx);
        set_draft(&ctx, "4");

        // The box is focused and draws; focus is then taken away, the way a
        // click on the canvas would take it.
        let _ = frame(&ctx, &status, RawInput::default());
        ctx.memory_mut(|m| m.surrender_focus(Id::new(PAGE_BOX_ID)));

        let actions = frame(&ctx, &status, RawInput::default());
        assert_eq!(actions, vec![Action::GoToPage(3)]);
    }

    /// ★ **An out-of-range commit clamps, navigates, and leaves a note.**
    #[test]
    fn an_out_of_range_commit_clamps_and_reports_it() {
        let ctx = Context::default();
        let status = opened();
        let _ = frame(&ctx, &status, RawInput::default());
        focus_page_box(&ctx);
        set_draft(&ctx, "99");

        let actions = frame(&ctx, &status, key_press(Key::Enter, Modifiers::NONE));
        assert_eq!(
            actions,
            vec![Action::GoToPage(3)],
            "a four-page document ends at index 3"
        );
        let state = page_state(&ctx);
        assert_eq!(
            state.note,
            Some(Note::Clamped {
                asked: 99,
                landed: 3
            }),
            "the clamp must be reported, or it is indistinguishable from the \
             box ignoring what was typed"
        );
        assert_eq!(state.draft, None, "the commit succeeded; 99 is not held");
    }

    /// ★ **A refused commit keeps what the operator typed.**
    ///
    /// Wiping the field to "helpfully" restore the current page destroys the
    /// evidence of what the operator meant, and leaves them unable to tell a
    /// rejection from a control that does nothing.
    #[test]
    fn a_refused_commit_keeps_what_the_operator_typed() {
        let ctx = Context::default();
        let status = opened();
        let _ = frame(&ctx, &status, RawInput::default());
        focus_page_box(&ctx);
        set_draft(&ctx, "thirty-seven");

        let actions = frame(&ctx, &status, key_press(Key::Enter, Modifiers::NONE));
        assert!(actions.is_empty(), "nothing to navigate to: {actions:?}");
        let state = page_state(&ctx);
        assert_eq!(state.draft.as_deref(), Some("thirty-seven"));
        assert_eq!(state.note, Some(Note::NotANumber));
    }

    /// A clamp note stops being shown once the operator is somewhere else.
    ///
    /// The note explains where *this* commit put them. Left in place it
    /// would attach that explanation to a page they reached with the ⏴
    /// button, which is a small lie told confidently.
    #[test]
    fn a_clamp_note_is_forgotten_once_the_operator_moves_away() {
        let ctx = Context::default();
        let status = opened();
        let _ = frame(&ctx, &status, RawInput::default());

        ctx.data_mut(|d| {
            d.insert_temp(
                Id::new(PAGE_STATE_ID),
                PageBox {
                    draft: None,
                    // The fixture is open at page 0; a note claiming to have
                    // landed on page 3 is therefore stale by construction.
                    note: Some(Note::Clamped {
                        asked: 99,
                        landed: 3,
                    }),
                },
            );
        });

        let _ = frame(&ctx, &status, RawInput::default());
        assert_eq!(
            page_state(&ctx).note,
            None,
            "a note about page 4 must not sit beside a box reading 1"
        );
    }

    /// The step buttons raise the same actions everything else does.
    ///
    /// Not a tautology: the whole value of a mirror surface is that it
    /// invokes the *same* command, and a status bar that raised its own
    /// page-stepping arithmetic would be a second navigation model to keep
    /// in step with the keyboard's.
    ///
    /// Asserted through the action type rather than by clicking, which would
    /// need synthesized pointer input at a rect this test has to predict.
    /// What is worth pinning is that the variants exist and are the ones
    /// `keyboard::collect` produces.
    #[test]
    fn the_step_buttons_raise_the_shared_navigation_actions() {
        let ctx = Context::default();
        let status = opened();
        let mut actions = Vec::new();
        let _ = ctx.run_ui(key_press(Key::PageUp, Modifiers::NONE), |ui| {
            actions = crate::app::keyboard::collect(ui.ctx(), Some(4));
            show(
                ui,
                &status,
                &mut crate::find::FindState::default(),
                &mut crate::canvas::pick::PickFilter::default(),
                &mut crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT.to_owned(),
                &mut crate::app::prefs::WheelPaging::default(),
                &mut actions,
            );
        });
        assert_eq!(actions, vec![Action::PrevPage]);
    }
}
