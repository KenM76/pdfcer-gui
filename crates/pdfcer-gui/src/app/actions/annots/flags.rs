//! # `app::actions::annots::flags` — one annotation-flag switch, applied
//!
//! Reads the annotation's `/F` as the document holds it now, turns one switch
//! through [`pdfcer_gui_base::annotflagswitch::switched`], and writes the whole
//! word back with `EditSession::set_annotation_flags` as one undoable edit.
//! A mark taken off screen is deselected, because the canvas can no longer
//! select it and an outline around nothing would be the only thing left.

use pdfcer_core::object::ObjId;
use pdfcer_gui_base::annotflagswitch::{FlagSwitch, switched};

use crate::app::state::OpenDoc;

/// Apply one switch.
pub(super) fn set_flag(doc: &mut OpenDoc, page: usize, id: ObjId, switch: FlagSwitch, on: bool) {
    let Some(page_id) = doc.pages.get(page).map(|p| p.id) else {
        return;
    };
    crate::app::actions::apply::vector_edit(doc, "set-annotation-flag", page, 1, |session| {
        let word = pdfcer_core::annot::page_annotations(&session.graph(), page_id)
            .into_iter()
            .find(|a| a.id == Some(id))
            .map_or(0, |a| a.flags.0);
        let next = pdfcer_core::annot::AnnotFlags(switched(word, switch, on));
        session.set_annotation_flags(id, next).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "set-annotation-flag-applied id={} subtype={} switch={} on={on} \
                     before={:#x} after={:#x}",
                    id.num,
                    change.subtype,
                    trace_name(switch),
                    change.before.0,
                    change.after.0
                )
            });
            Vec::new()
        })
    });
    if switch == FlagSwitch::OnScreen && !on {
        doc.selection.clear_annot();
    }
}

/// The switch's name in the trace.
const fn trace_name(switch: FlagSwitch) -> &'static str {
    match switch {
        FlagSwitch::OnScreen => "on-screen", // ui-text-exempt: diagnostic trace token
        FlagSwitch::Prints => "prints",      // ui-text-exempt: diagnostic trace token
        FlagSwitch::Locked => "locked",      // ui-text-exempt: diagnostic trace token
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::open_local_fixture;
    use pdfcer_core::annot::page_annotations;
    use pdfcer_core::annot_author::{Quad, RedactSpec};
    use pdfcer_core::page_tree::Rect;
    use pdfcer_core::vartext::Quadding;

    fn selectable(doc: &OpenDoc, id: ObjId) -> bool {
        let view = doc.session.view();
        crate::canvas::selection::annot::selectable_on(
            &view,
            &doc.pages[0],
            0,
            &std::collections::BTreeSet::new(),
            &std::collections::BTreeMap::new(),
        )
        .iter()
        .any(|c| c.target.id == id)
    }

    fn word(doc: &OpenDoc, id: ObjId) -> u32 {
        page_annotations(&doc.session.graph(), doc.pages[0].id)
            .into_iter()
            .find(|a| a.id == Some(id))
            .map(|a| a.flags.0)
            .expect("the mark is on the page")
    }

    /// Off screen takes the mark out of the canvas's reach, on screen brings
    /// it back, and Locked is written — through the shell's own apply path.
    #[test]
    fn the_switches_reach_the_file_and_the_canvas() {
        use pdfcer_core::annot::AnnotFlags;
        let mut doc = open_local_fixture("four-pages.pdf");
        let spec = RedactSpec {
            quads: vec![Quad::from_rect(Rect {
                llx: 100.0,
                lly: 100.0,
                urx: 200.0,
                ury: 120.0,
            })],
            fill: None,
            overlay_text: None,
            quadding: Quadding::Left,
        };
        crate::app::actions::apply::vector_edit(&mut doc, "mark", 0, 1, |session| {
            session.add_redaction(0, &spec).map(|_| Vec::new())
        });
        let id = page_annotations(&doc.session.graph(), doc.pages[0].id)
            .into_iter()
            .find_map(|a| a.id)
            .expect("a mark");
        assert!(selectable(&doc, id));
        set_flag(&mut doc, 0, id, FlagSwitch::OnScreen, false);
        assert_ne!(word(&doc, id) & AnnotFlags::NO_VIEW, 0);
        assert!(!selectable(&doc, id), "a hidden mark cannot be clicked");
        set_flag(&mut doc, 0, id, FlagSwitch::OnScreen, true);
        assert!(selectable(&doc, id), "Show on screen brings it back");
        set_flag(&mut doc, 0, id, FlagSwitch::Locked, true);
        assert_ne!(word(&doc, id) & AnnotFlags::LOCKED, 0);
    }
}
