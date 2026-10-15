//! The selection lifted into an `ObjectClip` without touching any clipboard:
//! what Copy stores and Duplicate pastes back at the offset.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/take.md`.

use std::sync::Arc;

use pdfcer_core::object::ObjId;
use pdfcer_core::vector::ObjectClip;

use super::annotclip::{self, Plan};
use super::clipboard::{Clipped, Refusal, anchor_of};
use crate::app::state::OpenDoc;

/// Which engine verb lifted the clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `copy_selection`: page objects and annotations.
    Selection,
    /// `copy_objects_in_form`: parts of one placed drawing.
    FormLeaves,
}

impl Kind {
    /// The trace's `kind=` value.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Selection => "selection",
            Self::FormLeaves => "form-leaves",
        }
    }
}

/// A lifted selection and what the engine left out of it.
#[derive(Debug)]
pub struct Taken {
    pub kind: Kind,
    pub clip: ObjectClip,
    /// The 0-based page it was lifted from.
    pub page: usize,
    /// Page objects, or drawing parts, that were selected.
    pub objects: usize,
    /// The engine's carrier decision for each annotation.
    pub plan: Plan,
    /// The annotations lifted, in `/Annots` order: the cut deletes these.
    pub annot_ids: Vec<ObjId>,
    /// Drawing parts selected beside page objects, which `copy_selection`
    /// cannot take.
    pub in_form_left: usize,
}

impl Taken {
    /// The clipboard's record of this clip.
    #[must_use]
    pub fn clipped(&self) -> Clipped {
        Clipped::Selection {
            count: self.clip.len(),
            annotations: self.plan.carried(),
            annot_ids: self.annot_ids.clone(),
            left_behind: self.plan.refused.clone(),
            thin: self.plan.thin,
            in_form_left: self.in_form_left,
            bytes: self.clip.to_bytes(),
            page: self.page,
            anchor: anchor_of(&self.clip),
        }
    }
}

/// Lift what is selected: parts of a placed drawing when nothing else is
/// selected, else page objects and annotations together.
pub fn selected(doc: &mut OpenDoc) -> Result<Taken, Refusal> {
    match super::formclip::leaves_only(doc) {
        Some((page, leaves)) => leaves_of(doc, page, &leaves),
        None => selection(doc),
    }
}

/// Lift the page objects and annotations selected, on the annotations' own
/// page where there are any.
///
/// The annotations' page wins over `view.page_index`: the view can be
/// scrolled onto another sheet with a comment still selected, and
/// `copy_selection` indexes into the page it is given.
pub fn selection(doc: &OpenDoc) -> Result<Taken, Refusal> {
    let annots = annotclip::selected(doc)?;
    let page = annots.first().map_or(doc.view.page_index, |a| a.page);
    let objects = doc.selection.object_indices_on(page);
    let in_form = doc.selection.leaf_indices_on(page).len();
    if objects.is_empty() && annots.is_empty() {
        if in_form > 0 {
            refused(&format!("inside-form n={in_form}")); // ui-text-exempt: a trace reason
            return Err(Refusal::InsideForm(in_form));
        }
        return Err(Refusal::NothingSelected);
    }
    let indices: Vec<usize> = annots.iter().map(|a| a.index).collect();
    // `copy_selection` commits nothing, which is what makes a cut one undo
    // entry: only its delete half is an edit.
    let clip = doc
        .session
        .copy_selection(page, &objects, &indices)
        .map_err(|_| Refusal::EngineRefused)?;
    let plan = Plan::of(&clip);
    if plan.nothing_to_carry(objects.len()) {
        refused(&format!("cannot-carry what={:?}", plan.refused)); // ui-text-exempt: a trace reason
        return Err(Refusal::CannotCarry(plan.refused));
    }
    Ok(Taken {
        kind: Kind::Selection,
        clip,
        page,
        objects: objects.len(),
        plan,
        annot_ids: annots.iter().map(|a| a.id).collect(),
        in_form_left: in_form,
    })
}

/// Lift `leaves` of the drawing on `page`. The engine bakes the drawing's
/// placement into each item, so the clip pastes as page content where and as
/// large as it was drawn.
pub fn leaves_of(doc: &mut OpenDoc, page: usize, leaves: &[usize]) -> Result<Taken, Refusal> {
    // `&mut` only because the engine reads its cached page model; a holder of
    // the session elsewhere refuses rather than waiting.
    let Some(session) = Arc::get_mut(&mut doc.session) else {
        refused("session-borrowed");
        return Err(Refusal::EngineRefused);
    };
    let clip = session
        .copy_objects_in_form(page, leaves)
        .map_err(|error| {
            refused(&format!("engine detail={error}")); // ui-text-exempt: a trace reason
            Refusal::EngineRefused
        })?;
    Ok(Taken {
        kind: Kind::FormLeaves,
        clip,
        page,
        objects: leaves.len(),
        plan: Plan::default(),
        annot_ids: Vec::new(),
        in_form_left: 0,
    })
}

fn refused(why: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("selection-take-refused reason={why}")
    });
}
