# DESIGNS.md

Designs that are argued and not yet built, one section per open
`OPERATOR_REQUESTS.md` row, written for the engineer who will build them.

A section is deleted when its design has been built **and driven**, in the same
commit. Until a driven check agrees with it, the section is the only record of
what the correct behaviour is supposed to be, and an implementation that
reintroduces the symptom is indistinguishable from one that fixed it.

Where a section says a question needs the operator's ruling, that is
load-bearing: do not build past it. Rulings are recorded in
`OPERATOR_REQUESTS.md`, and only the operator closes a row.

---

## O226–O229 — the OCR text-layer mode: two synced views, a PDF↔text slider, merge and split

### What the four rows are, as one build

O226 is the mode and its two views. O227 is merge-and-split, which is **not**
an OCR feature and must not be built as one. O228 is the selection echo between
the views. O229 is the overlay colour and its persistence. They ship together
because three of them are meaningless alone, and the fourth (O227) is the one
that will be wanted everywhere else afterwards.

### The measurement that decides the cost, taken before any of this was designed

The obvious fear is that a second view of one document is a rewrite, because
zoom and scroll are per-document everywhere in this shell. **It is not**, and
the reason is that the state was already gathered into one place:

- `viewer::ViewState` — the struct `OpenDoc::view` holds — already carries
  `zoom`, `fit`, `page_index`, `display`, `rulers`, `grid`, `guides`,
  `show_points`, `line_weights` and `off_page`. That is a per-**view** state
  object that happens to be stored one-per-document. Nearly everything a second
  pane needs to differ in is already inside it.
- `canvas::present::show` already takes a `&mut egui::Ui` and draws the whole
  canvas into it, with `show_in` beneath it drawing into the region the rulers
  left. It does not assume it owns the window.

So the shape of the work is **not** "make the canvas re-entrant". It is:

> `OpenDoc::view: ViewState` becomes a small owner of one or two `ViewState`s,
> and the four view fields that escaped onto `OpenDoc` move inside `ViewState`
> where they belonged in the first place.

**The four stragglers, and each is a per-view quantity mis-homed on the
document:** `last_scroll_offset`, `zoom_anchor`, `observed_zoom` and
`deep_zoom` (plus `zoom_commit_at` / `zoom_commanded`, which are the settle
clock for one view's zoom). Every one of them is frame bookkeeping about *a*
canvas, and with two canvases on screen a single copy is not a limitation —
it is a bug that makes pane B's scroll steal pane A's anchor. The move is
mechanical and the compiler finds every site.

⚠ **Do not make `views` a `Vec`.** Two is the number he asked for, two is the
number the sync rule is written for, and a vector invites a third pane nobody
has a design for. A `primary` plus an `Option<secondary>` says the same thing
and makes "is the split open?" a `.is_some()` rather than a length test that
reads as arbitrary.

### Why the selection echo (O228) is nearly free, and must stay that way

**Selection lives on the document, not on the view**, and that is correct and
should not change. Both panes therefore read one selection with no
synchronisation of any kind: pane B draws an outline around whatever the
document says is selected because that is the only selection there is.

This is worth stating because the tempting refactor — "each view gets its own
selection, and they sync" — would be a second selection model, would need a
conflict rule nobody wants to write, and would break the thing he actually
asked for, which is that the two panes are two looks at **one** editing
session. The echo is not a feature to build; it is what falls out of not
breaking the existing model.

What *does* need building is the outline itself in the non-focused pane: the
focused pane draws handles and the caret, the other draws **box outlines only**.
Handles in both panes would give two drag targets for one object.

### The scroll-and-zoom sync rule, and the three ways it goes wrong

The rule is in `OPERATOR_REQUESTS.md` O226 and is repeated here because this is
where it gets built: **the two views agree on the document point at their
centres and on the zoom — never on the scrollbar offsets.**

```
follower.zoom   = author.zoom
follower.centre = author.centre          // in PAGE coordinates, not pixels
```

A wider pane sees more page around the same centre. That is the whole rule.

**Failure 1 — the feedback loop, already recorded in `D:\dev\rag\egui\`.** A
view whose fit-zoom is computed from its own rect, driving a second view whose
rect influences the first, oscillates and never settles. The guard is that
**exactly one pane is the author per frame** — the one that received the
gesture — and the follower writes nothing back that frame. Model it as: the
author publishes `(zoom, centre)` into a one-frame outbox; the follower
consumes it before it lays out; a pane never publishes in a frame in which it
consumed.

**Failure 2 — fit modes are not shareable.** `FitMode::Width` in a 400 px pane
and a 900 px pane are different scales. Fit is resolved **in the pane that
issued it**, producing a concrete zoom, and that concrete zoom is what
propagates. The follower's `fit` becomes `FitMode::Actual`-equivalent (an
explicit zoom), never a copy of the author's fit mode — copying the mode is how
you get two panes that each "fit" and therefore never agree.

**Failure 3 — the harness goes stale on a divider drag.** The RAG already
records that harness coordinates go stale when a dock width changes. A
draggable divider between two canvases is that same hazard with a new name.
Every `ui-verify` check that clicks a canvas point must take its rect **this
frame**, and the split's own checks must drive the divider and re-measure.

**The lock.** A padlock on the divider, on by default, which breaks the sync so
each pane scrolls and zooms alone. This is not a nicety: the moment he wants
the scan at 400 % beside the whole page, a mandatory sync is a wall.

### The overlay: how the text layer is actually drawn

The slider is a single `f32` in `0.0..=1.0` per view, living in `ViewState`.
It drives **two** things, and conflating them is the error to avoid:

| slider | the page raster | the text overlay |
|---|---|---|
| 0.0 | full opacity | not drawn |
| 0.5 | half | half |
| 1.0 | not drawn | full opacity |

- **The raster is not re-rendered.** Its opacity is a tint on the already-cached
  texture. Re-rasterizing per slider position would put a render request behind
  a drag, and this shell has learned that lesson once already.
- **The overlay is vector, drawn every frame** from the run geometry, in the
  colour O229 sets. It is never baked into a texture, because it must stay crisp
  at every zoom and because a cached overlay is a second rendering path for
  content that has exactly one.
- **At slider 1.0 the field behind the text is blank**, not the scan at 1 %.
  He said *"only the text layer is visible"* and meant it — the point is to read
  the OCR output without the paper arguing with it.

### R8b, settled here so it is not re-litigated

The one-line test asks whether the canvas differs from the saved-and-reopened
document **because pdfcer is marking its own uncertainty**. It is not. This is
an operator-thrown X-ray switch over content that is already in the file,
the same class as Acrobat's *Show OCR text*. The binding guards:

1. The slider is at 0.0 in every other mode and leaving the mode restores the
   true render with no residue.
2. The mode is named off-canvas while it is on.
3. Nothing about the overlay reaches the file. O229's colour is a `Prefs`
   field, beside `shade_form_fields` and `chrome`, and the driven check owes
   exactly this: change the colour, save, reopen, **the bytes are unchanged.**

### The trap that will ship silently if it is not asserted

OCR text is invisible on purpose — text render mode 3, drawn beneath the scan.
**An edit that loses that mode prints the correction over the picture of the
paper**, in black, at full size, and the operator finds out at the printer.

This cannot be verified by reasoning and cannot be verified by a unit test on
the shell's side, because the shell is not what writes the run. It needs a
driven check on a real scanned fixture that edits a run, saves, reopens, and
asserts the run is still invisible. **The fixture does not exist yet and is
part of this work**, not a prerequisite someone else supplies.

### O227, and why it is bigger than the mode

He asked for merge-and-split across *"all of our text editing"*. Taken
literally — and the standing sibling-kinds expectation makes it literal — that
is free text, callouts, form field values, text-bearing markup, ce dimension
labels, page content text, and this mode's OCR runs.

**Merge** joins selected runs in reading order; the separator comes from the
geometry (same baseline → space; consecutive lines → space, and no hyphen
unless one was already there; paragraph end → break). The merged run's box is
the union of what it consumed.

**Split** is the inverse at the caret or at a selection boundary, each half
keeping the geometry its text occupied.

**The invariant that makes it safe:** undo restores the exact prior runs, not
an approximation. A merge that cannot be undone into the same six boxes is a
destructive edit wearing a formatting label, and OCR output is exactly where
someone will merge forty runs and then want three of them back.

**Build it as a text verb pair, not as an OCR gesture.** If it lands inside the
OCR mode it will be re-implemented the first time he wants it on a callout, and
the two copies will disagree about hyphens within a month.

### ★★★ There are two OCR engines, and confidence is the reason this mode exists

`pdfcer-core::ocr` already carries an engine-independent seam — the
`OcrEngine` trait, `RecognizedWord`, `OcrPage`, `layer::add_ocr_layer` and
`models`. `engine_ocrs` binds the `ocrs` engine today and **its
`reports_confidence` returns `false`**. A second engine, **OCRcer**
(`D:\dev\OCRcer\`, designed and being built from scratch, MIT code *and* MIT
model), is built to the same trait and **its `reports_confidence` returns
`true`** — per-word confidence as a calibrated margin, plus per-character
boxes.

That single difference reshapes this mode.

**The mode is the disclosure surface R8b already demands.** OCR output is the
textbook case of *"an inference the operator cannot see"* — invisible text,
placed under a picture, asserting what the paper says. R8b's ruling on that is
not "mark it up", it is **render normally and report off-canvas, both**. This
mode is that report, made interactive.

**The engine already has the primitive; do not invent a second one.**
`OcrPage::words_needing_review(threshold)` returns exactly the words below a
confidence bar, and `OcrPage::mean_confidence` summarises a page. The panel
beside the two views is a view of that list: click a row, both panes centre on
the word, the text pane has the caret in it. **That, not the slider, is the
fastest route through a bad page** — the slider is for reading, the list is for
working.

**`reports_confidence()` is a capability, and R8 decides how it is expressed.**
On a layer produced by an engine that reports no confidence, the review list
and every confidence column **are not drawn at all** — not greyed, not zeroed,
not shown as "n/a". R9 is explicit and the trait method exists precisely so
this is answerable rather than guessed. The rest of the mode — the split, the
slider, the overlay, selection, merge and split — is unaffected and must not
be made conditional on it.

⚠ **Do not bind any of this to OCRcer.** It is the second implementation of a
trait that already has two, which is the moment a shell hard-codes one by
accident. Everything the shell touches is `OcrEngine` / `RecognizedWord` /
`OcrPage`; nothing names an engine.

**Word granularity is why O227 is the right verb pair.** Both engines return
**words**, each with a rect. A sentence is therefore *n* runs by construction,
and merge-and-split is not a convenience on top of OCR — it is the only way the
operator gets from what the engine produced to what the document says. Build
merge and split before the slider if something has to be cut.

### What the engine survey settled, and what it blocks

**Read-back works.** `extract_page_view` against `session.view()` — the `_view`
twin, or the model describes the page as it was before the last accepted edit —
with `ExtractOptions::with_provenance(true)` returns positioned runs, and
`ExtractedGlyph::invisible` is populated unconditionally. `EditableTextModel::recognize`
then groups them into lines and blocks with a hit test. That is the whole read
side of this mode and it needs nothing new.

**Replace-in-place works and stays invisible.** `edit_text` rewrites only the
show operator's operand, so the ambient `3 Tr` survives by construction;
`format_text` goes further and restores it through the R88 ambient ladder,
refusing by name rather than dropping it. The common correction is safe.

**Three things are blocked, and they are filed:**

| gap | filed |
|---|---|
| no write path can set text render mode — so insert-a-missed-word and any rebuild produce **visible ink over the scan** | **G034** |
| no merge verb anywhere in the crate; `split_text_object` has no inverse | **G035** |
| nothing finds or removes a previously written OCR layer, so re-OCR **stacks** a second invisible copy of every word | **G036** |

⚠ **The consequence for O227, and it must not be quietly dropped.** Merge and
split were asked for across *all* text editing, and on an **invisible** layer
neither can be offered until G034 and G035 land — delete + `add_text` is the
only route and it writes visible text. Under R8/R9 the command is therefore not
registered for mode-3 runs and nothing is drawn. The rest of O227, on visible
text, is blocked only by the missing merge verb.

### Two more blockers, filed; two lesser gaps, not

**Filed.**

| gap | filed |
|---|---|
| the extraction and surgery models index the same page in two spaces with **no engine-supplied mapping** — the shell joins them by byte span and nothing guarantees the join | **G037** |
| **no box-resize verb** — `move_text_run` translates, nothing sets a target width, and `set_h_scale` takes a percentage the shell must derive by re-implementing the engine's metrics | **G038** |

G037 is the single largest hidden cost in this mode and the most likely source
of a wrong-object edit: a mismatch does not panic and does not refuse, it edits
a different run than the one the operator clicked.

**Not filed — recorded so they are on disk and attributable.**

1. **No run-level split.** `split_text_object` cuts between show operators
   only. Splitting one word's `Tj` into two positioned words — which is exactly
   what correcting an OCR run-together needs — is not expressible. It is a
   sub-case of **G035**'s territory and should ride with it rather than open a
   fourth file on the same subject.
2. **No greyscale helper.** The OCR trait takes 8-bit luma; the RGBA→Rec.601
   step exists only in `pdfcer-cli`'s `main.rs`, so every consumer rewrites it
   and any two can disagree about the coefficients. A nuisance, not a blocker.

### The coordinate trap, recorded before it is hit

`ocr::words_to_page_space` is **`/Rotate 0` only**, while `pdfcer-render` does
honour `/Rotate`. Use **`words_to_page_space_on(words, img_w, img_h, PagePlacement)`**,
always. The rotation-blind form yields a transposed invisible layer that looks
perfect on screen — because it is invisible — and is wrong everywhere it
matters.

Likewise `models::resolve_model_dir_with(..., required)` rather than
`resolve_model_dir`: an empty `models/ocrs` otherwise resolves and shadows a
good directory.

---

## O181 — installed fonts in Add Text

### What is true

`canvas/textedit/pen.rs` types `TextPen::face` as `Std14` and holds
`pub const FACES: &[Std14]`, fourteen entries written out by hand. An installed
font is not expressible on the pen, and the module header says a donor font is
deliberately not there.

The rest of the chain is built, on both sides of the crate boundary:

| piece | where |
|---|---|
| walks `%WINDIR%\Fonts` and the per-user font dir | `app/prefs/fonts.rs` `os_font_dirs()` |
| parses every file, takes every advertised face name | `app/fonts.rs`, via `FontProgram::parse` and `.face_names()` |
| holds name to path | `app/fonts.rs` `Library`, private `paths: BTreeMap<String, PathBuf>` |
| engine: subset a donor for the characters typed | `pdfcer-render` `font::subset::plan_subset(donor, face_index, chars, base_name, subset_tag)` |
| engine: the Add-Text seam that takes the result | `pdfcer-core` `NewTextFace::Embedded(Box<FontEmbedPlan>)`, `AddTextRequest::with_embedded_face(plan)` |

Nothing is missing on the engine side and no engine request is owed. The two
hard stops are both in this repository: the `Std14` type on the pen, and the
fact that `Library` exposes no way to list what it scanned. Its public surface
is `source`, `scan`, `scan_with`, `donor_for`, `len`, `is_empty` — a resolver
keyed by a `/BaseFont` the document already names, never an offer list.

### The build

1. **`Library::names() -> Vec<&str>`**, sorted, from the private `paths` map.
   The engine's `FontEnvironment::named_faces()` exists for the same purpose and
   is called nowhere in this workspace; expose exactly one of the two.
2. **Widen the pen's face** to `enum PenFace { Standard(Std14), Installed(String) }`.
   `pen::FACES` stays as the first group.
3. **The combo grows a second group**, in the shape the restyle chooser already
   ships in `panels/properties/face.rs`: page faces first, faces pdfcer would
   add to the document second, with the disclosure drawn in the popup before any
   addable row. The check
   `the_face_chooser_offers_a_face_the_document_does_not_contain` asserts that
   disclosure is on screen before the click and passes today. Copy that shape
   rather than inventing a second vocabulary for the same idea.
4. **Commit**: on an `Installed` face, read the bytes, `plan_subset` over the
   characters actually typed, `with_embedded_face(plan)`.

### Three things that will bite

- **CFF donors are refused by name.** `pdfcer-core/src/font_embed.rs` takes
  TrueType `glyf` outlines only (`OutlineKind::{TrueType, Cff}`), and a great
  many installed `.otf` files are CFF. The scan already parses each file, so the
  kind is known at scan time: decide there, and when a face is filtered out, the
  count of what was filtered belongs in the popup's disclosure. A silently short
  list is the defect wearing the other coat.
- **`use_os_fonts` defaults to `false`** (`app/prefs/mod.rs`), so the second
  group is empty until the operator finds a Settings checkbox. Default it to
  `true`. This is the same ruling as O180's trimming tick: defaulting it off
  fixes the report only for people who go looking in Settings.
- **No driven check enumerates what the Add Text font list contains.**
  `checks/properties_tool.rs` asserts the region is drawn and never opens the
  combo. That is the coverage hole behind this request.

### Driven check owed

`add_text_offers_a_font_the_machine_has` — arm Add Text, open the pen's font
combo, require a second group with at least one row that is not one of the
fourteen, pick it, type, and require the saved document to carry an embedded
program for it. SKIP, not fail, with a named reason when the machine's font
folders yield no embeddable TrueType face at all.

---

## O189 — bookmarks do not transfer when a page is dragged between documents

The engine has already written this rule down for a different verb, and the
rule should be the same one. `pdfcer-core`'s `pageops` decided that an *extract*
carries every outline entry whose destination lands on an extracted page, keeps
the ancestor headings above it as destination-less containers, and counts what
it dropped. A cross-document page drag is an extract that happens to land in an
already-open document, so it carries the same thing by the same rule. The one
genuinely new question — what happens to the source when the gesture was a move
— is the part that needs the operator's approval, not the carry itself.

### 1. The existing drag path

| stage | where |
|---|---|
| drag state that survives a document switch | `crates/pdfcer-gui/src/pagedrag.rs` |
| the drop resolves a gap and raises the action | `canvas/pagedrop.rs` — `pagedrag::end`, `wants_move`, `insert_position` |
| the cross-document arm | `app/actions/crossdoc.rs` — `PdfcerApp::apply_insert_from_open_document(source_slot, pages, position, take)` |
| the transfer itself | `app/actions/crossdoc.rs` calls `super::pages::insert_from_view(target, &view, pages, position)` |
| the shared insert half | `app/actions/pages.rs` — `pub(super) fn insert_from_view(doc, view, pages, position) -> usize` |
| the engine verb | `pdfcer_core::edit::EditSession::insert_pages(view, pages, position)` |
| the move's second half | `app/actions/crossdoc.rs` — `fn take_pages_from`, running `vector_edit(source, "page-move-take", ..)` around `super::pages::delete` |

One engine verb performs the transfer, and a move is that verb followed by
`delete_pages_with` on the source, each on its own undo stack. `crossdoc.rs`'s
header carries the reasoning for why the default is a copy.

**Why nothing comes across today.** `insert_pages` copies outward from the
pages. `/Outlines` is a catalog entry, unreachable from any page, so the copier
never sees it. The bookmarks are not lost in transit — they were never in the
set of objects being copied. Carrying them means replaying the source outline
through `add_outline_item`, which is what the Bookmarks panel already does by
hand. `text/pages.rs` states this in the module itself.

**What the operator is told today.** `insert_pages` returns `InsertOutcome`
whose `source_outline_dropped` field is computed as a test for whether the
source catalog contains an `/Outlines` key at all — not for whether any bookmark
pointed at a page that was dragged. The shell turns it into one clause in
`text/pages.rs`: *That file's bookmarks did not come across.* So an operator
dragging one page out of a bookmarked manual is already told the bookmarks did
not come across. The report is honest and already off-canvas. What O189 asks for
is not disclosure — it is the capability the disclosure is apologising for.

### 2. What the engine offers for outlines

`D:\Dev\pdfcer\docs\core-api\index.md` documents none of this surface; every
signature below comes from the source.

**Reading the source outline with destinations resolved to page indices.**
`pdfcer-core/src/outline.rs` — `pub fn read_outline<G: ObjectGraph + ?Sized>(graph: &G) -> Outline`.

- Infallible by contract: malformed input yields a partial tree plus populated
  `OutlineDiagnostics`, never a panic, never an unbounded walk. Bounded three
  ways — `MAX_OUTLINE_ITEMS`, `MAX_OUTLINE_DEPTH` (32), and no object visited
  twice.
- It builds a `page_slots`-derived `ObjId` to 0-based index map and resolves
  destinations through it, so items come back as
  `Destination::Page { page_index, view }`, already in the source's page index
  space.
- `DestView` carries the full Table 151 fit style: `Xyz { left, top, zoom }`,
  `Fit`, `FitH`, `FitV`, `FitR`, `FitB`, `FitBH`, `FitBV`. An explicit XYZ
  destination survives reading with its zoom and point intact.
- Unresolvable destinations are kept as the most specific variant the file
  supports: `Destination::UnmappedPage`, `::Named`, `::Remote`, `::NonNavigation`.

It takes any `&dyn ObjectGraph`, and `pdfcer_core::view::DocumentView`
implements `ObjectGraph` and exposes `graph()`. The parked source document the
cross-document arm holds **is** a `DocumentView`, so the source outline is
readable from exactly the borrow the drag already has — no new borrow shape, no
`&mut` on the parked session.

**Inserting an outline entry at a given parent and position.**
`EditSession::add_outline_item(parent: Option<ObjId>, title: &str, destination: Option<Destination>) -> Result<ObjId, EditError>`.
`parent: None` means top level. It maintains `/Count` up the ancestor chain and
opens a parent that was a leaf. It refuses by name rather than silently
dropping: `EditError::UnsupportedDestination` for `Remote` / `UnmappedPage` /
`NonNavigation`, `NamedDestinationNotFound` for a `Named` key nothing defines,
`PageOutOfRange` for a page the target does not have. The refusal happens before
anything is allocated, so a refused add leaves the session byte-for-byte
untouched. Placement anchors are `OutlinePlacement::{FirstChild{parent},
LastChild{parent}, Before{sibling}, After{sibling}}`.

**Two existing import verbs.**

*(a) The clipboard trio*, which already works between two documents:
`copy_outline_item(&self, item_id) -> Result<OutlineClip, EditError>`,
`cut_outline_item(&mut self, item_id) -> Result<OutlineClip, EditError>`,
`paste_outline_item(&mut self, clip, placement) -> Result<OutlinePasteOutcome, EditError>`.

- `OutlineClip` / `OutlineClipItem` are the logical subtree: `title: String`,
  `destination: Option<Destination>`, `open: bool`, `color: Option<[f64;3]>`,
  `style_flags: Option<i64>`, `children: Vec<OutlineClipItem>`. Built by
  `outline_item_to_clip`, a field-for-field copy of `OutlineItem`, so a
  `Destination::Page` carries its XYZ or FitR parameters.
- `OutlineClip::deepest_page()` lets the shell say, before the press, how many
  destinations will not survive in the target. `to_bytes` / `from_bytes` make
  the clip survive leaving the process.
- `paste_outline_item` is one undo entry however many bookmarks arrive; it folds
  with `coalesce_last`, and the engine records that counting intended commands
  instead of committed ones once silently produced three undo entries.
- Its per-item worker `paste_outline_subtree` drops a destination the target
  cannot honour rather than clamping it — a `Destination::Page` survives only
  while `page_index < pages`, otherwise `outcome.destinations_dropped` rises and
  the entry arrives destination-less. Reported as
  `OutlinePasteOutcome { items_pasted, destinations_dropped }`.

**The gap, stated precisely: `paste_outline_item` does not remap page indices.**
It takes `page_index` from the source's page space and tests it against the
target's page *count*. For O189 the pages land at a known offset, so every
carried `Destination::Page` needs `page_index` rewritten from source space to
target space before the paste. That rewrite exists nowhere in the engine.

The GUI already drives this trio: `panels/bookmarks/clip.rs` calls
`copy_outline_item` and anchors with `OutlinePlacement::LastChild`, and
`app/actions/bookmarks.rs` calls `paste_outline_item`. Acrobat cannot copy
bookmarks between files at all, so this is already an exceed.

*(b) The merge path*, which carries outlines wholesale.
`merge_document(&mut self, source: &DocumentView, position) -> Result<MergeOutcome, EditError>`
sets `MergeOutcome::outline_items_carried` from `merge_outline`, which imports
the source's top-level `/First` / `/Next` chain through `import_object` on the
same `PageSplice::mapping` the pages used — so `/Dest` page references re-point
automatically — then splices the arrivals onto the end of the target's top-level
chain via `append_outline_items`, creating `/Outlines` when the target has none.
The shell reports it in `text/pages.rs`: *N bookmark(s) came across, added after
this document's own.*

**This works only because a merge takes every page.** The mapping contains every
source page, so no `/Dest` can point outside it. For a page subset the same code
is actively dangerous — see the hazards list.

*(c) The policy that already answers the hard question.* The engine has written
this ruling down for extract and split in `pdfcer-core/src/pageops/outline.rs`
and, for the enum itself, `pdfcer-core/src/pageops/assemble.rs`:

```rust
pub enum OutlinePolicy {
    /// Carry every outline entry whose destination lands on a copied
    /// page, rewritten to point at the copy; drop the rest, counted.
    Subset,
    /// One top-level entry per source document, that source's entries nested.
    PerSource,
    /// No outline at all.
    Drop,
}
```

`pageops/outline.rs`'s header says why insert got neither: insert carries the
target's outline and does not import the source's, matching the recommended
default that plain Insert has no bookmark carryover and that a bookmark-carrying
insert should be a distinct, explicitly named mode if ever added. **A
cross-document page drag is that distinct, explicitly named mode.** It is not
plain Insert-from-file: the operator picked specific sheets out of a document
open in front of them, with its bookmark tree visible in a panel.

Two working parts of the `Subset` implementation, because the build below
reimplements them at the logical level:

- `Item::is_kept` — an entry is kept when its own destination is a kept page or
  any descendant's is. A container whose only value is holding kept children
  survives; dropping it would reparent its children to the root and destroy the
  hierarchy the operator can see on screen.
- `prune` — a kept container whose own destination left keeps its title and its
  children and simply has no destination: clicking it does nothing, expanding it
  works, which is what a chapter heading whose title page was not extracted
  should do. Dropped subtrees are counted whole by `count_all`, because
  reporting one dropped bookmark for a forty-entry chapter would be true and
  useless.

That answers the row note's objection — moving one page out of a five-page
chapter cannot carry the chapter heading without deciding what happens to the
other four — because pdfcer already decided it, in writing, for extract.

### 3. What happens today to a destination that pointed at a moved page

**In the source, after a move, a dangling entry is kept.** It is not fixed up,
and that is deliberate and documented. `delete_pages_with` runs `census_dangling`
before the splice and returns `DeleteOutcome.dangling: DanglingReport`
(`pdfcer-core/src/pageops/references.rs`):

| field | meaning |
|---|---|
| `outline_items` | outline items whose destination is a removed page |
| `links` | link annotations on surviving pages pointing at a removed page |
| `non_link_annotations` | non-link annots on surviving pages whose `/A /GoTo` names a removed page |
| `named_destinations` | named destinations resolving to a removed page |
| `page_labels_stale` | `/PageLabels` is now numerically stale |

Nothing repairs the outline; `delete_pages`' own doc contrasts it with
`DeleteOutcome::separations`, the one class of broken reference pdfcer does
repair.

It is already reported on the drag path. The move's second half captures the
source's notes and re-files them under the target's epoch, because `app::status`
draws only the active document's disclosure and a note filed against the parked
source would be recorded, correct, and invisible. The sentence is
`text/pages.rs`'s `deleted_dangling_bookmarks`: *N bookmarks now point at pages
that are no longer in this document.*

So a Shift-drag of a bookmarked page today produces, on the target's status row,
the removal sentence plus that dangling sentence — both true, and the second is
about the source while the operator is looking at the target. That is a live
wording defect, addressed below.

**In the target, after either a copy or a move**, nothing arrives. Its own
outline is untouched and the operator gets *That file's bookmarks did not come
across* whenever the source had an `/Outlines` key, whether or not any bookmark
pointed at a dragged page.

### 4. The rule — alternatives, argued

Four candidates. For each: what happens to the source, to a split chapter
heading, to an explicit XYZ destination, and to a target with no outline.

**Candidate 1 — flatten to the target's top level, in page order.** Carry only
entries whose own destination is one of the moved pages; discard ancestry;
append sorted by landing page.

- Source: unchanged on a copy; dangling entries left and counted on a move.
- Split chapter heading: lost. Dragging pages 3 and 4 of a five-page chapter
  gives two bookmarks named after the two sub-headings and no clue which chapter
  they were in. On a drawing set where every sheet's bookmark is *Sheet 12*, the
  target gains twelve identically shaped entries with no discipline heading.
- XYZ destination: survives; `DestView` is carried whole, only `page_index` is
  rewritten.
- Target with no outline: `/Outlines` is created.

Cheapest to build, and the only candidate that loses information the operator
can see in the Bookmarks panel at the moment they drag. **Rejected.**

**Candidate 2 — preserve ancestry, emitting the ancestor headings as containers.**
Carry every entry whose destination is a moved page, plus every ancestor of such
an entry, each surviving ancestor emitted as a destination-less container
carrying its original title.

- Source: identical to candidate 1.
- Split chapter heading: the heading comes across as a container with only the
  moved children under it. The other pages' entries stay where their pages are.
  Clicking the heading in the target does nothing; expanding it works. This is
  `prune`'s documented behaviour and what extract already ships. The wrinkle: if
  the heading's own destination happened to be one of the moved pages (a title
  page), it is kept with its destination, remapped — which is exactly what
  `prune`'s kept-pages filter gives.
- XYZ destination: survives whole.
- Target with no outline: created.
- Where: appended at top level, after the target's own entries, in the order the
  carried roots appeared in the source. This matches `merge_outline` and
  `append_outline_items`, and matches the existing merge sentence *added after
  this document's own*, so no second placement rule is introduced.

**This is the recommendation**, in order of weight:

1. **It is already pdfcer's ruling.** `OutlinePolicy::Subset` is exactly this,
   applied to extract and split. Deciding the opposite for a drag would mean the
   same operator, on the same five-page chapter, gets a chapter heading from
   Extract and no chapter heading from a drag — a divergence with no defence.
2. **The information exists and is free.** The source outline is readable from
   the `DocumentView` the arm already holds, destinations come back already
   resolved to page indices, and the target offset is already computed as
   `landing` inside `insert_from_view`.
3. **It answers the row note's objection honestly.** A chapter heading whose
   children were split is not a contradiction; it is a container in both
   documents, describing whichever pages each document actually holds.
4. **The failure mode is benign.** A destination-less heading is a legal §12.3.3
   shape — `paste_outline_item` relies on exactly that — and the worst an
   operator suffers is a click that does nothing on a node that expands.

**Candidate 3 — carry nothing, report off-canvas what was left behind.** This is
what ships today, and it is a correct rule, not a bug: the disclosure is
accurate and prominent. The complaint is that it apologises rather than doing
the work. **Rejected as the rule; retained as the decline path** for when the
carry cannot run.

**Candidate 4 — candidate 2, plus the mirror on the source.** When the gesture
is a move, also remove from the source the entries that were carried and whose
destination left with the page, by the same `is_kept` test, so the source is not
left with dangling bookmarks to clean by hand.

For it: a move that carries a bookmark to the target and leaves a broken copy in
the source has produced a third state, which is the exact failure `crossdoc.rs`
already names for pages.

Against it, and this is why it is the operator's call:

- It is a third command on the source's undo stack. A cross-document move is
  already two edits on two stacks with no single Ctrl+Z; `cut_outline_item`
  coalesces only its own pair.
- It deletes something the operator did not point at. A container heading whose
  destination left is still a heading for the pages that stayed.
- A narrower form exists: delete only leaf entries whose destination was a moved
  page and leave every container alone. That is defensible and small.

**Recommendation: ship candidate 2; put candidate 4 to the operator as a
follow-on, defaulted off.** Candidate 2 is purely additive and cannot make any
document worse than it is today; candidate 4 mutates a document the operator is
not looking at, on the gesture whose undo story is already the weakest in the
application.

**The rule, stated for the operator to approve.**

When pages are dragged from one open document into another, the bookmarks that
point at those pages come with them. A bookmark comes across when it points at
one of the pages dragged. Its parent headings come too, so the pages arrive
under the chapter they were in — but a heading whose own page stayed behind
arrives as a heading only: it expands, clicking it goes nowhere. Bookmarks
pointing at pages that were not dragged do not come.

The arriving bookmarks are added after the target document's own, and they keep
their zoom and position. A target with no bookmarks gets a bookmark list.

Bookmarks pdfcer could not carry — one that opens another file, or one whose
destination this file never defined — are left behind and counted, on the status
row, at the moment it happens.

The source document is not changed. On a Shift-drag the pages leave and its
bookmarks that pointed at them stay, now pointing at nothing, which pdfcer
already says. Cleaning those up is a separate decision.

### 5. The build

Three edits, one new text function, one new check. No engine change is required
— everything needed is public — but the last subsection argues for one.

**5.1 A new shell-side function: collect the carryable subtree, remapped.**
New in `app/actions/crossdoc.rs` (the only arm that reads two documents), or in
a sibling `crossdoc/outline.rs` if the arm gets long:

```
fn carried_outline(
    source: &DocumentView<'_>, // already held by the arm
    moved: &[usize],           // source page indices, as the action carries them
    landing: usize,            // target index where the first sheet lands
) -> (OutlineClip, CarryReport)
```

Algorithm, mirroring `pageops::outline::prune` at the logical level:

1. `let outline = pdfcer_core::outline::read_outline(source.graph());` Check
   `outline.diagnostics.is_faithful()` — a truncated tree must not be carried as
   if it were the document's own.
2. Build `remap: HashMap<usize, usize>` from `moved`: source page index to
   target page index. `copy_and_splice` iterates `source_pages` **in the order
   given**, without sorting, and inserts each imported page reference
   contiguously at the splice point, so the *k*-th entry of `moved` as the
   action carries it maps to `landing + k`. Do not sort `moved` first; sorting
   would be correct only where the caller's order happened to be ascending.
3. Depth-first over `outline.items`, recursion capped at `MAX_OUTLINE_DEPTH`:
   - kept = this item's destination is a `Destination::Page` whose `page_index`
     is in `remap`, or any descendant is kept;
   - a kept item emits an `OutlineClipItem` with `title`, `open`, `color`,
     `style_flags` copied verbatim (as `outline_item_to_clip` does), `children`
     = the kept children, and `destination` =
     `Some(Destination::Page { page_index: remap[old], view: dest_view })`, where
     `dest_view` is the item's own `DestView` carried through unchanged,
     when its own destination is a moved page, else `None`;
   - a dropped subtree increments `report.dropped` by `1 + count_all(children)`,
     because reporting one for a forty-entry chapter is true and useless;
   - a kept item whose destination is `Named`, `Remote`, `UnmappedPage` or
     `NonNavigation` emits with `destination: None` and increments a **separate**
     `report.unresolvable`, because the remedy differs.
4. Return the clip and the report.

Rebuild the clip rather than calling `copy_outline_item`: that verb takes one
`item_id`, copies a whole subtree unfiltered, has no remap, and needs `&self` on
the source *session*, which the arm does not hold — it holds a `DocumentView`.
`read_outline` on the view is the right borrow and the right granularity.

**5.2 Splice it into the arm.** In `apply_insert_from_open_document`, after
`insert_from_view` returns a non-zero count and while the target is still
borrowed:

```
let inserted = super::pages::insert_from_view(target, &view, pages, position);
if inserted == 0 { return; }                     // unchanged ordering discipline
let (clip, carry) = carried_outline(&view, pages, landing);
if !clip.is_empty() {
    target.session.paste_outline_item(&clip, OutlinePlacement::LastChild { parent: None })
}
```

through the existing `super::apply::vector_edit` funnel, so the epoch bump,
render-worker cancel and `pages::resync` happen as they do for every other edit.
That funnel is not optional even for a parked document.

**Ordering is insert-then-outline and cannot be reversed**, for the same reason
the arm gives for insert-then-delete: `paste_outline_item` tests `page_index`
against the target's *current* page count, so pasting first would drop every
destination.

**`landing` must be lifted out of `insert_from_view`.** It is computed locally
from the `InsertPosition` (Start to 0, End to `doc.pages.len()`, `Before(n)` to
`n`, `After(n)` to `n + 1`, otherwise the current view page) and never returned;
the function returns only a `usize` count. Change the return to
`{ inserted: usize, landing: usize }`. `insert_from_view` has exactly two
callers, in `app/actions/pages.rs` and `app/actions/crossdoc.rs`.

**5.3 Two commands on one stack, folded.** `insert_pages` commits one
`CommandKind::InsertPages`; `paste_outline_item` coalesces its own into one
`CommandKind::PasteOutlineItem`. They are on the **same** stack — the target's —
so unlike the insert/delete pair this one is foldable, and
`EditSession::coalesce_last(count, kind) -> bool` is **public**. Fold the two
into one entry with it, so a single Ctrl+Z undoes the arrival of pages and
bookmarks together. If the fold is declined, disclose the two-step undo in
words rather than leaving it silent.

**5.4 The engine change worth asking for (not required).** `insert_pages`
already computes the splice mapping and then discards it — there is a literal
`let _ = &mapping;` in the body, and that line is the hook. An
`insert_pages_with(..., OutlinePolicy)` reusing `pageops::outline`'s
`Item` / `is_kept` / `prune` against `PageSplice::mapping` would remap by object
identity instead of by position, removing the positional assumption above
entirely; fold to one command inside the engine; and put the rule in the one
place `pageops/outline.rs` already documents it, rather than in a second
shell-side implementation that will drift. File it as a request; ship the
shell-side build meanwhile.

**5.5 The text.** New function in `text/pages.rs`, beside `inserted` and
following `deleted_dangling_bookmarks`' singular/plural discipline — spelled
out, never `1 bookmark(s)`:

```
pub fn bookmarks_carried(carried: usize, dropped: usize, unresolvable: usize) -> Option<String>
```

- `carried > 0` — *N bookmarks came across with those pages, added after this
  document's own.* Reuse the merge sentence's phrasing verbatim so two verbs do
  not describe one outcome two ways.
- `dropped > 0` — *M bookmarks pointed at pages you did not drag and stayed in
  `<source>`.*
- `unresolvable > 0` — *K bookmarks opened another file or a destination this
  file does not define, and could not be carried.* A separate clause because the
  remedy is separate: nothing the operator can do here, versus drag the other
  pages too.
- Each clause is dropped when its count is zero, per the `Structures` ruling in
  the same module: a clause that always appears is a disclaimer rather than a
  disclosure.

**`Structures::outline_dropped` must stop firing when the carry ran.** It is set
from a bare `/Outlines` key test and prints *That file's bookmarks did not come
across*, which the carry makes false. The shell must suppress it on the
cross-document path whenever `carried > 0`; an engine-side `insert_pages_with`
would set it correctly instead.

### 6. Rule 4 — what gets reported, and where

The carry performs inferences the operator cannot see, and each owes a report:

| inference | report |
|---|---|
| which bookmarks matched the dragged pages | *N bookmarks came across…* |
| which were left behind because their pages stayed | *M bookmarks pointed at pages you did not drag and stayed in `<source>`.* |
| which could not be expressed at all (remote, named-undefined, unmapped) | *K bookmarks opened another file or a destination this file does not define, and could not be carried.* |
| a heading arrived without its destination because its own page stayed | *L headings came across without their own page — they group the sheets that arrived; clicking them goes nowhere.* |
| the source outline was truncated by `read_outline`'s guard rails | *That document's bookmark list is larger than pdfcer reads, so this carried only the part it can see.* |

Every clause is dropped at zero. `OutlineDiagnostics::is_faithful()` is the
single question for the last one, and `read_outline`'s contract is that a
truncated tree must be presented as truncated.

**Where:** the status-row edit disclosure, via
`super::record_edit_disclosure(Some(EditDisclosure { epoch, notes }))` in
`app/actions/disclosure.rs`, as the move arm already does. `app/status.rs` names
this lane as rule 4 — what a move or a delete had to change. Not a badge on a
page, not a modal, not a Bookmarks-panel decoration.

**Epoch:** the target's. `app::status` draws only the active document's
disclosure, and the target is the document on screen.

**Order:** what happened before what it cost — the carry's sentence first, then
the leftovers, then the source's dangling report on a move. That is the
application-wide shape the arm already states.

**One live wording defect this exposes.** On a Shift-drag the operator sees, on
the *target's* status row, *3 bookmarks now point at pages that are no longer in
this document* — where "this document" means the source. With the carry shipped
the target will also be gaining bookmarks in the same breath, and the two
sentences will read as contradicting each other. The sentence needs a
source-naming variant for the move path — *…no longer in `<source>`* — and that
variant is required by this change, not optional.

### 7. What will bite

1. **`paste_outline_item` refuses the whole paste above half `MAX_UNDO_DEPTH`
   items**, with `SelectionTooLargeForOneUndo`. Dragging fifty pages out of a
   densely bookmarked manual can trip it. The decline path is candidate 3 —
   carry nothing, say why, in words that name the limit.
2. **`add_outline_item` refuses on an encrypted or certified target**, and so
   does `paste_outline_item` through it — but `insert_pages` has already
   committed by then. The pages land and the bookmarks do not, so the decline
   must be caught and reported, never allowed to unwind the insert.
3. **`Destination::Named` is a trap.** A carried `Named` destination is refused
   by `add_outline_item` with `NamedDestinationNotFound` unless the key is
   defined in the target first, and `insert_pages` does not carry `/Dests` —
   only `merge_document` does, via `merge_named_destinations`. **Every `Named`
   destination is unresolvable on this path.** Route them to `destination: None`
   plus `report.unresolvable` rather than letting the paste refuse.
4. **Truncation.** A hostile or merely enormous `/Next` chain is capped at
   `MAX_OUTLINE_ITEMS`. Carrying a truncated tree silently is precisely the
   rule-4 violation `read_outline`'s contract warns about.
5. **Do not reach for `merge_outline` for a page subset.** It imports the outline
   dictionaries verbatim through `import_object`, which follows `/Dest`
   references; on a subset, a bookmark pointing at an undragged page would drag
   that page's object into the target as a loose object. It is safe for merge
   only because merge takes every page.
6. **The source's `is_modified` guarantee must not change.** The arm promises
   that on a copy nothing is written to the source and nothing is read out of it
   destructively — the whole reason the gesture is safe to try. `carried_outline`
   reads through `&DocumentView` only. Candidate 4 would break this promise on
   the move path, which is a second reason it is the operator's call.
7. **`/Count` arithmetic in the target.** `add_outline_item` maintains `/Count`
   up the chain and opens a leaf parent that gains a child. Pasting at
   `LastChild { parent: None }` touches only the root, which is the least
   surprising place for it — but a target whose root `/Count` was already wrong
   in the file will now be wrong by a different amount. Not a regression; worth
   knowing.
8. **The Bookmarks panel's own cached tree.** A paste raised from the
   cross-document arm rather than from the panel may not refresh it. Establish
   that before the check is written, or the check will pass against a stale
   panel.

### 8. The driven check owed

The existing cross-document drag check is `checks/page_drag_between_documents.rs`,
registered in `checks/mod.rs` and `checks/roster.rs` as two instances,
`PageDraggedBetweenDocuments::COPY` and `::MOVE`, whose `fn name` returns
`a_page_dragged_between_documents_is_copied` and
`a_shift_drag_between_documents_moves_the_pages`. For the current check count,
run `ls tools/ui-verify/src/checks/*.rs | wc -l`.

**New file:** `tools/ui-verify/src/checks/page_drag_carries_bookmarks.rs`,
three `const` instances on one struct following the `COPY` / `MOVE` pattern,
added to `checks/mod.rs` and `checks/roster.rs`.

**Fixture:** a source document with five pages and a three-level outline —
`Chapter One` (destination page 1) over `Section A` (page 2), `Section B`
(page 3), `Section C` (page 4), plus `Appendix` (page 5); a target document with
two pages, in two variants, one with no outline and one with a bookmark of its
own. Whether `tools/ui-verify/src/fixture.rs` can author an outlined fixture is
not established; it may need building.

| check | assertion, on dragging source page 3 into the target between its two sheets |
|---|---|
| `a_page_dragged_between_documents_brings_its_bookmark` | the target's Bookmarks panel gains `Chapter One` over `Section B`; `Section B`'s destination resolves to the target's page 2, the landing index, not page 3; the status row carries the carried-count clause |
| `a_bookmark_for_a_page_left_behind_stays_behind` | the target's panel does not contain `Section A`, `Section C` or `Appendix`; the status row states how many stayed behind and names the source |
| `a_chapter_heading_whose_page_stayed_arrives_as_a_heading` | `Chapter One` is present in the target and expands, and clicking it does not navigate, its own page 1 having stayed; the status row carries the heading-without-its-page clause |

Each also asserts the source's page count and bookmark count are unchanged on
the copy drag, and, on a Shift variant, that the dangling-bookmark sentence names
the source rather than saying "this document".

The existing check's header documents the four mechanisms no unit test can
reach; cite it rather than restate it, and add the fifth: **the outline carry
reads a document that is not on screen, through a borrow that exists only for
the duration of the drop.**

### 9. What needs the operator's decision

1. **Approve the rule as stated in §4** — candidate 2: carry matching entries,
   keep ancestor headings as destination-less containers, drop the rest and
   count them. It is the rule pdfcer already applies to Extract and Split.
2. **Should a Shift-drag also clean up the source's now-dangling bookmarks
   (candidate 4)?** Recommendation: no, not in this change. It adds a third undo
   entry to the gesture with the weakest undo story in the application, and it
   deletes from a document the operator is not looking at. If yes, take the
   narrow form — leaf entries only, containers untouched.
3. **Where the carried bookmarks land.** Recommendation: appended after the
   target's own top-level bookmarks, matching merge. The alternative — inserting
   them at the position corresponding to where the pages landed — reads better on
   paper and requires deciding what "corresponding" means when the target's
   outline order and page order disagree, which they frequently do.
4. **The dangling-bookmark sentence must start naming the source document.**
   This is a wording fix the change forces; it alters an existing shipped
   sentence and so needs sign-off.

---

## O183 — the nine-part ce-dimension request

**Rule 15 governs every line below.** A **ce dimension** is one pdfcer authors —
a `/Line` (or `/Polygon` / `/PolyLine`) annotation with `/IT /LineDimension`, a
baked `/AP`, and a `/PieceInfo /pdfcer` sidecar carrying its measurement model.
A **pdf dimension** is CAD-exported page content pdfcer reads and must not
silently alter. O183 is entirely about ce dimensions. No fix here touches page
content; anything that did would be a defect, not a feature.

**Six of the nine are already built.** The dominant failure is not missing
capability — it is that a click on a ce dimension opens the comment window
instead of leading the operator to the Properties panel where all of this
already lives. Item 7 is the keystone: a small addition at one gate, and closing
it makes items 2, 4, 8a and 9 stop being complaints without writing a single new
control. Item 1 is half-true and its real defect is narrower than reported.
Item 3 is true and is a recorded decision, not a gap. Item 5 is a genuine scope
question with a partial answer in the engine. Item 6 is half-shipped: the live
preview exists on both create and move, the text-versus-line disclosure does
not. Item 8 is half-shipped: radius-versus-diameter ships; leader location,
leader style and centre mark are absent from the engine and need a hand-off.

### 1. "Units cannot be changed after creation" — partly false, with a different defect underneath

**Unit is changeable on an existing ce-dimension group.** The control is
`egui::ComboBox::from_id_salt("dimension-group-unit")` in
`panels/dimension_groups/mod.rs`, populated from `Unit::all()`, raising
`Action::Dimension(DimensionAction::SetGroupScale { group, scale, format })`.
The group's scale is carried through unchanged — `set_group_scale` takes both,
and passing anything but the group's own scale would recalibrate the drawing
while the operator was only changing a unit. The action reaches
`session.set_group_scale(group, scale, format)` in `app/actions/dimensions.rs`.

**The real defect is the fraction mode.** The unit combo passes
`format: unit.default_format()`. The argument for that is sound — a
`NumberFormat` is a unit *and* how its fractional part is written, and carrying
eighths across a change to millimetres would produce a format nobody's drawing
uses — but the consequence is the report: an operator who has set
inches-in-sixteenths and then changes the unit loses the sixteenths, and there
is **no fraction or precision control anywhere on an existing group** to put it
back.

The only `fraction_combo` in the shell is in the Set-scale dialog
(`dialogs/scale.rs`), fed by `const FRACTIONS` in the same file — Decimal 0/1/2/3
and Fraction 8/16/32 — committed through `DimensionAction::SetGroupScale`. The
dimension-groups panel's rows are unit, scale and drafting standard, plus
rename/delete in `panels/dimension_groups/identity.rs`, and draw-into. There is
no fraction row.

The engine has no objection: `set_group_scale` takes a whole
`NumberFormat { unit, fraction, decimal_marker }`
(`pdfcer-core/src/dimension/units.rs`), so a fraction picker on an existing group
is purely a shell addition. The engine's feature table records that the unit is
writable only as a side effect of `set_group_scale`, which is the sanctioned
route rather than a hole.

**Class:** shipped-but-undiscoverable for the unit half; present in the engine
and unexposed in the shell for the fraction half.

**Fix, shell only.**

1. Add a fraction/precision row to `panels/dimension_groups/mod.rs` beside the
   unit combo, reusing `dialogs/scale.rs`'s `FRACTIONS` rather than inventing a
   second list.
2. When the unit combo changes, keep the operator's fraction mode if it is
   coherent with the new unit; otherwise fall to `default_format()` **and say so
   in the panel**. Rule 4: the substitution must be visible.
3. Per-ce-dimension precision already exists and overrides this
   (`panels/properties/dimension/overrides.rs`), so the group row is the default,
   not the only control.

**Driven checks owed:** `an_existing_group_offers_a_fraction_control`,
`changing_a_group_unit_discloses_a_fraction_it_could_not_keep`.

### 2. "The per-dimension override does nothing" — false as a capability claim; it is masked by item 7

All eleven cascade properties are drawn on the per-ce-dimension surface, by
`pub fn show(ui, group, overrides) -> bool` in
`panels/properties/dimension/overrides.rs`: unit, fraction/precision, decimal
marker, drafting standard, text height, line width, arrow length, arrow form,
colour, tolerance, tolerance precision. A test asserts the set is complete —
`const DRAWN: [&str; 11]` and `fn no_property_of_the_cascade_is_left_without_a_row`,
checked against the engine's
`StyleProvenance::each() -> [(&'static str, StyleSource); 11]`.

The action path is whole: `panels/properties/dimension/mod.rs` reads
`record.style`, calls `overrides::show`, and on a validated change raises
`DimensionAction::SetStyle { dimension, style }`; `app/actions/dimensions.rs`
routes it to `session.set_dimension_style`, wrapped in
`super::apply::vector_edit(doc, "set-dimension-style", 0, 1, ..)`. The panel is
drawn, not dead code — `panels/properties/mod.rs` calls `dimension::section` and
feeds its return into `something_drew`.

**Why the operator sees nothing.** Two candidates, both addressed elsewhere in
this survey rather than by building anything here:

1. **Item 7 masks it.** The click pops the comment window. The click also
   selects — the popup consumes nothing — so the Properties section *is*
   populated on the same click; the operator's attention is taken by the window
   that appeared on top of the canvas.
2. **Provenance can read as nothing happening.** `StyleSource::follows_group()`
   returns `true` for **both** `Factory` and `Group`, which the shell's own
   header flags as the easy thing to get wrong. A row whose checkbox is ticked
   but whose resolved value equals the inherited one draws identically. This is
   a hypothesis, not a measured defect; drive it before closing.

**One structural note that reads as a bug and is not one.** `unit`, `fraction`,
`decimal_marker` and `standard` have concrete fields on `Group` rather than
`Option`s, so their provenance can only ever be `Group` or `Dimension`, never
`Factory`, and the panel never prints a factory sentence for those four. That is
deliberate: saying "factory" for them would be a lie an operator could act on.

**Class:** shipped and undiscoverable. A discoverability fix is still owed, and
the cheapest one is item 7.

**Driven check owed:** `a_unit_override_on_one_ce_dimension_changes_what_is_drawn`.

### 3. "Tolerance is missing from the group entirely" — true, and it is a written decision

The engine supports it: `GroupStyle` (`pdfcer-core/src/dimension/style.rs`)
carries seven `Option` fields, of which two are `tolerance` and
`tolerance_places`.

The shell deliberately does not draw them. `panels/dimension_groups/style.rs`
draws only the other five and carries a section arguing the case: a tolerance is
a statement about one manufactured feature, and two holes on the same drawing
routinely carry different ones even though they share the drawing's units and
precision. So tolerance belongs on the per-ce-dimension surface, where it is
drawn, and a second control at group tier would invite an operator to set a
default that almost every member then overrides — the shape that makes the
moving-count read as no change on screen on nearly every press. It is reachable
from the CLI for the drawing that genuinely wants one.

That moving-count is real: `fn will_move` in the same file computes how many
members a group-tier change visibly moves, and the panel shows it per press.

**Class:** present in the engine, deliberately unexposed. This is an
operator's-decision item, not a bug and not a hand-off. If the decision is
overruled, the work is small — a tolerance row and a tolerance-precision row in
`panels/dimension_groups/style.rs` using the existing
`panels/properties/dimension/tolerance.rs` editor, routed through
`DimensionAction::SetGroupStyle` to `session.set_group_style`.

**The counter-argument is answerable.** SolidWorks carries a document-level
tolerance default, and the standing position is that SolidWorks is the floor, not
the ceiling. The shell's objection is not that a group default is wrong but that
it reads as a no-op press — which is a disclosure problem, and the panel already
solves disclosure problems the same way everywhere else: show the count of
members that will change, including when it is zero, and say why it is zero.

**Driven check owed:** `a_group_tolerance_default_reports_how_many_members_take_it`.

### 4. "Tolerance does not work on the override either" — false; same mask as item 2

`panels/properties/dimension/overrides.rs` draws the tolerance row
(`valid &= super::tolerance::show(ui, value, unit)`) and the tolerance-precision
row. `panels/properties/dimension/tolerance.rs` provides
`pub fn show(ui, value, unit) -> bool`, `const FORMS: [Tolerance; 7]`, a drag
speed constant, `fn same_form` and `fn reshape`. It renders fields, not a
specimen: the label is never previewed by concatenation, because a limit
tolerance suppresses the nominal and only `author_dimension` knows the exact
baked string.

**It raises nothing until it validates.** `overrides::show` returns `valid`, and
`panels/properties/dimension/mod.rs` pushes `SetStyle` only when
`valid && next != record.style`. An inverted limit pair is refused with the
engine's own sentence through `text/panels/dimension.rs`'s `tolerance_refused`,
with `tolerance_unit_note` carrying the note that tolerance values are in the
**displayed unit**, not points.

`Some(Tolerance::None)` is not `None`: the first is an override saying this ce
dimension carries no tolerance, the second is inherit. The checkbox is the
difference and both are legitimate states.

**Class:** shipped and undiscoverable. Closing item 7 closes this.

**Driven check owed:** `a_symmetric_tolerance_typed_on_one_ce_dimension_is_drawn`.

### 5. "Tolerance control is thinner than SolidWorks'" — the scope question

Do not promise anything here until SolidWorks is measured.

`pdfcer-core/src/dimension/tolerance.rs` implements seven of `swTolType_e`'s
thirteen forms: none, basic (boxed), bilateral/deviation, limit, symmetric, min,
max.

| SolidWorks form | Status | Reason recorded in the engine |
|---|---|---|
| `swTolFIT`, `swTolFITWITHTOL`, `swTolFITTOLONLY` | not implemented | ISO 286 fit classes; the reference RAG's class list is flagged unverified, and a wrong `H7/g6` deviation is a manufacturing defect |
| `swTolBLOCK`, `swTolGeneral` | not implemented | need a general-tolerance table pdfcer does not have |
| `swTolMETRIC` | not implemented | shares enum value 7 with `swTolFIT`; cannot be distinguished |

Item by item against what was asked:

| asked for | state |
|---|---|
| bilateral | shipped, as the deviation form |
| limit | shipped — and it suppresses the nominal; never preview a limit label by concatenation |
| symmetric | shipped |
| min / max | shipped |
| fit | not built; blocked on an unverified ISO 286 table |
| precision per side | not built, and it is a data-model change, not a control — there is exactly one `tolerance_places`, on `GroupStyle` and on `StyleOverrides`, where SolidWorks carries upper and lower precision independently |

**What must be measured in SolidWorks before any promise:**

1. Does SolidWorks let the upper and lower tolerance carry different decimal
   places on the same dimension, or is that only true across the two fields of a
   limit display? This decides whether "precision per side" is one new field or a
   twelfth cascade property, with everything that implies — `StyleOverrides` is
   eleven fields and a test counts them.
2. Which fit-class table does the shop use: ISO 286-2, ANSI B4.1, or a house
   table? A sourced table turns three tolerance forms from blocked into a day of
   typing.
3. Is block/general tolerance a per-dimension property in this workflow, or a
   title-block note?

Until those are answered this item has no estimate. It is the only item in O183
with no tranche, and the engine's refusal to guess is the right refusal.

### 6. "No live preview when creating or moving, and no way to tell TEXT from LINE" — first half false, second half real

**The preview ships, on create and on move, drawn by the same function that
draws the committed article.** `canvas/dimdrag.rs` states the rule: the segments
come from `measure::pick::dimension_preview_segments`, the same function a
committed dimension is drawn from.

- Create: `canvas/measure/pick.rs` — `pub fn placing_preview(&self, p: Point) -> Option<DimensionKind>`
  and `pub fn dimension_preview_segments(kind: &DimensionKind) -> Vec<(Point, Point)>`;
  wired from `canvas/measure/mod.rs` for the scale/line, linear and remaining
  tools, with the `Preview` struct and `pub(super) fn preview` in the same
  module. Asserted by the unit test
  `the_placing_preview_is_exactly_what_the_placing_click_authors`.
- Move and drag: `canvas/dimdrag.rs` returns
  `Some(dimension_preview_segments(&moved))` every frame that is not the
  committing frame. Nothing is previewed on the committing frame: the annotation
  is about to be regenerated and drawn for real, and a preview left over it would
  be a second copy of the same line, one frame stale. A vertex drag
  (`pub fn drag_vertex`) uses the same function plus a snap marker, and
  `pub fn annot_shapes` supplies the shapes for a selected ce dimension from the
  same segment source.
- Painted through `canvas/painting.rs`'s
  `pub dimension_preview: Option<&'a [(Point, Point)]>`, fed from
  `canvas/interact.rs`, stored in `canvas/previews.rs` and `canvas/dragroute.rs`.

**The second half is real: nothing says whether the text or the line is being
dragged.**

- **There is one grab box and it is the whole `/Rect`.** `canvas/dimdrag.rs`'s
  `pub fn grab_box` returns `map.rect_to_screen(annot.outline)` — label and lines
  together.
- **One drag moves both.** The commit pushes
  `DimensionAction::Place { dimension, offset, text_along }`: `offset` is the
  standoff perpendicular to the measured axis, `text_along` is where the label
  sits along the dimension line, and both are set from one gesture. This is
  `place_dimension`, the value-preserving verb, distinct from `move_dimension`,
  which translates the measured points.
- **The label is deliberately excluded from the selection shapes.**
  `annot_shapes`' doc says a dimension is selected by its lines, and that if that
  proves too strict in use, the fix is to ask the engine for the label's box
  rather than to guess one in the shell. That sentence is the hand-off this
  requirement needs.
- **No distinct cursor exists.** `canvas/cursor.rs` gives custom shapes only to
  Crosshair and Text/Ibeam; `canvas/tool/arm.rs` maps `DragKind` to `CursorIcon`
  with `Move`, `Rotate` and `MarkupVertex` all to `Grabbing` and `TextSelect` to
  `Text`. There is no ce-dimension-label case.
- `canvas/dimdrag.rs`'s header records that the label drag itself failed twice
  before it worked, so this surface is harder than it looks.

**Class: split.**

- **Shell-only and cheap:** two cursors or two hover highlights — the dimension
  line lit one way, the label anchor another. This is a pre-commit affordance
  Rule 4 welcomes; it changes no document state.
- **Engine-dependent:** the shell does not know the label's box. `annot.outline`
  is the whole `/Rect`. To highlight or hit-test the label region the shell must
  ask the engine for its rectangle, and guessing is forbidden.

**The intermediate that needs nothing from the engine is two grips rather than
two highlights.** Drag the dimension line to set `offset`; drag a small handle at
the label anchor to set `text_along`. The anchor position is derivable from
`text_along` and the measured axis, both of which the shell holds. Per R9, if the
label handle cannot be honoured for a kind — a circular ce dimension has no axis
to slide along, and `place_dimension` refuses it by name — the handle is **not
drawn**, never greyed.

**Driven checks owed:** `hovering_a_ce_dimension_line_says_it_is_the_line`,
`hovering_a_ce_dimension_label_says_it_is_the_label`,
`the_label_handle_moves_the_label_and_not_the_line`.

### 7. "Clicking a ce dimension pops the comment box" — the keystone

**True, root cause measured, and it is a small addition at one gate.**

1. **Every ce dimension is authored with a non-empty `/Contents`.**
   `pdfcer-core/src/dimension/author.rs` inserts the printed measurement as
   `/Contents`, and it is never empty. The dict shape is documented in the same
   module: `/Type /Annot /Subtype /Line /IT /LineDimension /Rect /L /C /Contents`.
2. **The popup's has-anything-to-say gate therefore always passes.**
   `canvas/notepopup/model.rs`:
   ```rust
   pub fn has_something_to_read(note: &NoteView) -> bool {
       if matches!(note.subtype.as_str(), "Text" | "FreeText") { return true; }
       note.contents.as_deref().is_some_and(|c| !c.trim().is_empty())
   }
   ```
   A ce dimension is `/Line`, so it falls to the second clause, and its
   `/Contents` is the label. The clause is always true.
3. **`clicked_on` toggles the window on that gate and nothing else.**
   `canvas/notepopup/mod.rs`'s `pub fn clicked_on(ctx, doc, page_index, point, map) -> Option<ObjId>`
   returns `None` only when `!model::has_something_to_read(note)`, and toggles
   immediately after — toggle, not open.
4. **The popup sits above the click ladder and consumes nothing.**
   `canvas/clicking.rs` is the single call site and says so. Selection still
   happens on the same click, so the Properties panel *is* populated; the popup
   merely competes for attention, and nothing has to be re-plumbed. The separate
   `annot_hit` is gated on `caps.author_markup`; `clicked_on` deliberately
   repeats the hit test so Read mode still opens notes.
5. **The popup already knows about ce dimensions.** `canvas/notepopup/mod.rs`
   carries `is_ce_dimension: bool` on the per-frame context — decided once per
   frame in `show`, never per popup — populated from
   `crate::panels::comments::model::ce_dimension_annots(&doc.session)`, which
   walks `dimension_model().dimensions()` and collects each `d.annot`, and it has
   bespoke ce-dimension heading and caption wording. The model's header is
   explicit that Rule 15 applies: a ce dimension is a `/Line` and it is not
   excluded there.
6. The empty-popup case was closed earlier by adding the gate that exists today.
   The ce-dimension case was seen and left in on purpose. O183 overrules that.

**The fix — shell-only, at the gate.** `clicked_on` already takes `doc`, so it
can reach the sidecar set the way `show` does:

- compute `panels::comments::model::ce_dimension_annots(&doc.session)` once per
  *click*, not per frame — `clicked_on` runs on press only, and its own cost note
  already budgets one `/Annots` walk per click;
- if the note is a ce dimension **and** the preference says do not pop, return
  `None`, so the press falls through to selection alone.

`ce_dimension_annots` deserializes the `/PieceInfo` sidecar and is bounded by the
number of ce dimensions; it is called once per frame by `panels::comments::body`,
never per row. One extra deserialization per click is acceptable; per frame would
not be. **There is no cheaper test available:** `NoteView` carries
`id, subtype, contents, author, modified, anchor, popup, authored_open, locked,
in_reply_to` and does **not** expose `/IT`, so the subtype marker cannot be read
off it. The fix stands on the once-per-click sidecar walk.

**The preference, which does not exist today.** `dialogs/settings/measuring.rs`
exposes exactly one function, `pub fn parallel(ui, draft)`. Its header says the
group exists rather than folding into Pages and printing because dimensioning is
a growing subject with an obvious next tenant — which is this preference's
invitation. That header also applies Rule 15 to operator-facing copy, saying
"new dimensions you draw" rather than the bare word; the new setting's wording
must do the same. `app/prefs/` carries no popup key. `dialogs/settings/` has a
`comments` module the setting could live in, but `measuring` is the
symptom-driven home: *my dimension opens a comment box* is a dimensioning
symptom.

**Which way the default goes.** Recommendation: default off — a ce-dimension
click does not pop. The popup's caption for a ce dimension is descriptive text
about a measurement; the operator's editing controls are in Properties, which
the same click already populates. The setting then exists for whoever wants the
old behaviour, which is the direction that does not need a second complaint to
discover.

**Class:** shipped and undiscoverable, plus a small deletion. The
highest-return, lowest-cost item in O183, and the one that makes items 2, 4, 8a
and 9 stop being complaints.

**Driven checks owed:**
`clicking_a_ce_dimension_does_not_open_the_comment_window`,
`clicking_a_cloud_with_a_comment_still_opens_it`,
`clicking_a_ce_dimension_selects_it_and_fills_the_properties_panel`,
`the_comment_window_on_a_ce_dimension_can_be_turned_back_on`.

### 8. "Radius ce dimensions have no controls at all" — two shipped, two absent

**8a. Radius versus diameter is shipped.** `panels/properties/dimension/mod.rs`
draws `display_toggle`, a radio pair raising
`DimensionAction::SetDisplay { dimension, show_diameter }` under the region
constant `REGION_DISPLAY = "properties.dimension.display"`; the measured readout
comes from `model.display(record.id)`, the one producer, and
`NO_SCALE_DISCLOSURE` is carried verbatim. `app/actions/dimensions.rs` routes it
to `session.set_dimension_display(dimension, show_diameter)`, and the engine's
feature table marks switching a placed circular dimension between radius and
diameter as `core [x] cli [x] gui [x]`. **Class: undiscoverable**, the same mask
as items 2, 4 and 9.

**8b. Leader location, leader style and centre mark are absent from the engine.**
In `pdfcer-core/src/dimension/`:

- `fn draw_circular` in `author.rs` authors a fitted circle outline (four kappa
  cubics), **one** radius leader from centre to rim, and **one** arrowhead at the
  rim. There is no parameter for which side the leader leaves on, no enum for
  leader style, and no centre mark drawn at all.
- `fn leader_endpoints` is the only leader geometry in the module;
  `fn draw_angular` is its angular sibling.
- Grepping the whole `dimension/` directory for
  `center_mark|centre_mark|CenterMark|Leader` returns no type, no field, no
  variant. There is nothing to expose.
- `ENGINE_BACKLOG.md` has no row for centre marks or leader style. This is a new
  hand-off, not an existing one.

**Class: absent from the engine.** Hand-off A below, not filed.

**Driven checks owed, for after the engine ships it:**
`a_radius_ce_dimension_offers_a_leader_side`,
`a_radius_ce_dimension_offers_a_centre_mark`.

### 9. "Arrow display is not controllable" — false; it is controllable on both tiers

- **Engine:** `pub enum ArrowForm { Filled, Open, Slash, Dot, None }` in
  `pdfcer-core/src/dimension/style.rs`, mapped in the file header to `swCLOSED`,
  `swOPEN`, `swSLASH`, `swDOT`, `swNO_ARROWHEAD`. Arrow length is a separate
  property: `GroupStyle::arrow_length`, `StyleOverrides::arrow_form` and
  `::arrow_length`.
- **Per ce dimension:** the arrow-form combo and arrow-length row in
  `panels/properties/dimension/overrides.rs`, named through
  `crate::text::dimension_groups::arrow_form_name`.
- **Per group:** the arrow-form row in `panels/dimension_groups/style.rs`, one of
  the seven `GroupStyle` properties the panel draws, with the members-will-move
  count from `fn will_move`.

**Class:** shipped and undiscoverable. Closing item 7 closes this.

**If "arrow display" meant something narrower** — first arrowhead versus second
independently, or arrows-inside versus arrows-outside, both of which SolidWorks
carries — **that is not built at any level.** Nothing in `style.rs` or
`author.rs` distinguishes the two ends. That would be a third engine hand-off,
not a shell row.

**Driven check owed:** `changing_the_arrow_form_on_a_group_redraws_its_members`.

### 10. Tranches, ordered by operator-visible return per unit of work

| Tranche | Contents | Size | Why it is here |
|---|---|---|---|
| T1 — stop the comment box | Item 7: the ce-dimension arm in `clicked_on`'s gate, the preference in `dialogs/settings/measuring.rs`, one key in `app/prefs/` | half a day | The single highest return. It makes items 2, 4, 8a and 9 stop being complaints, because it stops hiding the panel they all live in. Four of nine, closed by one gate. |
| T2 — say where the controls are | Pure discoverability: on selecting a ce dimension make the Properties section visibly the place — expand it by default, and if the panel is closed surface a one-line affordance pointing at it. No new control, no engine call. | half a day | T1 removes the distraction; T2 supplies the direction. They ship together — T1 alone is a subtraction. |
| T3 — fractions on an existing group | Item 1's real defect: a fraction/precision row beside the unit combo, reusing `dialogs/scale.rs`'s `FRACTIONS`, and disclosure of a fraction mode that could not survive a unit change | one day | Genuinely missing, cheap, shell only, and the half of item 1 that is true. |
| T4 — two grips, not one | Item 6's second half, the free part: a distinct grip and cursor for the label anchor versus the dimension line, so `text_along` and `offset` are separately draggable | two to three days | The disclosure Rule 4 wants, built from geometry the shell already holds. No engine dependency. |
| T5 — group-tier tolerance | Item 3, if the recorded decision is overruled: two rows in `panels/dimension_groups/style.rs` through the existing tolerance editor and `SetGroupStyle`, with the members-will-move count saying when it is a no-op and why | one day | Small because both the engine field and the editor exist. Gated on a decision, not on work. |
| T6 — label box from the engine | Item 6's engine half, hand-off B: a read verb returning the label rectangle, so the label can be highlighted rather than merely gripped | a week, mostly waiting on the engine | Only worth doing if T4 proves insufficient in use. It is the sanctioned fix; guessing the box is forbidden. |
| T7 — radius leaders and centre marks | Item 8b, hand-off A: leader side, a leader-style enum, and a centre-mark model in the engine first, then a shell section | a week minimum in the engine, then two to three days in the shell | Genuinely new capability with no engine substrate. The largest real work in O183 and the only class-(b) item. |

Item 5 has no tranche and no estimate: it is blocked on a SolidWorks
measurement, and promising anything before it would be promising an unverified
ISO 286 table into a manufacturing drawing.

**If only one thing ships: T1 and T2 together, in one day.** They close or
unmask five of the nine. T1, T2, T3 and T5 are day-scale; T4 is a few days; T6
and T7 are week-scale, and both are week-scale because of the engine, not the
shell.

### 11. What needs an engine change — drafted, not filed

Written in the shape `ENGINE_BACKLOG.md` uses. Not added to it. Filing is the
operator's call.

**Hand-off A — radius/diameter ce dimensions: leader control and centre marks.**
Three properties `draw_circular` has no substrate for:

1. **Leader location / side.** `draw_circular` draws exactly one leader, centre
   to rim, on a side the caller cannot choose. SolidWorks lets the leader leave
   on any side and lets the text sit inside the circle, outside it, or beyond a
   broken/shouldered leader. Needs a parameter — an angle, or a side/placement
   enum — on the circular kind, persisted in the `/PieceInfo` sidecar, honoured
   by `draw_circular`, and readable back.
2. **Leader style.** There is no enum. SolidWorks carries a set of leader glyphs
   (straight, bent/jogged, underlined/shouldered). Needs a `LeaderForm` enum
   alongside `ArrowForm`, most naturally as a **twelfth cascade property** so it
   inherits factory to group to ce dimension like every other one. That means
   extending `StyleOverrides` (eleven fields, with a `count()`), `GroupStyle`,
   and `StyleProvenance::each()`.
3. **Centre mark.** Nothing in `dimension/` mentions one. Needs a
   `CentreMark { shown, form, size }` model and drawing code in `draw_circular`.
   SolidWorks' forms are cross, cross-with-centrelines, and dashed centrelines;
   size is a document property there, which argues for the group tier.

**Why the shell cannot do it.** All three are drawn geometry inside the baked
`/AP`. The shell never authors a ce dimension's appearance — the engine does, and
`author_dimension` returning the exact baked article is the invariant that keeps
preview and commit identical. A shell-side centre mark would be a fourth thing
on screen that the document does not contain.

**Cascade note.** If leader style becomes the twelfth property, the shell change
is mechanical: `overrides.rs`'s `no_property_of_the_cascade_is_left_without_a_row`
test fails against `DRAWN: [&str; 11]` until the row is added — exactly the right
failure.

**Rule 15 check.** All of this authors ce dimensions. Nothing here reads,
rewrites or re-bakes a pdf dimension from a CAD export.

**Hand-off B — read a ce dimension's label rectangle.** A read verb returning the
rectangle the label occupies, in page space, for a given ce dimension id,
distinct from the annotation `/Rect`. `grab_box` returns the whole `/Rect`, which
covers label and lines; to hover-highlight or hit-test the label separately the
shell needs its box, and the module forbids guessing one. Read-only, no document
mutation, alongside `dimension_rects`, and derived from the same placement the
`/AP` was baked from so preview and reality cannot disagree. Lower priority than
A: T4 delivers most of item 6's value without it.

### 12. What needs the operator's decision

1. **Item 3 — overrule the recorded no-group-tolerance decision, or leave it?**
   The shell argues a group default would read as a no-op press on nearly every
   drawing. SolidWorks carries a document-level default, and SolidWorks is the
   floor. If parity is wanted, T5 is a day. If the real objection is the no-op
   disclosure, the existing members-will-move count already answers it. This is
   the only "missing" item in O183 that is a decision rather than work.
2. **Item 5 — the three SolidWorks measurements**: precision per side or per
   field of a limit; which fit-class table the shop uses; whether block/general
   tolerance is a per-dimension property or a title-block note. No estimate
   exists until these are answered.
3. **Item 7 — which way the default goes.** Recommended: a ce-dimension click
   does not open the comment window, with a setting to restore it. The
   alternative means the next operator meets the same complaint.
4. **Item 9 — did "arrow display" mean more than form and length?** Form and
   length are controllable on both tiers today. Independent first/second
   arrowhead, and arrows-inside versus arrows-outside, are SolidWorks features
   pdfcer does not have at any level. If those are what was meant, they are a
   third engine hand-off.
5. **Items 2, 4, 8a and 9 — is "undiscoverable" an acceptable close?** These are
   shipped. The position taken here is that a discoverability fix is still owed
   (T1 and T2) and that closing them as "already works" without it would be
   filing a record over a real complaint.

### 13. Open questions that outlive this survey

- **Whether the eleven override rows change the drawn article on the operator's
  own file.** The source path is whole from control to action to engine verb, and
  the engine's feature rows say the panel is reachable — but reaching the verb is
  not the operator seeing the number change. The `follows_group()` Factory-versus-
  Group trap is a live candidate for a row that looks inert. Drive it before
  closing items 2 and 4.
- **Whether the Properties panel is open by default**, and what it takes to find
  it. `panels/properties/mod.rs` calls `dimension::section`; whether the panel is
  visible at the moment of the click is a layout and session-state question. This
  is load-bearing for T2.
- **The ui-spec the overrides panel was built against.** `overrides.rs` and
  `panels/dimension_groups/style.rs` both cite it, the latter by clause (§C.11.1),
  but there is no `docs/ui_specs/` directory in this repository and
  `tool-options-dock-and-ce-dimension-properties.md` is not present. The spec is
  cited but absent where the code points, and one of the cited clauses reads as
  direct support for item 3. Find the spec before settling decision 1.
- **Whether any of this behaves differently on a rotated page or inside a
  form-wrapped CAD sheet.** Not in scope for the nine, not established.

### 14. Driven-check names, collected

Lowercase words joined by underscores, no digits.

```
clicking_a_ce_dimension_does_not_open_the_comment_window
clicking_a_cloud_with_a_comment_still_opens_it
clicking_a_ce_dimension_selects_it_and_fills_the_properties_panel
the_comment_window_on_a_ce_dimension_can_be_turned_back_on
an_existing_group_offers_a_fraction_control
changing_a_group_unit_discloses_a_fraction_it_could_not_keep
a_unit_override_on_one_ce_dimension_changes_what_is_drawn
a_symmetric_tolerance_typed_on_one_ce_dimension_is_drawn
a_group_tolerance_default_reports_how_many_members_take_it
changing_the_arrow_form_on_a_group_redraws_its_members
hovering_a_ce_dimension_line_says_it_is_the_line
hovering_a_ce_dimension_label_says_it_is_the_label
the_label_handle_moves_the_label_and_not_the_line
a_radius_ce_dimension_offers_a_leader_side
a_radius_ce_dimension_offers_a_centre_mark
```

---

## The Objects panel re-describes every object on every frame

`summary::describe_object` is called once per visible row with no cache, at two
points inside the row map in `panels/objects/mod.rs`, and it counts every anchor
of every subpath — its own doc in `panels/objects/summary.rs` says so. The cheap
classifier `object_kind`, in the same module, exists precisely because the full
description is not cheap, and it is what the panel's kind census uses instead.

The benchmark CAD sheet carries objects of 4,405, 4,972 and 6,681 anchors, so a
scroll over those rows re-counts them every frame. To re-derive those figures,
open `D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf` and read the anchor
counts off the Objects panel rows.

The design owed is a cache keyed by object identity and invalidated on the edit
epoch, with `object_kind` left as the census path either way. The driven check
owed is that scrolling the Objects panel over that sheet holds the frame budget.

---

## The GUI is one crate, and its five largest modules call each other in both directions

### What is true

`crates/pdfcer-gui` is **419,150 lines / 166,188 code lines across 720 files**,
compiled as a single unit. Nothing is over the 1,500-line file limit — the four
largest files sit at 1,486, 1,481, 1,473 and 1,471 — so `check-file-size.sh`
reports a well-factored crate. R2 bounds the file; nothing bounds the crate.

Two costs, and the second is the one that matters.

**The build cost, measured.** A leaf-file edit and a rebuild, both crates
incremental:

| crate | code lines | rebuild |
|---|---|---|
| `egui-shell` | 22,122 | **1.36s** |
| `pdfcer-gui` | 166,188 | **27.6s** (25.0s lib + 2.6s link) |

7.6× the code, 18× the wait. The relationship is superlinear, which is why
subdividing is worth more than the line counts suggest.

**The encapsulation cost.** A crate is Rust's only enforced boundary. One crate
means one boundary, and inside it everything can reach everything. Counting item
declarations by visibility —

```bash
grep -rhoE '^[[:space:]]*pub (fn|struct|enum|trait|const|static|type|mod|union) ' \
  --include='*.rs' crates/pdfcer-gui/src | wc -l          # and again for pub(crate)
```

— gives **5,924 bare `pub` against 322 `pub(crate)`**, in a crate whose sole
external consumer is its own `main.rs`. There is no layer a compiler will defend, so none formed.

### What that permitted, and it is the reason a split is not one commit

Cross-module reference counts, `crate::<module>` paths with comment lines
stripped — `python tools/module-graph.py`, which also prints the cycle list
below and is the only thing worth quoting, because these numbers move:

| module | code | depends on (top) |
|---|---|---|
| `canvas` | 43,977 | `app`:273, `viewer`:82, `text`:54, `panels`:46 |
| `app` | 32,135 | `text`:346, `canvas`:263, `shell`:96, `panels`:77 |
| `text` | 26,025 | `app`:27, `canvas`:24, `redact`:10 |
| `panels` | 23,581 | `app`:192, `canvas`:97, `text`:87, `pagedrag`:16 |
| `dialogs` | 18,977 | `app`:207, `text`:88, `canvas`:25, `sign`:20 |

Those five are 87% of the crate, and **every pair among them is mutually
recursive**: `app`→`canvas` 263 against `canvas`→`app` 273, `app`→`text` 346
against `text`→`app` 27, and so on. The crate as a whole carries **25 mutually
recursive module pairs, `app` in eleven of them**.

**Cargo forbids a dependency cycle between crates.** So no arrangement of these
five into separate crates compiles until the cycles are cut, and cutting them
is the work — the `git mv` afterwards is an afternoon.

The shape of the cycle is the same everywhere: `app` owns `PdfcerApp`, the
drawing modules take `&mut PdfcerApp` because they need the whole of it, and
`app` calls them to draw. State and painting are in one graph.

### The staged plan

**Stage 0 — done.** `incremental = true` on the release profile: 60s → 28s.

**Stage 1 — done.** `diag`, `units`, `secret` and `acrobat` — fan-out zero,
fan-in 16, 6, 4 and 3 — are a `pdfcer-gui-base` crate. The build win was near
zero — the base crate is 3,060 lines, 1,096 of them code — and was never the
point: a crate boundary is an
arrow the compiler enforces, so nothing down there can quietly call back up into
the application, which is the mechanism that produced the cycles above.

Two things it established that Stages 2 and 3 inherit.

**The call sites need not move.** `pdfcer-gui`'s crate root re-exports the four
names, so ~1,200 `crate::diag::…` spellings compile unchanged and the stage was
a handful of files instead of a several-hundred-file mechanical diff. This is not
a softening of the boundary: **the arrow is in the crate graph, not in how a
caller spells a path.** See the scheduling constraint below for what it buys and
for why it will not stretch as far in Stages 2 and 3.

**What silently breaks is the gates, not the code, and every later stage pays it
again.** Fourteen gate scripts hard-code `crates/pdfcer-gui/src` as their scan
root. One names its table file by path and fails loudly; the rest go **vacuously
green** — the subject leaves the root, the detector finds nothing, and a clean
report is indistinguishable from an empty one. Any stage that moves files
re-points those roots and then **falsifies each one against a planted violation
under the new root**. Re-running a re-pointed gate green proves nothing; it was
green while blind. A gate's own `--self-test` usually cannot supply the evidence
either, because it plants into a temp directory and therefore exercises the
scanner rather than the root list — the one part a move does not touch.

Two generated artefacts move with a workspace member and are named in no code
diff: `THIRD_PARTY_LICENSES.md` (`cargo-about` enumerates members, so a new
crate is a new line even when it adds no dependency) and the fold-in runbook's
copy list and manifest edits in `PROJECT_PLAN.md` §7.3.

**Stage 2 — what Stage 1 made acyclic.** `ocr` (1,144 code) references only
`units`, which is now beneath it, so `ocr` has no upward reference left and
follows. `trust`, `pagedrag` and `pagetree` are one small
edge each from the same position.

`icons` looks like a Stage 2 candidate and is not: it is in a cycle with
`shell` (`icons`→`shell` 1, `shell`→`icons` 4). One reference in one
direction is the whole obstruction, which is the argument for the gate in
Stage 1 — an edge that small is invisible to review and fatal to a split.
`stamps`, `sign`, `redact`, `protect` and `clipboard` reach up into `app` or
`text` and wait for Stage 3.

**Stage 3 — the one that pays, and the one with real risk.** Cut
`app` ↔ {`canvas`, `text`, `panels`, `dialogs`} by extracting the *state* the
painters need from the *wiring* that calls them, into a crate beneath both. The
target shape is: base → state → {canvas, text, panels, dialogs} as siblings →
a thin top crate that wires and owns `main`.

**The direction is load-bearing and easy to get backwards.** A split only pays
when the big pieces sit at the BOTTOM and their dependents are small: editing a
crate recompiles it and everything above it. Put `canvas` beneath a 32,000-line
`app` and a canvas edit still recompiles `app` — 18× on nothing. The top crate
must end up thin, or Stage 3 buys a directory rearrangement and no seconds.

### A scheduling constraint that applies to every stage

A stage that rewrites call sites touches every file that reaches the moved
module — 430 from `app` and 242 from `panels` for `diag` alone, before counting
the other fourteen. That is a wide, shallow, mechanical diff, and a wide
mechanical diff is the worst possible neighbour for feature work in the same
files. Run such a stage in one sitting against a clean tree, or not that day.

A re-export at the crate root avoids this entirely, as Stage 1 did. It is
available to Stages 2 and 3 as well, and the reason it will not stretch as far
there is that those stages split modules that are *called from* the top crate
as well as calling into it — a name re-exported at the root does not resolve a
cycle, it only hides which crate a name came from.

### The open question, which is the operator's

Stage 3 is a refactor of the crate's five largest modules, and the payoff is
developer seconds rather than anything he can see in the program. Whether it
runs before or after the current feature work is his call, and nothing in
Stages 1–2 forecloses either answer.

---

## Dragging a panel between compartments, and tearing one out by dragging it

Capability (d) of `MODES_AND_PANELS.md`'s register, and the half of **O126**
that was answered with commands rather than gestures: *"you understand that
there are options to float, close, and dock those panels … No shortcuts or lazy
half-implementation."* A tab can be activated, closed, floated and docked back;
it cannot be dragged anywhere.

### What is true

Built and driven: multiple columns per side, vertical stacks within a column,
tabs within a stack with a reserved overflow menu, draggable splitters, collapse
to a rail, persistence, named workspaces, and tear-out to a real OS window with
a home address it returns to.

The four gestures:

| | Gesture | What it lands on |
|---|---|---|
| **G1** | Drag a tab along its own strip | a new index in the same stack |
| **G2** | Drag a tab into another stack, between two stacks, past the end of a column, or onto the other side | a different address, possibly one that does not exist yet |
| **G3** | Drag a tab out of the window | a float, homed where the drag began |
| **G4** | Drag a float's header back over the dock | a docked address chosen by the pointer, not by the remembered home |

Three parts carry all four, and none of them is the layout structure:

1. **A queryable geometry.** `plan` is scalar-only — it resolves spans and plans
   tab strips and never sees an `egui::Rect` — so a drop that must ask *which
   compartment is under this point, and where exactly would the panel land*
   cannot be answered from it without parsing `dock::report`'s stringly-named
   `RectReport`. `DockGeometry` is the retained answer, rebuilt each frame and
   therefore one frame old by the time a drop reads it.
2. **A drop grammar** — take a panel from an address; insert it at a target that
   may be a tab index in an existing stack, a new stack splitting a column, a
   new column, or the far side. `DockLayout::move_panel` and `accepts_drop` are
   that grammar as a pure value, and `normalize` prunes what a move empties, so
   no caller has to reason about the column a drag leaves behind.
3. **A pointer the dock is told rather than reads, for G4 alone.** A float is
   drawn in a child viewport, so while its window is carried the pointer that
   matters is over the *application* window and `egui`'s pointer in the dock's
   context is not it. `DockState::set_float_drag` takes the panel, the point in
   the application window's own screen points, and whether this is the release
   frame; `dock::floatdrag` then resolves, draws and settles it with the same
   grammar a tab drag uses. Built, and falsified.

   **The gesture is a drag on the panel's own header strip inside the float
   window, and the point is arithmetic on two `inner_rect`s.** Measured by
   driving the release binary with a temporary probe on both pointer channels,
   pressing inside the float and dragging the real cursor out over the
   application window:

   - **The child viewport keeps reporting the pointer far outside its own
     bounds while the button is held.** The window is 320×480 and the child's
     own context reported `latest` as far as x=592 — 272 points past its right
     edge — with `has_pointer()` and `is_decidedly_dragging()` both true for
     every sample of the drag. The platform captures the pointer to the window
     the press landed in, and `egui` in that viewport sees all of it.
   - **The application's own context does not see the drag at all.** Across the
     same gesture the root reported `down=false` on every sample. So the dock
     cannot sense this pointer for itself, which is exactly why
     `set_float_drag` exists rather than the dock reading `ctx.pointer`.
   - **`ViewportInfo::inner_rect` is the window's content rect in monitor
     space**, in both the root and the child: each matched the platform's own
     client rectangle exactly. `outer_rect` matched the platform's *window*
     rectangle — the same rect plus decoration chrome — so using it on either
     side injects a title bar's worth of constant error.
   - **The conversion is `child_inner.min + local - app_inner.min`, and the
     root calibrated it.** For the last sample of the drag the child reported
     `(558, 335)`; the arithmetic gives `(610, 387)`; and the root's own
     context, for the same physical cursor, reported `latest=610,387`. The two
     sides agree to the point, and the confirming number came from a channel
     the arithmetic does not touch.

   Both rects are `Option` and are `None` wherever a window's position cannot be
   obtained, so a platform that reports neither degrades to the position-blind
   command route. Measured at `ppp = 1.0`; whether the two sides stay in
   agreement when the ui scale is not 1 is the one part of this not yet driven.

   **The window follows the pointer, and what makes that stable is an absolute
   target plus one declined pass.** `floatgrab::carry_to` puts the window where
   the grabbed point lands under the cursor, so once the window is where it
   should be the offset driving it is zero — where a per-frame delta oscillates,
   because the platform re-reports the cursor at a local position exactly as far
   back as the window just moved. That is necessary and not sufficient: the pass
   immediately after a commanded move reads a pointer measured across it and
   reports the residual as the negative of the one just acted on, which would
   send the window back every step for the whole gesture. `floatgrab::Settling`
   is what declines that one pass. `floatgrab`'s header carries both, with the
   `D:/dev/rag/egui/` finding that measured the second.

   The cost is that the window covers the compass it is being aimed with, which
   is an open question in `GUI_ROADMAP.md` rather than a defect in the gesture.

   The OS title bar is not the route. It runs a platform modal move loop in
   which `egui` sees no cursor, and it would need a platform crate to recover
   one; the header strip needs none.

   The float and the dock are drawn from the same `egui::Context` in the same
   frame, so whichever route supplies the point, the drag state itself is one
   field and not an IPC problem.

**R128 is not a prerequisite.** The earlier reading — that cross-dock drag needs
one wide tree spanning left ▸ canvas ▸ right, which puts the canvas in a
resizable pane, which fires the fit-zoom feedback loop — is a verdict about
`egui_tiles`, where drag identity is scoped to a `Tree` and two docks are two
trees. This shell never links it. One `DockState` owns both sides, so a drag
begun on the left is readable on the right, and the canvas does not move.

### What transfers from `QDockWidget`, and what does not

Qt's dock system is the most-copied answer to this problem, and the Qt Advanced
Docking System (ADS) is the community's correction of it. Three of its ideas are
worth taking, and one of its costs disappears here entirely.

**★★★ Take the gap-and-replay preview.** Qt does not draw a guessed rectangle
during a drag. `QDockAreaLayoutInfo::gapIndex` inserts a *placeholder item* into
the real layout, the real layout engine runs, and the rubber band is drawn over
the resulting `gapRect`. The preview is therefore the outcome by construction
and cannot drift from it.

Here that is cheaper still, because the layout is a plain value and the
arithmetic is pure: clone the `DockLayout`, apply the candidate drop, run the
same span resolution the real frame runs, and highlight the rect that comes
back. **Failure mode #2 — *"feedback must encode the outcome, not merely 'valid
target'"* — is then satisfied by construction rather than by care**, and the
whole grammar is testable with no window open, which is what makes the driven
checks a confirmation rather than the only evidence.

**★★ Take ADS's drop compass, not Qt's bare rubber band.** Qt's worst-reported
docking failure is inner-versus-outer ambiguity: the operator cannot tell
whether a release will tab the panel into the hovered group or split the column
around it. ADS answers with an overlay of five explicit zones over the hovered
area — four edges and a centre — plus a second, container-level overlay for the
outer edges. The five zones are the grammar made visible, and hovering one
previews exactly that outcome. Adopt the zones, drawn in this shell's own theme;
they are dock chrome and never reach the page.

**★★ Take ADS's floating drag preview, and do not open a window mid-drag.** ADS
drags a translucent preview and materialises a real window only on release.
That answers G3 and G4 without `ViewportCommand::StartDrag`, which `egui-winit`
gates on `window.has_focus()`, which requires the left button to have gone down
immediately prior, and whose effect is invisible until the next frame. For G4
the same trick runs backwards: the press lands in the float's header, the
preview takes over, and the window is destroyed on a successful dock.

**Take Qt's placeholder for persistence — later, not now.** `QMainWindow`'s
saved state keeps a placeholder item in the tree for a dock widget that is
closed or floating, so restoring puts it back where it was rather than where
there is room. `dock::float`'s `DockHome` is the address form of the same idea,
and its header already argues why an address can go stale and why `dock_back`
rebuilds rather than clamps. A placeholder is strictly better and is a separate
landing; nothing here depends on it.

**Take "fuzz the drop grammar" literally.** It is failure mode #9's design rule,
and Qt earned that rule with crashes in its stacking path. A drop grammar over a
plain value is the easiest thing in this repository to fuzz: generate a layout,
generate a drop, apply, normalize, assert the invariants — every panel appears
exactly once, no empty stack or column survives, shares sum to one, every
`active` indexes a tab that exists.

**What does not transfer is most of `QDockWidget`.** Reparenting widgets between
a dock area and a top-level window, native window handles, event filters, mouse
grabs, and a `QRubberBand` that is itself a window — an immediate-mode shell has
none of it. Moving a panel between compartments here is a mutation of a
serializable value that the next frame draws. That is why the steps below are
counted in days while Qt's implementation is counted in tens of thousands of
lines.

**One Qt default to reject.** Qt tears a dock widget out on any drag of its
title bar, which is failure mode #1 — an OS title bar and an application handle
stacked, where grabbing the wrong one silently does nothing. This shell draws
its own chrome and has no second title bar, so the tab itself is the one
unambiguous grab affordance: a drag that leaves the dock tears out, and there is
no separate handle to miss.

### The order to build it in

Each step ships on its own and leaves the program usable.

| | Step | | Why here |
|---|---|---|---|
| **0** | Retain the dock's geometry — address to rect for every side, column, stack, tab strip and tab, built during the draw it already performs and kept for the next frame, with a position-to-address query at each level. | **Built** | Nothing after it can be written without it, and it is also the honest answer to `dock::report`'s stringly names. |
| **1** | G1: reorder within a strip, with the insertion caret the page rail and the document strip already use, in its full and dimmed pair. | **Built** | The smallest useful gesture, and the recipe is proven twice in this repository. |
| **2** | The drop grammar and its fuzz, headless: take, insert, split, new column, normalize, invariants. | **Built** | A pure value, testable with no window. It comes before the overlay so the overlay has something true to preview. |
| **3** | G2: the pointer-to-`DropTarget` resolution over the retained geometry, then the compass overlay drawing it and the cross-compartment drop previewed by replay. | **Built** | The capability the register calls (d). Step 2 is what a replay preview applies to its clone, so this step has something true to show. |
| **4** | G3: a drag that leaves the dock tears out, homed at its origin. | **Built** | Sits on the float model already built and changes nothing underneath. |
| **5** | G4: drag a float back over the dock and drop it where the pointer says. | **Shell half built** — `dock::floatdrag` offers, previews and settles. Nothing calls `set_float_drag`, so the gesture is not reachable from the running program and the command route is still the only way home. What is owed is the caller: sense a drag on the panel's header strip inside the float body, convert by item 3's measured arithmetic, and feed `set_float_drag`. | Last because it settles with the grammar steps 2 and 3 built, and because it is the one gesture whose pointer the dock cannot sense for itself — so it is the one that needs an extension point. |

**What step 3 is made of.** Three parts, all in.

`dock::compass` answers *which `DropTarget` is under this point* over the
retained geometry, and returns the quadrilateral each of its five zones is hit
as — so the overlay that fills those quads has drawn the hit test rather than a
second description of it.

`dock::preview` answers *what would that do* — the gapRect trick made literal.
It clones the layout, applies the candidate with the same `move_panel` the
release calls, finds the panel in the result, and re-walks the rects. The walk
is the same function `Dock::show` lays its columns and stacks out with, so a
preview cannot describe a geometry the next frame will not produce. The side's
own rect is read from the retained geometry rather than recomputed, because
`egui`'s panel reservation less the banner and the rail settles it and no drop
this grammar can express moves it.

`dock::overlay` paints it: the five quads over the hovered compartment's body at
a resting wash, the armed one stronger, and an outline around the rect the replay
returns — dimmed, not withheld, at a release that would permute nothing, for the
reason the caret is. It runs once from `Dock::show` after both sides have drawn,
because the resolution needs the whole geometry and a `Ctx::geometry` filled in
draw order cannot answer about a compartment drawn later; it paints to a
foreground layer rather than into the hosting `Ui`, which by then is under every
panel body. It stands down whenever `drag::preview` published, which is the
ownership split between the two affordances.

**The one product defect this step exposed.** Every tab strip sits at the same y,
so a drag carried *sideways* onto a different strip stayed inside the reorder's y
band: the origin strip went on owning the gesture, drew a caret hundreds of
points from the pointer, and the overlay stood down for it, so the drop the
operator was reaching for was never offered. `drag::preview` now bounds x by the
strip exactly, with no slack — x is the axis the boundary is resolved from, and
`gap_in` answers for every x on the screen. The guard cannot be phrased as *"am I
over a different compartment"* for the draw-order reason above; both halves are
in `D:/dev/rag/egui/`.

### Traps this will hit, recorded before it is built

- **`drag_started()` fires after the threshold**, by which time the pointer has
  moved. Anything decided at gesture start — which tab was grabbed, which
  address it came from — reads `press_origin()`, never `interact_pointer_pos()`.
- **Drag predicates are button-agnostic.** A right-drag on a tab must not
  reorder it; every call site is `*_by(PointerButton::Primary)`.
- **A tab is a `Button` today and senses clicks only.** Sensing must become
  `click_and_drag`; `Sense::drag()` alone swallows the click that activates the
  tab, which the form tab-order list learned expensively.
- **The caret is resolved during layout and painted after the tabs.** A gap has
  no position until the tabs are placed. egui's own `dnd_drag_source` is the
  wrong tool twice over: it tints whole frames, and it re-runs the widget body
  into a tooltip layer under the cursor, which is the wrong affordance for a
  reorder and a second sense on an id already being interacted with.
- **A harness's coordinates go stale the moment a drop changes a dock width**,
  and a stale coordinate is indistinguishable from a broken conversion. Every
  driven step re-reads the rect it is about to click.
- **A child viewport's rects are relative to its own origin**, so anything G4
  publishes must be tagged and converted, and `show_viewport_immediate` may run
  its callback twice in one frame.
- **A drop indicator is a gesture-only overlay**, so the `ui-rect` trace records
  its appearance as a change and cannot report its disappearance. Assert on the
  appearance, and on the layout the release produced.

## Where the seam is in each file now crowding the size limit

R2's remedy is *find the seam*, never *raise the limit*, and the seam is a
property of the file rather than of the number. The files at risk are the ones
`find crates tools -name '*.rs' | xargs wc -l | sort -rn | head` puts within
about fifty lines of the 1,500 cap — the next ordinary edit to any of them turns
a green gate red mid-task. This section is the seam for each, so that work is a
patch rather than a re-derivation.

⚠ **Check a file's line endings before anchoring a patch script into it.** A
Python patch script that calls `write_text(s, encoding="utf-8")` on Windows
writes CRLF, because text mode defaults to `newline=None` and translates to
`os.linesep`; `read_text` hides it by reading universal newlines, so a round
trip that looks lossless converts the file. Git normalises at `add`, so the
commit is clean and the damage is invisible — until the *next* script whose
anchor ends in a newline matches zero times against a perfectly correct anchor.
Pass `newline="\n"` or use `write_bytes`.

⚠ **`i/lf w/crlf` is not by itself a defect here — ask the attribute first.**
`core.autocrlf` is `true` on this machine and `.gitattributes` opens with
`* text=auto`, so a file that carries no `eol` override is *supposed* to be CRLF
in the working copy. Only the pinned families — `.rs`, `.md`, `.sh`, `.py`,
`.toml`, `.html`, `.svg` and the rest of that list — are wrong when the working
copy is CRLF. `git check-attr eol -- <path>` answers which case a file is in,
and `git ls-files --eol <path>` then says whether it is in the right state.

⚠ **`git status` is not a content oracle; `git diff` is.** Rewriting a file with
byte-identical content still leaves it listed as modified, because the index
caches size and mtime and a rewrite invalidates both — and
`git update-index --refresh` does not settle it. The authoritative comparisons
are `git diff --name-only`, or `git hash-object --path <f> <f>` against
`git ls-files -s <f>`; identical hashes mean identical content whatever status
prints. Staging the file clears the noise, and stages nothing.

Line numbers below are an aid to the eye only. The anchor in every case is the
named item or the quoted banner, because a citation into a moving file goes
wrong silently.

**`panels/mod.rs` → `panels/state.rs`.** The catalog and the dispatch say which
panels exist and how the dock calls one; `PanelsState` is the per-frame scratch
the bodies keep between calls, and its header is entirely an argument about
cache lifetime. `PanelsState`, `ObjectTreeUi` and both impls move. `fn sync` is
private and `Panel::show` calls it, so it widens to `pub(super)`; `pub use
state::{ObjectTreeUi, PanelsState};` is mandatory for the call sites naming
`crate::panels::PanelsState`. `panels/tests.rs` uses `use super::*` and needs
nothing. ⚠ `FEATURES.md` cites this file **by line number** for `Panel::ALL`,
and `RESUME.md` greps it for `pub const ALL` — a cut below `Panel::ALL` leaves
both correct, a cut above it breaks the first silently. A second cut to
`panels/layout.rs` for `scroll_style`, `content_width`, `ELLIPSIS`,
`elide_to_width` and `text_width` is re-export-transparent, because sibling
modules already name them by full path.

**`app/dispatch.rs` → `app/dispatch/workspace.rs`.** Nine arms —
`file.properties`, `markup.comments`, `view.read_mode`, `view.fullscreen`,
`view.reset_layout`, the three `mode.*`, and the two panel guards — have the
shell's arrangement as their operand and none of them pushes an `Action`. That
sentence is `dispatch/panels.rs`'s header, verbatim. `tools.render_diagnostics`
stays: it is a dialog verb in the shape of `file.print`. ⚠ This is the
fail-closed one. The unreachable-command checker under `shell/commands/reach/`
requires a **free** `pub(super) fn dispatch` rather than an inherent method, a
`match` scrutinising a binding literally named `id`, and a guard named `handles`
or `claims` — both of which are already in `EVALUATED_GUARDS`, so registration
is one `if` in `reach/guards.rs`. `handles` must be `pub(crate)` because
`guards.rs` sits outside `app`. Add a fourth `include_str!` in `reach/mod.rs` so
the moved literals stay covered by `no_literal_arm_names_an_unregistered_command`.
Two orderings must survive the lift: `file.properties` and `markup.comments`
stay **above** `Panel::from_command_id` or they silently become toggles, and the
`panels::claims` guard stays **below** it. The reset-layout prose block sits
about eighty lines above the arm it explains; reunite them rather than lifting a
span. A second seam is `dispatch/document.rs` — bring a document in, put one
away.

**`app/actions/action.rs` — there is no seam, only a sub-enum.** The file is
three `use` lines and one enum of about sixty-five variants, and its own header
already states the growth path. The move that is available is a **correction**:
`actions/text.rs`'s header claims *"the caret commit, the free-text commit, the
restyle and the reflow"* while `TextAction` holds only three of those, so
`CommitTextEdit`, `CommitAddText` and `TextStyle` fold into it and the header
stops being a false statement. It costs about forty call-site rewrites.
`BeginTextAnnot` and `CommitTextAnnot` are an equally defensible second family.

**`tools/ui-verify/src/checks/mod.rs` — the deferral is spent; the durable
remedy is the subdirectory fold.** The file is one `pub mod` declaration per
check carrying its prose. `roster.rs`'s header already took the tempting seam
and forecloses it: moving a `pub mod` there renames the check and breaks every
reference to it. `CheckContext`, its impl and `trait Check` are now
`checks/harness.rs`, re-exported so nothing was renamed — that bought about 120
lines and was a **deferral, not a fix**; it cannot be taken twice.

⚠ `use crate::report::CheckReport;` stayed behind in `mod.rs` when the trait
moved, and must stay there. Most check modules spell the report type
`crate::checks::CheckReport`, which resolves because a private `use` is in scope
for the whole subtree beneath it; a name imported into `harness` is in scope for
`harness` alone.

The durable remedy is folding check families into subdirectories, for which
`checks/signing/` and `checks/raster_wall/` are precedent, at the cost of
rewriting the references to `crate::checks::driving::…`. ⚠ The other tempting
move — pushing per-check prose down into each check's own header — is
prose-shaving, which is the thing `check-file-size.sh` exists to refuse. That
refusal is about *moving* prose to buy lines; deleting a sentence that has
become false, or that restates `checks/conventions.rs`, is R5 and is owed anyway.

⇒ **The last two are the same shape twice.** A flat vocabulary — a module
declaration per check, sixty-five enum variants — has no prose seam, every
candidate cut renames the moved items' paths, and the only honest remedies are
the sub-enum and the subdirectory. The worst outcome for either is a cut chosen
to hit a line count.

---

## O219 / O221 — the view goes blank, and the ceiling moves with how many documents are open

### The two rows, in his words

> *"Also sometimes before this happens the view goes blank and when I zoom in a
> little more I get the error."*

> *"Zooming capability seems to be affected by the number for pdfs I have open,
> even if they are open in a new window. The more I have open, the less zoom I
> get, I think, but I could be wrong and it could be a bit random."*

★★★ **The last clause is the most informative sentence in the report and it
must not be discounted as hedging.** A fixed budget is never random. Live
graphics-memory pressure — which moves with what else is resident, what other
processes hold, and how the driver has fragmented its heap — is *exactly* what
"a bit random" feels like from the chair. The randomness is the evidence, and a
design that produces a deterministic ceiling is a design that has explained his
first sentence and contradicted his second.

### What is true — measured, not assumed

| fact | where it was measured |
|---|---|
| The whole-page tier has an **edge** budget and **no pixel** budget | `render::strategy::whole_page_raster_fits` checks `longest * raster_scale <= MAX_PIXMAP_EDGE - 1` and nothing else |
| So it admits `16383² = 268 Mpx` ≈ **1.07 GB** of RGBA | arithmetic on the line above |
| A texture too large on one axis **panics in release** | `egui_glow`'s `Painter::upload_texture_srgb` carries a bare `assert!`; `egui::Context::load_texture` guards size with `debug_assert!` only, so a release build hands it straight through |
| A texture of legal size that cannot be allocated is **silent** | `check_for_gl_error!` wraps its body in `if cfg!(debug_assertions)`, so `GL_OUT_OF_MEMORY` is never read in release. GL does not trap; the texture object stays bound with no storage and draws a blank rectangle at full frame rate, logging nothing |
| Every texture swap **transiently double-allocates** | `paint_and_update_textures` runs `textures_delta.set` → `paint_primitives` → `textures_delta.free` within one frame; only a frame boundary separates the peak from the release |
| The GL edge ceiling **is** reachable safely | `ctx.input(\|i\| i.max_texture_side)` — eframe's glow integration calls `Painter::max_texture_side()` and feeds it into `RawInput`, so no `unsafe` and no `glow` call is needed to read it. Nothing in this repo reads it today |
| A parked document **keeps its rasters**, by an argued decision | `app::documents::activate_slot` |
| The region tier is **scale-invariant** | `strategy::OVERSCAN` is a multiple of the viewport, not of the page — so capping the whole-page tier costs free panning and **loses no zoom** |
| The operator's own adapter reports **13.9 GiB** of dedicated memory | `HardwareInformation.qwMemorySize`, Intel Arc Pro B50. ⚠ WMI's `Win32_VideoController.AdapterRAM` is a `uint32` and saturates — it reports 2 GiB − 4 KiB on this machine, which is an artifact and not a measurement |

### ★★★ Why the obvious fix is the wrong one

The obvious fix is a byte budget: give `whole_page_raster_fits` a `max_bytes`
alongside its edge check and refuse a raster above it. It is half right — the
missing pixel budget **is** a real defect, and an edge limit genuinely is not a
memory budget — but as *the* answer to these two rows it fails three ways:

1. **No constant is right on two machines.** 256 MiB is punitive on a 13.9 GiB
   card and still optimistic on a 2 GiB laptop. Whatever number is chosen is
   wrong somewhere, and being wrong downward silently steals the zoom he
   already has.
2. **It cannot model accumulation, which is the whole of O221.** A per-raster
   cap does not notice that four documents are open. To model that it would
   have to become a pool, and a pool has to know what every resident texture
   costs — at which point it is tracking an allocator it does not own, against
   a total it cannot read.
3. **It contradicts his report.** A constant produces a ceiling that is the
   same every time. He says it is not.

### The design — learn the ceiling from the failure

This shell **already has** the machinery, and it was built for the same shape
of problem. `render::ceiling::RasterCeiling` exists because the `tiny-skia`
wall is content-dependent and unpredictable, so the only honest source of the
number is the refusal itself: one render fails, the shell writes down the
scale, and that page never goes that far again. Its header states the property
that makes one observation enough — the failures are **monotonic in the
scale**.

Graphics-memory exhaustion has that same property within a moment, and
`RasterCeiling` already backs off by `BACKOFF` and already discloses on the
bottom bar through `app::status::rasterstop`. So the repair is not a new
ceiling; it is **making this failure visible to the ceiling that already
exists.**

★ That also explains, rather than contradicts, the randomness: a learned
ceiling moves when the pressure moves, which is what he is describing.

#### 1. Make the failure observable — a fourth `native-*` crate

`pdfcer-gui` carries `#![forbid(unsafe_code)]`, `forbid` cannot be relaxed from
the inside, and every `glow` entry point is `unsafe`. That is the exact
argument `crates/native-clipboard` and `crates/native-window` were created
under, and it is written into the workspace manifest. This is the third
instance of the same pattern, not a new one.

The crate owns one call behind a safe function: drain `glGetError` in a loop
until it returns `GL_NO_ERROR` and report whether `GL_OUT_OF_MEMORY` was among
what it drained. `eframe::App::ui` already receives `frame: &mut eframe::Frame`
and currently discards it as `_frame`; `Frame::gl()` hands over the context.

⚠ **`glGetError` returns ONE error and clears it.** A single call is a bug:
the flag set may hold several, and reading one hides the rest. Loop.

⚠ **The flag is global to the context.** Reading it clears it for everyone.
In release that is safe — `check_for_gl_error!` is compiled out, so nothing
else is looking — but this is precisely the kind of "successful workaround"
that is a finding about the boundary and gets reported as one.

#### 2. ★★ Attribute the failure before acting on it

This is the part that will be got wrong. An `OUT_OF_MEMORY` drained at the top
of frame *N* was raised somewhere in frame *N−1*, and **any** upload could have
raised it — the font atlas, an icon sheet, a thumbnail. Clamping the page's
zoom for a failed glyph upload would take his zoom away for an unrelated
reason, and it would look exactly like the defect being fixed.

⇒ The ceiling may only be lowered when the previous frame **actually uploaded
a whole-page raster**, and it is lowered for *that* page at *that* scale — both
recorded when the upload was ordered, not reconstructed afterwards. If the
previous frame uploaded no page texture, the error is drained, traced, and
**not** attributed.

#### 3. The edge guard, which needs no policy number at all

Independently of the budget question, read `ctx.input(|i| i.max_texture_side)`
and never order a whole-page raster whose longer edge exceeds it. This is
device-sourced, so it invents nothing, and it closes a genuine unconditional
crash path: on any GPU reporting less than `MAX_PIXMAP_EDGE`, today's code
reaches `upload_texture_srgb`'s bare `assert!` and the process dies.

⚠ **Read it every frame; never cache it.** egui's value is `2048` until the
backend reports the real one, so a value latched too early would clamp the
program to a ceiling no hardware imposed. Read per frame and a spurious early
`2048` corrects itself on the next one.

⚠ On the operator's own machine this guard is expected to be **inert** — an
Arc Pro B50 reports a texture ceiling at or above `MAX_PIXMAP_EDGE`, so the
edge check can never bind there. It is correctness for other machines, and it
must not be reported as the fix for his symptom.

#### 4. What O221 needs beyond disclosure

**A reclaim, and disclosure is not a substitute for it.** `activate_slot`
deliberately keeps a parked document's rasters, which is right for switching
back quickly and wrong when the pressure it creates costs the active document
its zoom. The learned ceiling above will *respond* to that pressure, which
makes the symptom explicable; it does not give the memory back.

⇒ Under pressure — meaning after an attributed `OUT_OF_MEMORY` — parked
documents' whole-page rasters are dropped, and the drop is disclosed
off-canvas. Both halves are required. A row that is explained but not repaired
is not closed.

### What is built, and what building it corrected in this design

Parts 1 and 2 exist as **observability only**: `crates/native-gl` drains the
error flag behind a safe signature, and `render::pressure` decides what a
reading can be pinned on. The drain is the first statement of
`impl eframe::App for PdfcerApp`'s `ui`, because `eframe` runs the whole of
`ui` and only *then* paints — so an upload ordered in a frame is performed at
the end of it and its error is first readable at the top of the next. It emits
`gl-pressure` when the flag is dirty and `gl-max-texture-side` on change.
**Nothing lowers a ceiling and nothing drops a raster.** Parts 3 and 4 are not
built.

Three things the design above had wrong, all found by building it:

- **"Any upload could have raised it" understated the problem.** The canvas
  raster and the Pages panel's thumbnails share `render::raster::texture_from_pixels`
  and the same `RenderKey` type, so a thumbnail is a whole-page raster by every
  property the pixels expose — a real page index, a real scale, no region.
  Attribution keyed on the pixels alone would blame a 40 kB thumbnail, at
  whatever page scrolled into the panel, for a failure raised elsewhere. The
  surface is therefore passed in at the call site, which makes a third caller a
  compile error rather than a silent miscount.
- **★★ Elimination needs a COMPLETE census, and an incomplete one is worse
  than no instrument.** An uncounted upload does not cost one observation — it
  makes a two-upload frame look like a one-upload frame, so the rule stops
  refusing to guess and blames the upload it can still see. There are three
  `ctx.load_texture` sites, not one: the page raster, the print preview and the
  icon sheet. All record. `tools/gates/check-texture-census.py` is what keeps
  that true, because no unit test can see a call site and a hand-written list
  of upload sites is exactly the thing that rots.
- **★★ Part 4's reclaim cannot reach the case the row actually describes.**
  The row says *"even if they are open in a new window."* There is no command
  that opens a document in a second OS window: documents are slots in one
  process, `show_viewport_immediate` is used only by dialogs, and nothing
  launches a second copy of the program — the two `current_exe` readings are
  the path written into the file association, and the directory OCR searches
  for its models. So a second window is a second **process**,
  and the two share one GPU's memory. A reclaim inside one process cannot see
  the other's textures, let alone free them.

  ⇒ The reclaim keeps its place, but its claim shrinks to the single-process
  case. What covers the multi-process case is the clamp — degrade to the last
  scale that drew rather than refuse, whoever consumed the memory — which
  makes the O218 work the primary repair for this symptom and the reclaim a
  narrowing of it. It also means the disclosure must not say *"close some
  documents"* when the memory went to a second instance the program cannot
  see; it can only report what it asked for and what the device refused.

### What must be measured before any of this is defended

Neither row may be reported fixed on reasoning. The measurement is a zoom
**series**, not two endpoints, driven through the real binary:

- walk the zoom ladder on a fixed page and record the scale at which the canvas
  first goes blank — with **one** document open, then three, then six;
- the ceiling moving with the document count is O221's mechanism confirmed;
  the ceiling *not* moving refutes it, and the design above is then answering a
  question he did not ask.

⚠ A blank canvas has exactly one oracle — a captured screenshot. A green trace
line saying a raster was ordered is not evidence that anything was drawn.

### How the ladder is driven — the design, and the seam it needs

**Rung 1 exists.** `the_graphics_pressure_instrument_reports_a_real_device_limit`
launches a release build off the desktop, sends nothing, and reads the two
`render::pressure` lines. It asserts the standing `gl-max-texture-side` against
a plausibility floor and **reports** `gl-pressure` without asserting it. That
is the control: one document, fit zoom, no gestures. It costs the operator
nothing to run because it never touches his cursor.

Rungs 3 and 6 need something rung 1 does not: a way to **climb zoom** in a
process that cannot be driven by OS input, because its window is off the
desktop. `app::keyboard::scripted` is that seam and it carries its own
argument; what belongs here is why the ladder may use it and what it leaves
undone.

#### Why a keystroke seam rather than a registered command

**No zoom-step command is registered**, so `PDFCER_DIAG_INVOKE` — which rings
a command id — cannot climb zoom at all. The registered zoom verbs are
`view.zoom_actual`, `view.zoom_fit_height`, `view.zoom_fit_page`,
`view.zoom_fit_width`, `view.zoom_region` and `view.zoom_selection`, every one
of them **absolute**. `RIBBON_IA.md` §6 assigns zoom-in, zoom-out, next-page
and previous-page to the status bar and the keyboard instead, and
`shell::commands::reach::register` records those as their live routes.

⚠ A grep for `"view.zoom_in"` **returns hits**, and every one of them is prose
or a negative test case. Reading only the hit count says the command exists.

Registering one to suit a harness is a **ribbon decision, not a dispatch one**
— it changes what the operator can reach and where the verb lives, which
`RIBBON_IA.md` settles and this role proposes rather than improvises;
`shell::commands::reach::UNREACHED_ARMS` carries what it would take. ★ The
distinction that makes the seam legitimate where registering is not: a seam
substitutes for the *gesture the harness cannot make*, and registering a
command changes *what the product offers*. Only one of those is a claim about
the program.

#### ⚠ The shell gap the seam had to work around, and it is ours not the engine's

The seam paces its chords on `ctx.cumulative_pass_nr()`, twenty frames apart,
because **nothing in the shell reports that the canvas has settled**.
`ViewState` carries `zoom` and `fit` but no "the fit has been solved" flag, and
`crate::diag` counts UI rects per frame without exposing a reader for the
census. A chord delivered before the first layout is overwritten by that
frame's fit solve and is lost in silence.

⇒ The frame count is a **proxy for a readiness signal that does not exist**.
Either a `fit=solved` transition on `ViewState` or a `diag` predicate over the
per-frame rect census would let the seam wait for the thing it means, and both
are shell work rather than an engine request.

#### ⚠ What a rung may not conclude from the seam's own trace

`diag-keys index=k chord=… spelled=yes` means the key was **pushed**, and
nothing more — the seam runs before `app::keyboard::collect` and cannot know
what became of the press. A rung that counts rungs is asserting something both
outcomes satisfy. Read the effect from the application's own
`status … zoom=` / `render-spawn … scale=`, keyed on the rung's `index=`.

`the_scripted_keystroke_seam_climbs_the_zoom_ladder` is that reading, and it
is the seam's standing assertion: eight chords, each rung's effect taken from
the `status` lines bounded by its own `diag-keys index=k` and the next one's.
Its first arm is the pacing one — **no `status` line at all before rung 0**
means the application had not solved its layout when the first chord landed,
which is the defect's real signature and not a lost step in the middle. A rung
must also leave `fit=Page`, because a discrete zoom verb sets an explicit zoom.
A rung built here inherits that reader rather than writing a second one.

#### ★★ What this seam does NOT reach, and it is the row next door

**`Ctrl` + wheel is a different route with a different type.** `canvas::zoom`
reads `ctx.input(|i| i.zoom_delta())` — a **continuous** factor — where
`app::keyboard` raises a **discrete** `Action::ZoomIn`. They converge only at
the action funnel, and the funnel's own doc names five surfaces that can raise
a zoom. ⇒ a synthetic-keystroke seam climbs the discrete ladder and exercises
**one** of them.

That matters here rather than pedantically, because O220 is *"when the error
occurs it prevents me from pressing ctrl and using the zoom wheel to zoom back
out — I have to click the zoom out control on the bottom bar."* His sentence
names both routes and says one is trapped while the other still works. A rung
built on the keystroke seam is therefore driving the route he reports as
**working**, and it can produce a full green ladder while the reported defect
is untouched.

⇒ The ladder must not be read as progress on O220. That row is already driven
on the wheel, with real input, by
`the_raster_wall_stops_the_zoom_instead_of_painting_an_error`'s third part;
what it still lacks is its **precondition** — a canvas standing on a
`render-failed` refusal, on demand. The ladder's contribution to O220 is
exactly that and nothing else: if the document count moves the ceiling, it is
the lever that produces the state, and the wheel check that already exists
then runs from it.

#### What each background document has to have done before the rung means anything

- `place: false` and an off-desktop viewport for every background process.
  `Driver::confirm_uncovered` refuses a click a window owns, and every placed
  window lands at the same `(780, 40)` — so N visible sessions would stack.
- ⚠ **A document that has not rasterized holds no textures.** A rung that opens
  six documents and never lets them draw is measuring one document with five
  idle processes attached, and it will show no ceiling movement for a reason
  that has nothing to do with O221. Each background process must be settled at
  a zoom that demonstrably produced a whole-page raster — asserted from its own
  trace, not assumed from a frame count — before the subject starts climbing.
- The rungs must run **in one sweep on one machine**. What turns `gl-pressure`
  from a reading into evidence is the *difference between rungs*, and free
  graphics memory is not a constant across two runs an hour apart.

⚠ **Walk the zoom, never sample two ends of it.** The count axis is the series
1 / 3 / 6; within each rung the zoom is itself a series, because the scale at
which the canvas first goes blank is a transition and a transition hides
between two samples.

⇒ Until the series exists, **O219 and O221 stay FILED**, nothing measured here
is a cause, and the per-document cache reading remains a hypothesis with a
citation rather than an explanation to give the operator.

### Needs the operator's ruling

- **Is a single blank frame acceptable as the price of learning?** The design
  clamps *after* one failure, exactly as `RasterCeiling` does today. The
  alternative is a conservative pre-emptive budget that costs zoom he currently
  has on every document, whether or not it would ever have failed.
- **Should a parked document's rasters be dropped under pressure**, given the
  cost is a re-render when he switches back to it.
