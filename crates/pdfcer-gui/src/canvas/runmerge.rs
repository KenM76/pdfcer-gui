//! The operand of `format.merge_text_runs`: the runs the selection covers in
//! one page text object, and the engine's preflight answer for them.
//!
//! At the Part rung the unit is a line, so the runs are the union of the
//! selected lines' runs. At the Object rung a single selected text object that
//! is one line offers all of its runs. Form leaves are not offered: the engine
//! merges page objects only.

use pdfcer_core::vector::{VectorEditError, VectorObject};

use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::panels::objects::provider::ObjectModelProvider;

/// Two or more runs of one page text object, and whether the engine would merge them.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeOperand {
    /// The page object index.
    pub object: usize,
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
/// not two or more runs of one page text object.
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
    let (object, mut runs) = match selection.level() {
        SelectionLevel::Part => {
            let entered = selection.entered_object()?;
            if entered.page != page {
                return None;
            }
            let object = entered.object.page_object_index()?;
            let mut runs = Vec::new();
            for line in selection.selected_parts_on(page, entered.object) {
                runs.extend(targets.text_line_runs(object, line)?);
            }
            (object, runs)
        }
        SelectionLevel::Object => {
            let [only] = selection.entries() else {
                return None;
            };
            if only.page != page || only.subpath.is_some() {
                return None;
            }
            let object = only.object.page_object_index()?;
            if targets.text_line_count(object) != 1 {
                return None;
            }
            (object, targets.text_line_runs(object, 0)?.collect())
        }
        _ => return None,
    };
    runs.sort_unstable();
    runs.dedup();
    if runs.len() < 2 {
        return None;
    }
    let VectorObject::Text(text) = targets.page_objects().objects.get(object)? else {
        return None;
    };
    let refusal = pdfcer_core::vector::text_merge_refusal(text, &runs);
    Some(MergeOperand {
        object,
        runs,
        refusal,
    })
}
