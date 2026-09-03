//! # `canvas::markup::route` — which markup gesture one drag reaches
//!
//! Split out of [`crate::canvas::interact`] under **R2** on 2026-08-28, when the
//! text-following highlight took that file past 1,500 lines for the third time
//! in a day.
//!
//! ## ★★★ The seam, and it is the same one `canvas::dragroute` draws
//!
//! One outcome — a drag with a markup tool armed — reaches **three** different
//! gesture modules, and which one is decided by the kind and by what is under
//! the pointer:
//!
//! | armed kind | over | module | geometry |
//! |---|---|---|---|
//! | Freehand | anything | [`super::ink`] | a simplified pointer trail |
//! | **Highlight** | **text** | [`super::text`] | line-grouped **quads** |
//! | Highlight | blank paper | [`super::band`] | a rectangle |
//! | everything else | anything | [`super::band`] | two points |
//!
//! ★★ **Highlight is the row that made this a router.** It was two branches —
//! freehand or band — until the operator pointed out that dragging the
//! highlighter along text should follow the text, as Acrobat does
//! (`OPERATOR_REQUESTS.md` **O54**). A third destination on one gesture is what
//! `dragroute` already is for a move, and the same argument applies: a fork
//! whose branches can all decline eats the gesture, and the way to keep that
//! visible is to put the alternatives side by side.
//!
//! ★ The fallback ordering matters and is stated once here rather than inferred
//! from the nesting: **text first, band second.** A drag that finds no text is
//! not a failure — over a scan it is the common case, and an area highlight
//! there is what a drawing office wants and is more than the reference
//! application offers.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::Phase;
use crate::canvas::markup::{self, MarkupKind};

/// What one frame of a markup drag produced — at most one is `Some`.
#[derive(Default)]
pub struct Previews {
    /// A two-point band, if one would commit.
    pub band: Option<markup::band::Preview>,
    /// A simplified freehand trail, in canvas space.
    pub trail: Option<Vec<egui::Pos2>>,
    /// The lines a text-following highlight would cover, in canvas space.
    pub text_marks: Option<Vec<egui::Rect>>,
}

/// One frame of a markup drag.
///
/// ★ A struct because the list reached nine, and this module's whole subject is
/// keeping three alternatives legible side by side — a nine-argument call would
/// undo that at the one place a reader looks first. `gesture::Press`,
/// `resizing::Frame` and `dragroute::Frame` all took the same shape.
pub struct Drag<'a> {
    /// The egui context, for the freehand trail's own state.
    pub ctx: &'a egui::Context,
    /// The armed kind.
    pub kind: MarkupKind,
    /// The pen — ink for a stroke, highlighter for a wash.
    pub pen: markup::pen::Pen,
    /// The open document, for its text and its page.
    pub doc: &'a OpenDoc,
    /// The page on screen.
    pub page_index: usize,
    /// The drag's endpoints, in canvas space.
    pub from: egui::Pos2,
    /// See [`Self::from`].
    pub to: egui::Pos2,
    /// Where the gesture is.
    pub phase: Phase,
}

/// Route one frame of a markup drag to the gesture the kind and the page name.
pub fn drag(frame: Drag<'_>, actions: &mut Vec<Action>) -> Previews {
    let Drag {
        ctx,
        kind,
        pen,
        doc,
        page_index,
        from,
        to,
        phase,
    } = frame;
    let mut out = Previews::default();
    if kind.is_freehand() {
        out.trail = markup::ink::drag(
            pen,
            markup::ink::Stroke {
                ctx,
                kind,
                from,
                to,
                phase,
                page_index,
                page: doc.current_page(),
            },
            actions,
        );
    } else if let Some(marks) = markup::text::swept(
        markup::text::Swept {
            kind,
            pen,
            doc,
            page_index,
            from,
            to,
            phase,
        },
        actions,
    ) {
        // ★★★ THE HIGHLIGHT FOLLOWED TEXT. `OPERATOR_REQUESTS.md` O54.
        //
        // *"we should be able to drag it along to just highlight text
        // too like it works in adobe."* Both halves already existed and
        // were not connected: sweeping text produces line-grouped
        // quads, and authoring a highlight from quads works — but the
        // only route to them was to sweep with the SELECT tool and then
        // press a ribbon control.
        //
        // ⇒ With the tool named after the job armed, a drag drew a box.
        // That is O53's shape again: reachable through a panel-shaped
        // route rather than through the gesture the operator tries
        // first, which reads as missing because it is.
        out.text_marks = Some(marks);
    } else {
        // ★ The band, and it is the FALLBACK rather than the default
        // now — a drag that found no text under it. That case is worth
        // keeping and is better than the reference: Acrobat's highlight
        // draws nothing over a scan with no text layer, and an area
        // highlight there is exactly what a drawing office wants.
        out.band = markup::band::drag(
            pen,
            kind,
            from,
            to,
            phase,
            page_index,
            doc.current_page(),
            actions,
        );
    }
    out
}
