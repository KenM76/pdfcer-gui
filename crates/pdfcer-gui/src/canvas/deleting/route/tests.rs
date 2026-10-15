//! The Delete route's precedence and its capability gates.

use pdfcer_core::object::ObjId;
use pdfcer_gui_base::fieldset::FieldSet;

use super::{Ask, Route, route};
use crate::app::state::SelectedField;
use crate::canvas::selection::SelectionState;
use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

const EDIT: (bool, bool) = (true, true);
const REVIEW: (bool, bool) = (false, true);
const READ: (bool, bool) = (false, false);

fn field() -> SelectedField {
    SelectedField {
        field: "Name".to_owned(),
        widget: 0,
        page: 0,
    }
}

fn with_annot(locked: bool) -> SelectionState {
    let mut selection = SelectionState::default();
    selection.select_annot(AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id: ObjId::new(7, 0),
            kind: AnnotKind::Markup,
            subtype: "Square".to_owned(),
            locked,
        },
        outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
        oriented: None,
    });
    selection
}

fn ask<'a>(
    (edit_content, author_markup): (bool, bool),
    selected_field: Option<&'a SelectedField>,
    selection: &'a SelectionState,
    field_set: &'a FieldSet,
    refused: bool,
) -> Ask<'a> {
    Ask {
        edit_content,
        author_markup,
        selected_field,
        field_set,
        field_refused: refused,
        annot_refused: refused,
        selection,
    }
}

#[test]
fn a_field_goes_first_and_only_where_content_is_editable() {
    let (f, set, sel) = (field(), FieldSet::default(), with_annot(false));
    assert!(matches!(
        route(&ask(EDIT, Some(&f), &sel, &set, false)),
        Route::Fields(Some(_))
    ));
    assert!(matches!(
        route(&ask(EDIT, Some(&f), &sel, &set, true)),
        Route::FieldRefused(_)
    ));
    // Review cannot edit content, so the field is passed over for the markup.
    assert!(matches!(
        route(&ask(REVIEW, Some(&f), &sel, &set, false)),
        Route::Annots(Some(_))
    ));
}

#[test]
fn review_deletes_markup_and_read_deletes_nothing() {
    let (set, sel) = (FieldSet::default(), with_annot(false));
    assert!(matches!(
        route(&ask(REVIEW, None, &sel, &set, false)),
        Route::Annots(Some(_))
    ));
    assert!(matches!(
        route(&ask(REVIEW, None, &sel, &set, true)),
        Route::AnnotRefused(id) if id == ObjId::new(7, 0)
    ));
    assert!(matches!(
        route(&ask(READ, None, &sel, &set, false)),
        Route::ModeCannotEditContent
    ));
}

#[test]
fn content_is_reached_only_in_a_mode_that_edits_it() {
    let (set, sel) = (FieldSet::default(), SelectionState::default());
    assert!(matches!(
        route(&ask(EDIT, None, &sel, &set, false)),
        Route::Content
    ));
    assert!(matches!(
        route(&ask(REVIEW, None, &sel, &set, false)),
        Route::ModeCannotEditContent
    ));
}
