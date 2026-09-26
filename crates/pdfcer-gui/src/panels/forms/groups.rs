//! # `panels::forms::groups` — the Field-groups section, and the shell's only
//! route to deleting one
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/forms/groups.md`.

use pdfcer_core::forms::AcroForm;

use crate::app::actions::Action;
use crate::app::actions::forms::{FieldAction, groups as armed};
use crate::app::state::OpenDoc;
use crate::text::forms as t;

/// The region the collapsing header publishes, so a driven check can open it.
const REGION_HEADER: &str = "forms.groups.header"; // ui-text-exempt: trace region name, never displayed
/// The prefix each row's **Delete group…** control publishes under.
const REGION_ARM: &str = "forms.groups.arm."; // ui-text-exempt: trace region name, never displayed
/// The armed block's commit control.
const REGION_CONFIRM: &str = "forms.groups.confirm"; // ui-text-exempt: trace region name, never displayed
/// The armed block's cancel control.
const REGION_CANCEL: &str = "forms.groups.cancel"; // ui-text-exempt: trace region name, never displayed

/// How many rows the trace prints before it stops.
const MAX_TRACED_ROWS: usize = 200;

/// Draw the Field-groups section.
pub(super) fn section(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    form: &AcroForm,
    actions: &mut Vec<Action>,
) {
    // R83, asked once, before a single control is drawn. `deletion_refusal`
    // is a pure query — it reads the signature census and the trailer and
    // mutates nothing — so it is safe to call every frame from a UI, and core
    // says so in as many words.
    let refusal = doc.session.deletion_refusal();

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "form-groups nodes={} refused={}",
            form.groups.len(),
            u8::from(refusal.is_some()),
        )
    });

    if form.groups.is_empty() {
        // R124. Nothing at all — not a heading, not an empty state. A flat form
        // has no field groups in the same way it has no pages 3 through 9: the
        // absence is not news.
        return;
    }

    // Before the header, so the listing is in the trace whether or not the
    // operator opened it. See `trace_rows`.
    trace_rows(doc, form);

    let header = egui::CollapsingHeader::new(t::field_groups_heading())
        .id_salt("pdfcer-forms-groups")
        // Closed by default, like the Tab-order section beside it and for the
        // same reason: the panel's primary job is filling, and this answers an
        // occasional question about the form's shape. A driven check opens it
        // through `REGION_HEADER`.
        .default_open(false)
        .show(ui, |ui| {
            // EVERY DISCLOSURE ABOVE THE LIST, without exception — the rule
            // four other surfaces in this panel follow, for one reason: an
            // operator who reads a short list and stops has drawn their
            // conclusion by the time a footnote would reach them.
            ui.label(t::field_groups_explainer());

            match &refusal {
                // A refusal is a SENTENCE, never a silence — and never a
                // greyed button either. See the module header for why R9 sends
                // a permanently-refused capability to prose rather than to
                // greying, and why the sentence has to name the actual cause
                // rather than assuming certification.
                Some(error) => {
                    ui.add_space(4.0);
                    ui.colored_label(ui.visuals().warn_fg_color, t::field_groups_refusal(error));
                    ui.add_space(4.0);
                    // The nodes are still LISTED. Knowing what a form is
                    // organised into is a reading, not a change, and a
                    // certification signature forbids the second and not the
                    // first — so refusing to show the list as well would
                    // withhold information the document freely permits.
                    for node in &form.groups {
                        ui.label(t::field_group_row(
                            &node.fully_qualified_name,
                            form.descendants_of(&node.fully_qualified_name).count(),
                        ));
                    }
                }
                None => rows(ui, doc, form, actions),
            }
        });
    crate::diag::ui_rect(REGION_HEADER, header.header_response.rect);
}

/// **The per-row census, written from the MODEL rather than from the drawing.**
fn trace_rows(doc: &OpenDoc, form: &AcroForm) {
    if !crate::diag::enabled() {
        return;
    }
    let live = armed::armed(doc.edit_epoch);
    for node in form.groups.iter().take(MAX_TRACED_ROWS) {
        let name = &node.fully_qualified_name;
        let fields = form.descendants_of(name).count();
        let armed_here = live.as_ref().is_some_and(|a| a.preview.group_name == *name);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // The NAME is not carried: a field's name is the operator's own
            // words about their document, and `adopt-row` and `bookmark-add`
            // make the same ruling for the same reason. The object number
            // identifies the row for a check; the count is what a check asserts
            // against.
            format!(
                "form-group-row obj={} fields={fields} armed={}",
                node.id.num,
                u8::from(armed_here),
            )
        });
    }
}

/// One row per grouping node, and the armed block under whichever row owns it.
fn rows(ui: &mut egui::Ui, doc: &OpenDoc, form: &AcroForm, actions: &mut Vec<Action>) {
    let live = armed::armed(doc.edit_epoch);
    let mut raised: Option<FieldAction> = None;

    for node in form.groups.iter() {
        let name = &node.fully_qualified_name;
        // CORE'S walk, not a prefix match written here.
        //
        // `FieldGroupDeletion::nodes`' doc comment forbids a shell re-deriving
        // core's notion of descendant — *"the same argument applies with more
        // force here, because the shell would be re-deriving which ancestors a
        // cascade would have emptied"* — and it is right. But `descendants_of`
        // **is** core's notion, exposed for exactly this, so calling it is the
        // opposite of re-deriving it.
        //
        // It is only an indication all the same, and the row's wording keeps it
        // to one: how many fields are filed under this name. The boxes and the
        // emptied ancestors are the preview's to give, because a walk of the
        // field list cannot answer either.
        let fields = form.descendants_of(name).count();
        let armed_here = live.as_ref().is_some_and(|a| a.preview.group_name == *name);

        // `push_id` per node, or two rows' buttons share one egui id and the
        // wrong one responds to a hover — the collision `crate::panels::comments`
        // keys its rows against.
        ui.push_id(node.id.num, |ui| {
            // TWO LINES, not one, because this is a DOCK PANEL and not a
            // dialog. A fully-qualified name is unbounded and a button is
            // fixed-width; on one `horizontal` at the dock's default 320 points
            // the label would push the button off the right-hand edge, where a
            // driven run has already measured a control at x=1090 in a panel
            // ending at x=1100. A label wraps and a button does not, so the
            // identifying text gets its own line.
            ui.label(t::field_group_row(name, fields));
            let button = ui
                .button(t::field_group_delete_button())
                .on_hover_text(t::field_group_delete_hover(name));
            crate::diag::ui_rect(&format!("{REGION_ARM}{}", node.id.num), button.rect);
            if button.clicked() && raised.is_none() {
                raised = Some(FieldAction::ArmGroupDeletion(Some(name.clone())));
            }

            if armed_here {
                // Safe: `armed_here` is only true when `live` is `Some`.
                if let Some(a) = live.as_ref() {
                    disclosure(ui, a, &mut raised);
                }
            }
        });
        ui.separator();
    }

    if let Some(action) = raised {
        actions.push(action.into());
    }
}

/// **What the operator is about to lose**, drawn under the row they armed.
fn disclosure(ui: &mut egui::Ui, live: &armed::Armed, raised: &mut Option<FieldAction>) {
    let p = &live.preview;
    ui.add_space(4.0);
    ui.colored_label(
        ui.visuals().warn_fg_color,
        t::field_group_preview_summary(
            &p.group_name,
            p.terminals.len(),
            p.widgets_removed,
            p.nodes_removed,
        ),
    );
    if let Some(named) = t::field_group_preview_names(&p.terminals) {
        ui.label(egui::RichText::new(named).small());
    }
    ui.horizontal(|ui| {
        let confirm = ui.button(t::field_group_preview_confirm(p.terminals.len()));
        crate::diag::ui_rect(REGION_CONFIRM, confirm.rect);
        if confirm.clicked() && raised.is_none() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "form-group-delete-requested terminals={} widgets={} nodes={}",
                    p.terminals.len(),
                    p.widgets_removed,
                    p.nodes_removed,
                )
            });
            *raised = Some(FieldAction::DeleteGroup {
                group: p.group_name.clone(),
            });
        }
        let cancel = ui.button(t::field_group_preview_cancel());
        crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
        if cancel.clicked() && raised.is_none() {
            *raised = Some(FieldAction::ArmGroupDeletion(None));
        }
    });
    ui.add_space(4.0);
}
