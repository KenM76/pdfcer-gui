//! The operand of `format.split_text_lines`: one selected page text object
//! whose runs the engine would cut into two or more lines, and the part of the
//! engine's preflight the object model can answer.
//!
//! The cuts come from `pdfcer_core::vector::text_object_split_points` with
//! `SplitGranularity::Line`, the function `EditSession::text_object_split_plan`
//! calls, so the menu offers exactly the objects the press would cut. The
//! preflight is partial: `text_split_refusal` also needs the page's
//! `ContentStream` to see a `'`/`"` show operator or an open marked-content
//! section, and the provider holds none. Those two refusals arrive on the press,
//! on the status line. The engine request for a session-level preflight is
//! G114 (`request_G114_a_text_split_cannot_be_preflighted_from_a_session.md`).

use pdfcer_core::vector::{
    RunPositioning, SplitGranularity, VectorEditError, VectorObject, text_object_split_points,
};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::runmerge::{MenuRow, park_at, parked_at};
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::panels::objects::provider::ObjectModelProvider;
use crate::text::runsplit::RunSplitRefusal;

/// One page text object and where the engine would cut it.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitOperand {
    /// The page object index.
    pub object: usize,
    /// The run indices a new object would start at; never empty.
    pub cuts: Vec<usize>,
    /// The first cut the object model already knows the engine refuses.
    pub refusal: Option<VectorEditError>,
}

impl SplitOperand {
    /// Whether the command's row is drawn pressable.
    #[must_use]
    pub fn allowed(&self) -> bool {
        self.refusal.is_none()
    }
}

/// The split operand of `selection` on `page`, or `None` unless the selection
/// is exactly one page text object the engine would cut at least once.
#[must_use]
pub fn operand(
    targets: Option<&ObjectModelProvider>,
    selection: &SelectionState,
    page: usize,
) -> Option<SplitOperand> {
    let targets = targets?;
    if targets.page_index() != page || !matches!(selection.level(), SelectionLevel::Object) {
        return None;
    }
    let [only] = selection.entries() else {
        return None;
    };
    if only.page != page || only.subpath.is_some() {
        return None;
    }
    let object = only.object.page_object_index()?;
    let VectorObject::Text(text) = targets.page_objects().objects.get(object)? else {
        return None;
    };
    let cuts = text_object_split_points(text, SplitGranularity::Line);
    if cuts.is_empty() {
        return None;
    }
    let refusal = cuts.iter().find_map(|&index| {
        let run = text.runs.get(index)?;
        (run.positioned_by == RunPositioning::Inherited)
            .then_some(VectorEditError::SplitRunInheritsPosition { index })
    });
    Some(SplitOperand {
        object,
        cuts,
        refusal,
    })
}

/// The press: re-derive the operand from the selection, giving the action to
/// queue or why not. A parked menu row is never trusted as the operand.
///
/// # Errors
///
/// The refusal the status line words.
pub fn press(doc: &OpenDoc) -> Result<Action, RunSplitRefusal> {
    let page = doc.view.page_index;
    let split = {
        let targets = doc.page_objects();
        operand(targets.as_deref(), &doc.selection, page)
    };
    match split {
        Some(SplitOperand {
            object,
            refusal: None,
            ..
        }) => Ok(Action::Vector(
            crate::app::actions::VectorAction::SplitTextLines { page, object },
        )),
        Some(SplitOperand {
            refusal: Some(why), ..
        }) => Err(RunSplitRefusal::of_vector(&why)),
        None => Err(RunSplitRefusal::NotOneTextObject),
    }
}

const ROW_MEMORY_KEY: &str = "pdfcer-gui.canvas.runsplit.row";

/// Park the row the right-click found, for the reason
/// [`crate::canvas::runmerge::MenuRow`] gives.
pub fn park(ctx: &egui::Context, split: Option<&SplitOperand>) {
    let row = MenuRow {
        offered: split.is_some(),
        allowed: split.is_some_and(SplitOperand::allowed),
    };
    park_at(ctx, ROW_MEMORY_KEY, row);
}

/// The parked row; neither offered nor allowed before any right-click.
#[must_use]
pub fn parked(ctx: &egui::Context) -> MenuRow {
    parked_at(ctx, ROW_MEMORY_KEY)
}
