//! # `app::actions::forms::paste` — putting a copied form field back
//!
//! One verb, `EditSession::paste_field`, and the reason it is a module of its
//! own is R2: `super`'s file hit the 1,500-line ceiling the moment this arrived,
//! and this is the seam — a paste is *authoring from a source*, which is a
//! different subject from `super::author`'s *authoring from choices*.
//!
//! ## Why the shell does almost nothing here
//!
//! A paste routed through `add_text_field` and its siblings would have to
//! **re-author**: read the source field into a `canvas::formfield::Draft`,
//! mapping the field type, decoding the text strings, translating the flags.
//! `New*Field` is a **spec** — geometry plus a dozen booleans — so such a paste
//! could carry only what the spec can *express*, and these are readable on
//! `forms::Field` and writable nowhere: `/DA`, `/Q`, `/DV`, `/AA`, `/MK`'s
//! colours, `/BS`'s styles beyond solid, the `/Ff` bits no spec names, and the
//! baked `/AP`. `FieldClip` does not express properties; it **carries** them,
//! which is why `copy_field` / `paste_field` is the pair used here.
//!
//! ⇒ What is left here is the three things the *shell* owns and the engine
//! cannot know: which policy the operator's chord meant, what to call a new
//! field, and what to leave selected afterwards.
//!
//! ## Rule 4 — the disclosure is the engine's, verbatim
//!
//! `FieldPasteOutcome::disclosures` reaches the status row through
//! `vector_edit`, like every other verb's. Nothing here paraphrases it: *one
//! fact, one wording*, and the engine's version is the authoritative one because
//! it reports what the operation **did** rather than what the shell intended.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/forms/paste.md`.

use crate::app::state::OpenDoc;

/// **Author the form control that came off the clipboard.**
pub(super) fn paste(
    doc: &mut OpenDoc,
    page: usize,
    rect: pdfcer_core::page_tree::Rect,
    clip: &[u8],
    policy: &pdfcer_core::formclip::FieldPastePolicy,
) {
    use pdfcer_core::formclip::{FieldClip, FieldPastePolicy};

    // A copied RADIO button pasted into its own group: `paste_field` would
    // re-add the copied export value, which the engine refuses as
    // `RadioExportValueTaken` every time. `radiogroup` authors a member with a
    // free one instead.
    if let FieldPastePolicy::AdditionalWidget { existing } = policy
        && super::radiogroup::is_radio(doc, existing)
    {
        super::radiogroup::add_member(doc, existing, page, rect);
        return;
    }
    let clip = match FieldClip::from_bytes(clip) {
        Ok(c) => c,
        Err(e) => {
            // A clip this shell wrote itself failing to read back is not an
            // operator error and not a document error — it is a version skew
            // between the writer and the reader, which is exactly what the
            // format's magic and version word exist to catch. Traced and
            // reported, never silent.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("paste-field-refused page={page} reason=unreadable-clip err={e}")
            });
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::fieldclip::refusal(&crate::canvas::fieldclip::Refusal::EngineRefused(
                    e.to_string(),
                )),
            );
            return;
        }
    };

    // The name to select afterwards, taken BEFORE the move into the closure.
    let wanted = match policy {
        FieldPastePolicy::NewField { name, .. } => name.clone(),
        FieldPastePolicy::AdditionalWidget { existing } => existing.clone(),
        // `FieldPastePolicy` is `#[non_exhaustive]`, so a third policy the
        // engine adds later compiles here rather than breaking the build. It
        // reaches this arm with no name to select afterwards, which costs the
        // O53 re-selection and nothing else -- the paste itself still happens.
        // Traced, so a build that started taking this arm says so instead of
        // quietly losing the selection.
        _ => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "paste-field-unknown-policy reason=engine-added-a-variant".to_owned()
            });
            String::new()
        }
    };
    // NOTHING HERE READS `outcome.merged`, AND THAT IS DELIBERATE.
    //
    // `FieldPasteOutcome::merged` does **not** mean what
    // `FieldAuthorOutcome::merged` means, and on the two cases this shell has,
    // the two are INVERTED:
    //
    // | type | `merged` means |
    // |---|---|
    // | `FieldAuthorOutcome` (`add_*_field`) | this widget joined an EXISTING field |
    // | `FieldPasteOutcome` (`paste_field`)  | the field is in Shape A — one dict that is both field and widget (12.5.6.19) |
    //
    // Measured on the release binary:
    //
    //     Ctrl+V       (a NEW independent field)    -> merged=true
    //     Ctrl+Shift+V (another widget of the SAME) -> merged=false
    //
    // Both are correct under their own type's definition. The trap is that a
    // confirmation phrased from `FieldAuthorOutcome::merged` reads correctly
    // until it is pointed at a paste, where it prints exactly the wrong
    // sentence on exactly the wrong chord — and survives review, because the
    // field name is the same.
    //
    // ⇒ The sentence here is the ENGINE's, verbatim, through `disclosures`. The
    // field that says which of the two happened is `created`, not `merged`.
    let before = doc.edit_epoch;
    let widgets = std::cell::Cell::new(0usize);

    crate::app::actions::apply::vector_edit(doc, "paste-field", page, 1, |session| {
        session
            .paste_field(&clip, page, rect, policy)
            .map(|outcome| {
                widgets.set(outcome.widget_ids.len());
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "paste-field-applied page={page} merged={} created={} widgets={}",
                        outcome.merged,
                        outcome.created,
                        outcome.widget_ids.len()
                    )
                });
                outcome.disclosures
            })
    });

    // O53 — leave what was just placed selected. Guarded on the epoch, so a
    // refusal leaves the previous selection alone rather than pointing at a
    // field that was never created.
    if doc.edit_epoch != before && !wanted.is_empty() {
        let index = widget_index_after_paste(doc, &wanted);
        doc.selected_field = Some(crate::app::state::SelectedField {
            field: wanted,
            widget: index,
            page,
        });
    }
}

/// Which widget of `fqn` the paste just added — the last one.
fn widget_index_after_paste(doc: &OpenDoc, fqn: &str) -> usize {
    let view = doc.session.view();
    pdfcer_core::forms::parse_acroform(&view)
        .and_then(|form| {
            form.fields_named(fqn)
                .next()
                .map(|f| f.widgets.len().saturating_sub(1))
        })
        .unwrap_or(0)
}

/// One field of a several-field paste, decoded before the session is taken.
enum Step {
    /// `paste_field`.
    Field {
        page: usize,
        rect: pdfcer_core::page_tree::Rect,
        clip: pdfcer_core::formclip::FieldClip,
        policy: pdfcer_core::formclip::FieldPastePolicy,
    },
    /// Another button of a radio group, which `paste_field` refuses; see
    /// [`paste`].
    Radio(pdfcer_core::edit::NewRadioButton),
}

/// **Paste several copied fields as one undo step**, leaving them selected.
pub(super) fn many(doc: &mut OpenDoc, items: &[pdfcer_gui_base::fieldaction::FieldPaste]) {
    let Some(page) = items.first().map(|i| i.page) else {
        return;
    };
    let Some((steps, names)) = decode(doc, items) else {
        return;
    };
    let before = doc.edit_epoch;
    let mut report = (0_usize, false);
    crate::app::actions::apply::vector_edit(doc, "paste-fields", page, steps.len(), |session| {
        commit(session, &steps, &mut report)
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "paste-fields-applied pasted={}/{} one_step={}",
            report.0,
            steps.len(),
            report.1
        )
    });
    if doc.edit_epoch == before {
        return;
    }
    let placed: Vec<crate::app::state::SelectedField> = names
        .into_iter()
        .take(report.0)
        .map(|(field, page)| crate::app::state::SelectedField {
            widget: widget_index_after_paste(doc, &field),
            field,
            page,
        })
        .collect();
    doc.selected_field =
        doc.field_set
            .band(None, placed, pdfcer_gui_base::fieldset::BandMerge::Replace);
}

/// A pasted field's name and page.
type Placed = (String, usize);

/// The steps, and the field each one leaves to select with its page. `None`
/// when a clip cannot be read back, which is said.
fn decode(
    doc: &OpenDoc,
    items: &[pdfcer_gui_base::fieldaction::FieldPaste],
) -> Option<(Vec<Step>, Vec<Placed>)> {
    use pdfcer_core::formclip::{FieldClip, FieldPastePolicy};
    let (mut steps, mut names, mut claimed) = (Vec::new(), Vec::new(), Vec::<String>::new());
    for item in items {
        let name = match item.policy.as_ref() {
            FieldPastePolicy::NewField { name, .. } => name.clone(),
            FieldPastePolicy::AdditionalWidget { existing } => existing.clone(),
            _ => String::new(),
        };
        // A radio group with no widget to copy the look of falls through to
        // `paste_field`, whose refusal is then said rather than lost.
        let radio = matches!(
            item.policy.as_ref(),
            FieldPastePolicy::AdditionalWidget { .. }
        ) && super::radiogroup::is_radio(doc, &name);
        if let Some((spec, export)) = radio
            .then(|| super::radiogroup::member(doc, &name, item.page, item.rect, &claimed))
            .flatten()
        {
            claimed.push(export);
            steps.push(Step::Radio(spec));
        } else {
            let clip = match FieldClip::from_bytes(&item.clip) {
                Ok(c) => c,
                Err(e) => {
                    crate::app::actions::record_note(
                        doc.edit_epoch,
                        crate::text::fieldclip::refusal(
                            &crate::canvas::fieldclip::Refusal::EngineRefused(e.to_string()),
                        ),
                    );
                    return None;
                }
            };
            steps.push(Step::Field {
                page: item.page,
                rect: item.rect,
                clip,
                policy: (*item.policy).clone(),
            });
        }
        names.push((name, item.page));
    }
    Some((steps, names))
}

/// Run each step, then fold. `report` is (placed, folded), set even when a
/// verb refuses part-way, since what was placed stays placed.
fn commit(
    session: &mut pdfcer_core::edit::EditSession,
    steps: &[Step],
    report: &mut (usize, bool),
) -> Result<Vec<String>, pdfcer_core::edit::EditError> {
    use pdfcer_core::edit::{CommandKind, EditError, MAX_UNDO_DEPTH};
    if steps.len() > MAX_UNDO_DEPTH {
        return Err(EditError::SelectionTooLargeForOneUndo {
            targets: steps.len(),
            limit: MAX_UNDO_DEPTH,
        });
    }
    let (mut pushed, mut told, mut failure) = (0_usize, Vec::<String>::new(), None);
    for step in steps {
        let outcome = match step {
            Step::Field {
                page,
                rect,
                clip,
                policy,
            } => session
                .paste_field(clip, *page, *rect, policy)
                .map(|o| o.disclosures),
            Step::Radio(spec) => session.add_radio_button(spec).map(|_| Vec::new()),
        };
        match outcome {
            Ok(lines) => {
                pushed += 1;
                for line in lines {
                    if !told.contains(&line) {
                        told.push(line);
                    }
                }
            }
            Err(e) => {
                failure = Some(e);
                break;
            }
        }
    }
    if pushed == 0 {
        return failure.map_or(Ok(Vec::new()), Err);
    }
    let folded = session.coalesce_last(pushed, CommandKind::PasteFormField);
    *report = (pushed, folded);
    let mut said = vec![crate::text::fieldclip::fields_pasted(pushed)];
    said.extend(told);
    if let Some(e) = failure {
        said.push(crate::text::fieldclip::fields_partly_pasted(
            pushed,
            steps.len(),
            &e.to_string(),
        ));
    }
    if !folded {
        said.push(crate::text::fieldclip::fields_pasted_not_one_undo().to_owned());
    }
    Ok(said)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::open_local_fixture;
    use pdfcer_core::formclip::{FieldPastePolicy, PasteTooltip};
    use pdfcer_core::page_tree::Rect;
    use pdfcer_gui_base::fieldaction::FieldPaste;

    fn item(doc: &OpenDoc, source: &str, lly: f64, policy: FieldPastePolicy) -> FieldPaste {
        FieldPaste {
            page: 0,
            rect: Rect {
                llx: 300.0,
                lly,
                urx: 400.0,
                ury: lly + 20.0,
            },
            clip: doc.session.copy_field(source).expect("copy").to_bytes(),
            policy: Box::new(policy),
        }
    }

    fn new_field(name: &str) -> FieldPastePolicy {
        FieldPastePolicy::NewField {
            name: name.to_owned(),
            tooltip: PasteTooltip::Carry,
            copy_value: false,
            copy_actions: true,
        }
    }

    fn widgets(doc: &OpenDoc) -> Vec<(String, usize)> {
        pdfcer_core::forms::parse_acroform(&doc.session.view())
            .expect("a form")
            .fields
            .iter()
            .map(|f| (f.fully_qualified_name.clone(), f.widgets.len()))
            .collect()
    }

    fn undo(doc: &mut OpenDoc) {
        std::sync::Arc::get_mut(&mut doc.session)
            .expect("sole owner in a test")
            .undo()
            .expect("one undo");
    }

    /// Two new fields land, both end up selected, and one Undo removes both.
    #[test]
    fn several_new_fields_paste_as_one_undo_step_and_stay_selected() {
        let mut doc = open_local_fixture("all-field-kinds.pdf");
        let before = widgets(&doc);
        let items = [
            item(&doc, "TextOne", 100.0, new_field("TextOne2")),
            item(&doc, "CheckOne", 140.0, new_field("CheckOne2")),
        ];
        many(&mut doc, &items);
        let after = widgets(&doc);
        assert_eq!(after.len(), before.len() + 2, "{after:?}");
        let selected: Vec<String> = doc
            .field_set
            .all(doc.selected_field.as_ref())
            .into_iter()
            .map(|f| f.field)
            .collect();
        assert_eq!(selected, ["TextOne2", "CheckOne2"]);
        undo(&mut doc);
        assert_eq!(widgets(&doc), before, "one Undo did not remove both");
    }

    /// Another box of a radio group and of a text field, in one step: the
    /// radio goes by `add_radio_button`, which `paste_field` would refuse.
    #[test]
    fn another_box_of_a_radio_and_a_text_field_is_one_undo_step() {
        let mut doc = open_local_fixture("all-field-kinds.pdf");
        let before = widgets(&doc);
        let again = |name: &str| FieldPastePolicy::AdditionalWidget {
            existing: name.to_owned(),
        };
        let items = [
            item(&doc, "RadioGroup", 100.0, again("RadioGroup")),
            item(&doc, "TextOne", 140.0, again("TextOne")),
        ];
        many(&mut doc, &items);
        let grew: Vec<_> = widgets(&doc)
            .into_iter()
            .zip(&before)
            .filter(|(a, b)| a.1 == b.1 + 1)
            .map(|(a, _)| a.0)
            .collect();
        assert_eq!(grew.len(), 2, "both fields gain a box: {grew:?}");
        undo(&mut doc);
        assert_eq!(widgets(&doc), before, "one Undo did not remove both boxes");
    }
}
