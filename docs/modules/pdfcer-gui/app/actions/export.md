# `app::actions::export` — writing part of the document out as something
else

## Why this is its own file

The sixth sibling of [`super::apply`], drawn along the same seam the other
five are: *what class of thing does this verb act on?* — pages there,
annotations in `annots`, the dimensioning model in `dimensions`, page
content in `apply`, redaction marks in `redact`. This is **what leaves the
document**.

It is a real subject rather than a size-driven cut, and the evidence is the
property every verb here shares and no verb elsewhere does: **none of them
changes the document at all.** No `vector_edit`, no undo entry, no epoch
bump, no cache invalidation. They read the open file and write a different
one, which makes every rule the mutation funnel enforces irrelevant to them
and every rule about *file* handling — a save picker, an overwrite, a
partial write — apply instead.

`super::pages::extract` is the same shape and stayed in `pages` because its
subject is a page set. If a third export lands, that is the moment to move
it here.

## The third export landed, and the trigger above FIRED

[`image`] — `OPERATOR_REQUESTS.md` **O120**, PNG / JPEG / SVG — is the third,
and it is written here rather than in a module of its own, which is the
easier half of what the sentence above asks for. The harder half is
**`pages::extract` has not moved**, and that is recorded rather than quietly
not done:

* The condition is met. Three exports now exist and the family is real.
* Moving `extract` is a change to `pages`, to `apply`'s dispatch and to
  whatever names it, made in the same pass as a new feature — and this
  project's own record of what that produces is `RIBBON_IA.md`'s repeated
  lesson that a taxonomy move and a capability arriving together make a diff
  nobody can review as either.

⇒ So the trigger is left **armed and stated** rather than silently reset. The
next reader of this header is looking at a condition that has fired, with
the reason it was not acted on written beside it, which is the shape this
project uses everywhere else for a decision deferred on purpose. What must
not happen is the sentence above being read as still-waiting: it is not.

## Why an export is an `Action` at all

`super::apply`'s header answers it for `SaveCopy` and the answer is the same
here: **a native file dialog must not open inside a layout pass.** It is a
modal OS window that blocks the thread, and opening one from a widget's
`clicked()` branch means egui is part-way through building a frame that will
not finish until the operator has answered.

Nothing about the document is being ordered — there is nothing to order —
so the funnel's *invariant* does not apply. Its **reason** does.

## Item notes

### `fn suggested_form_path`

Beside the document and named after it, with `.fdf` — [`suggested_path`]'s
rule and its reason. The extension is the **default format** as well as a
suggestion, which is why it is FDF: see [`form_data`]'s header.

### `fn suggested_path`

Beside the document, named after it, with a `.dxf` extension. The same rule
`super::pages::suggested_path` follows for an extract, and for its reason: a
picker that opens in the last-used directory of some other application is a
picker that makes the operator navigate back to their own project every
time.

### `struct Output`

A struct rather than a tuple because the caller has to pick a *different
sentence* per kind, and a `(Vec<u8>, u32, u32, usize, Vec<String>)` would
carry two fields that are meaningless for one of the two branches.

### `enum Failed`

[`Self::Render`] is about the **page** — it will happen again whatever
format is chosen, and the resolution or the document is the thing to look
at. [`Self::Encode`] is about the **format**, and its commonest cause is
`ExportError::TooLargeForJpeg`: JPEG stores its dimensions in sixteen bits
(ITU-T T.81 §B.2.2), so a raster over 65,535 pixels on a side has no JPEG
form at all while having a perfectly good PNG one.

⇒ A single "export failed" would send an operator whose only problem is a
format's arithmetic limit off to investigate their drawing. Each carries
the engine's own message, which names the numbers.

### `fn emf_bytes`

# Why this is a third function and not a flag on [`svg_bytes`]

The two share a *recording* — `pdfcer_render::emf` walks the same export
display list `pdfcer_render::svg` does — and share nothing else. Different
options type, different outcome type, different disclosure, different
receipt line, and a different answer for every one of the five things EMF
cannot express. A `if metafile { … } else { … }` inside one function would
be two functions sharing a brace.

# The background follows [`svg_bytes`]'s rule, for the same reason

`EmfOptions::background` is `Option<Rgb>` exactly as `SvgOptions`' is, and
`None` is the transparent state — the engine's CLI calls it *"EMF's
natural state (nothing is drawn where nothing was painted)"*. So the same
flag drives both, and the `RenderOptions` backdrop the caller set is again
**not** what decides it.

⇒ This is the shape of mistake that ships a window promising a clear
background and a file with a white rectangle at the bottom of it, and it
is worth the second comment because the two option structs are the two
places in this file where the backdrop is a decoy.

# Nothing here validates the metafile, and that is a decision

`pdfcer_render::emf::walk_records` exists precisely so a consumer can
check a metafile's record structure before handing it to
`SetEnhMetaFileBits`, and it is deliberately not called on this path. A
**file** export hands the bytes to `std::fs::write`, which cannot be made
to misbehave by a malformed record; the reader that would choke on one is
somebody else's program, tomorrow. Walking every record to produce a
sentence nobody could act on would cost a second pass over the whole
metafile on every export.

The **clipboard** path is the one that owes this check, because there
`SetEnhMetaFileBits` is handed a raw buffer and a bad one is a GDI failure
rather than a refusal. See `crate::clipboard`, which is where that call
will live when the placement half is buildable.

### `fn honesty_notes`

A helper rather than four inline `if`s at two call sites, because **both**
the refusal path and the success path owe exactly this set. A document whose
every page is Identity-H-without-`/ToUnicode` refuses with a zero character
count that looks identical to a scan, and the counter is the only thing that
tells them apart.

`TextDiagnostics` carries roughly thirty counters and this takes four. The
other twenty-seven are true and are not **actionable**: `spaces_derived` and
`lines_derived` are facts about every extraction ever run, and the window
already said so in `loses_breaks` where an operator can read it before
deciding. A status bar that lists everything measured is one nobody reads,
and rule 4's whole value is in being read.

### `fn named`

`suggested_path` and `suggested_form_path` take an `OpenDoc`, which
carries a session and cannot be built in a unit test. Their arithmetic
is one line each and it is that line that was wrong, so it is
reproduced here **from the same expression** rather than re-derived —
if either site changes shape, this stops describing it and the comment
below is the instruction to whoever notices.

Not a seam worth extracting: a shared helper would be a third place
the rule lives, and the rule is `format!("{stem}.{ext}")`.

### `fn a_dotted_document_name_keeps_its_revision`

`Path::set_extension` replaces everything after the **last** dot, so
under that call `plan.rev2` becomes `plan` and the suggested name is
`plan.dxf`.

⇒ The consequence is data loss, not untidiness: `plan.rev2.pdf` and
`plan.rev3.pdf` would both suggest `plan.dxf`, so exporting the second
**overwrites the first** — behind nothing but the operating system's
generic "a file with that name already exists". `.rev2` / `.rev3` is an
ordinary CAD naming shape, and the two files that collide are the two an
operator is most likely to want side by side.

The reasoning that makes `set_extension` look safe reads as true until
the helper meets a stem with a dot in it. **A claim in a comment is not
a test.** This is that comment, executed.

### `fn an_ordinary_name_and_a_bare_one_both_gain_the_extension`

The old comment's one true claim was that a document called `plan`
with no extension *"would gain one only through this call"*. It still
does: the stem of `plan` is `plan`, and the format string appends
unconditionally.

### `fn dxf`

## What this owes the operator, and why it is not optional

`DxfOutcome` is the disclosure half, and two of its counts are the reason
this feature is worth having over any generic converter:

- **`skipped_images`** — DXF has no raster entity, so a picture on the page
  is simply not in the file. The engine's own words for why that must be
  said: *"an operator whose drawing was half annotation gets a DXF that
  looks like the geometry went missing, and 'the labels are not in this
  file' is a sentence they need **before** they open it in SOLIDWORKS, not
  after."*
- **`unreadable_text`** — text pdfcer could not decode, kept apart from
  `skipped_text` (which the operator asked for) because one is a choice and
  the other is a fact about the source PDF. Rolling them together would let
  the second hide inside the first.

## Why the geometry is decomposed here and not carried in the action

`decompose_all` decomposes every requested page from `session.view()` when
the queue drains, so an edit raised earlier in the same frame is in the
export, and each page is the page asked for rather than whichever page the
canvas's cache holds. All pages are decomposed before the save dialog: one
unreadable page declines the whole run (`export-dxf-declined page=
reason=no-decomposition`), so the operator never chooses a name for files
that will not all arrive. This is the engine CLI's all-or-nothing order.

## Several pages

One page writes the chosen file. Several write `<stem>_p<n>.dxf` beside it
(`dxf_page_path`), `n` 1-based and zero-padded to the widest page number in
the run, the engine CLI's `export-dxf --pages --output-dir` naming; the bare
chosen name is not written. Each file traces its own `export-dxf page= ...
path=` line. The status sentence sums the counts over the files
(`text::export_dxf::exported_pages`). A write failure stops the run and says
how many of the files were written (`stopped_part_way`).

### `fn form_data`

`file.export_form_data`. This command was registered, drawn on
File ▸ Export, and **inert**, behind a `SCAFFOLDED` reason claiming the
writer did not exist and citing a `FEATURES.md` row saying the
FDF/XFDF/CSV half was unbuilt.

# The recorded reason was false, and it is the sixth of these

Three writers exist: `fdf::FormData::to_fdf`, `to_xfdf`, and
`formcsv::to_csv`, reached through `EditSession::export_form_data`. The
`FEATURES.md` row the entry cited was itself stale, so the reason was a
**citation of a citation** and nothing had re-read either.

⇒ The rule now written on the allow-list's own assertion: *when you touch
that list for any purpose, re-derive the reason of the entry beside the one
you came for.* This one was found by doing exactly that.

# The format is chosen by the EXTENSION, not by a third dialog

One picker, three formats, decided by what the operator types or picks in
the *Save as type* box — which is how every application on this desktop
does it, and which `crate::text::tool`'s rule about conventions makes the
default answer rather than a shortcut.

The alternative — a format dialog, then a picker — is two modal windows for
one act, and it puts the choice **before** the operator has thought about
where the file goes, which is the order they think in reversed.

An unrecognised extension is **FDF**, and that is a decision rather than a
fallback: FDF is the format the standard defines for this data (§12.7.8),
it is what Acrobat writes, and it is the only one of the three that a
reader can import without being told what it is.

# The CSV disclosure is not optional, and it is about a spreadsheet
rather than about a PDF

`formcsv::to_csv` **neutralises** values that would otherwise be executed as
formulas when the file is opened in a spreadsheet — a leading `=`, `+`, `-`
or `@`. That is a real and well-known injection route, and pdfcer doing the
right thing silently would leave an operator believing their exported data
is byte-identical to what the form holds.

It is not. `neutralised` counts how many, `neutralised_fields` names them,
and both are reported. Rule 4's *"the half that survives is the point"*: an
inference the operator cannot see still owes them an off-canvas sentence.

# Nothing about the document changes

No `vector_edit`, no epoch bump, no cache invalidation — this module's
header explains why that is what makes these verbs a family. The
disclosures ride `record_edit_disclosure` at the current epoch, so they
stand until the next real edit moves past them.

### `fn image`

The operator, verbatim:

> *"can you add the ability to export page(es) to png, jpg, svg. note that
> there had better be full support (including transparency where
> supported!)."*

This module's header says the third export is the one that decides whether
the family is real. It is, and it is: nothing here changes the document, no
`vector_edit` runs, no epoch moves, no cache is dropped. What it shares with
its two siblings is the whole of what the module is for — **it reads the
open file and writes a different one.**

# The refusal comes FIRST, before the picker and before the render

[`crate::app::actions::imageexport::ImagePlan::impossible`] is asked before
anything else happens, and the reason is the engine's own instruction:

> **refuse a "transparent" JPEG by name in your UI, never flatten silently**

The window already prevents the combination — its checkbox goes dead when
JPEG is selected and says why — so reaching this branch means the window was
bypassed. That it is *unreachable today* is exactly why it is here: the
property that must hold is **pdfcer never puts a page on a white background
without saying so**, and a guard that lives only in a window makes that a
property of the window rather than of the program. A keymap, a restored
plan, or a later window with a different layout each walk past a window and
none of them walks past this.

⇒ And it *refuses*. Flattening would produce a file that opens, looks nearly
right, and carries a white rectangle the operator meets when the drawing is
already inside somebody else's document.

# Why there is no call to `pdfcer_render::export::flatten_over`

The engine offers it, this function does not use it, and that is worth
stating rather than leaving as an apparent omission.

Transparency is declined **at the source**, by rendering with
[`pdfcer_render::PageBackdrop::White`]. ISO 32000-1 §11.4.7 already makes
the page an isolated group composited over white, so the renderer's own
composite *is* the standard's; `flatten_over` is a second, later composite
over a buffer that has already been premultiplied. The two agree for
ordinary content and only one of them is the specification's, so that is
the one used. `flatten_over` earns its place in a caller holding a pixmap
it did not render — a clipboard paste, a region grab — and this is not one.

# The order is REFUSE, ASK, RENDER — and it differs from [`dxf`]'s

[`dxf`] does the whole write before opening the picker, on the rule *"the
operator is never asked where to put a file that turns out to be empty"*,
and it can afford to because a DXF write is pure and cannot fail.

A page render is neither pure nor cheap. It is the most expensive thing this
program does, it takes seconds on a dense CAD sheet, and fifty of them
before a picker would mean an operator who presses Cancel has waited for
nothing. So the picker comes second — and the property `dxf`'s ordering was
protecting is preserved by a different mechanism: **everything that could
make this export empty has already been said in the window**, beside the
control that causes it. The pixel count, the `MAX_PIXMAP_EDGE` ceiling, a
range naming no page, and the transparent-JPEG refusal are all on screen
before the button is pressable.

# Rule 4 — the disclosure, off-canvas and afterwards

Nothing is marked on the page or on the canvas. Every sentence goes to
[`super::record_notes`], the same slot [`dxf`] and [`form_data`] use,
stamped with the current epoch so it stands until the next real edit moves
past it.

What it carries, and why each is owed:

* **the resolution written into the file.** The engine's note: *"without
  `pHYs` Word places a 300 DPI page four times too large."* That the number
  was *chosen* is not the claim; that it *travelled* is.
* **whether transparency survived**, in either direction — the operator
  asked for it by name, so both answers are answers.
* **`ExportTally`, for SVG** — shadings rasterised, soft masks kept,
  overprint and non-separable blends drawn as their `Normal` approximation,
  dashed strokes pre-applied, blend modes Word's importer ignores.
* **that SVG text is glyph outlines**, which nothing counts, which no
  inspection of the file by an operator would reveal, and which is the
  single largest surprise the format holds.

### `fn text`

> *"also the engine can export PDFs as text. we should have export/import
> for that."*

Both halves of that sentence ship. `super::exporttext`'s header carries the
finding on which of the three senses of *"import text"* the other half is:
`EditSession::place_text` and `blank_document` are wired as
`file.import_text`, and `crate::app::actions::importtext` is the other end
of the round trip.

# It writes the CLIPBOARD's own string

At the plan's defaults, the bytes this writes are exactly what
`file.copy_document_text` puts on the clipboard: the settings funnel's
`ExtractOptions`, `plain_text()`, U+000C between pages, no BOM, no
line-ending rewrite. `app::dispatch::textcopy`'s header makes the argument
for its two verbs sharing one extraction; this is the same argument with a
file on the end of it. **Two answers to "what is the text of this document"
inside one program is worse than either**, because both of them look like
text and nothing on screen would say which one you have.

# The order is EXTRACT, REFUSE, ASK, WRITE — and the refusal is the
whole feature

[`dxf`]'s ordering, not [`image`]'s, and for [`dxf`]'s stated rule: *"the
operator is never asked where to put a file that turns out to be empty."*

Here that rule stops being a nicety and becomes the point of the feature.
**A scanned drawing has no text layer**, so extracting it succeeds, returns
nothing, and would write a zero-byte `.txt` — which is indistinguishable
from a successful export of a blank page. The operator finds out when they
open it, or worse, when whoever they sent it to does.

So a zero character count refuses **before the picker opens**, names why (the
page is a picture of its words rather than words — a fact about their file,
not a pdfcer failure) and names the remedy by its ribbon label,
`File ▸ Recognise text`. See `crate::text::export_text::no_text_at_all`.

⇒ [`image`] can afford the opposite ordering because a render is expensive
and everything that could make *it* empty is already stated in its window.
An extraction is 331–449 ms on this project's fixtures — `crate::find`'s own
measurement — and the thing that makes this one empty is a property of the
document that no window could have known in advance.

# `extract_pages_view`, for every scope, including "every page"

One entry point rather than two. `resolve_pages` already turns *every page*
into `0..count`, so branching to `extract_document_view` for that case would
buy nothing and would introduce the one thing this feature cannot afford: a
second path to the same string, differing in its failure semantics.
(`extract_document_view` swallows a bad index; `extract_pages_view` reports
`NoSuchPage`, which is the honest answer when a page vanished between the
press and the drain.)

`session.view()`, never `session.document()` — the operator is exporting the
document they are looking at, unsaved edits included (decision 018), which
is the same rule the two clipboard verbs and the print preview follow.

# Rule 4 — the disclosure, off-canvas and afterwards

Nothing is marked on the page. Every sentence goes to [`super::record_notes`]
at the current epoch, and the set is chosen so each one tells the operator
something they could **act** on:

* **the file, the page count and the character count** — the receipt;
* **which pages came out empty**, by number, because an empty page 4 in a
  six-page set is a scanned insert they can go and look at;
* **pages that could not be read at all**, kept apart from the above: an
  empty page is one pdfcer read and found nothing on, this is one pdfcer
  could not read, and rolling them together would let damage present as a
  scan;
* **fonts publishing no route to Unicode** — Type 3 without `/ToUnicode`,
  Identity-H without `/ToUnicode`. The text renders perfectly and is missing
  from the file, which is the standard's own answer (§9.10.2) and is exactly
  why it has to be said. Acrobat's answer to this case is silence;
* **characters that fell through the decoding ladder**, as a fraction, so
  40-in-200 and 40-in-400,000 do not read as the same event;
* **what pdfcer itself added**, when page markers were asked for.

### `fn standard_settings`

A rendering standard is applied with `RenderPreset::apply` to a clone of the
document's settings and the clone goes through the settings funnel; the
operator's settings are never written, as the engine CLI's `--standard`. The
keys it changed are traced (`export-image-standard`) and counted in the
receipt (`drawn_as`).

### `fn background_note`

The background a page was flattened onto, named in the receipt: kept clear,
white, or the `#rrggbb` chosen. A PNG on a non-white colour is rendered clear
and composited by `pdfcer_render::export::flatten_over`; a JPEG through
`JpegOptions::background`; SVG and EMF through `with_background`. All three
read `ImagePlan::flatten_colour`, so they cannot disagree.
