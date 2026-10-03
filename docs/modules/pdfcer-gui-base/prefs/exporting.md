# `prefs::exporting` — what the three export windows remember

Operator request **O196**, 2026-09-13: *"the export windows forget every
setting."* Three windows, one complaint, and it is the same complaint
**O166** made about the Print window three days earlier — so this module is
deliberately a port of [`super::printing`] rather than a fresh design, down
to the shape of its token functions and the position of its `remember` call.

## ⚠ This is NOT O192, and the difference decides what gets built

`OPERATOR_REQUESTS.md` says it in the row itself: **do not fold this into
O192.**

- **O192** is *"the window reads no current value **from the document**"* —
  the Set Scale dialogue opening with `1 : 1` while the drawing it is about
  is at `1 : 50`. The fix is a **read of the open document** at construction.
- **O196** is *"the window remembers no previous value **across
  sessions**"* — Export image opening on PNG/300 dpi every time, for an
  operator who exports EMF at 600 every time. The fix is a **read of a file**
  at construction.

They look alike from a distance and they fail in opposite directions: O192's
fix must be overruled by the document, O196's fix must be overruled by
nothing *except* the document. §"the DXF ordering rule" below is where the
two meet, and it is the one place in this module where getting the order
wrong produces a silently wrong file rather than a mildly annoying window.

## What the three windows were doing before, measured rather than assumed


| window | what it hard-coded |
|---|---|
| Export image | `Png`, `CurrentPage`, 300 dpi, transparent, quality 90 |
| Export text | `AllPages`, `FormFeed`, `AsExtracted`, no BOM |
| Export to DXF | `DxfOptions::default()` — `Inches`, `fit_arcs`, `Entities`, `R2000` |

So an operator who exports every drawing as an EMF at 600 dpi re-answered
both questions on every single export, and an operator who works in
millimetres re-picked millimetres every time the DXF window opened.

## The distinction that decides what is remembered

[`super::printing`]'s question, unchanged, because it is the right one:
*would this value still be right for a **different document**?*

| Remembered | Not remembered, and why |
|---|---|
| The image format, the resolution, transparency, JPEG quality | The page index the window froze — it names a page of *this* document |
| Which pages, as a **policy** (see below) | The typed page range, same reason |
| The text separator, the line endings, the byte-order mark | The largest-page measurement — a measurement of *this* document |
| The DXF units, arc fitting, whether text is written, the DXF version | **The DXF scale** — see below; this is the important one |
| | `DxfScaleSuggestion` — an inference about *this page's* ce dimension groups |
| | `arc_tolerance` — no control exists for it; see below |

### The DXF scale is NOT remembered, and that is the whole point of the window

`ExportDxfDialog` exists because a DXF carries no scale of its own: the
number in that box is the only thing standing between the operator and a
drawing that imports at 1/50th of its real size. It is **derived** on every
open from the page's own ce dimension groups (`suggest_scale_for_groups`),
and a remembered scale would reintroduce, silently and across sessions,
precisely the defect the window was built to prevent: a plausible number
already in the box, belonging to yesterday's drawing.

⇒ A remembered value is only safe when a wrong one is **visible**. A wrong
format is visible — the file has the wrong extension. A wrong scale is not:
the DXF opens, the geometry is all there, and it is the wrong size.

### The DXF ordering rule, which is the only way to get this wrong quietly

`ExportDxfDialog::open` seeds `units` from the *suggestion* when the page
carries a calibrated ce dimension group, because — its own words —
*"a candidate is a group's whole opinion; a 1:50 metre group and a 1:50 inch
group are different answers wearing the same number."* The remembered units
are the operator's **habit**; the suggested units are a **measurement of the
page**.

⇒ **The habit seeds first and the measurement overrules it.** Apply them the
other way round and the window shows millimetres beside a scale derived from
an inch group, and the DXF comes out wrong by a factor of 25.4 with nothing
on screen to say so. Asserted in [`crate::dialogs::export_dxf`]'s own tests;
stated here because this module is where the temptation to "just apply the
preferences last" lives.

### `PageScope::Typed` is remembered as its own fallback, deliberately

[`PageScope`] has three variants and only two of them are policies.
*"Current page"* and *"all pages"* are true of any document; *"the pages I
typed"* is meaningless without `range_text`, which names pages of one
document and is therefore not remembered.

So `Typed` is written to the file as the window's own shipped default
(`current` for images, `all` for text) — a deliberate reduction, exactly like
[`super::printing`]'s `PaperChoice::Form(_) => "device"`. The alternative is
a window that opens with the **Pages** radio selected and an empty box beside
it, which greys the Export button on open for no reason the operator can see.

### ⚠ `DxfOptions::arc_tolerance` is not remembered, because nothing can set it

It is a real field with a real default (0.05) and **no control in the
window**. A preference for it would be a key only a hand-editor could reach,
describing a setting with no UI — which is the file-format equivalent of the
disabled stub **R9** forbids. If a control is ever added, the key is added
with it, in this file, in one edit.

### Why the two page scopes are separate keys and not one

Export image defaults to **this page** and Export text defaults to **all
pages**, and `dialogs/export_text.rs` argues that divergence at length: a
picture of one sheet is the common want, and a text file of one page of a
forty-page document almost never is. One shared `export_pages` key would
force those two windows to agree, which would make the divergence
unexpressible and silently discard one of the two decisions.

## Where it is stored, and why not `settings.txt`

`preferences.txt`, beside the shell's other preferences: flat `key = value`,
hand-editable, per-key recovery, and already covered by the update
instruction *"replace the program files, keep your `userdata` folder"*.

Not `settings.txt`. Every entry in that file cites a clause the PDF standard
leaves to the implementation; whether this operator likes a byte-order mark
is not one of them.

## Why the values are the windows' own types and not a mirrored set

Same reason as [`super::printing`]: a mirrored enum is a second source of
truth that drifts. This module stores [`ImageFormat`], [`PageScope`],
[`PageSeparator`], [`LineEndings`] and the engine's own [`DxfUnits`] and
[`DxfText`] — the exact values the windows hold — so a variant added to any
of them is a compile error here rather than a silent round-trip to the
default.

All six are `pub` and none is `#[non_exhaustive]`, which is why every
`*_key` function below is an exhaustive `match` with **no `_` arm**. That is
deliberate and it is the difference from [`super::printing`]'s `scope_key`,
which needs a catch-all because `AnnotationScope` is a foreign engine enum
that may grow. Do not add a `_` arm here to silence a future compile error:
that error is the mechanism working.

## The token functions, and the property that binds them

Every enum has a `*_key` (value → token) and a `*_from_key` (token → value)
function, and each pair is asserted to round-trip over **every variant** in
this module's tests. That is the property the file format actually needs: a
writer that emits a token its own parser rejects turns the operator's
settings into a `BadValue` note on the next launch, which reads as pdfcer
forgetting them — the very complaint this module answers.

## Item notes

### `fn default`

The specification, not a coincidence: a fresh `userdata` folder must open
this window the way every previous build of pdfcer opened it. **Deleting
`preferences.txt` is a way to reset pdfcer, never a way to change what it
does.** Asserted rather than assumed — see
`tests::the_image_default_is_what_the_dialog_used_to_hard_code`.

### `fn default`

⚠ Written out field by field rather than `#[derive(Default)]`, and that
is not style. [`PageScope`] has **no** `Default` impl at all, and the
other three would take their own `#[default]` variants — which happen to
agree today and are not *specified* to. This impl is the specification;
the derive would have been a coincidence that compiles.

### `fn default`

Literals, deliberately: if the engine changes a default, this window's
behaviour must change **visibly, in a diff**, not silently on a
`cargo update`. The disagreement is then a failing test rather than a
different DXF.

### `const MIN_EXPORT_DPI`

These four constants are **the controls' bounds**, not separately reasoned
limits, and [`super::printing`]'s header states why that matters: *"a file
that refused a value the operator could produce by dragging the box would
silently discard a setting they had just made."* If a control's range
changes, change it here in the same commit — the round-trip is only honest
while the two agree.

### `struct ExportImagePrefs`

Five fields, every one of them an answer to *"how does this operator export
pictures"* rather than to *"what is in this document"*. `PartialEq` and not
`Eq` because [`Self::dpi`] is an `f32`; the derive is load-bearing rather
than decorative — [`crate::dialogs::export_remembered::remember_image`]
compares with `!=` and skips the file write when nothing moved.

### `struct ExportTextPrefs`

`PartialEq` for the same dirty-check reason as [`ExportImagePrefs`]; `Eq`
would be derivable here, and is deliberately not derived, so the three groups
present one shape to a reader.

### `struct ExportDxfPrefs`

Three fields out of `DxfOptions`' five. The other two — `scale` and
`arc_tolerance` — are argued in the module header, and the argument for
`scale` is the most important sentence in this file.

### `fn image_format_from_key`

`jpg` is **not** accepted as a synonym for `jpeg`. A second spelling is a
second thing the writer and the parser have to agree about, and the file
documents its own vocabulary in the block [`write_block`] emits.

### `fn page_scope_key`

**Lossy on purpose.** [`PageScope::Typed`] has no token, because the
typed range it depends on is not remembered; a window restored into `Typed`
with an empty range box would open with its Export button greyed and nothing
on screen to explain it. The caller decides what `Typed` degrades *to* — see
[`page_scope_key_or`], which is what both windows actually call.

### `fn page_scope_key_or`

`fallback` is the window's shipped default — `CurrentPage` for the image
window, `AllPages` for the text window — so a reduction lands on the answer
that window would have opened with anyway rather than on a third behaviour.

# Panics

Never in practice, and the `expect` says which contract would have to be
broken first: `fallback` must itself be a scope with a token. Passing
`PageScope::Typed` as the fallback is the one way to reach it, and no caller
does — both pass a `const` default.

### `fn dxf_units_key`

`millimetres` with the British spelling, matching the engine's own variant
name. The file is a vocabulary of its own and consistency with the type it
describes beats consistency with any other file on the machine.

### `fn dxf_units_from_key`

`mm` and `millimeters` are accepted **in addition**, and this is the one
place in this module that takes a synonym. The reason is not symmetry with
the writer — the writer emits exactly one spelling — it is that this is the
single key in the file whose British spelling an American hand-editor will
get wrong, and the cost of a `BadValue` here is an operator silently
exporting inches. The writer's block names the canonical spelling.

### `struct ExportPrefs`

Held on [`Prefs`](super::Prefs) as three separate fields rather than one, so
that a window reads only its own group and a future fourth export window adds
a struct rather than widening one. This type exists only so [`parse_key`] and
[`write_block`] take one argument instead of three.

### `fn parse_key`

# Why the parser for this group lives HERE and not in `prefs::file`

[`super::printing::parse_key`] argues this at length and the argument carries
over unchanged: the rule *"adding a preference is one edit to one file"* is
about **the parser and the writer staying together**, not about their being
in `file.rs` specifically. Twelve keys, all about exporting, in the file that
already owns their types, defaults and token vocabulary.

`file.rs` keeps one chained call that delegates here and one that delegates
to [`write_block`], so the round-trip tests over the whole of
[`Prefs`](super::Prefs) cover this group unchanged.

# The contract

`value` arrives already trimmed, as `file.rs` trims both halves before it
dispatches. Returns [`KeyOutcome`]; see its variants. [`KeyOutcome`] is
borrowed from [`super::printing`] rather than re-declared, for the reason
`offpage` borrows it: a second copy is a second thing that can drift.

# Out of range CLAMPS; unparseable is a `BadValue`

The numeric keys follow [`super::printing`]'s ruling exactly. `dpi = 99999`
becomes [`MAX_EXPORT_DPI`] silently, because the number is a legible
intention the control itself would have clamped. `dpi = fast` is a
[`KeyOutcome::BadValue`] reported at its line number, because it is not.

### `fn write_block`

Called once by `Prefs::write_to_string`. The comments are as long as they are
because the file is meant to be opened in a text editor, and
`export_dxf_units = millimetres` tells an operator nothing about what else
they could write there.

Written **unconditionally**, even on a fresh profile where every value is
the default. `offpage`'s own note is the reason: *"a preference nobody can
discover is a preference nobody has."*

### `export_image_background`

`#rrggbb`, parsed by `Rgb::parse_hex`; anything else is a bad value and the
default (white) stands.
