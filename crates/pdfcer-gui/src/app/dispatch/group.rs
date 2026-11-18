//! # `app::dispatch::group` — Group and Ungroup
//!
//! Page objects selected in a mode that edits content are grouped into one
//! form XObject, or one selected form is taken apart; parts selected inside a
//! placed drawing are grouped within it, or the group one part is directly
//! inside is taken apart; otherwise the selected
//! markups are grouped under the one selected last, or their group is
//! released. The chords are bound to the `markup.*` ids and reach both, as
//! `super::arrange`'s do.

use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{ObjId, Object};
use pdfcer_core::vector::{ImageSource, VectorObject};

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::group::GroupAction;
use crate::app::state::{OpenDoc, Status};
use crate::canvas::selection::AnnotKind;
use crate::text::grouping::GroupRefusal;

/// Set while two or more page objects, or two or more parts of placed
/// drawings, are selected in a mode that edits them.
pub(crate) const OBJECTS_GROUPABLE: &str = "selection.objects_groupable"; // ui-text-exempt: condition name
/// Set while the one selected page object is a group or a placed drawing, or
/// one part of a placed drawing is selected.
pub(crate) const FORM_UNGROUPABLE: &str = "selection.form_ungroupable"; // ui-text-exempt: condition name
/// Set while two or more markups are selected in a mode that authors them.
pub(crate) const MARKUPS_GROUPABLE: &str = "selection.markups_groupable"; // ui-text-exempt: condition name
/// Set while a selected markup is a member of a group.
pub(crate) const MARKUP_UNGROUPABLE: &str = "selection.markup_ungroupable"; // ui-text-exempt: condition name

/// Whether `id` is one of the four grouping commands.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    wants_group(id).is_some()
}

/// `Some(true)` for a Group command, `Some(false)` for an Ungroup one.
fn wants_group(id: &str) -> Option<bool> {
    // ui-text-exempt: registered command ids, never displayed.
    match id {
        "edit.group" | "markup.group" => Some(true),
        "edit.ungroup" | "markup.ungroup" => Some(false),
        _ => None,
    }
}

/// The page and the two or more page objects Group would gather.
#[must_use]
pub(crate) fn groupable_objects(doc: &OpenDoc) -> Option<(usize, Vec<usize>)> {
    super::arrange::restackable_objects(doc).filter(|(_, objects)| objects.len() >= 2)
}

/// The page and the two or more parts of placed drawings Group would gather
/// inside the drawing that holds them; the engine refuses parts of two.
#[must_use]
pub(crate) fn groupable_leaves(doc: &OpenDoc) -> Option<(usize, Vec<usize>)> {
    selected_leaves(doc).filter(|(_, leaves)| leaves.len() >= 2)
}

/// The page and the one selected part of a placed drawing.
#[must_use]
pub(crate) fn ungroupable_leaf(doc: &OpenDoc) -> Option<(usize, usize)> {
    match selected_leaves(doc)? {
        (page, leaves) if leaves.len() == 1 => Some((page, leaves[0])),
        _ => None,
    }
}

/// Parts of placed drawings selected whole, with no page object or markup.
fn selected_leaves(doc: &OpenDoc) -> Option<(usize, Vec<usize>)> {
    if doc.selection.level() != crate::canvas::selection::SelectionLevel::Object {
        return None;
    }
    crate::canvas::formclip::leaves_only(doc)
}

/// The page and the one selected page object, when it is a form XObject.
///
/// The page's decomposition answers only for the page in view; a selection on
/// another page is passed through and the engine decides.
#[must_use]
pub(crate) fn ungroupable_form(doc: &OpenDoc) -> Option<(usize, usize)> {
    let (page, objects) = super::arrange::restackable_objects(doc)?;
    let [object] = objects[..] else {
        return None;
    };
    if page == doc.view.page_index {
        let provider = doc.page_objects()?;
        let is_form = matches!(
            provider.page_objects().objects.get(object),
            Some(VectorObject::Image(image))
                if image.source == ImageSource::Form && image.xobject.is_some()
        );
        if !is_form {
            return None;
        }
    }
    Some((page, object))
}

/// The page, the primary (selected last) and the other markups Group would
/// join. Links and ce dimensions are not markups here; a locked markup is the
/// document's to keep as it is.
#[must_use]
pub(crate) fn groupable_markups(doc: &OpenDoc) -> Option<(usize, ObjId, Vec<ObjId>)> {
    let all = doc.selection.annots();
    let (primary, others) = all.split_last()?;
    if others.is_empty() {
        return None;
    }
    let fits = |a: &&crate::canvas::selection::AnnotSelection| {
        a.target.kind == AnnotKind::Markup
            && a.target.subtype != "Link"
            && !a.target.locked
            && a.target.page == primary.target.page
    };
    all.iter().all(fits).then(|| {
        (
            primary.target.page,
            primary.target.id,
            others.iter().map(|a| a.target.id).collect(),
        )
    })
}

/// The primary of the group annotation `id` is a member of: its `/IRT`, when
/// `/RT` is `/Group` (ISO 32000-1 §12.5.6.2). One dictionary read.
#[must_use]
pub(crate) fn group_primary(doc: &OpenDoc, id: ObjId) -> Option<ObjId> {
    let Some(Object::Dict(dict)) = doc.session.value(id) else {
        return None;
    };
    let graph = doc.session.graph();
    let grouped = matches!(
        dict.get(b"RT").map(|o| graph.resolve(o)),
        Some(Object::Name(n)) if n.as_bytes() == b"Group"
    );
    match dict.get(b"IRT") {
        Some(Object::Reference(primary)) if grouped => Some(*primary),
        _ => None,
    }
}

/// The page and a selected markup that is a member of a group.
#[must_use]
pub(crate) fn ungroupable_markup(doc: &OpenDoc) -> Option<(usize, ObjId)> {
    doc.selection
        .annots()
        .into_iter()
        .find(|a| a.target.kind == AnnotKind::Markup && group_primary(doc, a.target.id).is_some())
        .map(|a| (a.target.page, a.target.id))
}

/// Set this module's four conditions for the open document.
pub(crate) fn conditions(
    app: &PdfcerApp,
    doc: &OpenDoc,
    set: &mut egui_shell::commands::ConditionSet,
) {
    let can = app.capabilities();
    if can.edit_content {
        if groupable_objects(doc).is_some() || groupable_leaves(doc).is_some() {
            set.set(OBJECTS_GROUPABLE);
        }
        if ungroupable_form(doc).is_some() || ungroupable_leaf(doc).is_some() {
            set.set(FORM_UNGROUPABLE);
        }
    }
    if can.author_markup {
        if groupable_markups(doc).is_some() {
            set.set(MARKUPS_GROUPABLE);
        }
        if ungroupable_markup(doc).is_some() {
            set.set(MARKUP_UNGROUPABLE);
        }
    }
}

/// The page-object verb `group` asks for, if the selection offers one.
fn object_action(doc: &OpenDoc, group: bool) -> Option<GroupAction> {
    if group {
        groupable_objects(doc)
            .map(|(page, objects)| GroupAction::Objects { page, objects })
            .or_else(|| {
                groupable_leaves(doc)
                    .map(|(page, leaves)| GroupAction::ObjectsInForm { page, leaves })
            })
    } else {
        ungroupable_form(doc)
            .map(|(page, object)| GroupAction::UngroupForm { page, object })
            .or_else(|| {
                ungroupable_leaf(doc).map(|(page, leaf)| GroupAction::UngroupInForm { page, leaf })
            })
    }
}

/// The markup verb `group` asks for, if the selection offers one. Ungroup
/// with only a group's primary selected is passed to the engine, which knows
/// whether anything is grouped under it.
fn markup_action(doc: &OpenDoc, group: bool) -> Option<GroupAction> {
    if group {
        return groupable_markups(doc).map(|(page, primary, members)| GroupAction::Markups {
            page,
            primary,
            members,
        });
    }
    ungroupable_markup(doc)
        .or_else(|| {
            doc.selection
                .annot()
                .filter(|a| a.target.kind == AnnotKind::Markup)
                .map(|a| (a.target.page, a.target.id))
        })
        .map(|(page, markup)| GroupAction::UngroupMarkups { page, markup })
}

/// Dispatch one of the four.
pub(super) fn dispatch(app: &mut PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    let Some(group) = wants_group(id) else {
        return;
    };
    let can = app.capabilities();
    let Status::Open(doc) = &app.status else {
        return;
    };
    let chosen = can
        .edit_content
        .then(|| object_action(doc, group))
        .flatten()
        .or_else(|| {
            (can.author_markup && !id.starts_with("edit."))
                .then(|| markup_action(doc, group))
                .flatten()
        });
    match chosen {
        Some(action) => actions.push(Action::Group(action)),
        None => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("command-declined id={id} reason=nothing-to-group")
            });
            crate::app::status::decline::record_group(if group {
                GroupRefusal::TooFew
            } else {
                GroupRefusal::NotADrawing
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Four ids are claimed, two of each kind, and nothing else.**
    #[test]
    fn the_four_ids_and_no_other() {
        for id in ["edit.group", "markup.group"] {
            assert_eq!(wants_group(id), Some(true), "{id}");
        }
        for id in ["edit.ungroup", "markup.ungroup"] {
            assert_eq!(wants_group(id), Some(false), "{id}");
        }
        for id in [
            "edit.align",
            "markup.bring_to_front",
            "format.select_form",
            "",
        ] {
            assert!(!claims(id), "{id}");
        }
    }
}
