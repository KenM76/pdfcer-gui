//! # `canvas::links` — **following a `/Link`, which this program could not do
//! at all until now**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/links.md`.

use egui::{Pos2, Rect};
use pdfcer_core::outline::{Destination, RemoteTarget};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::links as t;

/// A link the pointer is over, lifted out of the cache.
///
/// **Owned, not borrowed.** `OpenDoc::page_links` hands back a `Ref`, and a
/// caller holding one cannot then take `&mut OpenDoc` — which every consumer
/// here eventually needs, directly or through the action funnel. One
/// `Destination` clone per hit is a handful of bytes on a gesture the operator
/// made deliberately; the borrow it avoids is a whole class of runtime panic.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// Where it goes. See [`follow`] for the five cases.
    pub destination: Destination,
    /// Its position in the page's `/Annots` array.
    ///
    /// **Not** its index in the resolved list — those disagree on any page
    /// carrying a non-link annotation, and this is the one the engine takes
    /// when an annotation has to be addressed back to it.
    pub annots_index: usize,
    /// The clickable box, in canvas coordinates.
    pub rect: Rect,
}

impl Hit {
    /// Whether this link is one this program can actually perform.
    ///
    /// The predicate the cursor is decided by, and deliberately narrow: only
    /// `Destination::Page` is navigable. See the module header on why a hand
    /// over the other four would be advertising a capability that is not there.
    #[must_use]
    pub const fn navigable(&self) -> bool {
        matches!(self.destination, Destination::Page { .. })
    }
}

/// The link under `point` on `page_index`, if any.
///
/// `point` is in **canvas/page coordinates**, the same space
/// `PageMapping::to_page` produces and the same space every other hit test on
/// this surface takes.
///
/// ## Last match wins
///
/// `/Annots` is painted in array order, so a later entry is drawn over an
/// earlier one and is the one under the pointer where two overlap. Overlapping
/// links are rare and are exactly the case a first-match scan gets backwards —
/// and a first-match scan looks correct on every document that does not have
/// them, which is almost all of them.
///
/// ## A link with no `/Rect` is skipped, and that is not a filter
///
/// §12.5.2 makes `/Rect` required, so a link without one has a destination it
/// can never be clicked to reach. The engine keeps it in the list rather than
/// dropping it — so a repair tool can see it — and a hit test must skip it,
/// because there is no box to be inside.
#[must_use]
pub fn under_pointer(doc: &OpenDoc, page_index: usize, point: Pos2) -> Option<Hit> {
    let page = doc.pages.get(page_index)?;
    let links = doc.page_links(page_index)?;
    let mut found = None;
    for link in &links.links {
        let Some(rect) = link.rect else { continue };
        let Some(canvas) = crate::canvas::mapping::annot_canvas_rect(
            [rect.llx, rect.lly, rect.urx, rect.ury],
            page,
        ) else {
            continue;
        };
        if canvas.contains(point) {
            found = Some(Hit {
                destination: link.destination.clone(),
                annots_index: link.annots_index,
                rect: canvas,
            });
        }
    }
    found
}

/// **Act on a link the operator clicked.**
///
/// Navigates when it can and says why when it cannot. Every branch does exactly
/// one of those two things and there is no silent arm — see the module header
/// on the four non-navigating variants.
pub fn follow(hit: &Hit, doc: &OpenDoc, actions: &mut Vec<Action>) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // The VARIANT is named, not merely "a link was clicked". The whole
        // defect class here is a viewer collapsing five destinations into two
        // behaviours, and a trace that said `link-followed` for all five would
        // be unable to show it.
        format!(
            "link-click page={page_index} index={index} kind={kind}",
            page_index = doc.view.page_index,
            index = hit.annots_index,
            kind = kind_token(&hit.destination),
        )
    });
    match &hit.destination {
        // The ONE navigable case, and it goes through the same pipeline a
        // bookmark does — `app::actions::destination::actions_for`, page first
        // and unconditionally, zoom before scroll. Reusing it is not tidiness:
        // a second implementation of Table 151 would be a second set of answers
        // to "what does a null `top` mean", and the two would drift.
        Destination::Page { page_index, view } => {
            crate::app::actions::destination::actions_for(*page_index, view, actions);
        }
        // The target is not a page of this tree. Usually a page delete.
        Destination::UnmappedPage { .. } => {
            crate::app::actions::record_note(doc.edit_epoch, t::unmapped_page().to_owned());
        }
        // A name neither §12.3.2.3 namespace defines.
        Destination::Named { .. } => {
            let name = hit.destination.name_lossy().unwrap_or_default();
            crate::app::actions::record_note(doc.edit_epoch, t::unresolved_name(&name));
        }
        // `/GoToR`. The remote name is NEVER resolved against this document's
        // name tree — §12.6.4.3 puts it in the target file's namespace, and a
        // document that happened to define the same name would otherwise
        // produce a confident, entirely wrong local page jump with no error
        // anywhere. The engine refuses to do it and so does this.
        Destination::Remote { file, target, .. } => {
            crate::app::actions::record_note(
                doc.edit_epoch,
                t::remote(&remote_label(file, target)),
            );
        }
        //
        // The engine already resolved a file specification for `/GoToR` and
        // **threw it away for `/Launch`** — the same key, the same question,
        // answered in one case and not the other. It now reads Table 203's `/F`
        // and the deprecated `/Win` fallback, so *"which file does this open?"*
        // has an answer where one is written.
        //
        // `/Launch` stays NON-navigation. It starts an application; it does
        // not go to a page, and pdfcer will not run it (R13). What changed is
        // only that the disclosure can now NAME the file instead of saying
        // "this is a Launch action" and stopping — which is the difference
        // between a sentence an operator can act on and one they cannot.
        Destination::NonNavigation { action, file } => {
            let named = action.as_ref().map_or_else(
                || unknown_action().to_owned(),
                |a| String::from_utf8_lossy(&a.0).into_owned(),
            );
            let note = match file.as_ref() {
                Some(bytes) => t::non_navigation_file(&named, &String::from_utf8_lossy(bytes)),
                None => t::non_navigation(&named),
            };
            crate::app::actions::record_note(doc.edit_epoch, note);
        }
        // `Destination` is `#[non_exhaustive]`. A variant added by a later
        // engine Pass must not silently become "nothing happened": this arm
        // says the link exists and pdfcer does not know what it is, which is
        // true and is a sentence somebody will report.
        _ => {
            crate::app::actions::record_note(doc.edit_epoch, t::no_destination().to_owned());
        }
    }
}

/// The word for a `NonNavigation` whose `/S` could not be read.
///
/// A separate function so `check-ui-strings` sees it as copy rather than as a
/// literal buried in a `map_or_else`.
fn unknown_action() -> &'static str {
    // ui-text-exempt: substituted into `text::links::non_navigation`, which is
    // the catalogued sentence; this is the noun that fills its hole.
    "unrecognised"
}

/// A `/GoToR`'s file and page, for the sentence.
fn remote_label(file: &Option<Vec<u8>>, target: &RemoteTarget) -> String {
    let name = file.as_ref().map_or_else(
        || unnamed_file().to_owned(),
        |bytes| String::from_utf8_lossy(bytes).into_owned(),
    );
    match target {
        // `+ 1`. `RemoteTarget::PageNumber` is 0-based and every page number
        // this program shows an operator is 1-based. The engine's own reply
        // flagged this as the conversion it nearly got wrong in its CLI.
        RemoteTarget::PageNumber(n) => t::remote_page(&name, n.saturating_add(1).unsigned_abs()),
        RemoteTarget::Named(bytes) => t::remote_named(&name, &String::from_utf8_lossy(bytes)),
        _ => name,
    }
}

/// The word for a `/GoToR` with no readable `/F`.
fn unnamed_file() -> &'static str {
    // ui-text-exempt: substituted into `text::links::remote`.
    "another document"
}

/// The trace's one-word name for a destination kind.
fn kind_token(destination: &Destination) -> &'static str {
    match destination {
        Destination::Page { .. } => "page", // ui-text-exempt: trace token
        Destination::UnmappedPage { .. } => "unmapped", // ui-text-exempt: trace token
        Destination::Named { .. } => "named", // ui-text-exempt: trace token
        Destination::Remote { .. } => "remote", // ui-text-exempt: trace token
        Destination::NonNavigation { .. } => "action", // ui-text-exempt: trace token
        _ => "unknown",                     // ui-text-exempt: trace token
    }
}

/// **Set the pointing hand over a link that can be followed.**
///
/// The whole of the discovery affordance, and the only thing this module puts
/// on screen. Called once per frame from `canvas::present`, after
/// `canvas::forms`' own cursor pass and before `canvas::interact` runs — so
/// `canvas::tool::cursor_for` still has the last word, which is right: a
/// cursor it has an opinion about is one describing a gesture already under
/// way, and that outranks a hover.
///
/// Does nothing in a mode that edits content, matching [`follow`]'s own gate.
/// A hand promising navigation in a mode where the click selects instead would
/// be a lie told sixty times a second.
pub(super) fn cursor(
    ctx: &egui::Context,
    doc: &OpenDoc,
    pages: &[crate::canvas::strip::PageView],
    edit_content: bool,
) {
    if edit_content {
        return;
    }
    let Some(pos) = ctx.pointer_latest_pos() else {
        return;
    };
    for view in pages {
        if !view.map.image_rect().contains(pos) {
            continue;
        }
        if under_pointer(doc, view.page, view.map.to_page(pos)).is_some_and(|hit| hit.navigable()) {
            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit_with(destination: Destination) -> Hit {
        Hit {
            destination,
            annots_index: 0,
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
        }
    }

    /// **Exactly one of the five variants is navigable.**
    #[test]
    fn only_a_resolved_page_is_navigable() {
        assert!(
            hit_with(Destination::Page {
                page_index: 3,
                view: pdfcer_core::outline::DestView::Fit,
            })
            .navigable()
        );
        for other in [
            Destination::UnmappedPage {
                page: None,
                view: pdfcer_core::outline::DestView::Fit,
            },
            Destination::Named {
                name: b"absent".to_vec(),
            },
            Destination::NonNavigation {
                action: None,
                file: None,
            },
        ] {
            assert!(
                !hit_with(other.clone()).navigable(),
                "{other:?} must not be offered as navigable — a hand cursor over it \
                 advertises a capability that does not exist"
            );
        }
    }

    /// Every variant gets its own trace token, and none of them collide.
    #[test]
    fn each_destination_kind_has_its_own_trace_token() {
        let tokens = [
            kind_token(&Destination::Page {
                page_index: 0,
                view: pdfcer_core::outline::DestView::Fit,
            }),
            kind_token(&Destination::UnmappedPage {
                page: None,
                view: pdfcer_core::outline::DestView::Fit,
            }),
            kind_token(&Destination::Named { name: Vec::new() }),
            kind_token(&Destination::NonNavigation {
                action: None,
                file: None,
            }),
        ];
        for (i, a) in tokens.iter().enumerate() {
            for b in &tokens[i + 1..] {
                assert_ne!(a, b, "two destination kinds share a trace token");
            }
        }
    }

    /// **A remote page number is reported 1-based.**
    #[test]
    fn a_remote_page_number_is_shown_one_based() {
        let label = remote_label(&Some(b"other.pdf".to_vec()), &RemoteTarget::PageNumber(0));
        assert!(label.contains("page 1"), "{label}");
        assert!(!label.contains("page 0"), "{label}");
        let later = remote_label(&Some(b"other.pdf".to_vec()), &RemoteTarget::PageNumber(11));
        assert!(later.contains("page 12"), "{later}");
    }

    /// A `/GoToR` with no readable `/F` still produces a sentence.
    ///
    /// The alternative — refusing to say anything — leaves a click that does
    /// nothing, which is the state this whole module exists to remove.
    #[test]
    fn a_remote_with_no_filename_still_names_something() {
        let label = remote_label(&None, &RemoteTarget::PageNumber(4));
        assert!(!label.is_empty());
        assert!(label.contains("page 5"), "{label}");
    }
}
