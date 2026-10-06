//! # `app::actions::forms` — everything done to a form FIELD
//!
//! A sibling of [`super::dimensions`], [`super::pages`], [`super::vector`] and
//! [`super::export`], and it owns both halves of its subject: the action enum
//! [`FieldAction`] and the apply logic every one of its variants reaches.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/forms.md`.

/// **Authoring a form control from the placement dialog's choices.**
///
/// Its sibling [`paste`] is authoring from a SOURCE; the two are not
/// duplicates, and each header says which input it starts from.
pub(super) mod author;
/// Deleting a **grouping node** — the two-press verb, its preview store and
/// both apply paths.
pub mod delete;
pub mod groups;
/// **Putting a copied form field back** — `EditSession::paste_field`. Its
/// header carries why the shell does almost nothing in it any more.
mod paste;
/// **A field's format, validate and calculate scripts.**
mod scripts;
/// **Verbs about the BOX rather than the field** — rotation today, and the
/// natural home for the next one. A field's identity and a widget's placement
/// are two subjects.
mod widget;

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::app::status::decline::{self, Declined};
use crate::text::status as t;

pub use pdfcer_gui_base::fieldaction::FieldAction;

/// Apply every form-field verb that needs the open document and nothing else.
pub(super) fn apply(doc: &mut OpenDoc, action: FieldAction) {
    match action {
        // Selection is VIEW STATE. It changes no document, bumps no epoch and
        // invalidates no page — which is why it does not go near the funnel.
        FieldAction::Select(selected) => doc.selected_field = selected,
        // One line: the panel already resolved left/right into a
        // counterclockwise angle, and `rotate_widget` owns the rest -- the
        // multiple-of-90 refusal, the normalisation and the appearance
        // regeneration it may or may not be able to do.
        FieldAction::RotateWidget {
            field,
            widget,
            degrees,
        } => widget::rotate(doc, &field, widget, degrees),
        FieldAction::Paste {
            page,
            rect,
            clip,
            policy,
        } => paste::paste(doc, page, rect, &clip, &policy),
        FieldAction::EditProperties {
            field,
            edit,
            touched,
        } => edit_properties(doc, &field, &edit, touched),
        FieldAction::EditWidget {
            field,
            widget,
            edit,
            touched,
        } => edit_widget(doc, &field, widget, &edit, touched),
        FieldAction::Import { path } => import_data(doc, &path),
        FieldAction::Rename { from, to } => rename(doc, &from, &to),
        FieldAction::SetButtonAction { field, action } => {
            set_button_action(doc, &field, *action);
        }
        FieldAction::PickButtonIcon { field, widget } => {
            super::buttonicon::pick(doc, &field, widget);
        }
        FieldAction::DeleteField { field } => delete::field(doc, &field),
        FieldAction::SetScript { field, edit } => scripts::set(doc, &field, *edit),
        FieldAction::MoveWidget {
            field,
            widget,
            dx,
            dy,
        } => move_widget(doc, &field, widget, dx, dy),
        FieldAction::DeleteWidget { field, widget } => delete::widget(doc, &field, widget),
        FieldAction::ReorderAnnotations { page, order } => {
            super::reorder::reorder_annotations(doc, page, &order);
        }
        FieldAction::SetPageTabs { page, tabs } => super::reorder::set_page_tabs(doc, page, tabs),
        // The arm changes no document and bumps no epoch, so it does not go
        // near `vector_edit`; the deletion does, like every other structural
        // form verb. See `groups`' header for why a query needs to be an action
        // at all.
        FieldAction::ArmGroupDeletion(group) => groups::arm(doc, group),
        FieldAction::DeleteGroup { group } => groups::delete(doc, &group),
        FieldAction::Adopt { page, widget, name } => adopt(doc, page, widget, name),
        FieldAction::Edit(edit) => crate::panels::forms::edit::apply(doc, &edit),
        FieldAction::AdjustHandSign {
            field,
            page,
            from,
            to,
        } => super::handsign::adjust(doc, &field, page, from, to),
        // Unreachable rather than unhandled, and named so the compiler will
        // say so if the split above is ever changed without changing this.
        FieldAction::Begin { .. }
        | FieldAction::Commit { .. }
        | FieldAction::Sign { .. }
        | FieldAction::SignWithId { .. }
        | FieldAction::HandSign { .. } => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "FieldAction::Begin, ::Commit and the signing actions are applied in super::apply, which holds the dialog and defaults state this function cannot reach"
            );
        }
    }
}

/// Register one unclaimed widget into the document's `/AcroForm`.
pub(super) fn adopt(doc: &mut OpenDoc, page: usize, widget: ObjId, name: Option<String>) {
    super::apply::vector_edit(doc, "adopt-widget", page, 1, |session| {
        match session.adopt_widget(widget, name.as_deref()) {
            Ok(outcome) => Ok(vec![t::adopted(
                &outcome.name,
                outcome.field_type.is_some(),
                outcome.acroform_created,
            )]),
            Err(error) => {
                if let Some(declined) = correctable(&error) {
                    decline::record_adopt_refusal(declined);
                }
                Err(error)
            }
        }
    });
}

/// Every field name the document already carries.
pub(super) fn field_names(doc: &OpenDoc) -> Vec<String> {
    let view = doc.session.view();
    pdfcer_core::forms::parse_acroform(&view)
        .map(|form| {
            form.fields
                .iter()
                .map(|f| f.fully_qualified_name.clone())
                .collect()
        })
        .unwrap_or_default()
}

// A dotted field name that crosses an existing TERMINAL field is refused by
// the engine, not by a pre-check here. `place_new_field_deferred` is the single
// choke point every `add_*` verb and `paste_field` reach, and it raises
// `FieldPathCrossesTerminal` before a byte is staged or an undo entry pushed.
//
// Do not reintroduce a shell-side prefix walk. The engine refuses only when the
// deepest existing node on the path is a **terminal** (`child_field_count ==
// 0`, §12.7.3.1's own definition); a walk that refuses on any prefix present in
// `AcroForm::fields` is a different, wider set. The two differ on the **mixed
// node** — a node carrying child fields *and* its own bare widget kids, which
// `pdfcer_core::forms`'s `walk_field` models and pdfcer's own same-name merge
// can generate. A second child there loses nothing and the engine allows it; a
// prefix walk refuses it while claiming a field would be destroyed.
//
// The shell's whole share of this is `correctable`'s `FieldPathCrossesTerminal`
// arm, which words the refusal with the name **the engine identified**. That
// answer cannot drift from the engine's, because it is the engine's.
//
// The sentence lives in `crate::text::fieldclip::name_crosses_a_field`. The
// engine's own `Display` prose names the victim and cites the clause, but it
// reaches `PDFCER_DIAG` and stops there: `check-ui-strings.sh`'s exclusion 3 is
// explicit that an error type's `Display` is not permission to route operator
// text through it. Without the arm, a correct engine refusal would reach the
// operator as the floor's generic *"That change was refused"*.

/// **Change one property of an existing field.**
pub(super) fn edit_properties(
    doc: &mut OpenDoc,
    field: &str,
    edit: &pdfcer_core::edit::FieldEdit,
    touched: &'static str,
) {
    let edit = edit.clone();
    let field = field.to_owned();
    super::apply::vector_edit(doc, "edit-field", 0, 1, move |session| {
        session.edit_field(&field, &edit).map(|outcome| {
            let mut lines = Vec::new();
            // Verbatim, and FIRST. It is the one line that says the
            // operator's stored data no longer matches the field's own rules,
            // which outranks every count.
            if let Some(why) = outcome.value_no_longer_fits {
                lines.push(why);
            }
            if outcome.password_value_removed {
                lines.push(crate::text::forms::field_password_value_removed(&field));
            }
            if outcome.sort_claim_unmet {
                lines.push(crate::text::forms::field_sort_claim_unmet().to_owned());
            }
            // A reorder the shell did not expect. `choiceopts` sorts the list
            // it sends with the engine's own exported sorter, so this is
            // normally false and says nothing — it is true only if the two
            // orderings have come apart, which is the drift `G026` exported the
            // comparator to prevent and which no test on either side can see.
            if outcome.options_sorted {
                lines.push(crate::text::forms::field_options_reordered().to_owned());
            }
            if outcome.widgets_affected > 1 {
                lines.push(crate::text::forms::field_widgets_affected(
                    outcome.widgets_affected,
                ));
            }
            // Nothing at all when the edit was ordinary, which is most of the
            // time. A bar that narrated every checkbox would stop being read,
            // and `vector_edit` treats an empty list as "no line".
            let _ = touched;
            lines
        })
    });
}

/// **Change one placement of a field** — its rectangle, its border, its
/// caption, its visibility, or either of its two `/MK` colours.
pub(super) fn edit_widget(
    doc: &mut OpenDoc,
    field: &str,
    widget: usize,
    edit: &pdfcer_core::edit::WidgetEdit,
    touched: &'static str,
) {
    let edit = edit.clone();
    let field = field.to_owned();
    super::apply::vector_edit(doc, "edit-widget", 0, 1, move |session| {
        session.edit_widget(&field, widget, &edit).map(|outcome| {
            // **The trace this verb never had** — `OPERATOR_REQUESTS.md`
            // O76. Its two siblings, `move-widget-applied` and
            // `rotate-widget-applied`, have always reported their outcome; this
            // one reported nothing, which is why a check box quietly stretching
            // its own artwork for weeks was invisible to every driven run.
            //
            // All three fields, because a screenshot cannot separate them: a
            // border that thickened because `/BS /W` changed and one that
            // thickened because §12.5.5's placement matrix scaled it are the
            // same pixels. `regenerated=` is the field that tells them apart,
            // and it is the assertion the O76 check is built on.
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "edit-widget-applied field={field} widget={widget} touched={touched:?} \
                     geometry={} resized={} redrawn={} foreign_replaced={} appearance={:?}",
                    outcome.rect_after.is_some(),
                    outcome.resized,
                    if matches!(
                        outcome.appearance,
                        pdfcer_core::edit::AppearanceOutcome::RecordedNotPainted(_)
                    ) {
                        "no"
                    } else {
                        "yes"
                    },
                    u8::from(outcome.foreign_appearance_replaced),
                    outcome.appearance,
                )
            });
            let mut lines = Vec::new();
            // **First**, because it is the only one about something on screen.
            //
            // Read off `AppearanceOutcome` rather than off `appearance_stale`
            // being `Some`, which is what the engine's own field doc asks for:
            // `appearance_regenerated: false` + `appearance_stale: None` meant
            // two different things — *nothing needed redrawing* and *something
            // did and pdfcer could not* — and this crate read the ambiguous
            // pair for the whole of the time `with_background` existed. An enum
            // cannot silently acquire a fourth meaning for an existing value.
            if let pdfcer_core::edit::AppearanceOutcome::RecordedNotPainted(why) =
                &outcome.appearance
            {
                lines.push(crate::text::forms::field_appearance_not_repainted(
                    outcome.resized,
                    why,
                ));
            }
            // **`rect_after`, not "always"**. This line was pushed
            // unconditionally, so every border, caption, visibility and — since
            // O202 — colour edit ended with *"The box was moved."* on the
            // status line, naming an act the operator had not performed. The
            // engine reports geometry as `Option`; a `None` there means the
            // edit never touched the rectangle, and there is nothing to say
            // about a move that did not happen.
            if outcome.rect_after.is_some() {
                lines.push(
                    crate::text::forms::field_widget_moved(
                        outcome.resized,
                        outcome.appearance_regenerated,
                    )
                    .to_owned(),
                );
            } else {
                lines.push(crate::text::forms::field_widget_property_changed(
                    touched,
                    outcome.appearance_regenerated,
                ));
            }
            if outcome.foreign_appearance_replaced {
                lines.push(crate::text::forms::field_foreign_appearance_replaced().to_owned());
            }
            if outcome.siblings_untouched > 0 {
                lines.push(crate::text::forms::field_siblings_untouched(
                    outcome.siblings_untouched,
                ));
            }
            lines
        })
    });
}

/// **Read an FDF, XFDF or CSV file and set this document's field values from
/// it.**
pub(super) fn import_data(doc: &mut OpenDoc, path: &std::path::Path) {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("import-form-data-failed stage=read detail={error}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_form::import_unreadable(&error.to_string()),
            );
            return;
        }
    };
    // The extension decides the parser, matching the export's rule exactly —
    // one convention for both halves of the round trip, so a file exported as
    // `.csv` and imported as `.csv` cannot land in a branch nobody chose. FDF
    // is the default for the same reason it is on the way out: it is the format
    // §12.7.8 defines for this data.
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let parsed = match extension.as_str() {
        // ui-text-exempt: file extensions, matched not displayed.
        "xfdf" => pdfcer_core::fdf::FormData::parse_xfdf(&bytes).map_err(|e| e.to_string()),
        "csv" => pdfcer_core::formcsv::parse_csv(&bytes).map_err(|e| e.to_string()),
        _ => pdfcer_core::fdf::FormData::parse_fdf(&bytes).map_err(|e| e.to_string()),
    };
    let data = match parsed {
        Ok(data) => data,
        Err(detail) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("import-form-data-failed stage=parse format={extension} detail={detail}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_form::import_unparseable(&detail),
            );
            return;
        }
    };

    let fields = data.fields.len();
    super::apply::vector_edit(doc, "import-form-data", 0, 1, move |session| {
        session.import_form_data(&data).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // `-applied`, NOT plain `import-form-data`.
                    //
                    // `vector_edit` writes its own line for the same edit —
                    // `import-form-data page=0 n=1 epoch=1 disclosures=…` —
                    // and trace matching is on the **exact event name**. Two
                    // lines sharing a name is how a check reads the wrong one
                    // and then reports failure about a gesture that worked: a
                    // driven check taking `.last()` gets `vector_edit`'s line,
                    // which carries no `applied=` key, and reports
                    // `applied=0` about an import that set every field it was
                    // given.
                    //
                    // **A module's own summary line takes a verb suffix; the
                    // funnel's label keeps the bare name.**
                    "import-form-data-applied read={fields} applied={} skipped={}",
                    outcome.applied, outcome.skipped
                )
            });
            let mut lines = vec![crate::text::export_form::imported(
                outcome.applied,
                outcome.skipped,
            )];
            if outcome.password_values_withheld > 0 {
                lines.push(crate::text::export_form::import_passwords_withheld(
                    outcome.password_values_withheld,
                ));
            }
            lines
        })
    });
}

/// **Give an existing push button an action**, as one undoable command.
fn set_button_action(
    doc: &mut OpenDoc,
    field: &str,
    action: Option<pdfcer_core::edit::ButtonAction>,
) {
    let name = field.to_owned();
    super::apply::vector_edit(doc, "set-button-action", 0, 1, |session| {
        session.set_button_action(field, action).map(|change| {
            vec![crate::text::buttonaction::changed(
                &name,
                change.replaced.as_deref(),
            )]
        })
    });
}

/// **Rename the selected field.**
pub(super) fn rename(doc: &mut OpenDoc, from: &str, to: &str) {
    let to = to.trim().to_owned();
    if to.is_empty() || to == from {
        return;
    }
    doc.selected_field = None;
    super::apply::vector_edit(doc, "rename-field", 0, 1, |session| {
        session
            .rename_field(from, &to)
            // `inspect_err`, so the error is still returned unchanged for the
            // funnel to trace and for the floor to word if `correctable`
            // declines to claim it. Recording is an addition to the error path,
            // never a substitution for it.
            .inspect_err(|error| {
                if let Some(declined) = correctable(error) {
                    decline::record_field_rename_refusal(declined);
                }
            })
            .map(|outcome| {
                let mut lines = vec![crate::text::forms::form_field_renamed(
                    &outcome.to,
                    outcome.descendants_renamed,
                )];
                // Rule 4: pdfcer rewrote buttons the operator did not touch.
                //
                // `/ResetForm` and `/SubmitForm` name their targets as fully
                // qualified NAME STRINGS, so a rename that did nothing else would
                // leave them pointing at nothing. `rename_field` repairs them —
                // correctly, invisibly, and not as anything the operator pressed.
                // No view in this shell shows an action's target list, so without
                // this sentence the repair is unobservable.
                //
                // Conditional, like every disclosure on this surface: a rename
                // that touched no action says one thing. A receipt that recites
                // "0 buttons updated" after every rename is a form, and by the
                // third one nobody reads the line that matters.
                if outcome.action_targets_retargeted > 0 {
                    lines.push(crate::text::forms::form_field_actions_retargeted(
                        outcome.action_targets_retargeted,
                    ));
                }
                lines
            })
    });
}

/// The status lines one authoring outcome owes the operator.
pub(super) fn disclosures(
    outcome: &pdfcer_core::edit::FieldAuthorOutcome,
    kind: crate::canvas::formfield::FormFieldKind,
) -> Vec<String> {
    let mut lines = vec![crate::text::forms::form_field_added(&kind.noun())];
    if outcome.merged {
        lines.push(crate::text::forms::form_field_merged());
    }
    if outcome.disclosures.tooltip_declined {
        lines.push(crate::text::forms::form_field_no_tooltip());
    }
    if outcome.disclosures.has_no_options {
        lines.push(crate::text::forms::form_field_no_options());
    }
    if outcome.disclosures.tagged_document || outcome.disclosures.structure_tab_order {
        lines.push(crate::text::forms::form_field_tagged_document());
    }
    lines
}

/// Which refusals the operator can do something about.
fn correctable(error: &pdfcer_core::edit::EditError) -> Option<Declined> {
    use pdfcer_core::edit::EditError as E;
    use pdfcer_core::forms_author::FormAuthorError as A;
    match error {
        E::FieldNameTaken { .. } => Some(Declined::FieldNameTaken),
        E::WidgetHasNoFieldIdentity { .. } => Some(Declined::WidgetHasNoName),
        // Unreachable from `adopt_widget` and the only refusal `author` can
        // raise — which is why this is a table shared by two surfaces rather
        // than a helper belonging to one of them.
        //
        // `EditError::FieldAuthoring` is `#[error(transparent)]`, so the inner
        // variant is the whole of the error and matching it is matching the
        // refusal. `terminal` is the fully-qualified name of the existing field
        // in the way, which the engine built by walking `/Parent` — it is not
        // derivable from the operator's string, which is exactly why it is
        // cloned out and carried rather than recomputed.
        //
        // Matched on the VARIANT, never on `error.to_string()`. The engine's
        // prose for this refusal is good and names the field, and it is still
        // the wrong thing to key on: prose is the part of an API with no
        // compatibility promise, so an arm keyed on words goes on firing after
        // the engine narrows the condition underneath it.
        E::FieldAuthoring(A::FieldPathCrossesTerminal { terminal, .. }) => {
            Some(Declined::FieldPathCrossesTerminal(terminal.clone()))
        }
        // A **floor**, not the first line of defence: no surface in this shell
        // currently reaches it, because each of the three routes to a partial
        // name greys its own control first.
        //
        // | route | the control | reaches this arm |
        // |---|---|---|
        // | `adopt_widget` | `panels::forms::tab_order::register`'s per-widget name box — free text, but the row calls `adopt_preview` every frame and `add_enabled(false, ..)`s the button on any `Err` | no — the button greys while the period is typed |
        // | `rename_field` | `panels::properties::formfield`'s name box — commit greyed on `validate_partial_name` | no |
        // | `sign` | no name box at all; the window lists fields that exist, and an existing FQN takes the engine's reuse branch where a period is legitimate | no |
        //
        // Kept rather than deleted because every `no` above is a claim about a
        // shell-side gate, not about the engine. Two are judgments a refactor
        // can relax; the third is a preview call that a `&self`-to-`&mut self`
        // change upstream would silently end. When one goes, the operator gets
        // a sentence instead of `Display` output with nobody having to notice.
        //
        // The general rule the table teaches: **a guard's placement decides
        // which surface has to explain it.** `reject_dotted_partial` lives
        // inside the engine's `adopt_plan`, and `adopt_preview` is that same
        // plan with the writes dropped, so the refusal arrives in the preview a
        // row already draws from — a hover, not the status bar. A panel that
        // greys on a refusal it cannot name falls into `refusal_hint`'s
        // catch-all, which is the program apologising for a rule it is
        // enforcing correctly; give it wording of its own instead.
        //
        // Measured by two deliberately separate tests: the grey by
        // `panels::forms::tab_order::register::tests::a_dotted_name_greys_the_register_button_and_the_hover_names_the_rule`,
        // this arm by `tests::adopting_under_a_dotted_name_is_refused_and_worded`.
        E::FieldAuthoring(A::DottedPartialName { supplied }) => {
            Some(Declined::DottedPartialName(supplied.clone()))
        }
        // A rename onto a name something already bears: the **most common**
        // way a rename fails, and the **only** refusal the rename surface can
        // reach that its own gate cannot predict, because predicting it needs
        // the field tree rather than the typed string.
        //
        // Mapped to `FieldNameTaken` rather than to a variant of its own,
        // under *one fact, one wording*: the engine raises two variants here
        // (`EditError::FieldNameTaken` from `adopt_widget`,
        // `FormAuthorError::RenameCollision` from `rename_field`) because they
        // are produced by different code, but from the operator's chair they
        // are one sentence — *something already has that name* — with one
        // remedy. See that variant's own doc for the table.
        E::FieldAuthoring(A::RenameCollision { .. }) => Some(Declined::FieldNameTaken),
        _ => None,
    }
}

/// **Move one widget by a page-space delta.**
pub(super) fn move_widget(doc: &mut OpenDoc, field: &str, widget: usize, dx: f64, dy: f64) {
    super::apply::vector_edit(doc, "move-widget", 0, 1, |session| {
        session.move_widget(field, widget, dx, dy).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // `-applied`, per the convention this module records at
                    // length: the funnel writes its own bare-named line for the
                    // same edit and `.last()` would read that one.
                    "move-widget-applied field={field} widget={widget} dx={dx:.3} \
                     dy={dy:.3} siblings={}",
                    outcome.siblings_left_behind
                )
            });
            if outcome.siblings_left_behind > 0 {
                vec![crate::text::forms::widget_siblings_unmoved(
                    outcome.siblings_left_behind,
                )]
            } else {
                Vec::new()
            }
        })
    });
}

#[cfg(test)]
mod tests;
