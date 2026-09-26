# `canvas::textsel` — selecting text on the page, and copying what was selected

The operator's rule for Read mode is *"the document shouldn't allow editing
and should allow only selecting of objects that acrobat reader would
allow."* `app::modes::capability` owns the first half — Read refuses every
content gesture. This module is the second half, and it is a widening rather
than a narrowing: **Reader allows text selection**, so a Read mode that
refused it would be less than "only what Reader allows", not a cautious
reading of it.

The gesture is also what makes the text-marking annotations reachable.
Underline, strikeout and squiggly mark *text*, and
`pdfcer_core::annot_author::MarkupSpec::TextMarkup` takes a `Vec<Quad>` —
which is [`TextSelection::page_quads`], produced by the same pass over the
same glyphs as the canvas boxes (§5.1). Nothing here authors anything; the
authoring lives in [`crate::canvas::markup::text`], and §6 says why.

---

## 1. The interaction decisions, and which application each came from

The operator's standing instruction is to *"make your best educated guesses
to match what inkscape, acrobat, and SolidWorks do"*, recording which one was
followed and why, and — where they disagree — saying which won. **Acrobat
wins ties about *reading*, because Acrobat is what pdfcer replaces.**

| Question | Acrobat | Inkscape | SolidWorks | Shipped | Why |
|---|---|---|---|---|---|
| drag selects a **range** or a **rectangle** | both (range default, `Alt` for rectangle) | range | range | **range** | unanimous on the default; the rectangle is deferred, §2 |
| **double**-click | word | word | word | **word** | unanimous |
| **triple**-click | paragraph | line | line | **line** | §1.1 — the disagreement is the interesting part |
| crosses **columns** | yes | n/a (one text object) | n/a (one note) | **yes** | falls out of content order, §4 |
| crosses **pages** | yes | n/a | n/a | **no** | §4 — and it is a cost decision, stated as one |
| visible **caret** | no (Select tool) | yes (text tool) | yes (note editing) | **no** | §1.2 |
| **Escape** | clears | deselects | leaves the field | **clears** | unanimous |
| **Ctrl+A** | all text | all objects | all in field | **all text on the page** | §1.3 |
| **Ctrl+C** | copies | copies | copies | **copies** | unanimous |
| Shift+click | extends from the anchor | extends | extends | **extends** | unanimous |

### 1.1 Triple-click: Acrobat says paragraph, and this ships a line

The one row where the reference applications disagree and Acrobat **did not
win**, so it needs the argument.

Acrobat selects a paragraph. Inkscape and SolidWorks select a line. A PDF
content stream contains neither: `pdfcer-core`'s own extraction documentation
is blunt that lines are **derived** (its S5 sourcing note — *"no line or
paragraph markers exist in a content stream, in tagged or untagged files
alike"*), and paragraphs are derived a second time, from the lines, by
`EditableTextModel`'s block recognition using a leading-gap ratio and an
indent ratio.

So the choice is between a unit this engine derives once and a unit it
derives twice. A triple-click that selected a *block* would be the
operator's most emphatic gesture resolved through the shakiest inference in
the stack, and when it got the paragraph wrong — on a drawing sheet's title
block, where "paragraph" means very little — there would be no smaller unit
to fall back to, because the double-click below it is a word. The line is
the honest middle rung, it is what two of the three reference applications
do, and [`EditableTextModel::line_range_at`] is a published verb for it
where a block range is not.

### 1.2 No caret, and that is a statement rather than an omission

Acrobat Reader's Select tool draws a highlight and **no blinking caret**; a
caret appears only where something can be typed (a form field). Inkscape and
SolidWorks both draw one, and both are *editing* text when they do.

Reading is the subject here, so Acrobat wins — and there is a second,
stronger reason that is about this shell rather than about convention: a
caret promises an **insertion point**, and there is nothing to insert.
Phase 5 (in-place text editing) is last in the operator's order and must not
be started early. A caret drawn before it exists is an affordance for a
feature that does not, which is the no-placeholders invariant read straight
(`PROJECT_PLAN.md` §3).

`pdfcer-core` already publishes everything a caret needs
([`EditableTextModel::caret_x`], `caret_left`, `caret_right`, `caret_up`,
`caret_down`) — written for Phase 5. None of it is called here. That is
where a caret comes from when there is something to type into.

### 1.3 Ctrl+A means "everything this gesture can select"

Acrobat selects all the text; Inkscape selects all the objects in the layer;
SolidWorks selects everything in the field being edited. All three are the
same rule — *select everything the thing you are currently selecting in* —
and this shell applies it: where a press selects text, Ctrl+A selects the
page's text ([`select_all`]).

**The other half is deliberately absent and is named rather than implied:**
in a mode that selects page content there is no select-all, because
`canvas::selection` has no "every object on the page" verb and inventing one
inside a keyboard handler would put a selection rule somewhere other than
the module that owns selection rules. So Ctrl+A does nothing in Edit: one
honest gap, with the shape of the fix recorded here.

---

## 2. What a drag does, and the rectangle that is not built

A drag selects the **range** between where the button went down and where
the pointer is — in the engine's content order, which is what makes it flow
round line ends and across a column break rather than sweeping a box.

Acrobat's second mode — `Alt`+drag for a **rectangular** text selection — is
genuinely useful on the drawing sheets this application exists for, where a
parts table's column is a rectangle and emphatically not a range. It is
**not built**, and the reason is this module's first rule: one derivation, so
what is shown and what is copied cannot diverge. A
rectangular selection is a second selection model — its copy is column-wise,
its reading order is its own, and it cannot be expressed as a
`(TextPosition, TextPosition)` pair at all — so it would be a second
[`resolve`] with a second quad derivation and a second copy path beside it.
That is the divergence this module is built to make impossible, bought for a
modifier.

What it would take, so the next hand does not re-derive it: a
`Selection::Rect(egui::Rect)` variant beside the `(anchor, focus)` pair
[`TextSelection`] carries today, resolved by filtering
`PageText`'s glyphs on their own geometry rather than by
[`EditableTextModel::resolve_range`], with the copy assembled per line from
the surviving glyphs. Both variants would then have to flow through one
`resolve` returning one [`TextSelection`], which is what keeps the promise
above.

---

## 3. THE MODE GATE

**[`gate`], and its header is the whole argument** — why text selection needs
no capability, why it still has to be told apart from the content marquee,
and what the rule yields mode by mode. [`takes_the_press`] and the tests that
are about it live there too.

The one-line version, so a reader here is not sent away for nothing:

> **A press means text when the text tool is armed, *or* when the select tool
> is active and the mode cannot select content** — and selecting text needs
> no capability at all, because it authors nothing.

---

## 4. One page, and content order

**A selection is a range on one page.** Acrobat's crosses pages; this one
does not, and it is a cost decision rather than a taste one.

`crate::find`'s header carries the measurement: a whole-document extraction
is 331–449 ms on this project's fixtures, which is why Find never searches
on a keystroke. A cross-page selection needs a document-wide index — every
page walked, tokenised and font-resolved — and it needs it live, because a
drag samples the pointer sixty times a second. Per **page** it is one
extraction cached on `(page, edit epoch)` and free thereafter
(`app::cache::PageTextCache`); per **document** it is Find's number
paid again on every page turn, to make a gesture work that ends at the
window edge anyway. The anchor and the focus would also live on pages with
different [`crate::canvas::mapping::PageMapping`]s, which `canvas::interact`
is single-page by construction and says so.

**Columns are a different matter and need nothing.** `PageText::runs` is in
page content order, and the engine inserts a derived line break at a
backward horizontal jump — its `backward_jump_ratio`, which exists because
*"a two-column page whose columns share baselines runs the two columns
together into one line with no separator at all"*. So a drag from the first
column into the second selects everything between them in content order,
with the columns separated, which is what Acrobat does. Neither this module
nor the operator has to know a column existed.

Content order is **not** appearance order, and the engine says so
(§14.8.2.3.1: the two orderings *"may or may not coincide"*). A file whose
producer emitted its text out of visual order will select out of visual
order. That is the file's ordering, faithfully reported; inventing a
geometric reading order here would be a third derivation on top of two.

---

## 5. One derivation: what is highlighted IS what is copied

The requirement, and the defect it names: the highlight is drawn from the
same quads the copy uses — **one derivation, so what is shown and what is
copied cannot diverge.**

[`resolve`] is that one derivation. It takes the ordered pair of
[`TextPosition`]s **once**, walks the covered runs **once**, and in that
single pass produces both halves of [`TextSelection`]: the string is sliced
from the runs' own text as the walk passes through them, and the boxes are
accumulated from the glyphs inside the same byte windows. There is no second
entry point, no "recompute the quads for drawing", and no way to ask for one
without the other — [`TextSelection`]'s fields are populated together or the
value does not exist.

That also makes the highlight free to draw. The quads are stored in **canvas
space**, which is zoom-independent, so a frame that merely paints an
existing selection runs no extraction, builds no model and does no
geometry — the same property `canvas::selection` relies on for its outlines
and `crate::find::Hit::canvas` for its wash.

### 5.1 The same pass produces a THIRD output, and that is why

[`TextSelection`] carries its boxes twice: [`TextSelection::quads`] in
**canvas space**, which is what the overlay paints, and
[`TextSelection::page_quads`] in **PDF user space**, which is what a
`/QuadPoints` text markup is authored from ([`crate::canvas::markup::text`]).
Both are `boxes` — the one `Vec` accumulated in [`resolve`]'s single walk —
and neither can exist without the other.

It would have been one field fewer to store the canvas boxes alone and let
the authoring site invert [`crate::viewer::canvas_to_pdf_space`] over their
corners. That is refused, for two reasons and the second is the one that
decides it:

* **It is a second derivation of the geometry**, arriving through the door
  §5 exists to lock. The rule is not *"do not extract twice"* — it is that
  what is shown and what is committed must be the same value, and two
  spellings of the same projection are exactly how they come to differ.
* **The inverse is not the identity on a rotated page.** The forward hop is
  `find::reveal::quad_to_canvas`, which maps all four corners and takes their
  bounds precisely because `/Rotate 90` sends `ul`/`lr` to two corners that
  are no longer the extremes. Inverting a *bounded* rect corner by corner
  gives back two opposite corners in an order `Rect::from_corners` is not
  promised to normalise — a mark that lands mirrored about the page's centre
  line, in the file, discovered after saving. That is the failure
  [`crate::canvas::markup`]'s own §1 is built around, reintroduced by an
  optimisation worth eight bytes a line.

### Why the highlight stays readable

The rule `overlay::CURRENT_ALPHA` carries is general: *the operator's next act
after finding a hit is to READ it*, so a wash opaque enough to cover the word
it marks has failed at its job.

It applies here with more force, not less — a selection is what you are
about to copy, and an operator who cannot read it cannot tell whether they
swept the right words. So the selection wash reuses the same themed colour at
the same low end (`overlay::TEXT_SELECTION_ALPHA`), under the compile-time
bound that keeps it there, and it is drawn **unstroked**: Find strokes
its current hit to distinguish it from its neighbours, and a text selection
has no neighbours to be distinguished from. A stroke per line box would also
draw a visible seam between two lines of one selection, which is a boundary
the operator did not make.

### The glyph box, and the constant that had two candidates

A glyph carries an origin, an advance and a size — not a box. `pdfcer-core`
approximates one in two places and **they do not agree**:

| site | ascent | descent |
|---|---|---|
| `EditSession`'s search quad (what `TextMatch::quad` is) | `+0.85 × size` | `−0.22 × size` |
| `TextRun::bbox` and `Line::bbox` | `+0.75 × size` | `−0.25 × size` |

There is no shared constant to inherit, so it is a choice, and it is made
**for the search quad's numbers**: `crate::find` draws its highlights from
`TextMatch::quad`, and Find is the surface an operator will see next to this
one — searching for a word and then selecting the same word must not produce
two boxes of visibly different heights over the same glyphs. Matching the
*bbox* numbers would instead match a box nothing paints.

---

## 6. This module authors nothing, and neither does copying

No function here takes `&mut EditSession`, raises an
[`crate::app::actions::Action`], or bumps `edit_epoch`. The selection lives
on [`crate::app::state::OpenDoc`] beside the object selection, which
`canvas`'s header already argues is not a document mutation: *"a selection
*names* parts of a document and changes nothing a save would write."*

Copying is the same class one step further out: it reads the extraction and
calls `egui::Context::copy_text`. It is not routed through the action funnel
for the reason `file.print` is not — the funnel exists for work that touches
a document or must not happen mid-frame, and this is neither.

Rule 4 (disclosure lives off-canvas) is satisfied the way `canvas::overlay`'s
header states it: with nothing selected this paints nothing at all, so *would
a screenshot of the canvas differ from a screenshot of the same document
saved and reopened?* answers no by construction. A selection wash is a
pre-commit affordance — the cursor, describing what a copy would take — in
exactly the category rule 4 admits alongside the rubber band and the snap
indicator.

## 7. Staleness

A [`TextPosition`] is `(run index, byte offset)` **into a particular
extraction**. An edit re-writes content streams, so run indices renumber and
byte offsets move: a position recorded before an edit can name different
glyphs, no glyphs, or the right glyphs in the wrong place. That is Find's
staleness problem exactly, and `crate::find`'s header rejects the same two
wrong answers — re-resolving automatically (an extraction per edit) and
drawing the old geometry anyway (*"a highlight that may be over the wrong
text, which is the one thing rule 4 forbids outright"*).

Find keeps the query and drops the geometry, because a query is something the
operator typed. A selection has no such half: it **is** geometry. So the
whole thing is dropped — [`TextSelection::epoch`] records the revision it was
resolved against, [`TextSelection::live`] answers `false` the instant that
moves, and the overlay is handed nothing.

**Authoring a text markup is itself an edit**, so marking a selection
makes that selection stale on the very next frame: `add_markup` goes through
`vector_edit`, which bumps `edit_epoch`, and the wash disappears. Acrobat
keeps its selection across a markup and this does not, which is a real
difference and is recorded rather than smoothed over. The alternative is a
second staleness rule — *"an edit that adds an annotation does not move the
text"* — living outside this module and free to disagree with the one here;
the epoch is the only signal there is, and refining it into kinds of edit is
a mechanism, not a line. What the operator loses is one re-sweep to underline
*and* strike out the same words.

## 8. Text that does not run along the page's x axis

A vertical file-path stamp in a CAD title block is the case that names the
two failures, in the operator's words: *"the I cursor doesn't reorient and it
pastes each letter onto its own line."*

The engine owns the hard half. `pdfcer_core::text_edit::Line::direction` is
the unit vector taken from the §9.4.4 text rendering matrix, shared by every
glyph on the line by construction, and the extraction resolves a baseline
step into the **line's own frame** rather than into page axes — so a 90° line
is one line and not one derived break per letter. What is left to this module
is banding: which frame a glyph's cell is measured in.

| rule | rotated text |
|---|---|
| §5, one derivation | **holds, and it is what makes the rotated path safe.** The direction is consulted once, inside [`resolve`], and both the boxes and the string are built in that same walk |
| §4, content order | **holds.** Nothing is reordered; a rotated line is the same glyphs in the same order, banded differently |
| box shape | a rotated line's glyph cells are accumulated **in the line's own frame** (`bands::Band::Rotated`) and emitted as one banded [`Quad`]; a horizontal line's are accumulated in page axes |
| the copied string | unfiltered. Every run the extraction emits is copied, because the extraction emits no derived break inside a rotated line for this module to have to skip |

**A page with no rotated text never reaches any of it.** `is_rotated`
answers `false` for every line, every glyph takes the page-axis branch, and
no line frame is ever built. That is deliberate and structural: one grouping
rule handling both cases would put every ordinary document's selection
through the rotated code to serve a minority of drawing sheets.

The canvas wash is a `Rect`, so for a **quadrant** rotation (90°, 180°,
270° — every rotated stamp a CAD exporter emits) the band is axis-aligned in
page space and the wash covers it exactly. At an arbitrary angle the band is
a parallelogram and the wash is its bounding box, which over-covers at the
corners. The authored `/QuadPoints` are the true parallelogram either way,
because [`TextSelection::page_quads`] carries corners rather than bounds.

## 9. Where the rest of this module is

The two chords (`Ctrl+A`, `Ctrl+C`), the guard in front of them and the one
function that writes the clipboard live in [`clipboard`], re-exported flat so
every call site still writes `textsel::copy` and `textsel::pending_key`.
The seam is there rather than anywhere else because [`clipboard::copy`] is
reached by two **ribbon commands** that have no selection at all — see that
module's header.

## Item notes

### `const GLYPH_ASCENT`

`pdfcer-core`'s **search-quad** number, deliberately, where its run and line
boxes use `0.75`. See the module header §5: `crate::find` paints
`TextMatch::quad`, and a selected word must not be a visibly different height
from the same word found. There is no shared constant in core to inherit, so
this is a decision rather than a reference.

### `const CLAMP_SCAN`

A scan rather than a straight bisection because the reachable set along a
drag is not an interval: a sweep that crosses a gap between two columns
leaves reach and re-enters it, and a bisection seeded from the anchor would
stop at the near edge of the gap and silently under-select. Scanning
**backwards from the pointer** finds the furthest text the operator has
actually dragged past, which is the one they mean.

### `fn clamp_to_text`

The clamp that makes overshooting a line harmless. Without it a sweep that
ran one line-height past the last character resolved no focus, [`drag`]
returned `None`, and the entire selection vanished mid-gesture — on a
drawing sheet, where a title-block run is 40 pt wide on a 2,384 pt page,
that is most sweeps.

# The engine is the oracle, and nothing here re-derives its geometry

*Where* the text ends is [`EditableTextModel::hit_test`]'s question — its
reach is one line-height around each line's box and is deliberately not a
parameter. So this does not compute a line end, an inflated box or a
direction: it asks `hit_test` at sample points and keeps the furthest one
that answered. A rotated line, a multi-line sweep and a future change to
the reach are all handled by construction, which is the property a
shell-side copy of the rule could not have.

`None` when no sample along the ray resolves — the caller then drops the
selection, which is [`drag`]'s unchanged behaviour for a sweep over paper.

### `fn model`

Rebuilt per gesture event rather than cached, and affordable for a
structural reason: the model **borrows** the `PageText` and owns no glyph
data, so recognition is a clustering pass over indices rather than a copy of
the page. The expensive half — the content-stream walk — is the thing that
*is* cached, on `(page, edit epoch)`, in [`crate::app::cache::PageTextCache`].

Caching the model instead would mean storing a value that borrows a
`RefCell`'s contents, which is the self-referential shape neither `Ref` nor
this crate's cache pattern can express.

`BlockRecognitionOptions::default()` and not a customized one: its ratios and
`ExtractOptions`' segmentation ratios are two halves of one derivation, and
tuning either alone would make the lines this shell paints and the lines the
engine derived describe different text.

### `fn hit`

Two hops, and the first is the one that is easy to get backwards: the canvas
speaks **Y-down from the page's top-left with `/Rotate` applied**, and every
glyph position `pdfcer-core` reports is in **PDF user space — Y-up, from the
un-rotated CropBox's lower-left**. `canvas::mapping`'s header names conflating
those two as *the classic silent defect*, and it is silent here in the worst
way: the page looks perfect, and a drag selects a mirrored line.

So the conversion goes through [`crate::viewer::canvas_to_pdf_space`], which
is the single bridge for that hop and works by inverting the **renderer's
own** device transform — so the geometry and the picture agree by
construction rather than by two implementations happening to match.

`None` when the page's transform will not invert, or when the point is out
of **reach** of every line — [`EditableTextModel::hit_test`] inflates each
line's box by one line-height and answers `None` outside all of them, so
this is a presence test and not a placement that never fails. A drag begun
in the margin beside a line still selects from it, which is Acrobat's
behaviour; a drag begun on open paper selects nothing.

⚠ A sweep whose *focus* leaves that reach must not cancel the gesture. See
[`clamp_to_text`], which is the caller's answer and the reason this
function is allowed to be strict.

### `fn is_rotated`

`true` for anything that is not left-to-right along +x. The test is on the
engine's own [`pdfcer_core::text_edit::Line::direction`], which is the unit
vector taken from the §9.4.4 text rendering matrix and shared by every glyph
on the line by construction.

Why a *tolerance* rather than exact equality with `(1, 0)`: a page that
rotates through the CTM rather than through `Tm`, and a fitted OCR baseline,
both produce a direction a hair off horizontal. Treating those as rotated
would send ordinary prose down the frame-accumulating path for no benefit;
the engine draws the same line at `text_extract::SAME_DIRECTION_COS` and
this matches it in spirit — near-horizontal is horizontal.

### `fn resolve`

One ordered pair in, one [`TextSelection`] out, and both of its halves
produced by the same walk over the same byte windows:

* the **string** is sliced out of each covered run's own `text`, so derived
  word spaces and line breaks — which are runs carrying no glyphs — are
  copied along with the characters they separate;
* the **boxes** are accumulated from the glyphs whose byte ranges intersect
  those same windows, grouped by the line the engine put each glyph on.

The glyph list comes from [`EditableTextModel::resolve_range`] rather than
being re-derived from the byte windows here, because that function already
owns the intersection rule (including its correct treatment of a zero-width
caret window, which selects nothing) and a second implementation of it is
precisely how a highlight comes to cover one glyph more than the copy does.

Returns `None` for a range covering no glyphs. That is the *only* way a
caller clears a selection through this module, which is what makes "an empty
selection is `None`" true everywhere rather than in most places.

### `fn ordered`

`TextPosition`'s own ordering key is private to `pdfcer-core`, so the tuple is
spelled here — once, in the one function that needs it, rather than at each
of [`resolve`]'s two uses of "the earlier one".
