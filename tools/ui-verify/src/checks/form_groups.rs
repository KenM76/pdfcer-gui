//! `form_groups` — **the two surfaces an audit found the engine had shipped and
//! this shell had never grown**, driven against the real binary.
//!
//! # What this module is about
//!
//! `EDITABLE_SURFACES.md` (2026-08-28) enumerated every `pub fn` on
//! `EditSession` and diffed it against every call site in the shell. Two of the
//! misses were forms:
//!
//! | engine | shell, before |
//! |---|---|
//! | `delete_field_group` + `field_group_deletion_preview` | **no route at all** — a grouping node has no widget to click, no row in the fill list and no entry in the tab order, so nothing could name one |
//! | `rename_refusal` (and, it turned out, `deletion_refusal`) | **consulted by nothing** — Rename and both Delete controls were drawn live on every document, including one that refuses them |
//!
//! Two checks, because the two are different claims about different surfaces and
//! a reader who sees one fail should not have to work out which half is broken.
//!
//! # ★★★ Why these need DRIVING and cannot be settled by a unit test
//!
//! Both defects are of the shape this harness exists for, and neither is
//! observable from inside the crate:
//!
//! - **The group route** is four links — a section that only draws when
//!   `AcroForm::groups` is non-empty, a button that raises an action, a funnel
//!   arm that runs a `&mut self` preview, and a *second* button that only
//!   exists once the first has been pressed. Every link has unit tests. The
//!   thing that ships broken is the wiring, and a two-press protocol is the
//!   most wiring any form verb in this shell has.
//! - **The refusal** is an *absence*: on a certified document the correct
//!   behaviour is that no control is drawn. A unit test asserting "the function
//!   returns Some" passes on a build where nothing calls the function — which
//!   is precisely the state the audit found, under 2,538 passing tests.
//!
//! ⇒ So [`FieldGroupDeleteRemovesTheSubtree`] presses the buttons, and
//! [`StructuralRefusalsAreSentencesNotControls`] reads the *absence* of two
//! regions beside the trace line that proves the gates were asked. That second
//! pairing is `crate::checks` rule 4 discharged: never treat an absence as
//! evidence unless you have shown the thing that would have produced it was
//! working.
//!
//! # The fixtures, and why each is the only one that would do
//!
//! | fixture | what it carries | why nothing else works |
//! |---|---|---|
//! | `forms/nested-form.pdf` | `Personal.Name`, `Personal.Address.City`, `Personal.Address.Zip` | **The only fixture with grouping nodes at all.** Core's own note on `AcroForm::groups` says it is empty for a flat form, *"which is every file in the Pass 7.0 census"* — and the two-level shape is what makes the **cascade** observable: deleting `Personal` also empties `Personal.Address`, a node nobody named |
//! | `../../fixtures/certified-nested-form.pdf` | `/DocMDP` at **`/P 2`**, over the SAME two-level tree | Certified **and** nested, which is the intersection nothing else in either corpus occupies. Filling permitted, restructuring refused. The engine's `PROVENANCE.md` says why `/P 1` would not do: it refuses *everything*, so a check written against it *"passes whether or not those gates differ at all"* — the fill controls would be gone too, and the check could not tell a correct build from one that disables the whole panel |
//!
//! ★★★ **The second fixture was `forms/certified-p2-form.pdf` and could not
//! serve.** That file is certified at the right `/P` and its fields are
//! **flat** (`FullName`, `Subscribe` — no dots), so `AcroForm::groups` is empty
//! and `panels::forms::groups::section` takes its early return before laying
//! out a single control. Phase F's arm-withholding assertion — *the section
//! draws no `forms.groups.arm.*` control* — was therefore true of a section
//! that never drew: an absence with nothing behind it, which `crate::checks`
//! rule 4 forbids. Every certified file in either corpus was flat
//! (`certified-comments.pdf`, `threaded-comments.pdf`, `certified-p2-form.pdf`)
//! and the only nested one (`nested-form.pdf`) was uncertified, so the fix was
//! a **fixture**, not a rewrite: `tools/gen-certified-nested-fixture.py` copies
//! `nested-form.pdf`'s field tree under `certified-p2-form.pdf`'s
//! certification. Its header carries the whole argument, and
//! `crates/pdfcer-gui/src/app/actions/forms/delete.rs`'s
//! `the_certified_nested_fixture_is_both_certified_and_nested` pins the four
//! properties it has to keep — it loads, `deletion_refusal` is `Some`,
//! `AcroForm::groups` is non-empty, and `fill_refusal` is `None`.
//!
//! # Phases
//!
//! ## `field_group_delete_removes_the_subtree` — `nested-form.pdf`
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | View ▸ Forms, then open the Field-groups header | `form-groups nodes=2 refused=0` |
//! | B | read the per-row census | one `form-group-row` per node, none armed |
//! | C | press the root node's **Delete group…** | `form-group-preview terminals=3 nodes=2`, and the row reports `armed=1` |
//! | D | press **Delete 3 fields** | `delete-field-group-applied terminals=3 widgets=3 nodes=2` |
//! | E | re-read the census | `nodes=0` — the subtree and both grouping nodes are gone |
//!
//! ## `structural_refusals_are_sentences_not_controls` — `certified-nested-form.pdf`
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | F | Edit mode, View ▸ Forms, then **open** the Field-groups header | `form-groups nodes=2 refused=1`, and — with the body laid out — **no** `forms.groups.arm.*` region |
//! | G | click a widget the canvas census names | `form-field-selected field=…` |
//! | H | File ▸ Properties, then read the Properties pane's gate census | `form-field-gates rename_refused=1 delete_refused=1`, and **neither** `properties.form_field.rename` nor `properties.form_field.delete` declared |
//!
//! ★★★ **H opens the Properties panel, and that is a correction rather than a
//! flourish.** Edit's default dock puts Properties and Forms in ONE tabbed
//! stack, and a tabbed stack draws only its active tab — so phase F's own
//! `View ▸ Forms` click pushed Properties to the back and
//! `panels::properties::formfield::section` stopped running. The 2026-08-29
//! sweep reported *"a field is selected and the Properties pane traced no
//! `form-field-gates` line"* about a pane the check had itself hidden; the
//! selection in that same trace is real. See `PROPERTIES_ITEM`.
//!
//! ★ Phase F is what makes phase H's absences readable: a build that simply
//! failed to draw the Properties section would produce the same missing
//! regions, and the `form-field-gates` line — written unconditionally, refused
//! or not — is what tells the two apart.
//!
//! ★★★ **The arm half of F is LIVE, and its history is worth keeping.** It has
//! been through three states, and each was a worse-looking fix than it sounds:
//!
//! 1. **Asserted unconditionally on a flat fixture** — and passed, having
//!    checked that a section which never drew drew no controls.
//! 2. **SKIPped the whole check** — the wrong half to cut. `refused` is traced
//!    ABOVE `groups::section`'s early return, so it is real evidence even on a
//!    flat form, and phases G and H (the Rename box and both Delete buttons,
//!    which are what this check's `defect()` actually names) need only a
//!    certified document with a widget. That left the check reporting SKIP on
//!    every run: zero coverage, counted as a check.
//! 3. **Conditional, with a note when it did not run** — honest, and still zero
//!    coverage of the one assertion, because the condition was never true.
//!
//! ⇒ The fixture closed it. `certified-nested-form.pdf` traces `nodes=2`, so
//! the arm assertion runs on every run — and a run that reports `nodes=0` is
//! now a **FAILURE** rather than a note, because on this fixture that means
//! either the panel stopped listing an interior the engine can see or the
//! fixture was regenerated wrong. Both are red, and a silent pass is the one
//! outcome this phase must never produce again.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_or_in_overflow, list,
    live_names,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The two-level form, in the **engine's** corpus. See the module header's
/// fixture table. Resolved by [`engine_fixture`].
const NESTED: &str = "forms/nested-form.pdf";
/// The `/P 2` certified form **over that same two-level tree**, in *this*
/// repository. See the module header's fixture table.
///
/// ★★★ **Relative to `CARGO_MANIFEST_DIR`, which is `tools/ui-verify`** — two
/// levels up, not three, and resolved by [`local_fixture`] rather than by
/// [`engine_fixture`]. `annot_delete_gate` and `field_delete_gate` both record
/// the same trap: written as `../../../fixtures/…` it resolves to a
/// `D:/Dev/fixtures/` that does not exist, and the check SKIPs on every run
/// while telling the reader to run a generator that writes somewhere else.
///
/// ★ It is in this repository and not the engine's because `D:/Dev/pdfcer` is
/// READ-ONLY for this project — see the workspace manifest — so a fixture this
/// project needs and that project does not have is authored here, by
/// `tools/gen-certified-nested-fixture.py`.
const CERTIFIED: &str = "../../fixtures/certified-nested-form.pdf";
/// The command that shows the Forms panel, and the tab it lives on.
const PANEL_TAB: &str = "view";
const PANEL_ITEM: &str = "ribbon.item.view.panel_forms";
/// The command that shows the **Properties** panel, and the tab it lives on.
///
/// ★★★ **Needed because the two panels share one tabbed dock stack, and a
/// tabbed stack draws only its active tab.** Edit's default arrangement puts
/// Properties, Comments, Forms, Redact, Dimension groups and Attachments in one
/// stack (`app::modes::defaults`), so phase F's own `ribbon.item.view.panel_forms` click — the
/// one that brings the Forms panel forward to read the Field-groups section —
/// **pushes Properties to the back of the same stack**, and
/// `panels::properties::formfield::section` stops running entirely.
///
/// That is why the 2026-08-29 sweep reported *"a field is selected and the
/// Properties pane traced no `form-field-gates` line"*: the selection was real
/// (`form-field-selected page=0 field=Personal.Address.Zip widget=0` is in the
/// trace) and the pane that would have written the census was a background tab
/// the check had itself put there. Phase H now brings it back before reading.
///
/// ★ `file.properties` is `show_panel`, not a toggle, so it is idempotent and
/// scrolls the tab into view — which matters, because the dock publishes a
/// `dock.tab.*` rect only for tabs its bar is currently showing, and clicking
/// that rect directly would work only when the bar happened to be scrolled
/// right. Going through the ribbon command asks for the panel by name instead.
const PROPERTIES_TAB: &str = "file";
const PROPERTIES_ITEM: &str = "ribbon.item.file.properties";
/// The mode the structural surfaces are reached in.
const MODE: &str = "edit";

/// The Field-groups section's collapsing header, which ships closed.
const REGION_HEADER: &str = "forms.groups.header";
/// The prefix each row's **Delete group…** control publishes under, suffixed
/// with the grouping node's object number.
const REGION_ARM: &str = "forms.groups.arm.";
/// The armed block's commit control.
const REGION_CONFIRM: &str = "forms.groups.confirm";
/// The Rename control in the Properties pane — published **only when drawn**.
const REGION_RENAME: &str = "properties.form_field.rename";
/// The Delete-field control in the Properties pane — published **only when
/// drawn**.
const REGION_DELETE: &str = "properties.form_field.delete";
/// The Properties pane's form-field section, whose rect is its own `min_rect`
/// and is therefore always inside the panel by construction. Used only to prove
/// the section drew at all.
const REGION_SECTION: &str = "properties.form_field";

/// The section's per-frame census: how many grouping nodes, and whether the
/// document refused.
const CENSUS: &str = "form-groups";
/// One line per grouping node drawn.
const ROW: &str = "form-group-row";
/// The funnel's line for the preview — the FIRST press.
const PREVIEWED: &str = "form-group-preview";
/// What the confirm button raises, before the funnel sees it.
const REQUESTED: &str = "form-group-delete-requested";
/// ★★★ The module's own summary line for the deletion — **`-applied`**, not the
/// bare `delete-field-group` the `vector_edit` funnel writes for the same edit.
///
/// Two lines sharing a name is how a check taking `.last()` reads the wrong one
/// and then reports failure about a gesture that worked. This project has made
/// that mistake twice (`text-style`, `import-form-data`); the suffix convention
/// is what stops the third.
const APPLIED: &str = "delete-field-group-applied";
/// The Properties pane's per-frame gate census.
const GATES: &str = "form-field-gates";
/// Traced when a canvas click selects an existing field.
const SELECTED: &str = "form-field-selected";
/// The canvas census naming every selectable widget, in canvas space.
const TARGETS: &str = "form-target";

/// A window tall enough that the Forms panel's third section is on screen
/// without a scroll hunt.
///
/// ★ The same remedy `form_field` applies for the same measured reason: the
/// harness's default window gives a dock slot a few hundred points, and the
/// Forms panel now draws a header, two disclosure blocks, three whole-form
/// controls, a Tab-order section and a Field-groups section above its fill
/// list. A check that failed because the *window* was small would be reporting
/// the wrong subject.
const VIEWPORT: &str = "0,0,1400,1300";

/// **Delete a field group and its whole subtree, from the Forms panel.**
pub struct FieldGroupDeleteRemovesTheSubtree;

impl Check for FieldGroupDeleteRemovesTheSubtree {
    fn name(&self) -> &'static str {
        "field_group_delete_removes_the_subtree"
    }

    fn defect(&self) -> &'static str {
        "a form's grouping nodes are reachable from nowhere in the shell, so a document \
         organised as `Personal.Address.Zip` can have its fields deleted only one at a time — \
         or the Delete-group control is drawn and never asks the engine what it would remove, \
         so the operator confirms a destructive cascade they were never shown"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_groups(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

/// **A document that refuses a rename or a delete says so, and offers
/// nothing.**
pub struct StructuralRefusalsAreSentencesNotControls;

impl Check for StructuralRefusalsAreSentencesNotControls {
    fn name(&self) -> &'static str {
        "structural_refusals_are_sentences_not_controls"
    }

    fn defect(&self) -> &'static str {
        "on a certified document the Rename box and both Delete buttons are drawn live, so the \
         operator types a new name, presses Rename, and the engine's refusal reaches the trace \
         and nothing else — the pure queries that would have answered before the control was \
         offered are consulted by nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_refusals(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// The group-delete route
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_lines)]
fn drive_groups(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (exe, ui_rect) = preflight(ctx)?;
    let pdf = engine_fixture(NESTED).ok_or_else(|| {
        Error::new(format!(
            "the engine fixture `{NESTED}` is not on disk. It is the ONLY fixture in either \
             corpus with grouping nodes in it — core's own note on `AcroForm::groups` says the \
             field is empty for a flat form, which is every other file — so there is nothing to \
             fall back to and nothing this check could assert against."
        ))
    })?;

    let session = launch(ctx, &exe, &pdf, "form_groups.trace.txt", None)?;
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- A: the panel, and the section inside it ---------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    open_from_tab(&session, &driver, ui_rect, PANEL_TAB, PANEL_ITEM)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(before) = census(&trace) else {
        return Ok(Some(format!(
            "the Forms panel drew and traced no `{CENSUS}` line, so the Field-groups section \
             was never reached. That is the state this check exists for: the engine has had \
             `delete_field_group` throughout and the shell had no surface that could name a \
             grouping node. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if before.refused {
        return Ok(Some(String::from(
            "the Field-groups section reported the document as refusing structural change, on \
             `nested-form.pdf` — which carries no encryption and no signature. Either \
             `deletion_refusal` has started answering `Some` for an ordinary document, which \
             would silently withdraw the delete controls from every file, or this check is \
             looking at the wrong fixture.",
        )));
    }
    if before.nodes == 0 {
        return Ok(Some(format!(
            "the section traced `{CENSUS} nodes=0` on `{NESTED}`, whose fields are \
             `Personal.Name` and `Personal.Address.Zip` — so the form parses to at least two \
             grouping nodes. A zero here means the panel is reading `AcroForm::groups` and \
             getting nothing, which would render the whole section invisible on the only \
             documents it is for."
        )));
    }
    report.note(format!("{} grouping node(s) listed", before.nodes));

    // The header ships closed on purpose, so the rows are not laid out until it
    // is opened. A check that assumed it open would report the feature missing
    // on a correct build.
    let header = declared(&trace, ui_rect, REGION_HEADER).ok_or_else(|| {
        Error::new(format!(
            "the section traced its census and declared no `{REGION_HEADER}` region, so there \
             is no header to open. Regions beginning `forms.groups`: {}.",
            list(&declared_names(&trace, ui_rect, "forms.groups"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(header))?;
    session.settle(24);

    // --- B: the rows, and none of them armed yet ---------------------------
    let trace = session.trace()?;
    let rows: Vec<_> = trace.events(ROW).collect();
    if rows.is_empty() {
        return Ok(Some(format!(
            "the header was opened and no `{ROW}` line followed, so the section counted \
             {} node(s) and drew none of them.",
            before.nodes
        )));
    }
    if rows
        .iter()
        .any(|l| l.get("armed").is_some_and(|v| v == "1"))
    {
        return Ok(Some(String::from(
            "a row reported itself as armed before anything was pressed. A destructive \
             confirmation that is open on arrival is one the operator can press by accident, \
             and it means the epoch rule that retires an armed preview is not being applied.",
        )));
    }
    let arms = live_names(&trace, ui_rect, REGION_ARM);
    let Some(first) = arms.first() else {
        return Ok(Some(format!(
            "{} node(s) were traced and no `{REGION_ARM}*` region is on screen, so the rows \
             were computed and never laid out — or were laid out past the bottom of the panel, \
             which is where this panel's own register rows were found on 2026-08-19. Regions \
             declared: {}.",
            before.nodes,
            list(&declared_names(&trace, ui_rect, "forms.groups"))
        )));
    };
    report.note(format!("{} arm control(s) on screen", arms.len()));

    // --- C: the FIRST press, which must change nothing and disclose ---------
    let rect = declared(&trace, ui_rect, first)
        .ok_or_else(|| Error::new(format!("the `{first}` region went away between phases.")))?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(preview) = trace.events(PREVIEWED).last() else {
        return Ok(Some(format!(
            "pressing `{first}` traced no `{PREVIEWED}` line, so the control is drawn and \
             inert — the engine was never asked what the deletion would remove. What an \
             operator meets is a button that does nothing, on the one verb in this panel whose \
             consequences are entirely invisible."
        )));
    };
    let terminals: usize = preview
        .get("terminals")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    let nodes: usize = preview
        .get("nodes")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    report.note(format!(
        "the preview reports {terminals} field(s), {} box(es) and {nodes} grouping node(s)",
        preview.get("widgets").unwrap_or_default()
    ));
    if terminals == 0 {
        return Ok(Some(String::from(
            "the preview resolved and reported ZERO fields beneath the node. Core rules that a \
             node with no terminals under it is not a grouping node at all, so a zero here \
             means the preview and the listing are walking the form differently — and the \
             listing is what the operator was offered a destructive button from.",
        )));
    }
    if let Some(refusal) = trace.events(&format!("{PREVIEWED}-refused")).last() {
        return Ok(Some(format!(
            "★ the section offered the control and the PREVIEW refused: {}. The panel asks \
             `deletion_refusal` before drawing a single control, and the preview runs the same \
             two gates through `group_deletion_preflight` — so reaching this means the pure \
             query and the preflight have come apart.",
            refusal.get("detail").unwrap_or_default()
        )));
    }
    // The row must now say so, or the disclosure block is not attached to the
    // row that produced it.
    let armed_rows = session
        .trace()?
        .events(ROW)
        .filter(|l| l.get("armed").is_some_and(|v| v == "1"))
        .count();
    if armed_rows == 0 {
        return Ok(Some(format!(
            "the preview was taken and no `{ROW}` line reports `armed=1`, so the answer reached \
             the store and the panel is not reading it back. The operator presses Delete \
             group…, the engine computes exactly what would go, and the screen does not change."
        )));
    }

    // --- D: the SECOND press ------------------------------------------------
    let trace = session.trace()?;
    let confirm = declared(&trace, ui_rect, REGION_CONFIRM).ok_or_else(|| {
        Error::new(format!(
            "the preview was taken and no `{REGION_CONFIRM}` region was declared, so the \
             disclosure block drew without its commit control. Regions beginning \
             `forms.groups`: {}.",
            list(&declared_names(&trace, ui_rect, "forms.groups"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(confirm))?;
    session.settle(40);

    let trace = session.trace()?;
    if trace.events(REQUESTED).last().is_none() {
        return Ok(Some(format!(
            "clicking `{REGION_CONFIRM}` traced no `{REQUESTED}` line, so the commit control is \
             drawn and inert — the worst place in this whole route for that to be true, \
             because the operator has already read a list of what they were about to lose."
        )));
    }
    if let Some(refusal) = trace.events("delete-field-group-refused").last() {
        return Ok(Some(format!(
            "★★ the preview succeeded and the DELETION refused: {}. Core shares one \
             `group_deletion_preflight` between the two precisely so this cannot happen — its \
             own words are that a preview which succeeds where the act fails \"invites the \
             operator to confirm something that cannot happen\".",
            refusal.get("detail").unwrap_or_default()
        )));
    }
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "the confirm raised its action and the funnel never traced `{APPLIED}`, so it was \
             queued and never applied."
        )));
    };
    let removed: usize = applied
        .get("terminals")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    if removed != terminals {
        return Ok(Some(format!(
            "the preview promised {terminals} field(s) and the deletion removed {removed}. The \
             operator confirmed a number they were shown and a different number happened, on a \
             verb whose entire result is invisible on the page — which is the exact failure \
             the preview exists to prevent."
        )));
    }
    // ★ `widgets=` and `nodes=`, not `epoch=`: `delete-field-group-applied`
    // carries the three counts and no epoch — the epoch lives on the funnel's
    // own `delete-field-group` line, which this check deliberately does not
    // read. Asking for a key the line does not carry printed "at epoch " with
    // nothing after it, which reads as an epoch of zero.
    report.note(format!(
        "{removed} field(s) removed, with {} widget(s) and {} grouping node(s)",
        applied.get("widgets").unwrap_or_default(),
        applied.get("nodes").unwrap_or_default()
    ));

    // --- E: and the listing agrees -----------------------------------------
    let Some(after) = census(&trace) else {
        return Ok(Some(format!(
            "the panel stopped tracing `{CENSUS}` after the deletion."
        )));
    };
    if after.nodes >= before.nodes {
        return Ok(Some(format!(
            "{} grouping node(s) before the deletion and {} after it. The engine reported \
             success, so the nodes are gone from the document and the panel is not reading it \
             back — which an operator meets as pressing a destructive button and watching \
             nothing change, on a surface where nothing changing is also what success looks \
             like everywhere else on screen.",
            before.nodes, after.nodes
        )));
    }
    report.note(format!(
        "{} grouping node(s) remain, from {}",
        after.nodes, before.nodes
    ));
    // The armed block must be gone: the epoch moved, and a confirmation
    // describing a group that no longer exists is worse than none.
    if declared(&trace, ui_rect, REGION_CONFIRM).is_some() {
        return Ok(Some(format!(
            "the deletion succeeded and `{REGION_CONFIRM}` is still declared, so the armed \
             preview outlived the edit it described. The epoch rule that retires it is not \
             firing, and the block on screen now names a subtree that has already gone."
        )));
    }

    capture(ctx, &session, report, "form_groups.png");
    Ok(None)
}

// ---------------------------------------------------------------------------
// The refusals
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_lines)]
fn drive_refusals(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (exe, ui_rect) = preflight(ctx)?;
    let pdf = local_fixture(CERTIFIED).ok_or_else(|| {
        Error::new(format!(
            "this repository's fixture `{CERTIFIED}` is not on disk. Build it with \
             `python tools/gen-certified-nested-fixture.py` from the repository root. \
             Nothing already on disk substitutes, and the two obvious candidates each fail \
             one half: `forms/certified-p2-form.pdf` is certified at the right `/P` and \
             FLAT, so the Field-groups section early-returns and phase F's arm assertion \
             would be about a section that never drew; `forms/nested-form.pdf` has the tree \
             and no certification, so there would be nothing to withhold. A `/P 1` document \
             will not substitute either: it refuses everything, so the fill controls would \
             be withdrawn too and this check could not tell a correct build from one that \
             disables the whole panel — the engine's `PROVENANCE.md` records that as the \
             reason the `/P 2` fixture was authored in the first place."
        ))
    })?;

    let session = launch(
        ctx,
        &exe,
        &pdf,
        "form_refusals.trace.txt",
        Some("mode.edit"),
    )?;
    report.note(format!(
        "launched {} as pid {} on the /P 2 certified, two-level form",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- F: the Forms panel's own gate -------------------------------------
    open_from_tab(&session, &driver, ui_rect, PANEL_TAB, PANEL_ITEM)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(seen) = census(&trace) else {
        return Ok(Some(format!(
            "the Forms panel drew and traced no `{CENSUS}` line, so the Field-groups section \
             never asked whether this document permits structural change. Trace: {}.",
            session.trace_path().display()
        )));
    };
    // ★★ `refused` is asserted on EVERY fixture, nodes or no nodes, and that is
    // sound rather than convenient: `groups::section` calls
    // `deletion_refusal()` and traces the census **above** its
    // `form.groups.is_empty()` early return, so this line is the document's own
    // answer to the gate question on a file whose form is flat. A build that
    // stopped asking, or that asked the /P-aware FILL gate instead, reports
    // `refused=0` here and goes red.
    if !seen.refused {
        return Ok(Some(String::from(
            "the Field-groups section reported this /P 2 certified document as permitting \
             structural change. `deletion_refusal` takes the STRICT certification gate, not \
             the /P-aware fill gate, so it must answer `Some` here — the fill controls staying \
             live is correct on the same file and is exactly what makes this fixture able to \
             tell the two gates apart.",
        )));
    }
    // ★★★ THE PRECONDITION FOR THE ARM ASSERTION, ASSERTED RATHER THAN
    // ASSUMED — and it is a FAILURE now, not a note.
    //
    // `groups::section` returns before laying out a single control when
    // `AcroForm::groups` is empty, so on a FLAT certified form `arms` is empty
    // because there were no rows, not because the refusal withheld them. Every
    // previous version of this phase ran against such a form
    // (`forms/certified-p2-form.pdf`: `FullName`, `Subscribe`, no dots) and so
    // reported R9 upheld by a section that never drew — `crate::checks` rule 4
    // exactly: an absence is not evidence unless the thing that would have
    // produced the presence was working.
    //
    // `certified-nested-form.pdf` is the fixture that closes it, and the
    // engine's answer on it is not in doubt:
    // `crates/pdfcer-gui/src/app/actions/forms/delete.rs`'s
    // `the_certified_nested_fixture_is_both_certified_and_nested` asserts
    // `AcroForm::groups == ["Personal.Address", "Personal"]` from inside the
    // crate, every `cargo test` run. So `nodes=0` here cannot mean "this
    // document is flat"; it means the PANEL stopped listing an interior the
    // engine can see, or the fixture was regenerated wrong. Both are red.
    //
    // ⇒ Reported as a FAILURE rather than as a note, deliberately. A note would
    // put the phase back where it started — technically honest, and passing
    // while its one assertion never ran, which is the outcome a SKIP at least
    // makes legible and a silent pass does not.
    if seen.nodes == 0 {
        return Ok(Some(format!(
            "the Field-groups section traced `{CENSUS} nodes=0` on `{CERTIFIED}`, whose field \
             tree is two levels deep — `Personal` over `Personal.Address` over three \
             terminals. `AcroForm::groups` collects the name tree's INTERIOR, and a unit test \
             beside this fixture's users asserts it is exactly \
             `[\"Personal.Address\", \"Personal\"]`. So either the panel stopped listing nodes \
             the engine reports, or the fixture was rebuilt into something flat — regenerate \
             it with `python tools/gen-certified-nested-fixture.py`. Reported as a failure \
             rather than as a note because with `nodes=0` the arm assertion below has nothing \
             to withhold and would pass having tested nothing."
        )));
    }
    // ★★★ THE HEADER MUST BE OPENED BEFORE THE ABSENCE IS READ, and this is the
    // SECOND level of the same vacuity trap.
    //
    // The Field-groups header ships **closed** — phase A clicks it for exactly
    // this reason, and egui does not run a `CollapsingHeader`'s body while it
    // is closed. So a phase that read `live_names(REGION_ARM)` without opening
    // it would find nothing on **every** document, refused or not: the arm
    // controls are not withheld, they are simply not laid out yet.
    //
    // ⇒ Getting the fixture right was necessary and not sufficient. Both halves
    // have to hold for the absence to mean anything: the section must have rows
    // (`nodes != 0`, asserted above) **and** its body must have run.
    let header = declared(&trace, ui_rect, REGION_HEADER).ok_or_else(|| {
        Error::new(format!(
            "the section traced `{CENSUS} nodes={}` and declared no `{REGION_HEADER}` region, \
             so there is no header to open and the absence below could not be read. Regions \
             beginning `forms.groups`: {}. Reported as a SKIP: without the header this phase \
             cannot make its observation at all, which is a harness precondition rather than \
             the defect under test.",
            seen.nodes,
            list(&declared_names(&trace, ui_rect, "forms.groups"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(header))?;
    session.settle(24);
    let trace = session.trace()?;

    // ★★★ THE ASSERTION THIS PHASE WAS PASSING WITHOUT, now live on every run.
    //
    // R9: a permanently-refused capability renders NOTHING. Three things are
    // true at this point and all three are needed:
    //
    //   * the document refuses — `refused=1`, asserted above from the census;
    //   * the section has rows to draw controls beside — `nodes != 0`;
    //   * the header is open, so `groups::section`'s body has run.
    //
    // Any `forms.groups.arm.*` region declared now is therefore a Delete-group
    // button offered on a document that will refuse every press.
    //
    // ★★ And the CONTROL for this absence is phase B, in this same module:
    // there, on `nested-form.pdf` — the same field tree without the
    // certification — the identical gesture produces arm controls and the check
    // fails if it does not. So "no arm control here" is a difference caused by
    // the `/Perms` dictionary, not a claim about a gesture that never works.
    let arms = live_names(&trace, ui_rect, REGION_ARM);
    if !arms.is_empty() {
        return Ok(Some(format!(
            "the section knows the document refuses, has {} grouping node(s) listed, and drew \
             {} arm control(s) anyway: {}. R9: a permanently-refused capability renders nothing \
             and says why in prose — a greyed button implies a state the operator could argue \
             their way out of, which a certification signature is not.",
            seen.nodes,
            arms.len(),
            list(&arms)
        )));
    }
    report.note(format!(
        "the Field-groups section listed {} grouping node(s), refused, and offered no control",
        seen.nodes
    ));

    // --- G: select a field the canvas draws --------------------------------
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    let trace = session.trace()?;
    let Some(target) = first_target(&trace) else {
        return Err(Error::new(format!(
            "the canvas published no `{TARGETS}` line, so this check has no widget to click. \
             `certified-nested-form.pdf` carries three merged text field-widgets plus the \
             certification's own, all on page 1, so either the census stopped being written or \
             the page has not rasterized. Reported as a SKIP: a missing census is a harness \
             precondition, not the defect under test."
        )));
    };
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.0)?;
    // The census is canvas space and `doc_to_window` takes PDF space; the flip
    // is the mapping's own formula read backwards, exactly as `form_field`
    // does it.
    let doc_y = page.height_pt - target.2;
    let point = mapping.doc_to_window(DocPoint::new(target.0, target.1, doc_y))?;
    driver.click_at(session.frame()?.to_screen(point))?;
    session.settle(30);

    let trace = session.trace()?;
    if trace
        .events(SELECTED)
        .filter(|l| l.get("field").is_some())
        .last()
        .is_none()
    {
        return Err(Error::new(format!(
            "clicking the widget the canvas named selected nothing — no `{SELECTED}` line. \
             That is `form_field`'s subject rather than this one's, so it is a SKIP: without a \
             selection the Properties pane draws nothing at all and every assertion below \
             would be about an empty panel."
        )));
    }

    // --- H: the Properties pane, and the two absences ----------------------
    //
    // ★★★ BRING THE PROPERTIES PANEL BACK TO THE FRONT FIRST. See
    // `PROPERTIES_ITEM` for the whole reason: phase F put the **Forms** panel
    // at the front of the tabbed stack that also holds Properties, and a
    // tabbed stack draws only its active tab, so the section whose census this
    // phase reads had not run on any frame since. Without this the phase reads
    // the absence of a line from a panel that was never asked to draw — the
    // exact vacuity `crate::checks` rule 4 forbids, one surface along from the
    // two instances phases E and F already guard against.
    //
    // ★ It is done AFTER the selection rather than before, because phase F–G's
    // work is all on the Forms panel and swapping the tabs twice is one more
    // gesture than swapping them once. The selection survives it: showing a
    // panel touches the dock, never `doc.selected_field`.
    open_from_tab(&session, &driver, ui_rect, PROPERTIES_TAB, PROPERTIES_ITEM)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(gates) = trace.events(GATES).last() else {
        return Ok(Some(format!(
            "a field is selected, `{PROPERTIES_ITEM}` was clicked so the Properties panel is the \
             front tab of its stack, and the pane traced no `{GATES}` line — so neither \
             `rename_refusal` nor `deletion_refusal` was asked. That is the audit's finding \
             exactly: two pure queries the engine ships, documented with this call site in \
             their own doctest, consulted by nothing — so Rename and both Delete buttons are \
             offered on a document that refuses all three."
        )));
    };
    let renamed_refused = gates.get("rename_refused").is_some_and(|v| v == "1");
    let delete_refused = gates.get("delete_refused").is_some_and(|v| v == "1");
    report.note(format!(
        "the pane reports rename_refused={} delete_refused={}",
        u8::from(renamed_refused),
        u8::from(delete_refused)
    ));
    if !renamed_refused || !delete_refused {
        return Ok(Some(format!(
            "the gates were asked and answered rename_refused={} delete_refused={} on a /P 2 \
             certified document. Both take the STRICT certification gate, so both must refuse; \
             a `0` here means a control is being offered whose every press returns the same \
             error to the trace and nothing to the operator.",
            u8::from(renamed_refused),
            u8::from(delete_refused)
        )));
    }

    // ★★ The absences, admissible because the section itself is declared and
    // the gate census above was written this same frame. A build that failed to
    // draw the section at all would be caught by the first of these, not
    // reported as a correct refusal.
    if declared(&trace, ui_rect, REGION_SECTION).is_none() {
        return Ok(Some(format!(
            "the pane traced its gates and declared no `{REGION_SECTION}` region, so the \
             section computed its answers and drew nothing. The refusal must be a SENTENCE, \
             never a silence: an operator who selects a field and gets an empty pane has found \
             a program that looks broken rather than a document that is protected."
        )));
    }
    for control in [REGION_RENAME, REGION_DELETE] {
        if declared(&trace, ui_rect, control).is_some() {
            return Ok(Some(format!(
                "both gates refused and `{control}` is still on screen. This is the defect \
                 named in the check's own description: the control is live, the operator uses \
                 it, and the engine's refusal reaches the trace and nowhere else. R83 — know \
                 before you offer."
            )));
        }
    }
    report.note("neither structural control is drawn, and the section is");

    capture(ctx, &session, report, "form_refusals.png");
    Ok(None)
}

// ---------------------------------------------------------------------------
// Shared plumbing
// ---------------------------------------------------------------------------

/// The two things every phase below needs, or the reason there is nothing to
/// drive.
fn preflight(ctx: &CheckContext) -> Result<(std::path::PathBuf, &'static str)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). These checks click a mode segment, two ribbon \
             controls and up to three panel controls. Reported as SKIPPED rather than passed: \
             a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    Ok((exe, ui_rect))
}

/// Launch on a named fixture, with the shell's own diagnostics on.
///
/// `invoke` rings commands on startup, one per frame — used by the refusals
/// check to enter Edit mode without a segment click, because a mode segment
/// that misses is a failure about the ribbon rather than about forms.
fn launch(
    ctx: &CheckContext,
    exe: &std::path::Path,
    pdf: &std::path::Path,
    trace_name: &str,
    invoke: Option<&str>,
) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace_name));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // ★ Through the PROFILE rather than by name. The env var is a property of
    // the binary under test, not of this check, and the legacy profile's binary
    // does not have one — a hard-coded name would silently do nothing there
    // while reading as though the window had been sized. See `VIEWPORT`.
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    }
    if let Some(commands) = invoke {
        spec.env
            .push(("PDFCER_DIAG_INVOKE".to_owned(), commands.to_owned()));
    }
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    Session::launch(&spec, ctx.profile.trace_prefix)
}

/// Click a tab, then the item on it — reaching into the overflow if the window
/// is narrow enough to have put it there.
fn open_from_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    tab: &str,
    item: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let tab_region = declared(&trace, ui_rect, &format!("ribbon.tab.{tab}")).ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.{tab}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab_region))?;
    session.settle(14);
    let found = declared_or_in_overflow(session, driver, ui_rect, item)?.ok_or_else(|| {
        Error::new(format!(
            "no `{item}` region on the {tab} tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                &format!("ribbon.item.{tab}.")
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(found))?;
    session.settle(20);
    Ok(())
}

/// The Field-groups section's last census line.
struct Census {
    nodes: usize,
    refused: bool,
}

fn census(trace: &Trace) -> Option<Census> {
    let line = trace.events(CENSUS).last()?;
    Some(Census {
        nodes: line.get("nodes").and_then(|v| v.parse().ok())?,
        refused: line.get("refused").is_some_and(|v| v == "1"),
    })
}

/// The first selectable widget the canvas named, as `(page, canvas_x,
/// canvas_y)` at its centre.
///
/// ★ The application's numbers, not the fixture's. `canvas/forms.rs` publishes
/// one line per widget precisely so a harness can aim at where the program says
/// the box is; a check that computed the rect from the PDF would be asserting
/// that two independent derivations agree and would report a disagreement as a
/// hit-test failure.
fn first_target(trace: &Trace) -> Option<(usize, f64, f64)> {
    trace.events(TARGETS).find_map(|l| {
        let page: usize = l.get("page")?.parse().ok()?;
        let raw = l.get("rect")?;
        let (min, size) = raw.split_once(")+(")?;
        let (x, y) = min.trim_start_matches('(').split_once(',')?;
        let (w, h) = size.trim_end_matches(')').split_once(',')?;
        let x: f64 = x.trim().parse().ok()?;
        let y: f64 = y.trim().parse().ok()?;
        let w: f64 = w.trim().parse().ok()?;
        let h: f64 = h.trim().parse().ok()?;
        Some((page, x + w / 2.0, y + h / 2.0))
    })
}

/// A picture, or a note saying why there is not one. Never a failure: the trace
/// assertions above are the oracle, and a capture that could not be taken says
/// nothing about the feature.
fn capture(ctx: &CheckContext, session: &Session, report: &mut CheckReport, name: &str) {
    let shot = ctx.out(name);
    match crate::capture::window_to_png(session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!(
                "the window could not be captured ({e}); the trace assertions above still hold"
            ));
        }
    }
}

/// A fixture from the engine's own corpus, which this repository builds against
/// by path.
fn engine_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(rel);
    path.is_file().then_some(path)
}

/// A fixture from **this** repository's `fixtures/`.
///
/// ★ Resolved from `CARGO_MANIFEST_DIR` — `tools/ui-verify` — rather than from
/// the process's working directory, which is whatever the operator happened to
/// be in. `reflow`, `text_edit`, `annot_delete_gate` and `field_delete_gate` all
/// do the same, and two of those record having got the *depth* wrong first:
/// `../../` reaches the repository root, `../../../` reaches `D:/Dev`, and the
/// second SKIPs on every run with a message telling the reader to run a
/// generator that writes to the first.
fn local_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    path.is_file().then_some(path)
}
