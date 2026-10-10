//! The operand of `format.merge_text_runs`: the runs the selection covers in
//! one text object, and the engine's preflight answer for them.
//!
//! At the Part rung the unit is a line, so the runs are the union of the
//! selected lines' runs. At the Object rung a single selected text object that
//! is one line offers all of its runs. The text object may be a page object or
//! a leaf inside a placed drawing; the engine has a merge for each.

use pdfcer_core::vector::{VectorEditError, VectorObject};

use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::panels::objects::provider::{ObjectModelProvider, TargetId};

/// Two or more runs of one text object, and whether the engine would merge them.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeOperand {
    /// The text object: a page object or a form leaf.
    pub target: TargetId,
    /// Ascending, deduplicated run indices.
    pub runs: Vec<usize>,
    /// `text_merge_refusal`'s answer; `None` means the merge may proceed.
    pub refusal: Option<VectorEditError>,
}

impl MergeOperand {
    /// Whether the command's row is drawn pressable.
    #[must_use]
    pub fn allowed(&self) -> bool {
        self.refusal.is_none()
    }
}

/// The merge operand of `selection` on `page`, or `None` when the selection is
/// not two or more runs of one text object.
#[must_use]
pub fn operand(
    targets: Option<&ObjectModelProvider>,
    selection: &SelectionState,
    page: usize,
) -> Option<MergeOperand> {
    let targets = targets?;
    if targets.page_index() != page {
        return None;
    }
    let (target, mut runs) = match selection.level() {
        SelectionLevel::Part => {
            let entered = selection.entered_object()?;
            if entered.page != page {
                return None;
            }
            let target = entered.object;
            let mut runs = Vec::new();
            for line in selection.selected_parts_on(page, target) {
                runs.extend(targets.text_line_runs_of(target, line)?);
            }
            (target, runs)
        }
        SelectionLevel::Object => {
            let [only] = selection.entries() else {
                return None;
            };
            if only.page != page || only.subpath.is_some() {
                return None;
            }
            let target = only.object;
            if targets.text_line_count_of(target) != 1 {
                return None;
            }
            (target, targets.text_line_runs_of(target, 0)?.collect())
        }
        _ => return None,
    };
    runs.sort_unstable();
    runs.dedup();
    if runs.len() < 2 {
        return None;
    }
    let VectorObject::Text(text) = targets.object_for(target)? else {
        return None;
    };
    let refusal = pdfcer_core::vector::text_merge_refusal(text, &runs);
    Some(MergeOperand {
        target,
        runs,
        refusal,
    })
}

/// The merge row's two answers — offered, and pressable — as the right-click
/// that opened the menu found them.
///
/// Parked because the page's object model is built only on the click frame
/// (`canvas::modelneed`): on every later frame the menu is open, `operand`
/// has no targets and would answer `None`, deleting the row one frame after
/// it was drawn. The press re-derives the operand, so a parked answer can
/// never merge anything the preflight has not just approved.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuRow {
    /// `shell::menus::TEXT_MERGE_OFFERED`.
    pub offered: bool,
    /// `shell::menus::TEXT_MERGE_ALLOWED`.
    pub allowed: bool,
}

impl MenuRow {
    /// The row `merge` implies.
    #[must_use]
    pub fn of(merge: Option<&MergeOperand>) -> Self {
        Self {
            offered: merge.is_some(),
            allowed: merge.is_some_and(MergeOperand::allowed),
        }
    }
}

const ROW_MEMORY_KEY: &str = "pdfcer-gui.canvas.runmerge.row";

/// Park the row the right-click found.
pub fn park(ctx: &egui::Context, row: MenuRow) {
    park_at(ctx, ROW_MEMORY_KEY, row);
}

/// The parked row; neither offered nor allowed before any right-click.
#[must_use]
pub fn parked(ctx: &egui::Context) -> MenuRow {
    parked_at(ctx, ROW_MEMORY_KEY)
}

/// Park `row` under `key`, for a menu row parked the same way as this one.
pub fn park_at(ctx: &egui::Context, key: &str, row: MenuRow) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(key), row));
}

/// The row parked under `key`; neither offered nor allowed before any.
#[must_use]
pub fn parked_at(ctx: &egui::Context, key: &str) -> MenuRow {
    ctx.data_mut(|d| {
        d.get_temp::<MenuRow>(egui::Id::new(key))
            .unwrap_or_default()
    })
}
