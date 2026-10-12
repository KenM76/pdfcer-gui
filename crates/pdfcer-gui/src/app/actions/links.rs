//! # `app::actions::links` — applying `LinkAction`
//!
//! Each arm is one `EditSession` link verb through the vector-edit funnel:
//! one undo entry, a trace a driven check reads, and the verb's disclosures
//! on the status line.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/links.md`.

use pdfcer_core::edit::{EditError, LinkBorder, LinkTarget};
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::linkaction::LinkAction;

use crate::app::state::OpenDoc;
use crate::text::linkedit as t;

/// Apply one link verb.
pub(super) fn apply(doc: &mut OpenDoc, action: LinkAction) {
    match action {
        LinkAction::Add {
            page,
            rect,
            target,
            border,
        } => add(doc, page, rect, &target, border.as_ref()),
        LinkAction::SetTarget { id, target } => set_target(doc, id, &target),
        LinkAction::SetBorder { id, border } => set_border(doc, id, border.as_ref()),
    }
}

fn add(
    doc: &mut OpenDoc,
    page: usize,
    rect: Rect,
    target: &LinkTarget,
    border: Option<&LinkBorder>,
) {
    crate::app::actions::apply::vector_edit(doc, "add-link", page, 1, |session| {
        session
            .add_link(page, rect, target, border)
            .map(|id| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "add-link-applied page={page} id={} target={} border={}",
                        id.num,
                        target_word(target),
                        border.is_some()
                    )
                });
                vec![t::added(page + 1)]
            })
            .map_err(Worded)
    });
}

fn set_target(doc: &mut OpenDoc, id: ObjId, target: &LinkTarget) {
    crate::app::actions::apply::vector_edit(doc, "set-link-target", 0, 1, |session| {
        session
            .set_link_target(id, target)
            .map(|change| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "set-link-target-applied id={} target={} replaced={}",
                        id.num,
                        target_word(target),
                        change.replaced_action.as_deref().unwrap_or("none")
                    )
                });
                let mut notes = vec![t::retargeted().to_owned()];
                notes.extend(change.replaced_action.as_deref().map(t::replaced_action));
                notes
            })
            .map_err(Worded)
    });
}

fn set_border(doc: &mut OpenDoc, id: ObjId, border: Option<&LinkBorder>) {
    crate::app::actions::apply::vector_edit(doc, "set-link-border", 0, 1, |session| {
        session
            .set_link_border(id, border)
            .map(|change| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "set-link-border-applied id={} visible={} width={} appearance_replaced={}",
                        id.num,
                        border.is_some(),
                        border.map_or(0.0, |b| b.width),
                        change.appearance_replaced
                    )
                });
                let mut notes = vec![t::bordered(border.is_some()).to_owned()];
                if change.appearance_replaced {
                    notes.push(t::appearance_replaced().to_owned());
                }
                notes
            })
            .map_err(Worded)
    });
}

/// The target's kind, for the trace.
fn target_word(target: &LinkTarget) -> String {
    match target {
        LinkTarget::Page { page_index, .. } => format!("page:{page_index}"),
        LinkTarget::Named(key) => format!("named:{}", String::from_utf8_lossy(key)),
        LinkTarget::Uri(uri) => format!("uri:{uri}"),
        _ => "other".to_owned(),
    }
}

/// An engine refusal, worded where the engine's sentence names a key.
struct Worded(EditError);

impl std::fmt::Display for Worded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            EditError::NamedDestinationNotFound { name } => f.write_str(&t::name_not_found(name)),
            EditError::LinkUriInvalid { reason } => f.write_str(&t::uri_invalid(reason)),
            EditError::LinkBorderWidthInvalid { .. } => f.write_str(t::width_invalid()),
            EditError::EmptyGeometry => f.write_str(t::no_area()),
            other => other.fmt(f),
        }
    }
}

impl crate::app::unlock::Refusal for Worded {
    fn encrypted(&self) -> bool {
        crate::app::unlock::Refusal::encrypted(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::open_local_fixture;
    use pdfcer_core::annot::page_annotations;
    use pdfcer_core::annot_author::Color;
    use pdfcer_core::outline::DestView;

    fn links_on_page_0(doc: &OpenDoc) -> Vec<ObjId> {
        page_annotations(&doc.session.graph(), doc.pages[0].id)
            .into_iter()
            .filter(|a| a.subtype.as_slice() == b"Link")
            .filter_map(|a| a.id)
            .collect()
    }

    fn page_ids(doc: &OpenDoc) -> Vec<ObjId> {
        doc.pages.iter().map(|p| p.id).collect()
    }

    /// The whole round: add, read back, re-point, re-border, all through
    /// the funnel and the reader the Properties section uses.
    #[test]
    fn a_link_is_added_read_back_retargeted_and_reframed() {
        let mut doc = open_local_fixture("four-pages.pdf");
        let rect = Rect {
            llx: 100.0,
            lly: 100.0,
            urx: 200.0,
            ury: 120.0,
        };
        apply(
            &mut doc,
            LinkAction::Add {
                page: 0,
                rect,
                target: LinkTarget::Page {
                    page_index: 2,
                    view: DestView::Fit,
                },
                border: None,
            },
        );
        let links = links_on_page_0(&doc);
        assert_eq!(links.len(), 1, "one link added");
        let id = links[0];
        let read = pdfcer_gui_base::linkread::read(&doc.session, id, &page_ids(&doc))
            .expect("a link dictionary");
        assert_eq!(read.goes, pdfcer_gui_base::linkread::Goes::Page(2));
        assert_eq!(read.border, None, "no border is /Border [0 0 0]");

        apply(
            &mut doc,
            LinkAction::SetTarget {
                id,
                target: LinkTarget::Uri("https://example.com".to_owned()),
            },
        );
        apply(
            &mut doc,
            LinkAction::SetBorder {
                id,
                border: Some(LinkBorder::new(2.0, Color::Rgb(1.0, 0.0, 0.0))),
            },
        );
        let read = pdfcer_gui_base::linkread::read(&doc.session, id, &page_ids(&doc))
            .expect("a link dictionary");
        assert_eq!(
            read.goes,
            pdfcer_gui_base::linkread::Goes::Uri("https://example.com".to_owned())
        );
        let border = read.border.expect("a visible border");
        assert!((border.width - 2.0).abs() < 1e-9);
        assert_eq!(border.rgb, Some([255, 0, 0]));
        assert!(!border.dashed);
    }
}
