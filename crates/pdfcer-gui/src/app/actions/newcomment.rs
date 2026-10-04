//! # `app::actions::newcomment` — route a [`NewComment`] to its verb

use pdfcer_gui_base::newcomment::NewComment;

use crate::app::prefs::Prefs;
use crate::app::state::OpenDoc;
use crate::canvas::markup::pen::Pen;
use crate::canvas::textannot::TextAnnotKind;

/// Author `comment` in the pen's colour and opacity for its kind.
pub(super) fn apply(doc: &mut OpenDoc, prefs: &Prefs, pen: Pen, comment: NewComment) {
    match comment {
        NewComment::Attachment {
            page,
            rect,
            file,
            icon,
            description,
        } => super::attachannot::place(
            doc,
            prefs,
            &super::attachannot::Placed {
                page,
                rect,
                file: &file,
                icon,
                description: description.as_deref(),
            },
            pen.text_annot_colour(TextAnnotKind::Attachment),
            pen.opacity_option(),
        ),
        NewComment::Sound {
            page,
            rect,
            file,
            icon,
            description,
            import,
        } => super::soundannot::place(
            doc,
            prefs,
            &super::soundannot::Placed {
                page,
                rect,
                file: &file,
                icon,
                description: description.as_deref(),
                import,
            },
            pen.text_annot_colour(TextAnnotKind::Sound),
            pen.opacity_option(),
        ),
        NewComment::Caret {
            page,
            at,
            text,
            paragraph,
        } => super::caretannot::place(
            doc,
            prefs,
            &super::caretannot::Placed {
                page,
                at,
                text: text.as_deref(),
                paragraph,
            },
            pen.text_annot_colour(TextAnnotKind::Caret),
            pen.opacity_option(),
        ),
        NewComment::ReplaceText {
            page,
            at,
            text,
            struck,
        } => super::caretannot::replace(
            doc,
            prefs,
            &super::caretannot::Replaced {
                page,
                at,
                text: &text,
                struck: &struck,
            },
            pen.text_annot_colour(TextAnnotKind::Caret),
            pen.opacity_option(),
        ),
    }
}
