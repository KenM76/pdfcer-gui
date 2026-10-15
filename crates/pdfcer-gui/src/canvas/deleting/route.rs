//! Which channel Delete takes: one decision for the Delete key and the
//! ribbon's `format.delete`, so Delete-the-key and Delete-the-command cannot
//! act on different things.
//!
//! Order: a selected form field, then a selected annotation, then page
//! content. A form selection and an annotation selection exclude each other by
//! construction, so the order states that rather than breaking a tie.
//!
//! **One predicate per capability.** A field is gated on `edit_content`, where
//! `canvas::forms` offers its selection; an annotation on `author_markup`,
//! because Review (`edit_content: false`) is the mode for markup and must keep
//! Delete for it; page content on `edit_content`. The content gate is not
//! redundant: Read can hold a content selection (a picture clicked to copy it),
//! and `format.select_form` can widen that to a whole form XObject.
//!
//! The field and annotation refusals are the Properties panel's own
//! (`formfield::refuses_delete`, `annotdelete::refuses`), which also withhold
//! the menu item. A refused press declines to the trace only: the sentence is
//! already on screen in the Properties panel from the moment of selection.
//! A field Delete is `DeleteWidget`: this box, not every box the field owns.
//!
//! Each caller words its own trace. Page content is resolved by
//! [`super::subject`], which the caller asks with the page model it holds.

use pdfcer_core::object::ObjId;
use pdfcer_gui_base::fieldset::FieldSet;

use crate::app::actions::Action;
use crate::app::state::SelectedField;
use crate::canvas::selection::SelectionState;

/// What the decision reads.
pub(crate) struct Ask<'a> {
    pub edit_content: bool,
    pub author_markup: bool,
    pub selected_field: Option<&'a SelectedField>,
    pub field_set: &'a FieldSet,
    pub field_refused: bool,
    pub annot_refused: bool,
    pub selection: &'a SelectionState,
}

/// Where Delete goes.
pub(crate) enum Route<'a> {
    /// The selected fields; `None` when none of them can be deleted.
    Fields(Option<Action>),
    FieldRefused(&'a SelectedField),
    /// The selected annotations; `None` when none of them can be deleted.
    Annots(Option<Action>),
    AnnotRefused(ObjId),
    /// Nothing claimed above, and the mode cannot edit page content.
    ModeCannotEditContent,
    /// Page content, for [`super::subject`].
    Content,
}

/// Decide where Delete goes.
pub(crate) fn route<'a>(ask: &Ask<'a>) -> Route<'a> {
    if ask.edit_content
        && let Some(field) = ask.selected_field
    {
        if ask.field_refused {
            return Route::FieldRefused(field);
        }
        return Route::Fields(
            crate::app::actions::forms::FieldAction::delete_selected(
                ask.field_set.all(Some(field)),
            )
            .map(Action::Field),
        );
    }
    if ask.author_markup
        && let Some(annot) = ask.selection.annot()
    {
        if ask.annot_refused {
            return Route::AnnotRefused(annot.target.id);
        }
        return Route::Annots(
            pdfcer_gui_base::annotaction::delete_selected(&ask.selection.annots())
                .map(Action::Annot),
        );
    }
    if ask.edit_content {
        Route::Content
    } else {
        Route::ModeCannotEditContent
    }
}

#[cfg(test)]
mod tests;
