//! # `canvas::links` — **following a `/Link`, which this program could not do
//! at all until now**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/links.md`.

use egui::{Pos2, Rect};
use pdfcer_core::edit::LinkTarget;
use pdfcer_core::object::ObjId;
use pdfcer_core::outline::{Destination, RemoteTarget};
use pdfcer_gui_base::outwardaction::{OutwardAction, RemoteGoTo};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::links as t;

/// A link the pointer is over, lifted out of the cache.
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
    /// The link annotation, when it is an indirect object; a `/URI` is read through it.
    pub id: Option<ObjId>,
}

impl Hit {
    /// Whether this link is one this program can actually perform.
    #[must_use]
    pub const fn navigable(&self) -> bool {
        matches!(self.destination, Destination::Page { .. })
    }

    /// Whether this is a `/URI` link, which asks before it opens.
    #[must_use]
    pub fn is_web(&self) -> bool {
        matches!(&self.destination, Destination::NonNavigation { action: Some(a), .. } if a.0 == b"URI")
    }

    /// Whether this is a `/Launch` naming a file, which asks before it opens.
    #[must_use]
    pub fn is_launch(&self) -> bool {
        matches!(&self.destination, Destination::NonNavigation { action: Some(a), file: Some(_) } if a.0 == b"Launch")
    }

    /// Whether this is a `/GoToR` naming a file, which opens it here.
    #[must_use]
    pub const fn is_remote_file(&self) -> bool {
        matches!(&self.destination, Destination::Remote { file: Some(_), .. })
    }

    /// Whether a click does something beyond a sentence: a page jump, the web
    /// prompt, or opening a file.
    #[must_use]
    pub fn followable(&self) -> bool {
        self.navigable() || self.is_web() || self.is_launch() || self.is_remote_file()
    }
}

/// The link under `point` on `page_index`, if any.
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
                id: link.id,
            });
        }
    }
    found
}

/// **Act on a link the operator clicked.**
pub fn follow(hit: &Hit, doc: &OpenDoc, actions: &mut Vec<Action>) {
    trace_click(hit, doc);
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
        // `/GoToR` naming a file: open it here and arrive at its place. The
        // remote name is resolved in the OPENED file's names, never in this
        // one's — §12.6.4.3 puts it in the target file's namespace, and a
        // document that happened to define the same name would otherwise
        // produce a confident, entirely wrong local page jump.
        Destination::Remote {
            file: Some(file),
            target,
            ..
        } => {
            actions.push(Action::Outward(OutwardAction::FileLink {
                file: resolve(doc, file),
                target: remote_go_to(target),
            }));
        }
        // No readable `/F`: there is no file to open, only a sentence.
        Destination::Remote { file, target, .. } => {
            crate::app::actions::record_note(
                doc.edit_epoch,
                t::remote(&remote_label(file, target)),
            );
        }
        //
        // `/URI` raises the Security Warning and `/Launch` naming a file the
        // Launch File prompt (`DESIGNS.md` § O320). Every other action, and a
        // `/Launch` naming no file, is said in words: what it is, and the file
        // it names where it names one (Table 203's `/F`, or `/Win`).
        Destination::NonNavigation { .. } if hit.is_web() => web(hit, doc, actions),
        Destination::NonNavigation {
            file: Some(file), ..
        } if hit.is_launch() => {
            actions.push(Action::Outward(OutwardAction::Launch {
                file: resolve(doc, file),
            }));
        }
        Destination::NonNavigation { action, file } => {
            crate::app::actions::record_note(
                doc.edit_epoch,
                described(action.as_ref(), file.as_deref()),
            );
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

/// Trace the click, naming the destination's variant.
fn trace_click(hit: &Hit, doc: &OpenDoc) {
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
}

/// The sentence for an action pdfcer does not perform: what it is, and the file
/// it names where it names one.
fn described(action: Option<&pdfcer_core::object::Name>, file: Option<&[u8]>) -> String {
    let named = action.map_or_else(
        || unknown_action().to_owned(),
        |a| String::from_utf8_lossy(&a.0).into_owned(),
    );
    match file {
        Some(bytes) => t::non_navigation_file(&named, &String::from_utf8_lossy(bytes)),
        None => t::non_navigation(&named),
    }
}

/// A `/URI` link: read its address through the engine and raise the prompt.
fn web(hit: &Hit, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let uri = hit
        .id
        .and_then(|id| doc.session.link(id).ok())
        .and_then(|reading| match reading.target {
            Some(LinkTarget::Uri(uri)) => Some(uri),
            _ => None,
        });
    match uri {
        Some(uri) => actions.push(Action::Outward(OutwardAction::WebLink { uri })),
        None => crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::outward::web_unreadable().to_owned(),
        ),
    }
}

/// A link's file specification as a path, relative to `doc`'s folder.
fn resolve(doc: &OpenDoc, spec: &[u8]) -> std::path::PathBuf {
    let folder = (doc.origin == crate::app::state::Origin::Opened)
        .then(|| doc.path.parent())
        .flatten();
    pdfcer_gui_base::outward::resolve_file(&String::from_utf8_lossy(spec), folder)
}

/// Where a `/GoToR` arrives. A `/D` that names no page (absent, negative,
/// unreadable) opens the file at its start rather than not at all.
fn remote_go_to(target: &RemoteTarget) -> RemoteGoTo {
    match target {
        RemoteTarget::Named(name) => RemoteGoTo::Named(name.clone()),
        other => other
            .page_index()
            .map_or(RemoteGoTo::Start, RemoteGoTo::Page),
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
        if under_pointer(doc, view.page, view.map.to_page(pos)).is_some_and(|hit| hit.followable())
        {
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
            id: None,
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
