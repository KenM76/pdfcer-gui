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
///
/// ★ A published region name is a cross-repo stability contract: the harness
/// asserts on it by string, so renaming one turns a check into a skip rather
/// than a failure.
const REGION_HEADER: &str = "forms.groups.header"; // ui-text-exempt: trace region name, never displayed
/// The prefix each row's **Delete group…** control publishes under.
///
/// ★★ Suffixed with the grouping node's **object number**, not its index in
/// `AcroForm::groups` and not its name.
///
/// - Not the index: deleting one node renumbers every node after it, so a check
///   that pressed "row 1" twice would press two different groups. This is the
///   same argument `tab_order::register` makes for keying on tab position
///   rather than list position.
/// - Not the name: a fully-qualified field name is the operator's own words and
///   may contain spaces, `=` and anything else `/T` permits (Table 220 makes it
///   a text string). Region names are parsed out of a `key=value` trace line,
///   so a name would break the parse on exactly the documents whose fields are
///   worth naming.
///
/// An object number is stable across the session, unique, and safe in a trace.
const REGION_ARM: &str = "forms.groups.arm."; // ui-text-exempt: trace region name, never displayed
/// The armed block's commit control.
const REGION_CONFIRM: &str = "forms.groups.confirm"; // ui-text-exempt: trace region name, never displayed
/// The armed block's cancel control.
const REGION_CANCEL: &str = "forms.groups.cancel"; // ui-text-exempt: trace region name, never displayed

/// How many rows the trace prints before it stops.
///
/// `pdfcer_core::forms::MAX_FORM_FIELDS` is 500,000 and a pathological form
/// could carry grouping nodes in proportion, so an uncapped per-row census
/// would bury every other line in a capture. The summary line is never capped,
/// so the *count* stays provable even when the enumeration stops — the same
/// rule [`super::tab_order`] caps its own row census under.
const MAX_TRACED_ROWS: usize = 200;

/// Draw the Field-groups section.
///
/// Called from [`super::body`] with the `/AcroForm` it has already parsed —
/// not re-derived here, because two parses of one form per frame is a cost with
/// no benefit and because a second parse could in principle disagree with the
/// one the rows above came from.
///
/// # ★ It renders NOTHING on a flat form, and that is R124 rather than an
/// oversight
///
/// `AcroForm::groups` is empty for a flat form, *"which is every file in the
/// Pass 7.0 census"* — so on the overwhelming majority of real documents this
/// section is not drawn, not collapsed-and-empty, not a heading over nothing.
/// Core's own doc comment on that field states the obligation: *"a consumer
/// that renders these must therefore render nothing when the list is empty
/// rather than an empty section."*
///
/// `actions` is pushed at most once per frame — see [`rows`] for why one press
/// per frame is enforced rather than assumed.
pub(super) fn section(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    form: &AcroForm,
    actions: &mut Vec<Action>,
) {
    // ★★★ R83, asked once, before a single control is drawn. `deletion_refusal`
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

    // ★ Before the header, so the listing is in the trace whether or not the
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
            // ★ EVERY DISCLOSURE ABOVE THE LIST, without exception — the rule
            // four other surfaces in this panel follow, for one reason: an
            // operator who reads a short list and stops has drawn their
            // conclusion by the time a footnote would reach them.
            ui.label(t::field_groups_explainer());

            match &refusal {
                // ★★★ A refusal is a SENTENCE, never a silence — and never a
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
///
///
/// This module's header promises the row lines are written *"whether or not the
/// collapsing header is open, so the listing is provable from a trace without
/// anyone having to click."* It was not true: the loop that wrote them sat
/// inside `CollapsingHeader::show`'s body, and egui does not run that closure
/// while the header is closed — and this section ships **closed**.
///
/// So a trace from a run that never opened the header carried the summary and
/// no rows, and a check reading it would conclude the form has no groups. The
/// promise was in prose, in a doc comment, checked by nobody.
///
/// ⇒ Lifted here, above the header, where the claim is true by construction.
/// The separation is also the more honest one and matches
/// `crate::panels::comments`: **a trace describes what the panel computed**,
/// not what it happened to paint. A surface that traced only what it drew would
/// go quiet exactly when a reader most wants to know what it decided — behind a
/// closed header, off the bottom of a scroll, inside a collapsed tree.
///
/// ★ Capped at [`MAX_TRACED_ROWS`], and the summary line above carries the real
/// total, so a truncated listing can never be mistaken for a short one.
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
///
/// # ★ At most one press per frame, and it is not an accident
///
/// The loop stops raising after the first press. Two presses in one frame would
/// queue two actions against a form parsed **before** either ran, and the
/// second would be acting on a set the first has already changed. The names
/// here are stable where indices are not, so the second action would in fact
/// still name the right node — the discipline is kept anyway, because *"queue
/// only what was computed against the state you have"* is worth holding
/// mechanically rather than re-deriving each time a queued verb is added. It
/// costs the operator nothing: physically, one press per frame is all there is.
///
/// # ★★ Order is core's, deepest-first, and is deliberately not re-sorted
///
/// `AcroForm::groups` is post-order — a child appears before its parent — and
/// core states it because *"it is the opposite of what DFS order suggests and a
/// consumer that assumed parents-first would render a breadcrumb backwards."*
/// It is also the useful order here: the deepest node is the smallest,
/// least-destructive removal, so the list reads from the safest press to the
/// most sweeping one.
///
fn rows(ui: &mut egui::Ui, doc: &OpenDoc, form: &AcroForm, actions: &mut Vec<Action>) {
    let live = armed::armed(doc.edit_epoch);
    let mut raised: Option<FieldAction> = None;

    for node in form.groups.iter() {
        let name = &node.fully_qualified_name;
        // ★★ CORE'S walk, not a prefix match written here.
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

        // ★ `push_id` per node, or two rows' buttons share one egui id and the
        // wrong one responds to a hover — the collision `crate::panels::comments`
        // keys its rows against.
        ui.push_id(node.id.num, |ui| {
            // ★★ TWO LINES, not one, because this is a DOCK PANEL and not a
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
///
/// # ★★★ This block is the whole point of the preview existing
///
/// The engine's own words for why: *"an operator looking at a collapsed tree
/// row cannot see how many that is or what they are called. This answers that
/// question against the live session, before anything changes."* And the reason
/// it is safe to draw from: the preview runs the **same gates** as the
/// deletion, because both go through one `group_deletion_preflight` — *"a
/// preview that succeeds where the act fails is worse than no preview: it
/// invites the operator to confirm something that cannot happen."*
///
/// # The order of what it says
///
/// Numbers, then names, then the two controls. The numbers decide *whether*,
/// the names decide *which*, and a control above either would be a button
/// offered before its own justification.
///
/// # ★ Cancel is a real control and not an implicit click-away
///
/// A destructive confirmation the operator can only escape by pressing
/// something else in the panel is one they can dismiss by accident and cannot
/// dismiss on purpose. It raises `ArmGroupDeletion(None)`, which changes
/// nothing and clears the block.
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
