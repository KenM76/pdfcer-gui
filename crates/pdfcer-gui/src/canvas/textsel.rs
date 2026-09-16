//! # `canvas::textsel` — selecting text on the page, and copying what was selected
//!
//! The operator's rule for Read mode is *"the document shouldn't allow editing
//! and should allow only selecting of objects that acrobat reader would
//! allow."* `app::modes::capability` owns the first half — Read refuses every
//! content gesture. This module is the second half, and it is a widening rather
//! than a narrowing: **Reader allows text selection**, so a Read mode that
//! refused it would be less than "only what Reader allows", not a cautious
//! reading of it.
//!
//! The gesture is also what makes the text-marking annotations reachable.
//! Underline, strikeout and squiggly mark *text*, and
//! `pdfcer_core::annot_author::MarkupSpec::TextMarkup` takes a `Vec<Quad>` —
//! which is [`TextSelection::page_quads`], produced by the same pass over the
//! same glyphs as the canvas boxes (§5.1). Nothing here authors anything; the
//! authoring lives in [`crate::canvas::markup::text`], and §6 says why.
//!
//! ---
//!
//! ## 1. ★ The interaction decisions, and which application each came from
//!
//! The operator's standing instruction is to *"make your best educated guesses
//! to match what inkscape, acrobat, and SolidWorks do"*, recording which one was
//! followed and why, and — where they disagree — saying which won. **Acrobat
//! wins ties about *reading*, because Acrobat is what pdfcer replaces.**
//!
//! | Question | Acrobat | Inkscape | SolidWorks | Shipped | Why |
//! |---|---|---|---|---|---|
//! | drag selects a **range** or a **rectangle** | both (range default, `Alt` for rectangle) | range | range | **range** | unanimous on the default; the rectangle is deferred, §2 |
//! | **double**-click | word | word | word | **word** | unanimous |
//! | **triple**-click | paragraph | line | line | **line** | §1.1 — the disagreement is the interesting part |
//! | crosses **columns** | yes | n/a (one text object) | n/a (one note) | **yes** | falls out of content order, §4 |
//! | crosses **pages** | yes | n/a | n/a | **no** | §4 — and it is a cost decision, stated as one |
//! | visible **caret** | no (Select tool) | yes (text tool) | yes (note editing) | **no** | §1.2 |
//! | **Escape** | clears | deselects | leaves the field | **clears** | unanimous |
//! | **Ctrl+A** | all text | all objects | all in field | **all text on the page** | §1.3 |
//! | **Ctrl+C** | copies | copies | copies | **copies** | unanimous |
//! | Shift+click | extends from the anchor | extends | extends | **extends** | unanimous |
//!
//! ### 1.1 Triple-click: Acrobat says paragraph, and this ships a line
//!
//! The one row where the reference applications disagree and Acrobat **did not
//! win**, so it needs the argument.
//!
//! Acrobat selects a paragraph. Inkscape and SolidWorks select a line. A PDF
//! content stream contains neither: `pdfcer-core`'s own extraction documentation
//! is blunt that lines are **derived** (its S5 sourcing note — *"no line or
//! paragraph markers exist in a content stream, in tagged or untagged files
//! alike"*), and paragraphs are derived a second time, from the lines, by
//! `EditableTextModel`'s block recognition using a leading-gap ratio and an
//! indent ratio.
//!
//! So the choice is between a unit this engine derives once and a unit it
//! derives twice. A triple-click that selected a *block* would be the
//! operator's most emphatic gesture resolved through the shakiest inference in
//! the stack, and when it got the paragraph wrong — on a drawing sheet's title
//! block, where "paragraph" means very little — there would be no smaller unit
//! to fall back to, because the double-click below it is a word. The line is
//! the honest middle rung, it is what two of the three reference applications
//! do, and [`EditableTextModel::line_range_at`] is a published verb for it
//! where a block range is not.
//!
//! ### 1.2 No caret, and that is a statement rather than an omission
//!
//! Acrobat Reader's Select tool draws a highlight and **no blinking caret**; a
//! caret appears only where something can be typed (a form field). Inkscape and
//! SolidWorks both draw one, and both are *editing* text when they do.
//!
//! Reading is the subject here, so Acrobat wins — and there is a second,
//! stronger reason that is about this shell rather than about convention: a
//! caret promises an **insertion point**, and there is nothing to insert.
//! Phase 5 (in-place text editing) is last in the operator's order and must not
//! be started early. A caret drawn before it exists is an affordance for a
//! feature that does not, which is the no-placeholders invariant read straight
//! (`PROJECT_PLAN.md` §3).
//!
//! `pdfcer-core` already publishes everything a caret needs
//! ([`EditableTextModel::caret_x`], `caret_left`, `caret_right`, `caret_up`,
//! `caret_down`) — written for Phase 5. None of it is called here. That is
//! where a caret comes from when there is something to type into.
//!
//! ### 1.3 Ctrl+A means "everything this gesture can select"
//!
//! Acrobat selects all the text; Inkscape selects all the objects in the layer;
//! SolidWorks selects everything in the field being edited. All three are the
//! same rule — *select everything the thing you are currently selecting in* —
//! and this shell applies it: where a press selects text, Ctrl+A selects the
//! page's text ([`select_all`]).
//!
//! **The other half is deliberately absent and is named rather than implied:**
//! in a mode that selects page content there is no select-all, because
//! `canvas::selection` has no "every object on the page" verb and inventing one
//! inside a keyboard handler would put a selection rule somewhere other than
//! the module that owns selection rules. So Ctrl+A does nothing in Edit: one
//! honest gap, with the shape of the fix recorded here.
//!
//! ---
//!
//! ## 2. What a drag does, and the rectangle that is not built
//!
//! A drag selects the **range** between where the button went down and where
//! the pointer is — in the engine's content order, which is what makes it flow
//! round line ends and across a column break rather than sweeping a box.
//!
//! Acrobat's second mode — `Alt`+drag for a **rectangular** text selection — is
//! genuinely useful on the drawing sheets this application exists for, where a
//! parts table's column is a rectangle and emphatically not a range. It is
//! **not built**, and the reason is this module's first rule: one derivation, so
//! what is shown and what is copied cannot diverge. A
//! rectangular selection is a second selection model — its copy is column-wise,
//! its reading order is its own, and it cannot be expressed as a
//! `(TextPosition, TextPosition)` pair at all — so it would be a second
//! [`resolve`] with a second quad derivation and a second copy path beside it.
//! That is the divergence this module is built to make impossible, bought for a
//! modifier.
//!
//! What it would take, so the next hand does not re-derive it: a
//! `Selection::Rect(egui::Rect)` variant beside the `(anchor, focus)` pair
//! [`TextSelection`] carries today, resolved by filtering
//! `PageText`'s glyphs on their own geometry rather than by
//! [`EditableTextModel::resolve_range`], with the copy assembled per line from
//! the surviving glyphs. Both variants would then have to flow through one
//! `resolve` returning one [`TextSelection`], which is what keeps the promise
//! above.
//!
//! ---
//!
//! ## 3. ★ THE MODE GATE
//!
//! **[`gate`], and its header is the whole argument** — why text selection needs
//! no capability, why it still has to be told apart from the content marquee,
//! and what the rule yields mode by mode. [`takes_the_press`] and the tests that
//! are about it live there too.
//!
//! The one-line version, so a reader here is not sent away for nothing:
//!
//! > **A press means text when the text tool is armed, *or* when the select tool
//! > is active and the mode cannot select content** — and selecting text needs
//! > no capability at all, because it authors nothing.
//!
//! ---
//!
//! ## 4. One page, and content order
//!
//! **A selection is a range on one page.** Acrobat's crosses pages; this one
//! does not, and it is a cost decision rather than a taste one.
//!
//! `crate::find`'s header carries the measurement: a whole-document extraction
//! is 331–449 ms on this project's fixtures, which is why Find never searches
//! on a keystroke. A cross-page selection needs a document-wide index — every
//! page walked, tokenised and font-resolved — and it needs it live, because a
//! drag samples the pointer sixty times a second. Per **page** it is one
//! extraction cached on `(page, edit epoch)` and free thereafter
//! (`app::cache::PageTextCache`); per **document** it is Find's number
//! paid again on every page turn, to make a gesture work that ends at the
//! window edge anyway. The anchor and the focus would also live on pages with
//! different [`crate::canvas::mapping::PageMapping`]s, which `canvas::interact`
//! is single-page by construction and says so.
//!
//! **Columns are a different matter and need nothing.** `PageText::runs` is in
//! page content order, and the engine inserts a derived line break at a
//! backward horizontal jump — its `backward_jump_ratio`, which exists because
//! *"a two-column page whose columns share baselines runs the two columns
//! together into one line with no separator at all"*. So a drag from the first
//! column into the second selects everything between them in content order,
//! with the columns separated, which is what Acrobat does. Neither this module
//! nor the operator has to know a column existed.
//!
//! Content order is **not** appearance order, and the engine says so
//! (§14.8.2.3.1: the two orderings *"may or may not coincide"*). A file whose
//! producer emitted its text out of visual order will select out of visual
//! order. That is the file's ordering, faithfully reported; inventing a
//! geometric reading order here would be a third derivation on top of two.
//!
//! ---
//!
//! ## 5. ★ One derivation: what is highlighted IS what is copied
//!
//! The requirement, and the defect it names: the highlight is drawn from the
//! same quads the copy uses — **one derivation, so what is shown and what is
//! copied cannot diverge.**
//!
//! [`resolve`] is that one derivation. It takes the ordered pair of
//! [`TextPosition`]s **once**, walks the covered runs **once**, and in that
//! single pass produces both halves of [`TextSelection`]: the string is sliced
//! from the runs' own text as the walk passes through them, and the boxes are
//! accumulated from the glyphs inside the same byte windows. There is no second
//! entry point, no "recompute the quads for drawing", and no way to ask for one
//! without the other — [`TextSelection`]'s fields are populated together or the
//! value does not exist.
//!
//! That also makes the highlight free to draw. The quads are stored in **canvas
//! space**, which is zoom-independent, so a frame that merely paints an
//! existing selection runs no extraction, builds no model and does no
//! geometry — the same property `canvas::selection` relies on for its outlines
//! and `crate::find::Hit::canvas` for its wash.
//!
//! ### 5.1 ★ The same pass produces a THIRD output, and that is why
//!
//! [`TextSelection`] carries its boxes twice: [`TextSelection::quads`] in
//! **canvas space**, which is what the overlay paints, and
//! [`TextSelection::page_quads`] in **PDF user space**, which is what a
//! `/QuadPoints` text markup is authored from ([`crate::canvas::markup::text`]).
//! Both are `boxes` — the one `Vec` accumulated in [`resolve`]'s single walk —
//! and neither can exist without the other.
//!
//! It would have been one field fewer to store the canvas boxes alone and let
//! the authoring site invert [`crate::viewer::canvas_to_pdf_space`] over their
//! corners. That is refused, for two reasons and the second is the one that
//! decides it:
//!
//! * **It is a second derivation of the geometry**, arriving through the door
//!   §5 exists to lock. The rule is not *"do not extract twice"* — it is that
//!   what is shown and what is committed must be the same value, and two
//!   spellings of the same projection are exactly how they come to differ.
//! * **The inverse is not the identity on a rotated page.** The forward hop is
//!   `find::reveal::quad_to_canvas`, which maps all four corners and takes their
//!   bounds precisely because `/Rotate 90` sends `ul`/`lr` to two corners that
//!   are no longer the extremes. Inverting a *bounded* rect corner by corner
//!   gives back two opposite corners in an order `Rect::from_corners` is not
//!   promised to normalise — a mark that lands mirrored about the page's centre
//!   line, in the file, discovered after saving. That is the failure
//!   [`crate::canvas::markup`]'s own §1 is built around, reintroduced by an
//!   optimisation worth eight bytes a line.
//!
//! ### Why the highlight stays readable
//!
//! The rule `overlay::CURRENT_ALPHA` carries is general: *the operator's next act
//! after finding a hit is to READ it*, so a wash opaque enough to cover the word
//! it marks has failed at its job.
//!
//! It applies here with more force, not less — a selection is what you are
//! about to copy, and an operator who cannot read it cannot tell whether they
//! swept the right words. So the selection wash reuses the same themed colour at
//! the same low end (`overlay::TEXT_SELECTION_ALPHA`), under the compile-time
//! bound that keeps it there, and it is drawn **unstroked**: Find strokes
//! its current hit to distinguish it from its neighbours, and a text selection
//! has no neighbours to be distinguished from. A stroke per line box would also
//! draw a visible seam between two lines of one selection, which is a boundary
//! the operator did not make.
//!
//! ### The glyph box, and the constant that had two candidates
//!
//! A glyph carries an origin, an advance and a size — not a box. `pdfcer-core`
//! approximates one in two places and **they do not agree**:
//!
//! | site | ascent | descent |
//! |---|---|---|
//! | `EditSession`'s search quad (what `TextMatch::quad` is) | `+0.85 × size` | `−0.22 × size` |
//! | `TextRun::bbox` and `Line::bbox` | `+0.75 × size` | `−0.25 × size` |
//!
//! There is no shared constant to inherit, so it is a choice, and it is made
//! **for the search quad's numbers**: `crate::find` draws its highlights from
//! `TextMatch::quad`, and Find is the surface an operator will see next to this
//! one — searching for a word and then selecting the same word must not produce
//! two boxes of visibly different heights over the same glyphs. Matching the
//! *bbox* numbers would instead match a box nothing paints.
//!
//! ---
//!
//! ## 6. This module authors nothing, and neither does copying
//!
//! No function here takes `&mut EditSession`, raises an
//! [`crate::app::actions::Action`], or bumps `edit_epoch`. The selection lives
//! on [`crate::app::state::OpenDoc`] beside the object selection, which
//! `canvas`'s header already argues is not a document mutation: *"a selection
//! *names* parts of a document and changes nothing a save would write."*
//!
//! Copying is the same class one step further out: it reads the extraction and
//! calls `egui::Context::copy_text`. It is not routed through the action funnel
//! for the reason `file.print` is not — the funnel exists for work that touches
//! a document or must not happen mid-frame, and this is neither.
//!
//! Rule 4 (disclosure lives off-canvas) is satisfied the way `canvas::overlay`'s
//! header states it: with nothing selected this paints nothing at all, so *would
//! a screenshot of the canvas differ from a screenshot of the same document
//! saved and reopened?* answers no by construction. A selection wash is a
//! pre-commit affordance — the cursor, describing what a copy would take — in
//! exactly the category rule 4 admits alongside the rubber band and the snap
//! indicator.
//!
//! ## 7. Staleness
//!
//! A [`TextPosition`] is `(run index, byte offset)` **into a particular
//! extraction**. An edit re-writes content streams, so run indices renumber and
//! byte offsets move: a position recorded before an edit can name different
//! glyphs, no glyphs, or the right glyphs in the wrong place. That is Find's
//! staleness problem exactly, and `crate::find`'s header rejects the same two
//! wrong answers — re-resolving automatically (an extraction per edit) and
//! drawing the old geometry anyway (*"a highlight that may be over the wrong
//! text, which is the one thing rule 4 forbids outright"*).
//!
//! Find keeps the query and drops the geometry, because a query is something the
//! operator typed. A selection has no such half: it **is** geometry. So the
//! whole thing is dropped — [`TextSelection::epoch`] records the revision it was
//! resolved against, [`TextSelection::live`] answers `false` the instant that
//! moves, and the overlay is handed nothing.
//!
//! ★ **Authoring a text markup is itself an edit**, so marking a selection
//! makes that selection stale on the very next frame: `add_markup` goes through
//! `vector_edit`, which bumps `edit_epoch`, and the wash disappears. Acrobat
//! keeps its selection across a markup and this does not, which is a real
//! difference and is recorded rather than smoothed over. The alternative is a
//! second staleness rule — *"an edit that adds an annotation does not move the
//! text"* — living outside this module and free to disagree with the one here;
//! the epoch is the only signal there is, and refining it into kinds of edit is
//! a mechanism, not a line. What the operator loses is one re-sweep to underline
//! *and* strike out the same words.
//!
//! ## 8. ★★ Text that does not run along the page's x axis
//!
//! A vertical file-path stamp in a CAD title block is the case that names the
//! two failures, in the operator's words: *"the I cursor doesn't reorient and it
//! pastes each letter onto its own line."*
//!
//! The engine owns the hard half. `pdfcer_core::text_edit::Line::direction` is
//! the unit vector taken from the §9.4.4 text rendering matrix, shared by every
//! glyph on the line by construction, and the extraction resolves a baseline
//! step into the **line's own frame** rather than into page axes — so a 90° line
//! is one line and not one derived break per letter. What is left to this module
//! is banding: which frame a glyph's cell is measured in.
//!
//! | rule | rotated text |
//! |---|---|
//! | §5, one derivation | **holds, and it is what makes the rotated path safe.** The direction is consulted once, inside [`resolve`], and both the boxes and the string are built in that same walk |
//! | §4, content order | **holds.** Nothing is reordered; a rotated line is the same glyphs in the same order, banded differently |
//! | box shape | a rotated line's glyph cells are accumulated **in the line's own frame** (`bands::Band::Rotated`) and emitted as one banded [`Quad`]; a horizontal line's are accumulated in page axes |
//! | the copied string | unfiltered. Every run the extraction emits is copied, because the extraction emits no derived break inside a rotated line for this module to have to skip |
//!
//! ★ **A page with no rotated text never reaches any of it.** `is_rotated`
//! answers `false` for every line, every glyph takes the page-axis branch, and
//! no line frame is ever built. That is deliberate and structural: one grouping
//! rule handling both cases would put every ordinary document's selection
//! through the rotated code to serve a minority of drawing sheets.
//!
//! ★ The canvas wash is a `Rect`, so for a **quadrant** rotation (90°, 180°,
//! 270° — every rotated stamp a CAD exporter emits) the band is axis-aligned in
//! page space and the wash covers it exactly. At an arbitrary angle the band is
//! a parallelogram and the wash is its bounding box, which over-covers at the
//! corners. The authored `/QuadPoints` are the true parallelogram either way,
//! because [`TextSelection::page_quads`] carries corners rather than bounds.
//!
//! ## 9. Where the rest of this module is
//!
//! The two chords (`Ctrl+A`, `Ctrl+C`), the guard in front of them and the one
//! function that writes the clipboard live in [`clipboard`], re-exported flat so
//! every call site still writes `textsel::copy` and `textsel::pending_key`.
//! The seam is there rather than anywhere else because [`clipboard::copy`] is
//! reached by two **ribbon commands** that have no selection at all — see that
//! module's header.

use std::collections::HashMap;

use egui::{Pos2, Rect};
use pdfcer_core::annot_author::Quad;
use pdfcer_core::page_tree::{Page, Rect as PdfRect};
use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel, TextPosition};
use pdfcer_core::text_extract::PageText;

use bands::{Accum, Band};

/// The two keyboard verbs and the clipboard write. See §9.
pub mod clipboard;

/// **Who owns the primary button** — the mode gate and the whole argument for
/// it. See §3.
pub mod gate;

/// **How a selection's glyph cells become the boxes it paints and marks** — the
/// accumulation half of §5, in the two frames §8 made necessary.
mod bands;

/// **The rotated-text page §8's rules are tested on** — test-only.
#[cfg(test)]
pub mod fixture;

pub use clipboard::{TextKey, apply_key, copy, pending_key};

/// Re-exported flat, so every call site still writes `textsel::takes_the_press`
/// and nothing outside `canvas/` learns that the module was split. The same
/// contract [`clipboard`]'s re-export above honours, for the same reason.
pub use gate::takes_the_press;

/// How far above the baseline a glyph's box reaches, as a fraction of its
/// effective font size.
///
/// `pdfcer-core`'s **search-quad** number, deliberately, where its run and line
/// boxes use `0.75`. See the module header §5: `crate::find` paints
/// `TextMatch::quad`, and a selected word must not be a visibly different height
/// from the same word found. There is no shared constant in core to inherit, so
/// this is a decision rather than a reference.
const GLYPH_ASCENT: f32 = 0.85;

/// How far below the baseline a glyph's box reaches, as a fraction of its
/// effective font size. The descender half of [`GLYPH_ASCENT`]'s pairing.
const GLYPH_DESCENT: f32 = 0.22;

/// **A range of characters on one page, and everything derived from it.**
///
/// The two halves of §5's promise travel together: [`Self::quads`] is what the
/// overlay paints and [`Self::text`] is what a copy writes, and both are
/// produced by one pass of [`resolve`] over one ordered pair of positions.
/// There is no constructor that fills one without the other.
///
/// Empty selections do not exist as values: [`resolve`] returns `None` when the
/// range covers no glyphs, so a plain click — which collapses the range —
/// clears the field rather than storing a selection with nothing in it. That is
/// what makes `Option<TextSelection>` on the document a two-state question
/// instead of a three-state one.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSelection {
    /// Which page the range is on. A selection is single-page — module header
    /// §4 — so this is a fact about the whole value rather than about one end.
    pub page: usize,
    /// Where the gesture started. Held so a drag or a Shift+click can extend
    /// **from** it: the anchor is the end the operator is not moving, and
    /// re-deriving it from the quads would be impossible once the focus has
    /// crossed it.
    anchor: TextPosition,
    /// Where the pointer is now. The end a drag moves.
    focus: TextPosition,
    /// The [`crate::app::state::OpenDoc::edit_epoch`] the positions above were
    /// resolved against. See the module header §7 — this is the whole of the
    /// staleness mechanism.
    epoch: u64,
    /// The selected glyphs' boxes, **in canvas space**, one per line of the
    /// selection.
    ///
    /// Canvas space (Y-down, page top-left, `/Rotate` applied) rather than PDF
    /// user space, and projected once here rather than per frame, for the
    /// reason `crate::find::Hit::canvas` gives for doing the same: page
    /// geometry cannot change while a document is open, so the answer is
    /// constant for the life of the selection, and the paint path becomes a
    /// projection with no PDF concepts in it at all.
    ///
    /// One box per line rather than one per glyph — a hundred adjacent
    /// rectangles paint as one band anyway, and merging them is what lets a
    /// selection over a paragraph cost four boxes instead of four hundred.
    pub quads: Vec<Rect>,
    /// ★ **The same boxes, in PDF user space** — ready to become a text
    /// markup's `/QuadPoints`.
    ///
    /// One entry per entry of [`Self::quads`], in the same order, from the same
    /// accumulation in [`resolve`]. Not a conversion *of* that field and not a
    /// second walk: the walk produces one `Vec` of PDF-space rectangles and both
    /// of these are built from it, which is what makes *"what is highlighted is
    /// what is marked"* true by construction rather than by two functions
    /// agreeing. Module header §5.1 carries the argument, including why
    /// inverting the canvas projection at the authoring site is the wrong answer
    /// on a rotated page.
    ///
    /// `Quad` rather than `Rect` because that is the type
    /// [`pdfcer_core::annot_author::MarkupSpec::TextMarkup`] takes, and building
    /// it here — once, from the rectangle the glyphs actually produced — leaves
    /// the authoring site with nothing geometric to decide.
    pub page_quads: Vec<Quad>,
    /// **Exactly the characters those boxes cover**, ready for the clipboard.
    ///
    /// Includes the engine's derived word spaces and line breaks, because they
    /// are runs in their own right and the walk passes straight through them —
    /// which is what makes a copied paragraph read as a paragraph rather than
    /// as one unbroken word.
    pub text: String,
}

impl TextSelection {
    /// Whether this selection still describes the revision it was made
    /// against.
    ///
    /// The gate the overlay and every copy path ask before spending it. See the
    /// module header §7: after an edit the positions inside name runs that have
    /// moved, and painting the stored quads anyway is the one thing rule 4
    /// forbids outright.
    #[must_use]
    pub fn live(&self, epoch: u64) -> bool {
        self.epoch == epoch
    }

    /// ★★ **Which runs of the page's extraction this selection covers**, low
    /// to high, or nothing when the revision has moved.
    ///
    /// The operand of every restyle. `crate::app::actions::textstyle` turns
    /// this list into one `format_text` call per run.
    ///
    /// # Why a list of ordinals and not the two `TextPosition`s
    ///
    /// The positions are the *anchor* and the *focus*, which are in gesture
    /// order — the focus is behind the anchor on a right-to-left sweep. Every
    /// consumer outside this module wants content order, and half of them would
    /// get the ordering wrong exactly once. `ordered` already exists here and is
    /// already the one place that decides it.
    ///
    /// # ★ Why the byte offsets are dropped
    ///
    /// `format_text` restyles **one whole show operator**. There is no verb that
    /// restyles half of one, so a caller handed byte offsets could only ignore
    /// them or misuse them. Publishing exactly what the engine can act on is
    /// what stops a panel implying a precision the file cannot carry — a sweep
    /// through the middle of a word restyles the word, and the shell must not
    /// pretend otherwise.
    ///
    /// # ★ Why the staleness gate is here and not left to the caller
    ///
    /// Same rule as [`Self::highlights`] and for a worse reason: a stale quad
    /// paints a wash in the wrong place, and a stale run ordinal **restyles the
    /// wrong text**. The caller stops by being handed an empty list rather than
    /// by remembering a check.
    #[must_use]
    pub fn runs(&self, epoch: u64) -> Vec<usize> {
        if !self.live(epoch) {
            return Vec::new();
        }
        let (start, end) = ordered(self.anchor, self.focus);
        (start.run..=end.run).collect()
    }

    /// The quads to paint on `page`, or nothing at all.
    ///
    /// Nothing when the page is not this selection's, and nothing when the
    /// revision has moved — so the overlay stops drawing by being handed an
    /// empty slice rather than by a check of its own, exactly as
    /// `find::FindState::page_highlights` arranges.
    #[must_use]
    pub fn highlights(&self, page: usize, epoch: u64) -> &[Rect] {
        if self.page == page && self.live(epoch) {
            &self.quads
        } else {
            &[]
        }
    }

    /// A selection built from nothing but a page, a revision and a list of
    /// page-space boxes — for the tests of the modules that **consume** one.
    ///
    /// # Why this exists rather than a fixture
    ///
    /// [`resolve`] is the only constructor, deliberately (see the type's docs),
    /// and it needs a `PageText` — which `pdfcer-core` makes
    /// `#[non_exhaustive]`, so this crate cannot build one and every test here
    /// drives a real extraction of a real file. That is right for *this* module
    /// and wrong for [`crate::canvas::markup::text`], whose rules are about a
    /// selection's **page, revision and boxes** and nothing else: forcing it to
    /// open a fixture and hunt for a page whose glyphs happen to sit where the
    /// assertion needs them would make its tests slower, flakier and about the
    /// fixture instead of about the rule.
    ///
    /// `#[cfg(test)]`, so it cannot become a second production constructor —
    /// which is the property that keeps "an empty selection is `None`" true of
    /// every value the application can actually hold.
    ///
    /// The canvas boxes are filled with the page boxes' own numbers rather than
    /// a projection, because there is no page here to project through. They are
    /// therefore **not** what a real selection would paint; what is faithful is
    /// the one property a consumer depends on — that the two vectors have the
    /// same length and the same order.
    #[cfg(test)]
    #[must_use]
    pub fn for_test(page: usize, epoch: u64, page_quads: Vec<Quad>) -> Self {
        let quads = page_quads
            .iter()
            .map(|q| {
                Rect::from_min_max(
                    Pos2::new(q.ll.0 as f32, q.ll.1 as f32),
                    Pos2::new(q.ur.0 as f32, q.ur.1 as f32),
                )
            })
            .collect();
        Self {
            page,
            anchor: TextPosition::new(0, 0),
            focus: TextPosition::new(0, 0),
            epoch,
            quads,
            page_quads,
            // Not read by anything this constructor exists for; a copy is what
            // `resolve` produces from real runs, and inventing plausible prose
            // here would make a test look like it was about the text when it is
            // about the geometry.
            text: String::new(),
        }
    }

    /// ★ **The quads a text markup would be authored from**, or nothing at all.
    ///
    /// [`Self::highlights`]'s twin, and deliberately the same shape: the caller
    /// is handed an empty slice rather than being asked to check a revision for
    /// itself, so a stale selection cannot be marked by a caller who forgot —
    /// which is the *"a highlight that may be over the wrong text"* failure
    /// (module header §7) with an annotation written into the file instead of a
    /// wash drawn over it.
    ///
    /// There is no page argument, where [`Self::highlights`] takes one: the
    /// overlay draws a *particular* page and has to be told which, while an
    /// authoring caller is asking *"where would this go"* and the answer
    /// includes [`Self::page`]. Handing back the quads without the page would be
    /// the invitation to pair them with `doc.view.page_index`, which is the
    /// current page and not necessarily this selection's.
    #[must_use]
    pub fn marks(&self, epoch: u64) -> &[Quad] {
        if self.live(epoch) {
            &self.page_quads
        } else {
            &[]
        }
    }

    /// How many characters are selected. For the trace line and for tests.
    ///
    /// Byte length rather than a `char` count, deliberately and to match the
    /// `chars=` trace field: it is the length of the string a copy puts on the
    /// clipboard, so a trace and a clipboard cannot disagree, and it is the unit
    /// `TextPosition` already speaks (`pdfcer-core` keys glyphs by byte offset
    /// because one code may decode to many code points).
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the selection covers nothing.
    ///
    /// Always `false` for a value that exists — [`resolve`] returns `None`
    /// rather than an empty selection, which is the invariant everything else
    /// here rests on. It is written anyway because clippy asks for it beside a
    /// `len`, and asking for it is right: a reader meeting `len()` is entitled
    /// to the companion, and the honest implementation *states* the invariant
    /// instead of leaving it to be inferred from four call sites.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// The facts about one page that every entry point below needs.
///
/// A struct rather than four parameters for the reason
/// `canvas::interact::Frame` is one: they are settled together, once, by the
/// caller that has the document, and passing them separately invites a call
/// site to fetch one of them for itself — which for `text` would mean a second
/// extraction and for `epoch` would mean a selection that outlives the
/// revision it describes.
#[derive(Clone, Copy)]
pub struct PageContext<'a> {
    /// The page's extracted text, from
    /// [`crate::app::state::OpenDoc::page_text`]. **The** extraction — see
    /// that method's docs on why there is only one.
    pub text: &'a PageText,
    /// The page itself, for the PDF-user-space → canvas projection.
    pub page: &'a Page,
    /// Which page this is, in the session's page space.
    pub index: usize,
    /// The revision the extraction describes, stamped onto any selection this
    /// produces. Module header §7.
    pub epoch: u64,
}

/// **Run one frame of a sweep gesture against the open document**, and trace it.
///
/// The body of [`crate::canvas::interact`]'s `GestureOutcome::TextSelect` arm.
/// It lives here rather than there because the arm is *wiring*: every rule it
/// applies — which extraction options, what a degenerate drag means, when a
/// range has changed enough to trace — is a rule this module owns.
///
/// # Returns
///
/// The new selection, or `None`. ★ `None` is returned when the page has **no
/// extractable text or no such page**, and the caller must assign it: a sweep
/// over a page with nothing on it clears whatever was selected, which is what
/// the operator asked for by sweeping there. Returning the *old* selection on a
/// miss would make a sweep across a blank page do nothing at all, which reads as
/// the gesture being broken rather than as there being nothing to select.
///
/// # ★ Why the trace fires on every frame and not only at the release
///
/// `trace::text_selection` collapses the frames where the range did not move, so
/// what reaches the channel is the sequence of *distinct* states the selection
/// passed through. A harness watching `chars=` grow can see the sweep happen;
/// one line at the end could only see that it had.
#[must_use]
pub fn sweep(
    doc: &crate::app::state::OpenDoc,
    page_index: usize,
    from: Pos2,
    to: Pos2,
    phase: crate::canvas::gesture::Phase,
) -> Option<TextSelection> {
    let selection =
        if let (Some(page_text), Some(page)) = (doc.page_text(), doc.pages.get(page_index)) {
            // ★ THE page's extraction, from `OpenDoc::page_text` — never a fresh
            // one built here. A second extraction with its own options would
            // segment lines differently from the one the page was drawn and
            // cached from, so a sweep would select against text that does not
            // match what is on screen.
            let ctx = PageContext {
                text: &page_text,
                page,
                index: page_index,
                epoch: doc.edit_epoch,
            };
            drag(&ctx, from, to)
        } else {
            None
        };
    crate::canvas::trace::text_selection(
        page_index,
        selection.as_ref(),
        if matches!(phase, crate::canvas::gesture::Phase::Complete) {
            "drag" // ui-text-exempt: trace token, never displayed
        } else {
            "sweep" // ui-text-exempt: trace token, never displayed
        },
    );
    selection
}

/// ★★★ **Re-resolve a selection against the revision that just replaced it.**
///
/// What makes a restyle repeatable without re-sweeping. Pressing Bold is an
/// edit; an edit bumps [`crate::app::state::OpenDoc::edit_epoch`];
/// [`TextSelection::live`] then answers `false`, the wash vanishes and
/// [`TextSelection::runs`] returns an empty list. Without this, pressing Italic
/// straight afterwards restyles nothing and the operator has to sweep the same
/// words again between every pair of presses (`OPERATOR_REQUESTS.md` O198).
///
/// # ★★★ THE STALENESS RULE IS NOT RELAXED. THE GEOMETRY IS REBUILT.
///
/// Module header §7 rejects two wrong answers, and this is neither of them.
/// It does **not** re-stamp the old selection with a new epoch — that is
/// "draw the old geometry anyway", and a restyle that changed a point size
/// moves every glyph after it, so the stored quads would wash the wrong pixels.
/// It does not re-resolve on a timer or per frame either. It re-runs the FULL
/// resolution — a fresh extraction, a fresh range, fresh quads, fresh text —
/// from the two positions the operator's own gesture set, at exactly one
/// moment: immediately after an edit that claimed not to change the text.
///
/// # ★★★ THE GUARD, AND WHY IT IS THE COVERED CHARACTERS
///
/// A restyle changes how text looks and never what it says. So the covered
/// string is an invariant the caller can check, and this function checks it:
/// **if the re-resolved selection does not cover character-for-character what
/// the old one covered, `None` is returned and the selection is dropped.**
///
/// That is what makes this safe against the thing module header §7 is really
/// afraid of — a `(run, byte)` position naming different glyphs after the run
/// indices renumber. If the engine ever splits, merges or re-orders runs on a
/// restyle, the covered text moves and this declines. The shell does not have
/// to know whether `format_text` renumbers; it measures.
///
/// ★★ It is deliberately NOT a check that the run ORDINALS are unchanged. A
/// producer that re-emits a title block as two operators instead of three has
/// renumbered nothing the operator can see, and the characters are the thing
/// the operator swept.
///
/// # Returns
///
/// `None` when the positions no longer resolve to anything, when they resolve
/// to different characters, or when the page carries no extractable text. The
/// caller assigns the result, so `None` means "drop the selection" — reached by
/// measurement rather than by assumption.
#[must_use]
pub fn reresolve(ctx: &PageContext<'_>, previous: &TextSelection) -> Option<TextSelection> {
    if previous.page != ctx.index {
        return None;
    }
    let model = model(ctx);
    let renewed = resolve(&model, ctx, previous.anchor, previous.focus)?;
    (renewed.text == previous.text).then_some(renewed)
}

/// **Update the selection from a drag** — press at `from`, pointer now at `to`,
/// both in canvas space.
///
/// The anchor is re-derived from `from` on every frame rather than kept from the
/// press, and that is not laziness: `PointerFrame::press_origin` guarantees
/// `from` is *where the button actually went down* (its own header records the
/// 94-point error that guarantee exists to close), so re-deriving is exact, and
/// it removes the one piece of state a drag could otherwise carry across frames
/// and get wrong.
///
/// Returns `None` when the drag covers no glyphs — a sweep across blank paper
/// selects nothing rather than the nearest word, which is what Acrobat does and
/// what stops a stray drag on a drawing sheet's margin producing a selection the
/// operator did not make.
pub fn drag(ctx: &PageContext<'_>, from: Pos2, to: Pos2) -> Option<TextSelection> {
    let model = model(ctx);
    let anchor = hit(&model, ctx, from)?;
    // A pointer that has run off the end of the text does not cancel the
    // sweep; it stops extending it. See [`clamp_to_text`].
    let focus = match hit(&model, ctx, to) {
        Some(focus) => focus,
        None => clamp_to_text(&model, ctx, from, to)?,
    };
    resolve(&model, ctx, anchor, focus)
}

/// How many points along `from`..`to` are tried before the clamp gives up,
/// and how many halvings sharpen the one that answered.
///
/// A scan rather than a straight bisection because the reachable set along a
/// drag is not an interval: a sweep that crosses a gap between two columns
/// leaves reach and re-enters it, and a bisection seeded from the anchor would
/// stop at the near edge of the gap and silently under-select. Scanning
/// **backwards from the pointer** finds the furthest text the operator has
/// actually dragged past, which is the one they mean.
const CLAMP_SCAN: u32 = 32;
/// The reciprocal, stated rather than divided, so the sample positions are
/// exact f32 and no cast appears in the scan.
const CLAMP_STEP: f32 = 1.0 / 32.0;
const CLAMP_SHARPEN: u32 = 8;

/// **The furthest point along `from`..`to` that still lands in text.**
///
/// The clamp that makes overshooting a line harmless. Without it a sweep that
/// ran one line-height past the last character resolved no focus, [`drag`]
/// returned `None`, and the entire selection vanished mid-gesture — on a
/// drawing sheet, where a title-block run is 40 pt wide on a 2,384 pt page,
/// that is most sweeps.
///
/// # The engine is the oracle, and nothing here re-derives its geometry
///
/// *Where* the text ends is [`EditableTextModel::hit_test`]'s question — its
/// reach is one line-height around each line's box and is deliberately not a
/// parameter. So this does not compute a line end, an inflated box or a
/// direction: it asks `hit_test` at sample points and keeps the furthest one
/// that answered. A rotated line, a multi-line sweep and a future change to
/// the reach are all handled by construction, which is the property a
/// shell-side copy of the rule could not have.
///
/// `None` when no sample along the ray resolves — the caller then drops the
/// selection, which is [`drag`]'s unchanged behaviour for a sweep over paper.
fn clamp_to_text(
    model: &EditableTextModel<'_>,
    ctx: &PageContext<'_>,
    from: Pos2,
    to: Pos2,
) -> Option<TextPosition> {
    let at = |t: f32| from + (to - from) * t;
    // Backwards from the pointer. `t = 1.0` is known to fail — it is the
    // caller's own miss — so the scan starts one step in.
    let mut hit_t = None;
    let mut t = 1.0 - CLAMP_STEP;
    for _ in 1..CLAMP_SCAN {
        if hit(model, ctx, at(t)).is_some() {
            hit_t = Some(t);
            break;
        }
        t -= CLAMP_STEP;
    }
    let mut lo = hit_t?;
    // Sharpen towards the far edge of the text, so the clamped caret sits at
    // the end of the run rather than up to one scan step short of it.
    let mut hi = (lo + CLAMP_STEP).min(1.0);
    let mut best = hit(model, ctx, at(lo))?;
    for _ in 0..CLAMP_SHARPEN {
        let mid = f32::midpoint(lo, hi);
        match hit(model, ctx, at(mid)) {
            Some(pos) => {
                best = pos;
                lo = mid;
            }
            None => hi = mid,
        }
    }
    Some(best)
}

/// **Update the selection from a click.**
///
/// The four cases, in the order they are tested, which is also the order of
/// increasing emphasis:
///
/// | gesture | result | from |
/// |---|---|---|
/// | triple-click | the **line** under the pointer | Inkscape / SolidWorks — §1.1 |
/// | double-click | the **word** under the pointer | all three |
/// | Shift+click | extend the existing selection, keeping its anchor | all three |
/// | plain click | collapse — i.e. **clear** | all three |
///
/// `current` is the selection as it stands; it is read only by the Shift case,
/// which needs the anchor to extend *from*. A Shift+click with nothing selected
/// falls through to a plain click, because there is no anchor to extend and
/// inventing one at the top of the page would select a paragraph the operator
/// never pointed at.
#[must_use]
pub fn click(
    ctx: &PageContext<'_>,
    current: Option<&TextSelection>,
    point: Pos2,
    shift: bool,
    double: bool,
    triple: bool,
) -> Option<TextSelection> {
    let model = model(ctx);
    let at = hit(&model, ctx, point)?;
    if triple {
        // `line_range_at` answers `None` for a position on a run carrying no
        // clustered glyph, which `hit_test` does not produce — but a `None`
        // here must clear rather than fall through to the word case, or an
        // emphatic gesture would quietly become a weaker one.
        let (start, end) = model.line_range_at(at)?;
        return resolve(&model, ctx, start, end);
    }
    if double {
        let (start, end) = model.word_range_at(at);
        return resolve(&model, ctx, start, end);
    }
    if shift && let Some(current) = current.filter(|c| c.page == ctx.index && c.live(ctx.epoch)) {
        return resolve(&model, ctx, current.anchor, at);
    }
    // A plain click collapses the range onto one caret slot, which covers no
    // glyphs, so `resolve` answers `None` and the caller clears. Expressed as a
    // degenerate range rather than as an early `return None` on purpose: there
    // is one rule for what a range means, and "a click is an empty range" is a
    // statement in that rule rather than an exception beside it.
    resolve(&model, ctx, at, at)
}

/// **Select every character on the page** — Ctrl+A. Module header §1.3.
///
/// The range runs from the first byte of the first run to the last byte of the
/// last: [`EditableTextModel::resolve_range`] orders and clamps the pair itself,
/// and walks whole intervening runs, so this needs no knowledge of where the
/// glyphs actually are. A page whose extraction produced no runs at all answers
/// `None`, which clears — the honest result for a page with no text on it.
#[must_use]
pub fn select_all(ctx: &PageContext<'_>) -> Option<TextSelection> {
    let model = model(ctx);
    let last = ctx.text.runs.len().checked_sub(1)?;
    let end = ctx.text.runs.get(last)?.text.len();
    resolve(
        &model,
        ctx,
        TextPosition::new(0, 0),
        TextPosition::new(last, end),
    )
}

/// Build the derived line/column/block structure over `ctx.text`.
///
/// Rebuilt per gesture event rather than cached, and affordable for a
/// structural reason: the model **borrows** the `PageText` and owns no glyph
/// data, so recognition is a clustering pass over indices rather than a copy of
/// the page. The expensive half — the content-stream walk — is the thing that
/// *is* cached, on `(page, edit epoch)`, in [`crate::app::cache::PageTextCache`].
///
/// Caching the model instead would mean storing a value that borrows a
/// `RefCell`'s contents, which is the self-referential shape neither `Ref` nor
/// this crate's cache pattern can express.
///
/// `BlockRecognitionOptions::default()` and not a customized one: its ratios and
/// `ExtractOptions`' segmentation ratios are two halves of one derivation, and
/// tuning either alone would make the lines this shell paints and the lines the
/// engine derived describe different text.
fn model<'a>(ctx: &PageContext<'a>) -> EditableTextModel<'a> {
    EditableTextModel::recognize(ctx.text, &BlockRecognitionOptions::default())
}

/// **Is `canvas` inside the box of any text run on this page?**
///
/// # ★★★ CONTAINMENT, and it must not be [`hit`]
///
/// [`hit`] falls back to the nearest line **within one line-height** when no
/// box contains the point — deliberately, because that is Acrobat's behaviour
/// for a sweep begun in the margin. It therefore answers `Some` over a band of
/// blank paper around every line, and a caller asking *"is there a word
/// here?"* would get "yes" in the margin beside one.
///
/// This is the other question, and the two must not be confused. Its one caller
/// is `canvas::clicking`, deciding whether a click in Read mode means the
/// picture underneath or the words on top of it — and answering "words"
/// everywhere would make a scanned page's image unselectable, which is the
/// mirror image of the defect it exists to fix.
///
/// ## ★★ Artifacts count
///
/// A run flagged as an artifact — a running head, a folio — is still text an
/// operator can see and expects to select. `include_artifacts` governs what
/// goes into extracted *plain text*, which is a different question from what is
/// under the pointer.
///
/// ## ★ A run with no `bbox` is skipped rather than guessed at
///
/// `TextRun::bbox` is `Option` because a run whose glyphs carry no usable
/// geometry has no honest box. Treating that as a hit would put the answer back
/// where [`hit`]'s fallback already is.
#[must_use]
pub fn word_at(ctx: &PageContext<'_>, canvas: Pos2) -> Option<()> {
    let pdf = crate::viewer::canvas_to_pdf_space(canvas, ctx.page)?;
    let (x, y) = (f64::from(pdf.x), f64::from(pdf.y));
    ctx.text
        .runs
        .iter()
        .filter_map(|run| run.bbox)
        .any(|b| x >= b.llx && x <= b.urx && y >= b.lly && y <= b.ury)
        .then_some(())
}

/// Where a canvas-space point lands in the page's text.
///
/// Two hops, and the first is the one that is easy to get backwards: the canvas
/// speaks **Y-down from the page's top-left with `/Rotate` applied**, and every
/// glyph position `pdfcer-core` reports is in **PDF user space — Y-up, from the
/// un-rotated CropBox's lower-left**. `canvas::mapping`'s header names conflating
/// those two as *the classic silent defect*, and it is silent here in the worst
/// way: the page looks perfect, and a drag selects a mirrored line.
///
/// So the conversion goes through [`crate::viewer::canvas_to_pdf_space`], which
/// is the single bridge for that hop and works by inverting the **renderer's
/// own** device transform — so the geometry and the picture agree by
/// construction rather than by two implementations happening to match.
///
/// `None` when the page's transform will not invert, or when the point is out
/// of **reach** of every line — [`EditableTextModel::hit_test`] inflates each
/// line's box by one line-height and answers `None` outside all of them, so
/// this is a presence test and not a placement that never fails. A drag begun
/// in the margin beside a line still selects from it, which is Acrobat's
/// behaviour; a drag begun on open paper selects nothing.
///
/// ⚠ A sweep whose *focus* leaves that reach must not cancel the gesture. See
/// [`clamp_to_text`], which is the caller's answer and the reason this
/// function is allowed to be strict.
fn hit(model: &EditableTextModel<'_>, ctx: &PageContext<'_>, canvas: Pos2) -> Option<TextPosition> {
    let pdf = crate::viewer::canvas_to_pdf_space(canvas, ctx.page)?;
    // ★★ ONE call, and no shell-side rotated-band pass in front of it.
    // `EditableTextModel::hit_test` projects the point onto the line, so a press
    // in the middle of a 90° letter lands on that letter.
    //
    // Do not reintroduce a private band here as a fallback: a shell copy of a
    // rule the engine owns keeps compiling and keeps returning something
    // plausible long after the two have diverged, and the symptom — a sweep one
    // letter short, or a sweep that selects nothing — looks like a gesture bug
    // rather than a duplicated rule.
    //
    // The nearest-line fallback inside `hit_test` is bounded at one
    // line-height and that bound is load-bearing: it is what makes this a
    // presence test. `clamp_to_text` handles the overshoot the bound creates.
    model.hit_test(f64::from(pdf.x), f64::from(pdf.y))
}

/// ★★ **Which way the text under `canvas` runs**, in CANVAS space, as an angle
/// in degrees from the horizontal — or `None` where the pointer is not over
/// rotated text.
///
/// The cursor's whole question, and the reason it is answered here rather than
/// in `canvas::cursor`: turning the I-beam needs the page's **extraction**, and
/// `cursor` is a bitmap generator that must not learn what a PDF is.
///
/// # The two hops, and why the second one cannot be skipped
///
/// [`EditableTextModel`] measures directions in **PDF user space** — Y-up, from
/// the un-rotated CropBox's lower-left. The cursor lives in **canvas space** —
/// Y-down, page top-left, with the page's `/Rotate` applied. A direction is not
/// a point, so it cannot be projected by [`crate::viewer::pdf_space_to_canvas`]
/// directly; what is projected is the **two ends of a short segment along it**,
/// and the direction is their difference.
///
/// Doing it that way rather than by adding `/Rotate` to the angle by hand is
/// the same decision `hit` makes for the same reason: `viewer` inverts the
/// renderer's own device transform, so the cursor and the picture agree by
/// construction instead of by two implementations happening to match. A page
/// with `/Rotate 90` turns its vertical stamp into a horizontal one on screen,
/// and the I-beam has to follow the picture, not the file.
///
/// # ★ `None` is the common answer and is not a failure
///
/// Ordinary horizontal text answers `None`, because the upright beam is already
/// right for it and saying so would mean every ordinary page paying for a
/// bitmap lookup to be told nothing changed.
///
/// ★★ Blank paper answers `None` for a stronger reason, and it is why this
/// function does **not** use [`hit`]. `EditableTextModel::hit_test` falls back
/// to the *nearest* line when no line contains the point, which is right for a
/// drag — Acrobat does it — and wrong for a cursor: the empty inches beside a
/// vertical stamp would turn the beam sideways over blank paper the operator is
/// not pointing at any text on. So the test here is **containment**, and the
/// difference between the two is deliberate rather than an inconsistency.
#[must_use]
pub fn tilt_at(ctx: &PageContext<'_>, canvas: Pos2) -> Option<f32> {
    let pdf = crate::viewer::canvas_to_pdf_space(canvas, ctx.page)?;
    // ★ The engine's own answer: `Line::direction` is the unit vector every
    // glyph on that line shares, sourced from the §9.4.4 text rendering matrix
    // rather than recovered from glyph origins. A shell-side census over origins
    // can come up empty on exactly the sparse rotated stamp it would be written
    // for, which is why the direction is read and never inferred.
    //
    // Containment, not nearest-line — see the doc above. `Line::bbox` is
    // computed in the line's own frame, so for a 90° line it is the tall narrow
    // box the ink actually occupies.
    let model = model(ctx);
    let dir = model
        .lines()
        .iter()
        .find(|line| {
            let (x, y) = (f64::from(pdf.x), f64::from(pdf.y));
            line.bbox.llx <= x && x <= line.bbox.urx && line.bbox.lly <= y && y <= line.bbox.ury
        })
        .map(|line| line.direction)
        // Horizontal text needs no tilt, and answering `0.0` for it would make
        // every ordinary page pay for a bitmap the cursor already has.
        .filter(|dir| (dir.1).abs() > f32::EPSILON || dir.0 < 0.0)?;
    // One point on the direction and one a short way along it. The length is
    // arbitrary — only the difference is used — but it is a whole point rather
    // than an epsilon so the subtraction below is nowhere near `f32`
    // cancellation at page coordinates.
    let from = crate::viewer::pdf_space_to_canvas(egui::pos2(pdf.x, pdf.y), ctx.page)?;
    let to =
        crate::viewer::pdf_space_to_canvas(egui::pos2(pdf.x + dir.0, pdf.y + dir.1), ctx.page)?;
    let step = to - from;
    if step.length_sq() < f32::EPSILON {
        return None;
    }
    Some(step.y.atan2(step.x).to_degrees())
}

/// **Does this line run in a direction the page-axis box would get wrong?**
///
/// `true` for anything that is not left-to-right along +x. The test is on the
/// engine's own [`pdfcer_core::text_edit::Line::direction`], which is the unit
/// vector taken from the §9.4.4 text rendering matrix and shared by every glyph
/// on the line by construction.
///
/// ★ Why a *tolerance* rather than exact equality with `(1, 0)`: a page that
/// rotates through the CTM rather than through `Tm`, and a fitted OCR baseline,
/// both produce a direction a hair off horizontal. Treating those as rotated
/// would send ordinary prose down the frame-accumulating path for no benefit;
/// the engine draws the same line at `text_extract::SAME_DIRECTION_COS` and
/// this matches it in spirit — near-horizontal is horizontal.
fn is_rotated(model: &EditableTextModel<'_>, line: usize) -> bool {
    model.lines().get(line).is_some_and(|line| {
        let (dx, dy) = line.direction;
        dy.abs() > f32::EPSILON || dx < 0.0
    })
}

/// ★ **The one derivation** — module header §5.
///
/// One ordered pair in, one [`TextSelection`] out, and both of its halves
/// produced by the same walk over the same byte windows:
///
/// * the **string** is sliced out of each covered run's own `text`, so derived
///   word spaces and line breaks — which are runs carrying no glyphs — are
///   copied along with the characters they separate;
/// * the **boxes** are accumulated from the glyphs whose byte ranges intersect
///   those same windows, grouped by the line the engine put each glyph on.
///
/// The glyph list comes from [`EditableTextModel::resolve_range`] rather than
/// being re-derived from the byte windows here, because that function already
/// owns the intersection rule (including its correct treatment of a zero-width
/// caret window, which selects nothing) and a second implementation of it is
/// precisely how a highlight comes to cover one glyph more than the copy does.
///
/// Returns `None` for a range covering no glyphs. That is the *only* way a
/// caller clears a selection through this module, which is what makes "an empty
/// selection is `None`" true everywhere rather than in most places.
fn resolve(
    model: &EditableTextModel<'_>,
    ctx: &PageContext<'_>,
    anchor: TextPosition,
    focus: TextPosition,
) -> Option<TextSelection> {
    let covered = model.resolve_range(anchor, focus);
    if covered.is_empty() {
        return None;
    }

    // Which line the engine clustered each glyph onto. Built from
    // `model.lines()` rather than by re-clustering on baseline y: the engine's
    // lines already account for the backward-jump split that separates two
    // columns sharing a baseline (module header §4), and a box drawn from a
    // second clustering would span a column gap the copy does not.
    let mut line_of: HashMap<(usize, usize), usize> = HashMap::new();
    for (index, line) in model.lines().iter().enumerate() {
        for gref in &line.glyphs {
            line_of.insert((gref.run, gref.glyph), index);
        }
    }

    // The boxes, in the order their lines are first met, so a selection's quads
    // are in the same content order as its text. `Vec` rather than a map keyed
    // on the line index: the count is one per line of the selection, so a linear
    // scan is cheaper than hashing, and the order is the point.
    let mut boxes: Vec<(Band, Accum)> = Vec::new();
    for gref in &covered {
        let Some(glyph) = model.glyph(*gref) else {
            continue;
        };
        // ★ Which frame this glyph's cell is measured in, decided on the
        // ENGINE's `Line::direction`. A glyph on a rotated line is banded with
        // its own line, in that line's axes; every other glyph keeps the
        // engine's line and page axes. The two never mix, because a `Band`
        // carries which it is.
        //
        // A glyph the line clustering did not claim still has to be drawn, or a
        // selection would silently highlight less than it copies. `Band::Loose`
        // keyed per glyph gives those a box each — visibly correct, and rare
        // enough that the cost is not worth a second clustering rule.
        let band = match line_of.get(&(gref.run, gref.glyph)) {
            Some(&line) if is_rotated(model, line) => Band::Rotated(line),
            Some(&line) => Band::Engine(line),
            None => Band::Loose(gref.run, gref.glyph),
        };
        let cell = match band {
            // The engine's own approximation of a glyph box, with the ascent
            // and descent fractions chosen in the module header §5.
            Band::Engine(_) | Band::Loose(..) => {
                let (x0, x1) = (glyph.x, glyph.x + glyph.advance);
                Accum::Page(PdfRect::from_corners(
                    f64::from(x0.min(x1)),
                    f64::from(glyph.y - glyph.size * GLYPH_DESCENT),
                    f64::from(x0.max(x1)),
                    f64::from(glyph.y + glyph.size * GLYPH_ASCENT),
                ))
            }
            // The same box, in the line's frame: one advance along the writing
            // direction, and the same ascender/descender span across it. For a
            // horizontal direction this reduces to the expression above, which
            // is the check that it is a generalisation and not a second rule.
            Band::Rotated(line) => {
                let dir = model.lines()[line].direction;
                Accum::Frame {
                    dir,
                    origin: (glyph.x, glyph.y),
                    along: (0.0, glyph.advance),
                    perp: (-glyph.size * GLYPH_DESCENT, glyph.size * GLYPH_ASCENT),
                }
            }
        };
        match boxes.iter_mut().find(|(k, _)| *k == band && k.merges()) {
            Some((_, accum)) => accum.absorb(&cell),
            None => boxes.push((band, cell)),
        }
    }

    // The characters, from the runs themselves. `get(..)` rather than indexing:
    // core guarantees a `TextPosition`'s offset is on a glyph boundary and
    // therefore on a UTF-8 boundary, and a stale position that is not must
    // contribute nothing rather than panic in the middle of a drag.
    let (start, end) = ordered(anchor, focus);
    let mut text = String::new();
    for index in start.run..=end.run.min(ctx.text.runs.len().saturating_sub(1)) {
        let Some(run) = ctx.text.runs.get(index) else {
            break;
        };
        let lo = if index == start.run {
            start.byte_offset
        } else {
            0
        };
        let hi = if index == end.run {
            end.byte_offset
        } else {
            run.text.len()
        };
        // ★★ Every run is copied verbatim, including the extraction's derived
        // word spaces and line breaks. There is deliberately no filter here for
        // spurious breaks inside a rotated line: the extraction resolves a
        // baseline step into the line's own frame, so it emits none. A filter
        // against something that is no longer produced is a filter that will one
        // day remove something real.
        if let Some(slice) = run.text.get(lo..hi) {
            text.push_str(slice);
        }
    }

    // ★ The projection into canvas space, through `find::reveal::quad_to_canvas`
    // — the SAME function Find projects its hits with. Reusing it rather than
    // mapping two corners here is what makes a selection box and a find box over
    // the same word land in the same place on a rotated page: it maps all four
    // corners and bounds them, because `/Rotate 90` sends the `ul`/`lr` pair to
    // two corners that are no longer the extremes.
    //
    // ★ **Both spaces are kept, and they are pushed in the same iteration** —
    // module header §5.1. A box whose projection declines contributes to
    // *neither*: the two vectors are index-aligned by construction, and a
    // `filter_map` on one with a plain `map` on the other would let the wash and
    // the authored mark describe different sets of glyphs, in the direction
    // nobody would notice (the mark is in the file; the wash is gone by the next
    // frame).
    let mut quads: Vec<Rect> = Vec::with_capacity(boxes.len());
    let mut page_quads: Vec<Quad> = Vec::with_capacity(boxes.len());
    for (_, accum) in boxes {
        let quad = accum.quad();
        if let Some(canvas) = crate::find::reveal::quad_to_canvas(&quad, ctx.page) {
            quads.push(canvas);
            page_quads.push(quad);
        }
    }
    if quads.is_empty() {
        return None;
    }

    Some(TextSelection {
        page: ctx.index,
        anchor,
        focus,
        epoch: ctx.epoch,
        quads,
        page_quads,
        text,
    })
}

/// The two positions in content order.
///
/// `TextPosition`'s own ordering key is private to `pdfcer-core`, so the tuple is
/// spelled here — once, in the one function that needs it, rather than at each
/// of [`resolve`]'s two uses of "the earlier one".
fn ordered(a: TextPosition, b: TextPosition) -> (TextPosition, TextPosition) {
    if (a.run, a.byte_offset) <= (b.run, b.byte_offset) {
        (a, b)
    } else {
        (b, a)
    }
}

/// **Answer the text selection's own two chords, Ctrl+A and Ctrl+C.**
///
/// They live here rather than in the caller because every rule they enforce is a
/// rule about *this* module, and a caller holding them would have to know all of
/// them to get any of them right.
///
/// ★ These two live apart from [`crate::canvas::keys::canvas_keys`] because
/// both need the page's **extraction** — one to build a range over it, one to
/// read a string out of a selection made against it — and `canvas_keys` is
/// deliberately a document-free function that a headless `egui::Context` can
/// drive end to end. Escape stays there, where its precedence question is
/// answered.
///
/// ★ Gated on [`takes_the_press`], the same predicate the press is gated on, so
/// a mode whose primary button does not select content does not answer Ctrl+A
/// with a text selection the operator has no gesture to clear. §1.3 of this
/// module's header records that the *other* half of Ctrl+A — select every
/// object — is a known gap rather than an oversight.
///
/// ★★ **[`pending_key`] FIRST, and the ordering is load-bearing.** The chord is
/// read off `egui::InputState` — one map lookup — and the page's extraction is
/// fetched **only** when one fired. Asking for the extraction in order to
/// discover that no chord was pressed builds it on the first frame of every
/// reading canvas: **392 ms at open** on `ncored-benchmark-cad-drawing.pdf`,
/// paid by an operator who has touched nothing. It is the same gate
/// `canvas::interact` step 4 puts in front of `page_objects()`, for the same
/// reason.
pub fn keys(
    ctx: &egui::Context,
    doc: &crate::app::state::OpenDoc,
    page_index: usize,
    active_tool: crate::canvas::tool::CanvasTool,
    caps: crate::app::modes::Capabilities,
    selection: &mut Option<TextSelection>,
) {
    if let Some(key) = pending_key(ctx)
        && takes_the_press(active_tool, caps)
        && let (Some(page_text), Some(page)) = (doc.page_text(), doc.pages.get(page_index))
    {
        let text_ctx = PageContext {
            text: &page_text,
            page,
            index: page_index,
            epoch: doc.edit_epoch,
        };
        apply_key(ctx, &text_ctx, key, selection);
    }
}

#[cfg(test)]
mod tests;
