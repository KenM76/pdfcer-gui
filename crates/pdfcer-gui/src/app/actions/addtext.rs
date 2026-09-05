//! # `app::actions::addtext` — placing NEW page text, and the width question
//!
//! One verb, `Action::CommitAddText`, and the one decision it has to make that
//! nothing upstream can: **how wide is the text the operator just typed?**
//!
//! ## Why it is its own file, on the day it grew
//!
//! **R2.** [`super::apply`] stood at 1,494 lines of its 1,500-line ceiling when
//! `OPERATOR_REQUESTS.md` **O127** arrived, and the arm this file holds was the
//! one that had to grow. The seam is a real one rather than a size-driven cut,
//! and it is the same seam [`super::funnel`] was cut along: `apply` is a
//! **router** — it answers *"which module handles this action?"* — and this
//! answers *"what does placing text mean?"*. The arm below now decides
//! something; a router arm should not.
//!
//! ## ★★★ A PDF HAS NO PARAGRAPH, and that is the whole subject
//!
//! Every visible line of text in a PDF is its own show operator at its own
//! absolute position. There is no object that means *"this text, flowing"*. So
//! something has to decide where a second line starts — a **width to wrap
//! against** — and that decision has to be made by the time the engine is
//! called, because `AddTextRequest` offers exactly two shapes:
//!
//! | request | what the engine does with `\n` |
//! |---|---|
//! | **point** — `new(page, origin, text)` | **refuses it, by name.** `\n` has no code in any standard encoding, so `encode_str` raises `Refusal { trigger: TargetAbsent, character: Some('\n') }` and the whole add fails |
//! | **boxed** — `…with_box(x, y, w, h)` | splits on it: each `\n` is a hard paragraph break, each paragraph is wrapped independently to the box's width, top-anchored from the box's top edge |
//!
//! ⇒ **A multi-line add MUST be boxed.** That is not a preference, and it is
//! why dragging a rectangle was the multi-line gesture in the first place: a
//! drag says how wide.
//!
//! ## ★★ What changed on 2026-09-04, and the argument it had to answer
//!
//! The operator: *"can the enter key create new lines when we are editing or
//! creating text?"* Enter now inserts a line break at a **clicked** caret too —
//! and a click has no extent, so this arm meets a multi-line draft with no
//! width for the first time.
//!
//! The shell's own standing rule, from `canvas::textedit::place`'s header, is
//! that a width may not be **invented**: *"a click would have to invent a
//! width."* That rule is kept. The width is not invented — it is **read off the
//! operator's own page**:
//!
//! ```text
//! left   = where they clicked
//! right  = the crop box's right edge
//! top    = where they clicked          (so the first line starts at the caret)
//! bottom = the crop box's bottom edge
//! ```
//!
//! Every number is a fact about their document. Nothing here chooses a margin,
//! a column width or a default; the sheet does.
//!
//! ★ And the promotion is **conditional**. A single-line point add is still a
//! point add, byte for byte, taking the exact path it took before — which is
//! what keeps the common case unchanged and makes this addition impossible to
//! regress into. See [`request`]'s three-way match.
//!
//! ## ★ Rule 4: the promotion is DISCLOSED
//!
//! A rectangle that is not drawn is a rectangle the operator cannot see, and
//! where a long line breaks depends on it. `crate::text::textedit::point_text_became_a_block`
//! is the sentence, shown once at the commit and not while typing, and it names
//! the gesture that puts the width back in their hands.
//!
//! It rides the ordinary disclosure list — the same `Vec<String>` the engine's
//! own disclosures travel in — rather than getting a channel of its own,
//! because an edit *did* happen and this is the part of it they cannot see.
//! That is precisely what `⚑ About your last edit:` is for, and it is the
//! distinction O127's other two defects were both on the wrong side of.

use super::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::canvas::textedit::pen::TextPen;

/// **What the operator placed**, gathered off the action so the two functions
/// below take one argument rather than five.
///
/// The same argument [`super::textannot::Placement`] makes: `page`, `origin`,
/// `text` and `wrap` are **one thing the operator did**, and a signature that
/// listed them in a row would let a caller transpose two `f64`s silently.
pub(super) struct Placed {
    /// The 0-based page.
    pub page: usize,
    /// Where, in PDF user space. For a dragged box this is its lower-left
    /// corner; for a click it is the click.
    pub origin: (f64, f64),
    /// What they typed, newlines and all.
    pub text: String,
    /// The face, size and colour, sampled from the pen at the commit.
    pub pen: TextPen,
    /// The dragged rectangle as `(llx, lly, urx, ury)`, or `None` for a click.
    pub wrap: Option<(f64, f64, f64, f64)>,
}

/// **Author the text**, wrapped or not, and disclose it if this arm chose the
/// rectangle.
///
/// # ★ The whole body is two calls and a funnel, deliberately
///
/// Every decision is in [`request`], which is pure and therefore provable
/// without a document, a session or a window. This function owns only the
/// things that need `&mut OpenDoc`: the crop box it reads, the funnel it calls,
/// and the disclosure it appends. That is the split every geometry rule in this
/// crate is written to, and it is what lets the interesting half be tested.
pub(super) fn commit(doc: &mut OpenDoc, placed: Placed) {
    // ★ The page's own rectangle, and `None` when there is no such page — which
    // the engine will then refuse by index, in its own words. Guessing a
    // cropbox here would turn a refusal ABOUT a missing page into a refusal
    // about a rectangle this shell invented, which is `textstyle::reflow`'s
    // recorded reasoning for the same choice.
    let crop = doc.pages.get(placed.page).map(|p| p.crop_box);
    let page = placed.page;
    let (req, promoted) = request(&placed, crop);
    let lines = req.text.split('\n').count();
    vector_edit(doc, "add-text", page, lines, |session| {
        session.add_text(&req).map(|report| {
            let mut notes = report.disclosures;
            if promoted {
                notes.push(crate::text::textedit::point_text_became_a_block().to_owned());
            }
            notes
        })
    });
}

/// **Build the engine request, and say whether this arm chose the rectangle.**
///
/// Pure, and separate from [`commit`] for the standing reason: it is the part
/// that could be wrong in a way the operator would notice — text laid into the
/// wrong box, a paragraph authored as one long line, a font forgotten — and a
/// `&mut EditSession` is not available to a test that only wants to ask what
/// was built.
///
/// # The three cases, and why they are three
///
/// | draft | `wrap` | `\n`? | request |
/// |---|---|---|---|
/// | dragged box | `Some` | either | boxed, at the **operator's** rectangle |
/// | clicked point, one line | `None` | no | **point** — unchanged, and this is the common case |
/// | clicked point, several lines | `None` | yes | boxed, at the **sheet's** rectangle, and disclosed |
///
/// ★★ The second row is the one to protect. It is what an operator does dozens
/// of times an hour — click, type a label, click away — and it takes exactly
/// the path it took before this function existed. A build that boxed every add
/// would wrap a one-line label at whatever width it invented, and the width
/// would have to be invented, because a click has no extent.
///
/// # ★★★ `with_box` takes ORIGIN AND EXTENT, not two corners
///
/// A signature worth reading rather than assuming: `(x, y, w, h)` and
/// `(llx, lly, urx, ury)` are four `f64`s either way and transposing them
/// compiles, runs, and puts the text somewhere plausible and wrong. The
/// subtraction happens here, once, at the boundary — [`tests::a_dragged_box_reaches_the_engine_as_origin_and_extent`]
/// is the assertion that keeps it honest.
///
/// # ★ Why a degenerate sheet falls back to a point rather than to a zero box
///
/// If the click is at or past the crop box's right or bottom edge there is no
/// rectangle to lay text into, and a zero-width box is a request the engine
/// would refuse for a reason that has nothing to do with what the operator did.
/// The honest answer is the request that *can* be made — a point — and the
/// engine's own `\n` refusal then names the real problem. Reaching for a
/// made-up minimum width here would be inventing exactly the number this
/// module's header refuses to invent.
pub(super) fn request(
    placed: &Placed,
    crop: Option<pdfcer_core::page_tree::Rect>,
) -> (pdfcer_core::text_edit::AddTextRequest, bool) {
    // ★★ The three fields the engine has carried since `AddTextRequest` shipped
    // and this arm did not always set. The pen is `canvas::textedit::pen`,
    // edited from the Tool panel and sampled at the commit, so this computes
    // nothing — it routes three values it was handed.
    let req = pdfcer_core::text_edit::AddTextRequest::new(
        placed.page,
        placed.origin,
        placed.text.clone(),
    )
    .with_font(placed.pen.face)
    .with_size(placed.pen.size())
    .with_color(placed.pen.engine_colour());

    if let Some((llx, lly, urx, ury)) = placed.wrap {
        // The operator drew this rectangle. Nothing to decide and nothing to
        // disclose: they can see the box they dragged.
        return (req.with_box(llx, lly, urx - llx, ury - lly), false);
    }
    if !placed.text.contains('\n') {
        // One line from a click — the common case, unchanged.
        return (req, false);
    }
    let (x, y) = placed.origin;
    let Some(crop) = crop else {
        return (req, false);
    };
    let (width, height) = (crop.urx - x, y - crop.lly);
    if !(width > 0.0 && height > 0.0) {
        // See the header note on a degenerate sheet.
        return (req, false);
    }
    // ★ `y - height` is the crop box's bottom, spelled as a subtraction from
    // the click so that the box's TOP is exactly the click: `with_box` anchors
    // the first line from `y + h`, so the operator's first line begins where
    // they pressed, which is the one property they can check by looking.
    (req.with_box(x, y - height, width, height), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crop() -> pdfcer_core::page_tree::Rect {
        pdfcer_core::page_tree::Rect {
            llx: 0.0,
            lly: 0.0,
            urx: 612.0,
            ury: 792.0,
        }
    }

    fn placed(text: &str, wrap: Option<(f64, f64, f64, f64)>) -> Placed {
        Placed {
            page: 0,
            origin: (100.0, 700.0),
            text: text.to_owned(),
            pen: TextPen::default(),
            wrap,
        }
    }

    /// ★★★ **A one-line click is still a POINT add**, and this is the
    /// regression guard for the whole change.
    ///
    /// The commonest gesture in the program — click, type a label, click away —
    /// must take exactly the path it took before Enter learned to make a line
    /// break. A build that boxed every add would wrap a short label at a width
    /// nobody chose, and would do it silently.
    #[test]
    fn a_single_line_click_is_not_promoted() {
        let (req, promoted) = request(&placed("SHEET 1 OF 4", None), Some(crop()));
        assert!(
            req.wrap_box.is_none(),
            "a one-line click must reach the engine as a point add, with no rectangle"
        );
        assert!(!promoted, "and it has nothing to disclose");
    }

    /// ★★★ **A multi-line click IS promoted, and the box runs to the sheet's
    /// own edges.**
    ///
    /// The fix for O127's defect 2 at the point where it meets the engine:
    /// `\n` in a point add is a **named refusal** — `\n` has no code in any
    /// standard encoding — so without this the operator's second line would
    /// lose the whole add, with an error about a character they cannot see.
    ///
    /// Every number in the assertion is the operator's or the page's. That is
    /// the property being pinned: no margin, no default, nothing chosen here.
    #[test]
    fn a_multi_line_click_is_boxed_from_the_click_to_the_sheet_edges() {
        let (req, promoted) = request(&placed("first\nsecond", None), Some(crop()));
        let laid = req.wrap_box.expect("a multi-line click must be boxed");
        assert!(promoted, "and the operator is owed the sentence saying so");
        assert!(
            (laid.llx - 100.0).abs() < 1e-9,
            "the box starts where they clicked, not at the margin: {laid:?}"
        );
        assert!(
            (laid.urx - crop().urx).abs() < 1e-9,
            "and runs to the sheet's own right edge — a width read, not invented: {laid:?}"
        );
        assert!(
            (laid.ury - 700.0).abs() < 1e-9,
            "its TOP is the click, because `with_box` anchors the first line from the top \
             edge — so the first line begins where they pressed: {laid:?}"
        );
        assert!(
            (laid.lly - crop().lly).abs() < 1e-9,
            "and it reaches the bottom of the sheet: {laid:?}"
        );
    }

    /// ★★ **The newline survives to the engine.**
    ///
    /// The fact the whole feature rests on: `with_box` splits on `\n` and wraps
    /// each paragraph independently. A build that joined the lines with a space
    /// on the way here would author plausible, wrong, single-line text — which
    /// is the failure mode O127 named in advance (*"rather than silently
    /// joining with spaces"*).
    #[test]
    fn the_hard_newline_reaches_the_engine_intact() {
        let (req, _) = request(&placed("first\nsecond", None), Some(crop()));
        assert!(
            req.text.contains('\n'),
            "the break must survive: it is what splits the paragraphs"
        );
    }

    /// ★★★ **A dragged box reaches the engine as origin AND EXTENT.**
    ///
    /// The transposition that compiles. The action carries corners, because
    /// that is what a dragged rectangle is; `with_box` takes `(x, y, w, h)`.
    /// Getting it wrong puts the text somewhere plausible and wrong, on a page
    /// nobody will look at again until it is printed.
    #[test]
    fn a_dragged_box_reaches_the_engine_as_origin_and_extent() {
        let (req, promoted) = request(
            &placed("note", Some((100.0, 200.0, 340.0, 290.0))),
            Some(crop()),
        );
        let laid = req.wrap_box.expect("a dragged box is boxed");
        assert!((laid.llx - 100.0).abs() < 1e-9, "{laid:?}");
        assert!((laid.lly - 200.0).abs() < 1e-9, "{laid:?}");
        assert!(
            (laid.urx - 340.0).abs() < 1e-9,
            "240 of width, not 340 — the subtraction happens once, here: {laid:?}"
        );
        assert!((laid.ury - 290.0).abs() < 1e-9, "{laid:?}");
        assert!(
            !promoted,
            "the operator drew this rectangle, so there is nothing to disclose"
        );
    }

    /// **A page whose rectangle cannot be read declines to invent one.**
    ///
    /// The engine then refuses by page index, in its own words, which is a
    /// sentence about the real problem. A guessed cropbox would turn it into a
    /// sentence about a rectangle this shell made up.
    #[test]
    fn no_cropbox_means_no_invented_box() {
        let (req, promoted) = request(&placed("first\nsecond", None), None);
        assert!(req.wrap_box.is_none());
        assert!(!promoted);
    }

    /// **A click past the sheet's edge falls back rather than authoring a
    /// degenerate box.**
    ///
    /// A zero- or negative-width rectangle is a request the engine would refuse
    /// for a reason unrelated to what the operator did. Reaching for a minimum
    /// width would be inventing the one number this module refuses to invent.
    #[test]
    fn a_click_outside_the_sheet_falls_back_to_a_point() {
        let mut p = placed("first\nsecond", None);
        p.origin = (700.0, 700.0);
        let (req, promoted) = request(&p, Some(crop()));
        assert!(req.wrap_box.is_none(), "no box, rather than an empty one");
        assert!(!promoted);
    }

    /// ★ **The pen reaches the engine in every one of the three cases.**
    ///
    /// The failure this arm's own history names: *"two branches each building a
    /// request would be two places for a font to be forgotten."* There are now
    /// three exits, so the property is asserted across all of them rather than
    /// argued for in a comment.
    #[test]
    fn every_route_carries_the_pen() {
        let pen = TextPen::default();
        for wrap in [None, Some((10.0, 10.0, 200.0, 60.0))] {
            for text in ["one line", "two\nlines"] {
                let (req, _) = request(&placed(text, wrap), Some(crop()));
                assert!(
                    (req.size - pen.size()).abs() < 1e-9,
                    "the pen's size was dropped for wrap={wrap:?} text={text:?}"
                );
            }
        }
    }
}
