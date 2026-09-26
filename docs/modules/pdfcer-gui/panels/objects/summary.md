# `panels::objects::summary` — the ONE description of a page object

Turns a `pdfcer_core::vector::VectorObject` into a small, GUI-shaped
**fact record** ([`ObjectSummary`]) that every surface which has to say
*"what is this thing?"* reads.

## Its consumers, and why "one description path" is the whole point

Four surfaces have to say what a selected object is, and none of them may
disagree with another: a Path's fill colour must never be described one way
in a tree row and a different way in Properties.

| Consumer | Where | State |
|---|---|---|
| Objects panel row label | [`crate::panels::objects`] | live |
| Properties panel | [`crate::panels::properties`] | live |
| Status-bar selection readout | `app/status.rs` | arrives with the selection model |
| Canvas selection overlay | `canvas/overlay.rs` | arrives with the selection model |

Two consumers is already enough for the rule to bite: the Objects panel
says *"Path · blue · 4 nodes"* in a row, and the Properties panel says
the same three facts in a list, and they are the same three field reads
rather than two pieces of code that happen to agree today.

Decision 011 names this exact failure shape — *"two decompositions quietly
diverge"* — and two *descriptions* of one decomposition is that defect one
layer up. This module is the structural answer to it.

## Why a fact record and not a `String`

Because every operator-visible string lives in [`crate::text`]. So this
module deliberately holds **no prose at all** — it classifies, measures
and counts, and [`crate::text::panels::objects`] alone renders. That
split is also what makes it unit-testable without an egui frame: the
tests below assert on enum variants and numbers, never on wording that a
copy edit would break.

## This module is rule 4's disclosure half, and nothing else

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s first
non-negotiable, as narrowed:

> **Disclosure lives off-canvas**: a status line, a results panel, a
> report after the command, a properties field. … **No badge, tint, red
> flag, dashed outline or "provisional" layer drawn into the page view.**

[`ObjectNote`] is that disclosure, and a panel is its correct home.
[`ObjectSummary::bounds_are_approximate`] therefore drives **a sentence in
a panel** and never a dashed outline on the canvas: a dashed outline around
content that is merely *described* imprecisely would be pdfcer marking its
own uncertainty on the page, which is precisely what rule 4 forbids. The
predicate earns its place because the question — *is this box an
approximation?* — is the right one to ask once, in one place.

A pre-commit affordance — a selection handle, a hover highlight, a
rubber band — is explicitly still welcome; those are the cursor, not the
content. None of them are drawn from here.

## What it can and cannot say

`pdfcer_core::vector::TextObject` carries a decoded [`TextPreview`] and a
[`TextFont`], and `ImageObject` carries `pixel_size`. This module
surfaces them — and surfaces their **absence** just as loudly, because
the interesting cases are the ones where a value is missing:

| Core says | This module reports | Why not something friendlier |
|---|---|---|
| `TextPreview::Decoded { lossy: false, .. }` | [`ObjectSummary::text`] = the string | — |
| `TextPreview::Decoded { lossy: true, .. }` | the string **plus** [`ObjectNote::TextPartlyUndecodable`] | The `\u{fffd}`s in the row are real; a note is what turns them from "pdfcer is broken" into "this font's encoding is incomplete". |
| `TextPreview::Undecodable` | `text = None` **plus** [`ObjectNote::TextUndecodable`] | A row of replacement characters looks like a defect. The honest answer is *"this text cannot be read, here is why"*. |
| `TextPreview::Unavailable` | `text = None`, no note | Nothing was attempted (no font resolver — the headless/unit-test path). The GUI always resolves fonts, so an operator never sees this state; disclosing it would be noise about a code path they are not on. |
| `TextPreview::Empty` | `text = None`, no note | The object really does show nothing. |
| `font: None` | [`ObjectSummary::font`] = `None` | No `Tf` was in effect. Never invented. |
| `pixel_size: None` | [`ObjectSummary::pixels`] = `None` | A form XObject has no samples; a malformed image's `/Width`/`/Height` are unusable. Deriving a number from the bbox would state a resolution the file does not have. |

The one thing still not said is a text object's **exact** extent. The
bbox is laid out from the font's own metrics — per-code advances from
`/Widths`/`/W`/the standard-14 AFM tables, height from
`/FontDescriptor` — so it is where a conforming reader puts the run, but
it is not measured glyph ink, and for a font with no usable metrics it
falls back to a coarse em box around the run's origin.
[`ObjectNote::ApproximateTextBounds`] is on every text object and carries
which of the four constructions produced this one.

## [`ObjectNote`] — the point of the whole module

The operator's report was *"sometimes I click and get a box highlighting
on the screen that doesn't seem to correspond to anything."* Three causes
of that were hit-testing bugs and are fixed. The residue is
**legibility**: a selection can be entirely correct and still enclose
apparently-empty paper. Every such case is a *known, already-computed
fact* about the object, and [`describe_object`] emits one note per
applicable case:

| Note | Real cause of a "box over nothing" |
|---|---|
| [`ObjectNote::ApproximateTextBounds`] | `TextObject`'s bbox is never measured glyph ink, so it can enclose paper the operator can see is empty (a font's designed ascent sits above most lowercase letters) and, in the `EmBox` fallback, can miss visible glyphs entirely. `approximate` is always `true`, so this note is on every text object; its payload says which construction produced the box, and therefore which of four sentences explains it. |
| [`ObjectNote::PaintsNothing`] | An `n`-op path (a clip, or a discarded construction) is a real, selectable object that paints no pixels at all (`PaintStyle::is_invisible`). |
| [`ObjectNote::DegenerateBounds`] | A horizontal or vertical rule has a bbox of zero height or width. It is selectable and correct — and a zero-extent outline rect strokes **nothing**, so without the note the operator sees a click that appears to do nothing at all. |
| [`ObjectNote::NoBounds`] | The object has no finite geometry, so no outline can be drawn anywhere. Rare, and without the note indistinguishable from a dead click. |
| [`ObjectNote::FormNotDecomposed`] | A form XObject is ONE opaque object: its outline covers the whole nested drawing, and its children are not individually listed or clickable. |

What is deliberately **not** here: a same-colour ("white on white")
heuristic. Whether a fill matches its background cannot be decided from
`PathObject`'s own fields — the backdrop may be another filled shape, an
image, or blank paper — and the ui-spec names that as an honest limit
rather than a guess to make. The readout states the object's own colour
verbatim instead and lets the operator draw the conclusion.

## Three standing constraints on this file

1. **It does not use egui.** Classification, measurement and counting need
   no frame, and keeping the dependency out is what lets the tests below
   run without one.
2. **`ObjectNote::ALL` and `ObjectKind::ALL` carry no `#[allow(dead_code)]`.**
   This crate is a library, so a `pub` const is never dead, and an allow
   that suppresses nothing is a lie about the code it sits on.
3. **Never write a bare "dimension" here.** Project rule 15 reserves the
   word: **ce dimensions** are the ones pdfcer authors and **pdf
   dimensions** are CAD content it reads. An image's sample count is
   neither, so it is a "pixel size" or a "sample count", in prose and in
   test names alike.
