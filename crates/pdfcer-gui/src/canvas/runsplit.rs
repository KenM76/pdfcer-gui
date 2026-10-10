//! The operand of `format.split_text_lines`: one selected text object, on the
//! page or inside a placed drawing, whose runs the engine would cut into two
//! or more lines, and whether the engine would refuse the cut.
//!
//! The cuts come from `pdfcer_core::vector::text_object_split_points` with
//! `SplitGranularity::Line`, the function `EditSession::text_object_split_plan`
//! calls, so the menu offers exactly the objects the press would cut. The
//! refusal is `EditSession::text_object_split_refusal`, asked once per
//! selection and edit epoch by [`refresh`] on the frame-level `&mut` and read
//! back from `OpenDoc::split_preflight`. Until that answer exists (the render
//! worker holds the session) only the refusal the object model can see, an
//! inherited-position run at a cut, greys the row; the press still words any
//! other refusal on the status line. The engine's preflight takes a page
//! object index, so a form leaf is greyed only by what the model can see.

use pdfcer_core::vector::{
    RunPositioning, SplitGranularity, VectorEditError, VectorObject, text_object_split_points,
};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::runmerge::{MenuRow, park_at, parked_at};
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::panels::objects::provider::{ObjectModelProvider, TargetId};
use crate::text::runsplit::RunSplitRefusal;
use pdfcer_gui_base::opendoc::splitpreflight::SplitPreflight;

/// One text object and where the engine would cut it.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitOperand {
    /// The text object: a page object or a form leaf.
    pub target: TargetId,
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
/// is exactly one text object the engine would cut at least once.
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
    let target = only.object;
    let VectorObject::Text(text) = targets.object_for(target)? else {
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
        target,
        cuts,
        refusal,
    })
}

/// [`operand`] for the document's selection on its current page, with the
/// engine's stored preflight answer when it is about this object, these cuts
/// and this edit epoch.
#[must_use]
pub fn operand_of(doc: &OpenDoc) -> Option<SplitOperand> {
    let page = doc.view.page_index;
    let mut split = {
        let targets = doc.page_objects();
        operand(targets.as_deref(), &doc.selection, page)?
    };
    if let (None, Some(object)) = (&split.refusal, split.target.page_object_index()) {
        let answer = doc
            .split_preflight
            .as_ref()
            .and_then(|p| p.answer((page, object), &split.cuts, doc.edit_epoch));
        split.refusal = answer.flatten().cloned();
    }
    Some(split)
}

/// The engine's stored refusal for the one object `selection` holds, when the
/// stored answer is about that object at this edit epoch. For the canvas menu,
/// which has the parked row but no object model on later frames, and whose
/// selection is lent out of `doc` while it draws.
#[must_use]
pub fn engine_refusal<'a>(
    doc: &'a OpenDoc,
    selection: &SelectionState,
) -> Option<&'a VectorEditError> {
    let page = doc.view.page_index;
    let [only] = selection.entries() else {
        return None;
    };
    let object = only.object.page_object_index()?;
    doc.split_preflight
        .as_ref()
        .filter(|p| p.page == page && p.object == object && p.edit_epoch == doc.edit_epoch)?
        .refusal
        .as_ref()
}

/// Ask the engine whether the selection's split would be refused, unless the
/// stored answer is already about it. Silent while the render worker shares
/// the session; the next frame asks again.
pub fn refresh(doc: &mut OpenDoc) {
    let Some(split) = operand_of(doc) else {
        return;
    };
    let Some(object) = split.target.page_object_index() else {
        return;
    };
    let page = doc.view.page_index;
    let epoch = doc.edit_epoch;
    if doc
        .split_preflight
        .as_ref()
        .is_some_and(|p| p.answer((page, object), &split.cuts, epoch).is_some())
    {
        return;
    }
    let Some(session) = std::sync::Arc::get_mut(&mut doc.session) else {
        return;
    };
    // An `Err` is a document- or page-level refusal the press words itself;
    // it greys nothing here.
    let answer = session.text_object_split_refusal(page, object, &split.cuts);
    let token = match &answer {
        Ok(refusal) => refusal.as_ref().map_or("none", refusal_token),
        Err(_) => "error",
    };
    let refusal = answer.ok().flatten();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            "split-preflight page={page} object={object} cuts={} refusal={}",
            split.cuts.len(),
            token,
        )
    });
    doc.split_preflight = Some(SplitPreflight {
        page,
        object,
        edit_epoch: epoch,
        cuts: split.cuts,
        refusal,
    });
}

/// A one-word trace token for a refusal.
fn refusal_token(e: &VectorEditError) -> &'static str {
    match e {
        VectorEditError::SplitRunInheritsPosition { .. } => "inherits-position",
        VectorEditError::SplitAtLineShowOperator { .. } => "line-show-operator",
        VectorEditError::SplitInsideMarkedContent { .. } => "marked-content",
        VectorEditError::EmptySplit => "empty",
        _ => "other",
    }
}

/// The press: re-derive the operand from the selection, giving the action to
/// queue or why not. A parked menu row is never trusted as the operand.
///
/// # Errors
///
/// The refusal the status line words.
pub fn press(doc: &OpenDoc) -> Result<Action, RunSplitRefusal> {
    let page = doc.view.page_index;
    let split = operand_of(doc);
    match split {
        Some(SplitOperand {
            target,
            refusal: None,
            ..
        }) => Ok(Action::Vector(
            crate::app::actions::VectorAction::SplitTextLines { page, target },
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
