//! # `app::actions::forms::author` — making a form control from the operator's choices
//!
//! One function. It is the path the **placement dialog** commits through, and
//! its sibling [`super::paste`] is the path a **clipboard** commits through.
//!
//! ## ★★ Why those are two paths and not one
//!
//! `paste`'s own header carries the long form; in a sentence: `New*Field` is a
//! **spec** — geometry plus a dozen booleans — so authoring from it can only
//! carry what the spec can *express*, and that is exactly right when the
//! operator has just chosen every value in a dialog. It is exactly wrong when
//! the values came from an existing field, which is why the paste route carries
//! a clip instead.
//!
//! Split out of `super` under R2 on 2026-08-30, when widget rotation took that
//! file past 1,500 lines for the second time in one session.

use crate::app::state::OpenDoc;

pub(in crate::app::actions) fn author(
    doc: &mut OpenDoc,
    page: usize,
    rect: pdfcer_core::page_tree::Rect,
    draft: &crate::canvas::formfield::Draft,
) {
    use crate::canvas::formfield::FormFieldKind as K;
    use pdfcer_core::edit::{
        BorderSpec, BorderStyle, ChoiceOption, NewCheckBox, NewChoiceField, NewPushButton,
        NewRadioButton, NewTextField, TooltipChoice,
    };

    // ★★★ REFUSE A NAME THAT WOULD SWALLOW AN EXISTING FIELD, before anything
    // is written. See `group_is_a_field` — this is a shim for an engine gap and
    // it guards unrecoverable data loss, so it runs first.
    if let Some(victim) = super::group_is_a_field(doc, draft.name.trim()) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("add-form-field-refused reason=group-is-a-field victim={victim}")
        });
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::fieldclip::name_would_swallow(&victim),
        );
        return;
    }

    let name = draft.name.trim().to_owned();
    // ★ Empty means DECLINED, not undecided. See the header — this one line is
    // the difference between a feature and a nine-day blocker.
    let tooltip = if draft.tooltip.trim().is_empty() {
        TooltipChoice::Declined
    } else {
        TooltipChoice::Text(draft.tooltip.trim().to_owned())
    };
    // A zero width is how PDF spells "no border", so the operator's choice
    // travels as a number rather than as a second boolean that could disagree
    // with it.
    let border = BorderSpec {
        style: BorderStyle::Solid,
        width: draft.border_width.max(0.0),
    };
    let kind = draft.kind;
    // ★★★ The epoch BEFORE, so the selection below is set only if the field was
    // actually authored. `vector_edit` bumps it on success and leaves it alone
    // on a refusal, which is the one signal available here -- the closure's
    // `Result` is consumed inside the funnel.
    let before = doc.edit_epoch;
    let placed_name = draft.name.trim().to_owned();
    // What the button is to do, and the name to address it by afterwards.
    //
    // ★ `fqn` is the trimmed name, which for a field authored here IS the
    // fully-qualified one: `add_push_button` places a top-level field, so there
    // is no parent to prefix. A field nested under a parent would need the
    // dotted path, and that case cannot arise from this dialog.
    let action = draft.action.clone();
    let fqn = placed_name.clone();
    // Carried out of the closure: a refusal on the SECOND verb, which leaves a
    // correctly placed button with nothing to do. The closure's own `Result`
    // reports the first verb, and reporting a second failure through it would
    // claim the button was not placed.
    let mut refused: Option<String> = None;
    // ★ Whether the two commands became one undo entry. `true` until something
    // says otherwise, because the ordinary case is the fold succeeding and a
    // sentence is owed only when it does not.
    let mut folded = true;

    crate::app::actions::apply::vector_edit(doc, "add-form-field", page, 1, |session| {
        let outcome = match kind {
            K::Text => {
                let mut spec = NewTextField::new(page, name, rect);
                spec.value = draft.value.clone();
                spec.max_len = draft.max_len;
                spec.tooltip = tooltip;
                spec.multiline = draft.multiline;
                spec.password = draft.password;
                // ★ Gated on `comb_ok` rather than on the flag alone, so the
                // dialog's rule and the authored field cannot disagree: comb
                // divides the width into `max_len` cells, and without a
                // maximum there is nothing to divide by.
                spec.comb = draft.comb && draft.comb_ok();
                spec.read_only = draft.read_only;
                spec.required = draft.required;
                spec.border = border;
                session.add_text_field(&spec)
            }
            K::CheckBox => {
                let mut spec = NewCheckBox::new(page, name, rect);
                spec.on_state = draft.export_value.clone();
                spec.checked = draft.checked;
                spec.tooltip = tooltip;
                spec.read_only = draft.read_only;
                spec.required = draft.required;
                spec.border = border;
                session.add_check_box(&spec)
            }
            K::Radio => {
                let mut spec = NewRadioButton::new(page, name, rect, draft.export_value.clone());
                spec.selected = draft.checked;
                spec.tooltip = tooltip;
                // `no_toggle_to_off` and `radios_in_unison` are left at the
                // engine's defaults rather than exposed: they are properties of
                // a GROUP, not of the widget being placed, so offering them
                // per-widget would let two members of one group carry
                // contradictory answers. They belong on a group editor, which
                // is the properties pane's business.
                spec.read_only = draft.read_only;
                spec.required = draft.required;
                spec.border = border;
                session.add_radio_button(&spec)
            }
            K::Choice => {
                // Export value and display text the same, deliberately — which
                // is what `ChoiceOption::plain` means. They differ only when a
                // form is submitted to a system that wants a code rather than a
                // label, which is a second column this dialog does not offer
                // and must not guess at.
                let options: Vec<ChoiceOption> = draft
                    .options()
                    .into_iter()
                    .map(ChoiceOption::plain)
                    .collect();
                let mut spec = NewChoiceField::new(page, name, rect, options);
                spec.combo = draft.combo;
                spec.editable = draft.editable;
                spec.multi_select = draft.multi_select;
                spec.sort = draft.sort;
                spec.tooltip = tooltip;
                spec.read_only = draft.read_only;
                spec.required = draft.required;
                spec.border = border;
                session.add_choice_field(&spec)
            }
            K::PushButton => {
                let mut spec = NewPushButton::new(page, name, rect, draft.caption.clone());
                spec.tooltip = tooltip;
                spec.read_only = draft.read_only;
                spec.border = border;
                let placed = session.add_push_button(&spec);
                // ★★★ AND THEN GIVE IT SOMETHING TO DO.
                //
                // Creation authors an INERT button, deliberately and
                // permanently: `pdfcer-core`'s decision 009 posture A says a
                // button must not gain behaviour as a side effect of being
                // drawn, and `NewPushButton` is untouched by `Pass 183.0`.
                // Giving one an action is a separate, named verb a caller has
                // to go out of its way to call — which is exactly what this is.
                //
                // ★★★ TWO COMMANDS, ONE UNDO ENTRY — since 2026-09-01.
                //
                // This read *"two commands, therefore two undo entries, and it
                // is a workaround rather than a design"*, because
                // `EditSession::coalesce_last` was private. It was reported per
                // decision 058 rather than absorbed silently, and `pdfcer-core`
                // made it `pub` the same day, choosing the general fix over a
                // combined verb for the reason the request argued: the
                // two-verbs-one-gesture shape recurs, and a combined verb fixes
                // one instance of it.
                //
                // The fold is below, after both verbs have run.
                //
                // ★ The action is written ONLY when one was chosen. The default
                // is `Nothing`, whose `to_core` is `None`, and calling
                // `set_button_action(name, None)` on a button that has never
                // had one would be a second command that changes nothing — an
                // undo entry for an act that did not happen.
                match (placed, action.to_core()) {
                    (Ok(outcome), Some(deed)) => {
                        match session.set_button_action(&fqn, Some(deed)) {
                            // ★★ The success line, and it exists for one
                            // reason: a driven check has no other oracle. The
                            // button on the page is drawn identically whether
                            // or not `/A` was written — which is rule 4 working
                            // correctly, and is also what makes a screenshot
                            // useless here. `replaced` is carried because it
                            // names what was destroyed, including a script
                            // pdfcer will not write back.
                            Ok(change) => {
                                let kind = action.kind;
                                let replaced = change.replaced.clone();
                                crate::diag::trace(|| {
                                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                                    format!(
                                        "button-action-applied name={fqn} kind={kind:?} replaced={}",
                                        replaced.as_deref().unwrap_or("none")
                                    )
                                });
                                // ★★ FOLD, IMMEDIATELY, and check the answer.
                                //
                                // Three things the public contract states that
                                // the private one never had to, and all three
                                // are honoured here:
                                //
                                // 1. **Check the return.** `false` means every
                                //    change was applied and only the GROUPING
                                //    failed — the undo stack was shorter than
                                //    `count`. Disclose it; do not retry, do not
                                //    ignore.
                                // 2. **`count` counts the commands just
                                //    pushed.** Two: the button and its action.
                                //    The guard covers a short stack; it does
                                //    NOT cover miscounting, which would fold an
                                //    unrelated earlier edit in with nothing to
                                //    detect it.
                                // 3. **Fold before anything else can push.**
                                //    There is no handle saying which commands
                                //    are ours, so this is the only safe moment.
                                //
                                // ★ The label names the GESTURE, not the last
                                // verb — which is what a label is for, and why
                                // `cut_field` relabels a single-target cut
                                // rather than leaving it saying "undo delete".
                                // `AddFormField` is the gesture: the operator
                                // drew a field. There is no `AddPushButton`
                                // kind and there should not be — an undo
                                // control saying "undo add form field" is
                                // right for all five kinds.
                                if !session
                                    .coalesce_last(2, pdfcer_core::edit::CommandKind::AddFormField)
                                {
                                    folded = false;
                                }
                                Ok(outcome)
                            }
                            // ★★★ The button IS placed and the action is not.
                            // Say so: silence here is the exact defect this
                            // feature removes — a button that looks right and
                            // does nothing — arriving by a different door. The
                            // engine's own words are carried because they name
                            // the condition (a page past the end, a field that
                            // is not there, a target that is a grouping node).
                            Err(why) => {
                                crate::diag::trace(|| {
                                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                                    format!("button-action-refused name={fqn} why={why}")
                                });
                                refused = Some(crate::text::buttonaction::action_refused(
                                    &fqn,
                                    &why.to_string(),
                                ));
                                Ok(outcome)
                            }
                        }
                    }
                    (other, _) => other,
                }
            }
        };
        outcome.map(|o| super::disclosures(&o, kind))
    });

    // ★★ The second verb's refusal, off-canvas, after the funnel has committed
    // the first. `record_note` keys on the epoch the funnel has just bumped, so
    // the sentence retires with the next edit the way every other note does.
    //
    // Rule 4: the button on the page is drawn exactly as the saved file will
    // draw it — no badge, no tint, nothing marking it as "placed but
    // behaviourless". What it does NOT do is reported here instead.
    if let Some(note) = refused {
        crate::app::actions::record_note(doc.edit_epoch, note);
    } else if !folded {
        // ★★ Said, not swallowed. The engine's contract is explicit that a
        // `false` here means the work is done and only the grouping failed —
        // so the button is placed and does what it was asked to do, and the
        // only thing wrong is that undoing it takes two presses. An operator
        // who presses Ctrl+Z once and sees a button that still exists deserves
        // to know why rather than to conclude undo is broken.
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::buttonaction::two_undo_entries(&placed_name),
        );
    }

    // ★★★ SELECT WHAT WAS JUST PLACED. `OPERATOR_REQUESTS.md` **O53**.
    //
    // Every program in this class leaves a newly drawn object selected --
    // Acrobat, Word, PowerPoint, Visio, Illustrator, Inkscape -- and they
    // disagree about whether the TOOL stays armed. So the arming is a taste
    // question with a convergent default (`dialogs::formfield` takes Acrobat's)
    // and this is not: it is the half none of them differ on.
    //
    // ★★ It is what makes the operator's next gesture work. He drew a checkbox
    // and reported *"I can't select it on the canvas to move or resize"*; with
    // the tool put down AND the field selected, the grips are already there and
    // the drag is already live. Requiring a click to select something he just
    // created is a step no other editor asks for.
    //
    // ★ Widget 0, because a field authored here has exactly one -- `add_*_field`
    // places a single widget. A field with several is one that grew later,
    // through `merge_document` or a hand-edited file, and there is no "the new
    // one" to name in that case.
    if doc.edit_epoch != before && !placed_name.is_empty() {
        doc.selected_field = Some(crate::app::state::SelectedField {
            field: placed_name,
            widget: 0,
            page,
        });
    }
}
