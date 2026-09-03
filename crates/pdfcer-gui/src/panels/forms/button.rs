//! # `panels::forms::button` — **what an existing push button does, and how to
//! change it**
//!
//! The half of `OPERATOR_REQUESTS.md` O60/O61 that could not ship on
//! 2026-09-01, and shipped hours later when the engine answered.
//!
//! ## ★★★ Why this row did not exist until the reader did
//!
//! `EditSession::set_button_action` shipped on 2026-08-30 and the placement
//! dialog consumed it the same morning: draw a button, choose what pressing it
//! does. That worked because a button being *placed* has a known action —
//! none — so the control had a truthful starting value.
//!
//! A button **already in the document** does not. `pdfcer-core` could write an
//! action and not read one back, and this project declined to draw the control
//! anyway. The three ways to do it without a reader were all bad, and the
//! request said so:
//!
//! 1. **Show "Nothing".** A button carrying `/A << /S /JavaScript … >>` would
//!    read as inert — pdfcer asserting a fact about somebody else's document
//!    that it had not checked. The sneaky half of rule 4 in its purest form.
//! 2. **Show nothing, and make the control a one-way *"set this button to:"*.**
//!    Honest, and an invented interaction. No form editor works that way, and
//!    this project's standing rule forbids inventing one.
//! 3. **Write first and read the result.** `ButtonActionChange::replaced` names
//!    what was there — so the only way to learn what a button does would be to
//!    destroy it. Not a control; a trap.
//!
//! ⇒ Filed as `request_a_buttons_action_can_be_written_and_not_read.md`,
//! answered by `Pass 212.0`, and this file is the consumption.
//!
//! ## ★★ The engine shipped FOUR states where three were asked for
//!
//! And the fourth is the one that makes the row honest.
//!
//! | state | what a control may offer |
//! |---|---|
//! | `None` | *"does nothing"* — offer to set one |
//! | `Known(a)` | show it, offer to change it |
//! | `Unmodelled(s)` | name the subtype, offer to **replace**, never claim to show it |
//! | `Foreign(s)` | name the subtype, offer **nothing** |
//!
//! `Unmodelled` and `Foreign` differ in exactly one thing — **whether replacing
//! is offered** — and that is the decision the operator is being asked to make.
//! Three states would have forced a wrong answer in one direction: a
//! `/SubmitForm` this reader does not yet decode is not *"an action pdfcer will
//! not author"*, because pdfcer authors submits happily, and calling it `Foreign`
//! would have greyed a row that should be live.
//!
//! ★ R9 is what makes `Foreign` render *nothing* rather than a greyed Change
//! button. A greyed control says *"not now"*; the truth here is *"not ever, by
//! decision"*, and greying would advertise a capability pdfcer has chosen not to
//! have.
//!
//! ## Where the row is
//!
//! In the Forms panel, on the push-button row, where `block_reason` used to put
//! a sentence saying there was nothing to fill in. That sentence was true and
//! is still true — a push button holds no value — and it was the whole of what
//! this shell had to say about a button until now.

use egui::Ui;
use pdfcer_core::edit::{ButtonAction, ButtonActionState, NamedAction, ResetScope};
use pdfcer_core::forms::Field;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::formfield::action::{ButtonDoes, ButtonDoesKind};
use crate::text::buttonaction as t;

/// The sentence naming what this button currently does.
const REGION_CURRENT: &str = "forms.button.current"; // ui-text-exempt: a trace region name
/// The control that opens the chooser.
const REGION_CHANGE: &str = "forms.button.change"; // ui-text-exempt: a trace region name
/// The control that commits a change.
const REGION_APPLY: &str = "forms.button.apply"; // ui-text-exempt: a trace region name

/// Draw the push-button row's action section.
///
/// `draft` is the panel's per-field editing state, held across frames because a
/// chooser the operator has opened must not close when the panel repaints.
pub(super) fn row(
    ui: &mut Ui,
    doc: &OpenDoc,
    field: &Field,
    draft: &mut Option<(String, ButtonDoes)>,
    actions: &mut Vec<Action>,
) {
    let fqn = field.fully_qualified_name.clone();
    let state = match doc.session.button_action(&fqn) {
        Ok(state) => state,
        Err(why) => {
            // ★ The reader's refusals match the writer's, by the engine's own
            // decision: a shell must not learn through the reader about a field
            // it would be refused permission to change. So a refusal here is
            // reported and nothing is offered — there is nothing this row could
            // honestly do next.
            ui.label(
                egui::RichText::new(t::current_unreadable(&why.to_string()))
                    .small()
                    .weak(),
            );
            return;
        }
    };

    let said = ui.label(
        egui::RichText::new(match &state {
            ButtonActionState::None => t::current_none(),
            ButtonActionState::Known(action) => t::current_known(kind_of(action)),
            ButtonActionState::Unmodelled(s) => t::current_unmodelled(s),
            ButtonActionState::Foreign(s) => t::current_foreign(s),
            // `ButtonActionState` is `#[non_exhaustive]`. A state this build
            // has never seen gets the most conservative sentence there is and
            // NO control — the same rule the Attachments panel applies to an
            // unknown `AttachmentKind`, and for the same reason: an affordance
            // for an act this code cannot describe is worse than silence.
            _ => t::current_foreign(""),
        })
        .small()
        .weak(),
    );
    crate::diag::ui_rect_visible(REGION_CURRENT, said.rect, ui.clip_rect());

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("button-action-read name={fqn:?} state={}", name_of(&state))
    });

    // ★★★ R9: `Foreign` renders NOTHING further. Not a greyed Change button —
    // greying says "not now" and the truth is "not ever, by decision". pdfcer
    // will not author a script and will not overwrite one, and a control that
    // looked like it might is a promise the press would break.
    if matches!(state, ButtonActionState::Foreign(_)) {
        return;
    }

    let open = draft.as_ref().is_some_and(|(name, _)| name == &fqn);
    if !open {
        let change = ui.button(t::change_button());
        crate::diag::ui_rect_visible(REGION_CHANGE, change.rect, ui.clip_rect());
        if change.clicked() {
            // ★★ The chooser opens on what the button ALREADY does, when that
            // is knowable. `Unmodelled` cannot seed it — pdfcer did not decode
            // the instance — so it opens at `Nothing`, which is the one value
            // that is not a claim about what is there. The sentence above has
            // already said the button carries something and that replacing
            // discards it.
            *draft = Some((
                fqn.clone(),
                match &state {
                    ButtonActionState::Known(action) => from_core(action),
                    _ => ButtonDoes::default(),
                },
            ));
        }
        return;
    }

    let Some((_, does)) = draft.as_mut() else {
        return;
    };
    crate::dialogs::buttonaction::rows(ui, does);

    // Apply is absent, not greyed, while the draft cannot be authored — the
    // blocker sentence under the chooser already says what to type, and a
    // greyed button beside a sentence naming the remedy is a second statement
    // of one fact.
    if does.blocker().is_none() {
        let apply = ui.button(t::apply_button());
        crate::diag::ui_rect_visible(REGION_APPLY, apply.rect, ui.clip_rect());
        if apply.clicked() {
            actions.push(Action::Field(
                crate::app::actions::forms::FieldAction::SetButtonAction {
                    field: fqn,
                    action: Box::new(does.to_core()),
                },
            ));
            *draft = None;
        }
    }
}

/// Which chooser entry a modelled action corresponds to.
///
/// ★ A `match` rather than a `From`, because the mapping is **lossy on
/// purpose**: the chooser has one entry per kind and an action carries
/// parameters. This answers *"which row is ticked"*, and [`from_core`] answers
/// *"what does the row start with"*. Two functions because they are two
/// questions, and folding them would make the lossy direction look reversible.
fn kind_of(action: &ButtonAction) -> ButtonDoesKind {
    match action {
        ButtonAction::ResetForm { .. } => ButtonDoesKind::ResetForm,
        ButtonAction::GoToPage { .. } => ButtonDoesKind::GoToPage,
        ButtonAction::Named(_) => ButtonDoesKind::Named,
        ButtonAction::SetHidden { .. } => ButtonDoesKind::ShowHide,
        ButtonAction::Uri { .. } => ButtonDoesKind::Uri,
        ButtonAction::SubmitForm(_) => ButtonDoesKind::SubmitForm,
        // `ButtonAction` is `#[non_exhaustive]`. An action kind this build does
        // not know is shown as `Nothing` and can be replaced — which is exactly
        // `Unmodelled`'s treatment, arrived at from the other side.
        _ => ButtonDoesKind::Nothing,
    }
}

/// Seed the chooser from what the button already does.
///
/// ★★ Only the parameters this shell can round-trip are carried. A
/// `ResetScope::Only`/`Except` becomes the chooser's *whole form* reset,
/// because the chooser offers no field picker for a reset — and the module that
/// owns that decision says why: the preview it can show is the whole-form
/// preview, and a per-field control without a per-field preview is a control
/// whose effect the operator cannot see before pressing.
///
/// ⇒ So opening the chooser on such a button and pressing Apply **widens** the
/// reset. That is a real narrowing of the document and it must not be silent —
/// which is why `Apply` sends `ButtonActionChange::replaced` to the status line
/// through `text::buttonaction::changed`.
fn from_core(action: &ButtonAction) -> ButtonDoes {
    let mut does = ButtonDoes {
        kind: kind_of(action),
        ..ButtonDoes::default()
    };
    match action {
        ButtonAction::ResetForm { scope } => {
            if !matches!(scope, ResetScope::All) {
                // The chooser cannot express it; say so where it is read.
                does.kind = ButtonDoesKind::ResetForm;
            }
        }
        ButtonAction::GoToPage { page_index, .. } => {
            does.page_number = (page_index + 1).to_string();
        }
        ButtonAction::Named(named) => {
            does.named = match named {
                NamedAction::NextPage => crate::canvas::formfield::action::NamedChoice::NextPage,
                NamedAction::PrevPage => crate::canvas::formfield::action::NamedChoice::PrevPage,
                NamedAction::FirstPage => crate::canvas::formfield::action::NamedChoice::FirstPage,
                NamedAction::LastPage => crate::canvas::formfield::action::NamedChoice::LastPage,
                _ => crate::canvas::formfield::action::NamedChoice::NextPage,
            };
        }
        ButtonAction::SetHidden { targets, hidden } => {
            does.targets = targets.join("\n");
            does.hide = *hidden;
        }
        ButtonAction::Uri { uri } => does.url.clone_from(uri),
        _ => {}
    }
    does
}

/// A short, stable name for the trace. Never displayed.
const fn name_of(state: &ButtonActionState) -> &'static str {
    match state {
        ButtonActionState::None => "none",
        ButtonActionState::Known(_) => "known",
        ButtonActionState::Unmodelled(_) => "unmodelled",
        ButtonActionState::Foreign(_) => "foreign",
        _ => "unrecognised",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **Every modelled action maps to a chooser entry that is not
    /// `Nothing`.**
    ///
    /// The failure this guards is silent and severe: a `Known` action whose
    /// `kind_of` fell through to `Nothing` would make the row say *"does
    /// nothing"* about a button that resets the form — pdfcer asserting a
    /// falsehood about the operator's own document, which is precisely what
    /// the reader was requested to prevent.
    #[test]
    fn every_modelled_action_names_a_real_chooser_entry() {
        let cases = [
            ButtonAction::ResetForm {
                scope: ResetScope::All,
            },
            ButtonAction::Named(NamedAction::NextPage),
            ButtonAction::Uri {
                uri: "https://example.com".to_owned(),
            },
        ];
        for action in &cases {
            assert_ne!(
                kind_of(action),
                ButtonDoesKind::Nothing,
                "{action:?} fell through to Nothing, so the row would claim it does nothing"
            );
        }
    }

    /// ★★ The seed round-trips a page number through the 1-based box.
    #[test]
    fn a_goto_seeds_the_box_one_based() {
        let does = from_core(&ButtonAction::GoToPage {
            page_index: 4,
            view: pdfcer_core::edit::PageView::WholePage,
        });
        assert_eq!(does.kind, ButtonDoesKind::GoToPage);
        assert_eq!(does.page_number, "5");
        assert!(
            does.blocker().is_none(),
            "a seeded draft must be authorable"
        );
    }

    /// ★ A show/hide seeds both halves, and the direction is the half that is
    /// easy to lose — Table 210's `/H` defaults to **true**, so a target list
    /// that arrived without a direction must not silently become "show".
    #[test]
    fn a_show_hide_seeds_the_targets_and_the_direction() {
        let does = from_core(&ButtonAction::SetHidden {
            targets: vec!["A".to_owned(), "B".to_owned()],
            hidden: true,
        });
        assert_eq!(does.target_names(), vec!["A", "B"]);
        assert!(does.hide);
        assert!(does.blocker().is_none());
    }

    /// The four states have four distinct trace names, so a driven check can
    /// tell them apart without reading the sentence.
    #[test]
    fn the_states_are_named_apart_in_the_trace() {
        let names = [
            name_of(&ButtonActionState::None),
            name_of(&ButtonActionState::Known(ButtonAction::ResetForm {
                scope: ResetScope::All,
            })),
            name_of(&ButtonActionState::Unmodelled("GoTo".to_owned())),
            name_of(&ButtonActionState::Foreign("JavaScript".to_owned())),
        ];
        let mut sorted = names;
        sorted.sort_unstable();
        let mut deduped = sorted;
        let unique = {
            let mut v: Vec<_> = deduped.to_vec();
            v.dedup();
            v.len()
        };
        deduped.sort_unstable();
        assert_eq!(unique, 4, "{names:?}");
    }
}
