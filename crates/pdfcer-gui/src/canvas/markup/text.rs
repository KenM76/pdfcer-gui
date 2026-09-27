//! # `canvas::markup::text` — underline, strikeout and squiggly: markup whose
//! operand is a **selection**, not a drag
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/markup/text.md`.

use pdfcer_core::annot_author::{Color, MarkupSpec, Quad};

use crate::app::actions::Action;
use crate::canvas::textsel::TextSelection;

pub use pdfcer_gui_base::markupkind::TextMarkKind;

/// Why a text-markup command authored nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// There is no text selection to mark.
    ///
    /// **Unreachable from the ribbon**, because the commands are registered
    /// `enabled_when("selection.text")` and that condition is published from
    /// exactly this state's negation. Reachable from a chord in a customized
    /// manifest, which is why it exists rather than being an `expect`.
    NoSelection,
    /// A selection exists and describes a revision that has moved.
    ///
    /// The one refusal an operator can reach *by accident*: mark a selection,
    /// and the edit that lands bumps the epoch, so pressing a second text-markup
    /// command without re-sweeping asks to mark glyphs whose positions were
    /// recorded against the previous revision. Declining is the only honest
    /// answer — `canvas::textsel` §7 — and writing the annotation anyway would
    /// put a mark over *possibly* the wrong words into the file, which is the
    /// one thing rule 4 forbids outright.
    Stale,
    /// The selection resolved to no quads at all.
    ///
    /// Structurally unreachable — `textsel::resolve` returns `None` rather than
    /// a selection with an empty box list — and refused explicitly anyway,
    /// because the alternative is `EditError::EmptyGeometry` coming back from
    /// the engine for a shell that promised never to send it geometry that draws
    /// nothing. The guard is ours, upstream of theirs, exactly as
    /// [`super::Refusal::NoExtent`] is for the drag kinds.
    NoQuads,
}

/// Build the `pdfcer-core` spec one text-markup command authors.
#[must_use]
pub fn spec(kind: TextMarkKind, quads: Vec<Quad>, pen: super::pen::Pen) -> MarkupSpec {
    let (r, g, b) = kind.rgb(pen);
    MarkupSpec::TextMarkup {
        kind: kind.subtype(),
        quads,
        color: Color::Rgb(r, g, b),
    }
}

/// **The ONE action a text-markup command becomes** — the whole rule, pure.
pub fn mark(
    kind: TextMarkKind,
    selection: Option<&TextSelection>,
    epoch: u64,
    pen: super::pen::Pen,
) -> Result<Action, Refusal> {
    let selection = selection.ok_or(Refusal::NoSelection)?;
    let quads = selection.marks(epoch);
    if quads.is_empty() {
        // `marks` returns an empty slice for BOTH "stale" and "no quads", and
        // the two are told apart here rather than by two accessors: the
        // staleness rule belongs to `textsel` and asking it once is what keeps
        // this module from carrying a second copy of it.
        return Err(if selection.live(epoch) {
            Refusal::NoQuads
        } else {
            Refusal::Stale
        });
    }
    Ok(Action::CommitTextMarkup {
        page: selection.page,
        kind,
        quads: quads.to_vec(),
        pen,
    })
}

/// Report a text-markup command that authored nothing, with the reason.
pub fn decline(kind: TextMarkKind, reason: Refusal) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-markup-declined kind={kind:?} reason={reason:?}")
    });
}

/// Report a text markup that is about to be authored.
pub fn trace_commit(kind: TextMarkKind, page: usize, quads: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-markup-commit kind={kind:?} page={page} quads={quads}")
    });
}

/// **A markup drag that found text under it: the quads it would cover, and the
/// commit on release.** `OPERATOR_REQUESTS.md` **O54**.
pub struct Swept<'a> {
    /// The armed markup kind. Only `Highlight` is answered — see the docs.
    pub kind: super::MarkupKind,
    /// The pen, for the wash.
    pub pen: super::pen::Pen,
    /// The open document, for its text and its page.
    pub doc: &'a crate::app::state::OpenDoc,
    /// The page on screen.
    pub page_index: usize,
    /// The drag's two endpoints, in canvas space.
    pub from: egui::Pos2,
    /// See [`Self::from`].
    pub to: egui::Pos2,
    /// Where the gesture is.
    pub phase: crate::canvas::gesture::Phase,
}

pub fn swept(frame: Swept<'_>, actions: &mut Vec<Action>) -> Option<Vec<egui::Rect>> {
    let Swept {
        kind,
        pen,
        doc,
        page_index,
        from,
        to,
        phase,
    } = frame;
    if kind != super::MarkupKind::Highlight {
        return None;
    }
    let page_text = doc.page_text()?;
    let page = doc.pages.get(page_index)?;
    // The SAME options the extraction ran with — `textsel::PageContext::opts`
    // — so the runs this drag sweeps are segmented exactly as the runs the
    // canvas paints and the find bar searches.
    let ctx = crate::canvas::textsel::PageContext {
        text: &page_text,
        page,
        index: page_index,
        epoch: doc.edit_epoch,
    };
    let selection = crate::canvas::textsel::drag(&ctx, from, to)?;
    let marks = selection.highlights(page_index, doc.edit_epoch);
    if marks.is_empty() {
        // No quads is NOT the same as no text: a drag that began and ended
        // inside one glyph selects nothing, and so does one over a page whose
        // text could not be extracted. Both mean *"this gesture is not
        // following text"*, and both fall through to the band — which is the
        // honest answer rather than a highlight of nothing.
        return None;
    }
    let marks = marks.to_vec();
    if phase == crate::canvas::gesture::Phase::Complete {
        match mark(
            TextMarkKind::Highlight,
            Some(&selection),
            doc.edit_epoch,
            pen,
        ) {
            Ok(raised) => {
                trace_commit(TextMarkKind::Highlight, page_index, marks.len());
                actions.push(raised);
            }
            Err(reason) => decline(TextMarkKind::Highlight, reason),
        }
        // Nothing is previewed on the frame that commits: the annotation is
        // about to be drawn for real, and a wash left over it would be a second
        // copy of the same colour, one frame stale.
        return None;
    }
    Some(marks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::annot_author::TextMarkupKind;
    use pdfcer_core::page_tree::Rect as PageRect;

    /// One line's worth of quad, at a plausible page position.
    fn quad(y: f64) -> Quad {
        Quad::from_rect(PageRect::from_corners(72.0, y, 300.0, y + 10.0))
    }

    /// A selection over `lines` lines of the given page, stamped with `epoch`.
    fn selection(page: usize, epoch: u64, lines: usize) -> TextSelection {
        crate::canvas::textsel::selection_for_test(
            page,
            epoch,
            (0..lines).map(|i| quad(700.0 - 12.0 * i as f64)).collect(),
        )
    }

    /// The default pen, for the tests whose subject is not the colour.
    fn pen() -> crate::canvas::markup::pen::Pen {
        crate::canvas::markup::pen::Pen::default()
    }

    // -----------------------------------------------------------------
    // The subtype, the colour and the quads
    // -----------------------------------------------------------------

    /// **Each kind authors its own `/Subtype`, and none borrows another's.**
    #[test]
    fn each_kind_authors_its_own_subtype() {
        let expected = [
            (TextMarkKind::Underline, TextMarkupKind::Underline),
            (TextMarkKind::StrikeOut, TextMarkupKind::StrikeOut),
            (TextMarkKind::Squiggly, TextMarkupKind::Squiggly),
        ];
        for (kind, want) in expected {
            let MarkupSpec::TextMarkup { kind: got, .. } = spec(kind, vec![quad(700.0)], pen())
            else {
                panic!("{kind:?} must author a /QuadPoints text markup");
            };
            assert_eq!(got, want, "{kind:?}");
        }
        assert_eq!(
            TextMarkKind::ALL.len(),
            expected.len(),
            "a fourth kind must be given a subtype here, not left to inherit one"
        );
    }

    /// **The quads are carried through untouched, in order and in number.**
    #[test]
    fn the_selections_quads_are_authored_unchanged() {
        let quads: Vec<Quad> = (0..4).map(|i| quad(700.0 - 12.0 * f64::from(i))).collect();
        let MarkupSpec::TextMarkup {
            quads: authored, ..
        } = spec(TextMarkKind::StrikeOut, quads.clone(), pen())
        else {
            panic!("a text mark must author a /QuadPoints text markup");
        };
        assert_eq!(authored, quads, "the boxes must arrive as they left");
    }

    /// **The operator's pen reaches every text kind.**
    #[test]
    fn the_operators_pen_reaches_every_text_kind() {
        let chosen = planted_pen();
        for &kind in TextMarkKind::ALL {
            let MarkupSpec::TextMarkup { color, .. } = spec(kind, vec![quad(700.0)], chosen) else {
                panic!("{kind:?} must author a /QuadPoints text markup");
            };
            let Color::Rgb(r, g, b) = color else {
                panic!("{kind:?} authored a non-RGB colour");
            };
            let expected = chosen.colour_of(kind.slot());
            assert!(
                (r - expected.0).abs() < 1e-9
                    && (g - expected.1).abs() < 1e-9
                    && (b - expected.2).abs() < 1e-9,
                "{kind:?} ignored its own pen and authored ({r}, {g}, {b}) instead of \
                 {expected:?} — the Markup ▸ Style swatch moves shapes and not text marks again"
            );
        }
    }

    /// **Each text kind takes its OWN pen, and no two share one.**
    #[test]
    fn each_text_kind_takes_its_own_pen() {
        use crate::canvas::markup::pen::PenSlot;
        // Every variant, including Highlight — which is deliberately NOT in
        // `TextMarkKind::ALL` (it is reached by a band gesture) and is exactly
        // the one a careless edit would route somewhere else.
        let every = [
            (TextMarkKind::Highlight, PenSlot::Highlighter),
            (TextMarkKind::Underline, PenSlot::Underline),
            (TextMarkKind::StrikeOut, PenSlot::StrikeOut),
            (TextMarkKind::Squiggly, PenSlot::Squiggly),
        ];
        // IDENTITY: each kind takes the slot named after it. This is the row
        // the separation check below cannot make — a kind routed to a slot that
        // belongs to a different FAMILY (the shape pen, the note) is still
        // "distinct from the other three".
        for (kind, slot) in every {
            assert_eq!(
                kind.slot(),
                slot,
                "{kind:?} is authored from the {:?} pen — Acrobat keeps a \
                 `c{kind:?}` key of its own, and a text mark that moves when the \
                 operator recolours a rectangle is a mark drawn by the wrong tool",
                kind.slot()
            );
        }
        // SEPARATION: and no two of them share one.
        for i in 0..every.len() {
            for j in (i + 1)..every.len() {
                assert_ne!(
                    every[i].0.slot(),
                    every[j].0.slot(),
                    "{:?} and {:?} share a pen slot — Acrobat gives each its own \
                     key, and collapsing two is how a per-kind palette becomes a \
                     single pen again",
                    every[i].0,
                    every[j].0
                );
            }
        }
        // …and Highlight is the wash, whichever gesture reached it. A
        // text-following highlight and an area highlight must come out of one
        // swatch or the feature changes colour depending on how it was drawn.
        assert_eq!(
            TextMarkKind::Highlight.slot(),
            crate::canvas::markup::pen::PenSlot::Highlighter
        );
        assert_eq!(
            crate::canvas::markup::pen::PenSlot::of(crate::canvas::markup::MarkupKind::Highlight),
            crate::canvas::markup::pen::PenSlot::Highlighter
        );
    }

    /// A pen whose eight slots hold eight distinguishable values.
    fn planted_pen() -> crate::canvas::markup::pen::Pen {
        use crate::canvas::markup::pen::{Pen, PenSlot};
        let mut pen = Pen::default();
        for (i, slot) in PenSlot::ALL.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let step = (i as u8) * 16 + 8;
            // NOT A THEME COLOUR: eight distinguishable test values, so an
            // assertion says which slot was taken rather than which default
            // happened to match.
            pen.set_colour(*slot, egui::Color32::from_rgb(step, step, step));
        }
        pen
    }

    // -----------------------------------------------------------------
    // The rule: what a command does with the selection it finds
    // -----------------------------------------------------------------

    /// **A live selection becomes exactly one action, on ITS page.**
    #[test]
    fn a_live_selection_marks_its_own_page() {
        let sel = selection(7, 3, 2);
        let raised =
            mark(TextMarkKind::Underline, Some(&sel), 3, pen()).expect("a live selection marks");
        let Action::CommitTextMarkup {
            page, kind, quads, ..
        } = raised
        else {
            panic!("a text-markup command must raise CommitTextMarkup: {raised:?}");
        };
        assert_eq!(page, 7, "the selection's page, not the visible one");
        assert_eq!(kind, TextMarkKind::Underline);
        assert_eq!(quads.len(), 2, "one quad per line of the selection");
        assert_eq!(quads, sel.page_quads, "the selection's own boxes");
    }

    /// **A selection made before an edit is refused, not marked.**
    #[test]
    fn a_stale_selection_is_refused_and_says_so() {
        let sel = selection(0, 4, 1);
        assert_eq!(
            mark(TextMarkKind::Squiggly, Some(&sel), 5, pen()),
            Err(Refusal::Stale),
            "one edit later, the boxes may be over other glyphs"
        );
        assert_eq!(
            mark(TextMarkKind::Squiggly, None, 5, pen()),
            Err(Refusal::NoSelection),
            "…and no selection at all is a different fact with a different answer"
        );
    }

    /// A selection carrying no boxes authors nothing rather than handing the
    /// engine geometry that draws nothing.
    #[test]
    fn a_selection_with_no_boxes_authors_nothing() {
        let empty = crate::canvas::textsel::selection_for_test(0, 1, Vec::new());
        assert_eq!(
            mark(TextMarkKind::Underline, Some(&empty), 1, pen()),
            Err(Refusal::NoQuads)
        );
    }

    /// **Every kind behaves identically at the rule level.**
    #[test]
    fn every_kind_marks_and_refuses_alike() {
        let live = selection(2, 9, 3);
        for &kind in TextMarkKind::ALL {
            let raised = mark(kind, Some(&live), 9, pen()).unwrap_or_else(|e| {
                panic!("{kind:?} refused a live selection: {e:?}");
            });
            assert!(
                matches!(raised, Action::CommitTextMarkup { kind: k, .. } if k == kind),
                "{kind:?} raised {raised:?}"
            );
            assert_eq!(
                mark(kind, None, 9, pen()),
                Err(Refusal::NoSelection),
                "{kind:?}"
            );
            assert_eq!(
                mark(kind, Some(&live), 10, pen()),
                Err(Refusal::Stale),
                "{kind:?}"
            );
        }
    }
}
