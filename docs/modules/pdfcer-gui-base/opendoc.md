# `opendoc` — what is open

One thing lives here: [`Status`] and [`OpenDoc`] — the shape of "what, if
anything, is open", and everything one open document owns.

## What is NOT here, and where it went

The **raster bookkeeping** — the per-frame decision about whether the
cached page texture is still a picture of what the operator is looking at,
and if not whether to re-rasterize now or wait for a zoom gesture to settle
— was the second half of this file until Phase 4 and now lives in
[`crate::app::settle`]. Phase 4 made it considerably larger (one texture
became a texture plus a bounded strip cache, and one staleness question
became two), and the seam is the one this header had already named:
everything here answers *"what is open, and what is the operator looking
at?"*, and everything there answers *"what does the picture need to be, and
what should be done about it this frame?"*. Only the second belongs in
`render/`, beside the worker it schedules.

The **fields** it reads are still declared below, because that is what
bounds their lifetime and is the whole point of their living on the
document; only the methods moved.

The two **derived caches** an open document owns — the page decomposition
and the font inventory — live in [`crate::app::cache`], along with the
four methods that read them. Their fields are still declared on [`OpenDoc`]
below, because that is what bounds their lifetime and is the whole point of
their move onto the document; only the types, the accessors and their
argument moved out.

The seam is a real one rather than a cut made to satisfy rule R2's
1,500-line gate: everything left in this file answers *"what is open, and
what is the operator looking at?"*, while everything in `cache.rs` answers
*"what expensive derived value do several surfaces need, and how is it
computed once?"* — a different question, with its own shared hazard
(staleness against [`OpenDoc::edit_epoch`]), its own shared device (a `Cell`
key beside a `RefCell` payload) and its own documented exemption from the
`&mut`-only mutation rule. `cache.rs`'s header carries that argument in
full. Splitting anywhere else would have left half an explanation on each
side of the cut.

## Three ways to fail, three ways to say so

[`Status`] distinguishes what most viewers conflate, and the
distinction is carried across from the old shell because it is one of
the things pdfcer does that its competitors do not:

- [`Status::Failed`] — the *file* is wrong: damaged, truncated, not a
  PDF. "Something is wrong with your document."
- [`Status::Unsupported`] — the file is fine and **pdfcer** is not
  finished. `pdfcer-core` detects such a file and refuses it cleanly
  rather than misparsing it into plausible-looking garbage. Presenting
  that as "failed to open" would tell the operator a lie about their own
  file.
- [`Status::NeedsPassword`] — a third thing again: pdfcer *can* decrypt
  this document and has not been told how.

The branch between them is made on **structured error data** from
`pdfcer-core`, never by matching on a message string. That is exactly what
"core errors are stable, structured diagnostics" is *for*, and it is what
makes the distinction reliable rather than a heuristic that decays as error
prose is edited.

The branch itself now lives in [`crate::app::lifecycle`], with
`is_unsupported_structure` and the two methods that move `Status` between
these variants. This file kept the **shape** of the answer (the enum, and
why it has these variants); that one has *when each one is produced*. See
that module's header for the seam.

## Rendering happens on state change, never per frame

egui redraws continuously; rasterizing a PDF page at 60 Hz would be
absurd. The canvas holds one cached [`PageTexture`] for the **current
page**, plus — under a continuous page-display mode only — a bounded cache
of the *other* visible pages ([`OpenDoc::strip_rasters`]). Each texture
carries the [`RenderKey`] it was rendered from — page, raster scale,
annotation visibility, layer-override generation — and staleness is that
key compared against the one the view wants, with deliberately no second
field list to keep in step with it (see [`RenderKey`]'s own docs; a key
compared on one side and not the other is a control that ticks and redraws
nothing).

**The comparison, the zoom debounce and the strip's scheduling all live in
[`crate::app::settle`]**, which carries the full argument for both
staleness policies. What lives here is the state they read.

[`OpenDoc`] also carries the page decomposition and the font inventory,
moved off `crate::panels::PanelsState` at S4 so the document's own
lifetime bounds them and no identity key is needed — see
[`OpenDoc::page_objects`] — and, from the same stage, the **canvas
selection**, moved off `egui::Memory` for the same reason. See
[`OpenDoc::selection`].

## Item notes

### `fn assemble`

[`Self::new`]'s own argument — *"a `reset()` method would be a second,
weaker way to achieve the same thing"* — applies with equal force to a
second struct literal. Two constructors that each listed thirty fields
would drift the moment one of them gained a field, and the drift would
be invisible: the compiler is satisfied by both.

### `fn policy_token`

# Why this exists rather than `{:?}` on the engine's enum

Because a `Debug` rendering belongs to `pdfcer-core`, and a driven check
keyed on one is asserting a formatting detail of somebody else's crate. This
project has already shipped a machine-read field that inverted its meaning
when an upstream `Debug` impl changed shape, and the check kept quoting the
truth while reporting the opposite of it.

So: two tokens, owned here, changed only deliberately. They are **not**
operator copy and never reach a surface — `crate::text::anomalies` owns the
sentences a person reads.

⚠ The `_` arm is not laziness. [`pdfcer_core::parser::DuplicateKeyPolicy`]
is `#[non_exhaustive]`, and `Refuse` — which this shell must never send to a
loader, see `crate::panels::docprops` — is a third variant today.

### `fn new`

Everything starts fresh, deliberately: opening a document
constructs a whole new `OpenDoc`, so a cached texture or a page
index can never refer to a page from a previous file. A `reset()`
method would be a second, weaker way to achieve the same thing and
an invitation to reuse an `OpenDoc` across documents — which is
exactly the stale-state bug that constructing fresh state prevents
by design.

`pub(crate)` rather than private so a panel's own test can build the
document state its body reads, through the **same** constructor
[`PdfcerApp::open_path`] uses. A test-only alternative constructor
would be a second way to assemble an `OpenDoc`, which is precisely
what this function's own argument says not to have.

### `fn created`

`name` is what the document is called, not where it is —
`crate::text::files::untitled`. See [`Origin::Created`].

A sibling of [`Self::new`] rather than a flag on it, because the two
read differently at the call site and one of them is rare: `open_path`
and `new_document` each say which they mean, and neither passes a
boolean whose meaning a reader has to look up.

### `fn stored_under`

The single predicate that separates [`Origin::Opened`] from
[`Origin::Created`] at every site that cares, and it is deliberately
shaped as *"give me the path if there is one"* rather than as
`is_created()`: the three call sites all want the path, so a boolean
would leave each of them reaching for `self.path` afterwards and one of
them eventually forgetting the test.

Its three readers, and what each would do wrong without it:

| site | without this |
|---|---|
| `PdfcerApp::open_path` → `RecentFiles::remember` | the Recent menu gains a row for a file that does not exist, whose whole promise is *"this worked before"* |
| `viewer::remembered` (read at open, written by `SetPageDisplay`) | a page-display choice stored against a fabricated path, and inherited by the next document that happens to be called the same thing |
| `canvas::guides` (read in [`Self::assemble`], written by `SetGuides`) | the same, for guide positions |

It is **not** consulted by the forms cache key, the Pages panel caption
or the trace, and that is correct rather than an omission: those want an
identity or a label, and a name is both.

### `fn current_extent`

Falls back to a US Letter shape for a document with no pages, so the
fit arithmetic has something finite to divide by. Nothing is drawn
in that state — the canvas shows [`crate::text::canvas_no_pages`] —
so the value is never seen; it exists so the arithmetic upstream of
the check does not have to special-case an empty document as well.

### `fn set_annotations_visible`

A staleness key, so changing it makes the cached texture stale and the
page re-rasterizes on the next frame — see [`RenderKey`]. That is the
whole difference between this being a control and being a bool nobody
can see.

**Deliberately does NOT bump [`Self::edit_epoch`]**: nothing about the
document has changed, only what is drawn of it. Bumping would throw
away the decomposition and the font inventory to no purpose, and would
make an `objects n=` line re-trace as though an edit had happened.

### `fn strip`

Built from the page vector and the view state, so it cannot disagree
with either. The convenience over calling
[`crate::viewer::strip::Strip::new`] at each site is not brevity: it is
that the three arguments after `pages` are all view state, and a call
site that passed its own idea of the display mode or the zoom would be
laying out a strip the rest of the frame does not agree with.
