//! # `panels::preflight::results` — findings grouped by rule, and the jump
//!
//! Rows are drawn in profile rule order, each rule with its count; a row is
//! labelled by its kind so a violation, a finding a person must judge and a
//! check that could not finish never read alike, and none reads as a pass.
//! Nothing here marks the page (R8b): a pick navigates and, for a page
//! object, makes the canvas's ordinary selection.

use pdfcer_core::preflight::{
    Finding, FindingKind, FindingTarget, PreflightReport, Profile, rule_matches,
};
use pdfcer_gui_base::objectprovider::TargetId;
use pdfcer_gui_base::subactions::SelectionAction;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::panels::preflight as t;

/// Rows drawn per rule; the rest are counted, and the copied report has them.
const ROWS_PER_RULE: usize = 200;

/// Why the view moves, for the reveal's trace.
const WHY: &str = "preflight-finding"; // ui-text-exempt: diagnostic token, never displayed

/// One rule's findings, as indices into the report.
pub(super) struct Group {
    /// The rule's id.
    pub id: String,
    /// The rule's severity, as the engine spells it.
    pub severity: &'static str,
    /// Indices into `PreflightReport::findings`, in report order.
    pub findings: Vec<usize>,
    /// The rule has a fix the panel can offer.
    pub fixable: bool,
}

/// Group `report`'s findings by `profile`'s rules, leaving out rules with
/// none. A finding no rule claims gets a group of its own, named by its rule
/// id, rather than disappearing.
pub(super) fn groups(profile: &Profile, report: &PreflightReport) -> Vec<Group> {
    let mut claimed = vec![false; report.findings.len()];
    let mut out = Vec::new();
    for rule in &profile.rules {
        let findings: Vec<usize> = report
            .findings
            .iter()
            .enumerate()
            .filter(|(i, f)| !claimed[*i] && rule_matches(&f.rule_id, &rule.id))
            .map(|(i, _)| i)
            .collect();
        if findings.is_empty() {
            continue;
        }
        for &i in &findings {
            claimed[i] = true;
        }
        out.push(Group {
            id: rule.id.clone(),
            severity: rule.severity.as_str(),
            fixable: !pdfcer_gui_base::preflightfix::rule_fixups(rule).is_empty(),
            findings,
        });
    }
    for (i, f) in report.findings.iter().enumerate() {
        if !claimed[i] {
            out.push(Group {
                id: f.rule_id.clone(),
                severity: f.severity.as_str(),
                findings: vec![i],
                fixable: false,
            });
        }
    }
    out
}

/// The kind label of a finding.
pub(super) fn kind_label(kind: FindingKind) -> &'static str {
    match kind {
        FindingKind::Violation => t::kind_violation(),
        FindingKind::NeedsPerson => t::kind_needs_person(),
        FindingKind::Incomplete => t::kind_incomplete(),
    }
}

/// A target's kind, for traces.
pub(super) const fn target_token(target: FindingTarget) -> &'static str {
    // ui-text-exempt: trace tokens, never displayed
    match target {
        FindingTarget::Document => "document",
        FindingTarget::Page => "page",
        FindingTarget::PageObject(_) => "object",
        FindingTarget::FormLeaf(_) => "leaf",
        FindingTarget::Annotation(_) => "annotation",
        FindingTarget::Font(_) => "font",
        FindingTarget::ColourSpace(_) => "colour-space",
        FindingTarget::Image(_) => "image",
        FindingTarget::GraphicsState(_) => "graphics-state",
        FindingTarget::StructElem(_) => "struct-elem",
        _ => "other",
    }
}

/// One row's text.
fn row_text(f: &Finding) -> String {
    let at = f
        .page
        .map_or_else(|| t::at_document().to_owned(), t::at_page);
    let measured = f
        .measured
        .map(|m| t::measured(m.value, m.limit, m.unit.as_str()));
    t::row(kind_label(f.kind), &at, &f.message, measured.as_deref())
}

/// Draw one group's rows. `row` is the running row number across groups,
/// which names each row's region for the driven checks.
pub(super) fn draw_rows(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    report: &PreflightReport,
    group: &Group,
    row: &mut usize,
    actions: &mut Vec<Action>,
) {
    for &i in group.findings.iter().take(ROWS_PER_RULE) {
        let f = &report.findings[i];
        let response = ui
            .add(egui::Button::selectable(false, row_text(f)).wrap())
            .on_hover_text(t::finding_hover());
        // ui-text-exempt: diagnostic region name, never displayed
        crate::diag::ui_rect(&format!("preflight.row.{row}"), response.rect);
        *row += 1;
        if response.clicked() {
            pick(doc, f, actions);
        }
    }
    if group.findings.len() > ROWS_PER_RULE {
        let more = t::more_rows(group.findings.len() - ROWS_PER_RULE);
        ui.label(egui::RichText::new(more).weak());
    }
}

/// One line listing every drawn row as `row:rule@page:target`.
pub(super) fn trace_rows(report: &PreflightReport, groups: &[Group]) {
    crate::diag::trace(|| {
        let mut list = Vec::new();
        for g in groups {
            for &i in g.findings.iter().take(ROWS_PER_RULE) {
                let f = &report.findings[i];
                let page = f.page.map_or_else(|| "-".to_owned(), |p| p.to_string());
                list.push(format!(
                    "{}:{}@{page}:{}",
                    list.len(),
                    f.rule_id,
                    target_token(f.target)
                ));
            }
        }
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("preflight-rows n={} list={}", list.len(), list.join(","))
    });
}

/// Go to a finding: its page, its object selected when it names one, and its
/// rect revealed when it has one.
fn pick(doc: &OpenDoc, f: &Finding, actions: &mut Vec<Action>) {
    let page = f
        .page
        .map(|p| p.saturating_sub(1))
        .filter(|p| *p < doc.pages.len());
    let selected = match (page, f.target) {
        (Some(_), FindingTarget::PageObject(i)) => Some(i),
        _ => None,
    };
    crate::diag::trace(|| {
        let page = page.map_or_else(|| "-".to_owned(), |p| (p + 1).to_string());
        let selected = selected.map_or_else(|| "-".to_owned(), |i| i.to_string());
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "preflight-picked rule={} page={page} target={} selected={selected}",
            f.rule_id,
            target_token(f.target)
        )
    });
    let Some(page) = page else {
        return;
    };
    if doc.view.page_index != page {
        actions.push(Action::GoToPage(page));
    }
    if let Some(i) = selected {
        actions.push(Action::Selection(SelectionAction::SelectObject {
            page,
            object: Some(TargetId::Object(i as u64)),
        }));
    }
    reveal(doc, page, f, actions);
}

/// Scroll the finding's rect into view, when it has one.
fn reveal(doc: &OpenDoc, page: usize, f: &Finding, actions: &mut Vec<Action>) {
    let (Some(r), Some(info)) = (f.rect, doc.pages.get(page)) else {
        return;
    };
    if let Some(canvas) =
        crate::canvas::geometry::pdf_rect_to_canvas((r.llx, r.lly, r.urx, r.ury), info)
    {
        let (min, max) = crate::canvas::minreveal::fracs_for_canvas_rect(canvas, info);
        actions.push(Action::RevealRect {
            page,
            min,
            max,
            why: WHY,
        });
    }
}
