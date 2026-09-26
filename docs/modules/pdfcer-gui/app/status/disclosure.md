# `app::status::disclosure` — the three rule-4 lines in the status bar


> The left half carries four things, and only the first is the narrator. The
> others look similar and are governed by different rules.

Everything here is **rule 4**: pdfcer did something the operator did not ask
for and cannot see, so it says so — off-canvas, never on the page.

## Three lines, and they are INDEPENDENT

| line | answers |
|---|---|
| [`fill_disclosure`] | what a form fill had to **infer** — an auto-size chosen, characters that could not be encoded |
| [`edit_disclosure`] | what a move or delete had to change about an object's **form** to express the request |
| [`recovered_disclosure`] | how this **file** was assembled, before anything was drawn |
| [`load_anomalies_disclosure`] | what this **file** said twice, and which reading pdfcer used |

⚠ The table names the four rule-4 lines. Two further tenants have since
joined the module and are **states rather than disclosures** —
[`blend_space_disclosure`] and [`line_weights_disclosure`] — each arguing its
own obligation in its own header; and [`catching_up`] is narration. The
module is now "the bar's left-hand sentences", and only the four above are
governed by the rule this header opens with.

The obvious mistake, adding a third beside two, is an `else if` chain that
shows whichever fires first. A document opened from a damaged index, then
edited, with a form filled, owes the operator all three —
`disclosure_independence` in the parent asserts they cannot collide.

The last two are the odd ones out and the reason for this module's
header: the first two are about **something the operator just did**, and
those two are about **what the file was before they touched it**. They are
also the only ones that persist for the life of the document rather than
until the next edit — see [`load_anomalies_disclosure`]'s lifetime section
for why an `edit_epoch` key would be actively wrong for them.

## Item notes

### `fn fill_disclosure`

# Why this is not behind the disclosure triangle beside it

The render notes are *narration* — a census of what a raster contained —
and `DEFECTS.md` §5's complaint was their prominence: the first thing an
operator read was the application talking about itself. Demoting them was
right.

These two sentences are the opposite kind of thing. They are the surviving
half of rule 4: **an inference the operator cannot see still owes a
report.** `applied_autosize` means pdfcer chose a point size the document
asked it to choose; `unencodable_chars` means the operator's own typing is
not what the page now says. Neither is re-derivable from the saved file
afterwards — both look exactly like the author's decision — so a
disclosure the operator has to *open something* to find is a disclosure
that did not happen.

# Why the status bar rather than the Forms panel alone


# It retires itself

Keyed on [`OpenDoc::edit_epoch`] **after** the fill, so any later edit —
including an undo — moves the document past it and the sentence
disappears with no code remembering to clear it. That is deliberate:
state that must be cleared is state that will one day be shown against
the wrong document.

Elided at the same fraction as the notes line, whole text on hover, and
**it does not make the bar taller** — R128, exactly as for its neighbour.

### `fn edit_disclosure`

# What it says, and who wrote it

Every vector verb — the three move verbs and Delete — returns a list of
operator-facing sentences alongside its success, non-empty when the surgery
had to change an operator's *form* to express the request: an `re` rectangle
rewritten as four lines so one corner could move independently, an
implicitly-started subpath's `m` materialised, a curve dropped along with
the point it ran into. **The drawing is unchanged and the bytes are not
recoverable by reversing the gesture** — dragging the corner back does not
restore the rectangle form — which is precisely the condition rule 4 exists
for: pdfcer inferred a representation, and the operator would otherwise
learn it from a diff.

The sentences are `pdfcer-core`'s own and are passed through verbatim; this
module contributes the framing, and only the framing. See
[`crate::text::status::edit_disclosure_line`].

# Why it is here rather than only in the trace

It *was* only in the trace. `crate::app::actions::vector_edit`'s header
named that as the outstanding half in as many words — *"a disclosure that
only ever reaches `PDFCER_DIAG` has been recorded and not disclosed"* — and
this function is the half it was waiting for. The trace is unchanged and
still carries the full list; what has changed is that an operator who is
not running with `PDFCER_DIAG` set can now read it, which is every operator.

# Why the status bar rather than a panel or the canvas

Two constraints, and together they leave one surface. Rule 4 puts a
disclosure **off-canvas** — the one-line test is whether a screenshot of the
editing canvas would differ from a screenshot of the same document saved and
reopened, and a note drawn over the page would make it differ. And the
gesture that raises one is a **canvas drag**, available in Edit and Review
with any panel arrangement including none, so a panel could not be relied on
to be mounted. The bar is the one surface present in every mode.

# It retires itself, and it cannot collide with its neighbour

Keyed on [`OpenDoc::edit_epoch`] **after** the edit, exactly as
[`fill_disclosure`] is: any later edit — including an undo — moves the
document past it and the sentence disappears with no code remembering to
clear it. One edit bumps the epoch once and records at most one kind of
disclosure, so the fill line and this one can never both be live for the
same revision; see
[`crate::app::actions::last_edit_disclosure`]'s section.

**It does not make the bar taller** — R128, asserted by
[`tests::the_bar_is_exactly_as_tall_open_as_closed`].

### `fn catching_up`

`OPERATOR_REQUESTS.md` **O63**, and the piece that makes the request's own
words — *"live preview for everything we do"* — true rather than
aspirational.

# Why this is the general answer and the drawn preview is not

`canvas::shapes` draws a real preview, exactly, at pointer speed — and only
where the shell holds the geometry: a path being moved, resized, rotated or
node-edited. That is a large share of canvas work and **none** of the rest of
the program. There is no shape to slide when the operator changes a fill
colour, presses Bold, deletes a run of text, marks a redaction or rotates a
page.

For those the shell cannot draw the answer, and it cannot get one from the
renderer either: `BENCHMARK.md` measures a **two-pixel** region render at
691 ms on the operator's own drawing, because ~99 % of render cost is
content-stream interpretation rather than fill. There is no arrangement of
the existing renderer that produces a correct picture inside a second.

⇒ So the honest general answer is not a worse picture. It is **saying that
the picture is not the answer yet** — the third of the three options the
operator chose between, and the only one with no failure mode. It applies to
every edit in the program, including the ones a drawn preview will never
reach.

# It is a STATE, not an event, and that changes two things

Every other line in this file is keyed on [`OpenDoc::edit_epoch`] and
retires when the document moves past it. This one is live for as long as its
condition holds and stops the moment the raster lands — so it needs no
retirement rule at all, and it can appear for one edit and not the next
depending only on how hard the page was to draw.

And it is **silent under 400 ms** ([`OpenDoc::page_is_catching_up`]),
because the picture is behind after every edit and a line that flashed on
each one would be noise that costs every other sentence this bar carries.

**It does not make the bar taller** — R128, the same constraint every line
here is under and for the same reason: it arrives without the operator
asking for anything, and a bar that grew on its own would re-fit the page at
the moment a gesture completed.

### `fn recovered_disclosure`

# Why it is in the status bar as well as in Properties

Operator ruling, 2026-08-26: *"disclose it."*

Properties already carries the detail — how many objects were recovered, how
many were defined more than once, how many needed repairing. But **a
disclosure the operator has to go looking for is half a disclosure**, and
this is the one fact that changes how much they should trust what is on
screen. A rebuilt index is a *best reading of damaged bytes*: where an object
was defined twice pdfcer had to pick one, and on a drawing a wrong pick is a
line in the wrong place on a page that renders perfectly.

# How it avoids being the nagging the old shell was criticised for

1. **Off-canvas.** A line in the status bar, never a badge on the page. The
   document is not in doubt as *drawn*; what is in doubt is how it was
   *assembled*, and marking the page would be a second rendering path for
   content that is fine — decision 059's whole subject.
2. **It only appears for a file that was actually rebuilt**, which is rare. A
   healthy document shows nothing; verified by opening one.
3. **It states the fact and stops.** No icon, no colour alarm, no modal at
   open. One sentence, and the operator decides whether it matters to the job
   in front of them.

The counters stay in Properties. The status bar answers *"is there
something I should know?"*; the panel answers *"what exactly?"* — and a line
long enough to carry three numbers would push the zoom and page controls off
a narrow window.

### `fn load_anomalies_disclosure`

# What changed under the shell, and why silence was no longer an option

Until `Pass 283.0` a PDF whose catalog named `/PageMode` twice with two
different values was **refused whole**. The operator hit it on a 46 KB
drawing that opens in Acrobat, and his ruling is the reason the loader now
opens it:

> *"acrobat just picks one — but what if it is the wrong one? … We should be
> making pdfcer so that it opens pdfs that have errors, and have a way that
> it manages those errors such that they aren't fatal, and if the user can
> intervene in a decision that should always be an option along with them not
> having to intervene."*

The half of that sentence a shell can get wrong is the *last* clause. A
loader that quietly picks one of two values and says nothing has made the
file open — and has also made pdfcer's choice invisible, which is the exact
shape rule 4 forbids: **an inference the operator cannot see still owes them
a report.** Before this line, a file with a doubled key opened, looked
perfect, and disclosed nothing anywhere in the program.

# Why this is not the recovered-index line with different words

[`recovered_disclosure`] fires when the stored cross-reference table could
not be parsed and pdfcer rebuilt the index by scanning. This fires when an
**object** contradicted itself, which happens on files whose index is
perfect — the operator's file among them. The two are disjoint in both
directions and can be live at once; see [`super::anomalies`]' header for the
table. Folding them into one line would mean either claiming the index was
damaged when it was not, or going quiet on the case that motivated the whole
Pass.

# It is a DISCLOSURE, not a prompt — and never a modal

The engine's own notice puts the constraint in the imperative: *"The document
is live and usable the instant it opens; what pdfcer guessed goes in a status
line or a panel, off-canvas, and never gates the open. Do **not** build a
modal in front of it. That is the shape he rejected by name."* So:

1. **Off-canvas** (R8b, rule 4 as narrowed by decision 059). A line in the
   bar, never a badge, tint or outline on the page. The page as *drawn* is
   not in doubt; what is in doubt is which of two values the file offered was
   used, and marking the drawing would be a second rendering path over
   content that is fine.
2. **Never blocking.** Nothing here is asked, so nothing waits for an answer.
3. **It only appears for a file that actually contained a contradiction.** A
   sound file shows nothing — no all-clear, no placeholder (R9). See
   [`crate::text::anomalies`]' header for why the true "opened cleanly"
   sentence is deliberately not written.

# Its lifetime is the document's, and it carries no epoch key

[`fill_disclosure`] and [`edit_disclosure`] retire on the next edit because
they describe something the operator just did. This describes **what the file
was before they touched it**, which stays true through every edit, undo and
save — so keying it on [`OpenDoc::edit_epoch`] would un-tell the operator the
first time he nudged a line. It reads
[`pdfcer_core::document::Document::load_anomalies`] live from the open
document instead, exactly as [`recovered_disclosure`] reads `recovery()`:
there is no cached state, so there is nothing to clear and no way for one
file's anomalies to be shown against another's.

The census only, never the detail. Which object and which two values is
Document properties' job; a line long enough to carry a `/PageMode` pair
would push the zoom and page controls off a narrow window, and the shared
[`disclosure_line`] gives this all four R128 defences — bounded width, fixed
row height, truncation rather than wrapping, whole sentence on hover.

### `fn blend_space_disclosure`

# What the operator sees without this, and why it reads as a bug

Reported 2026-08-26: *"seems I get different results depending on Zoom
level. The [shading] boxes … on zoom out the colors between our
rendering and the references don't match, but they do when I am zoomed in.
up to 474% they are mismatched, but at 579% they match."*

Measured the same day, and his bracket contains the answer exactly.
`pdfcer-render` composites a page with transparency in a **subtractive CMYK
buffer** — the correct space for it — and that buffer has a documented
ceiling of 256 MiB at 20 B/px, i.e. **13,421,772 pixels**. Past it the
renderer falls back to compositing in sRGB and counts that it did
(`cmyk_buffer_refused`, `blends_in_wrong_space`).

On an A4 page that ceiling is crossed at **zoom 534 %**, dead centre of the
474–579 % band he bracketed. Either side of it the same page renders
different colours: measured on the conformance suite's composite page, up to
**16 levels out of
255** in the transparency patches, by box-averaging every pixel of both
renders into a common grid so that resampling could not masquerade as the
effect.

# Why this is a disclosure and not just a fix


`render::strategy::for_page` now asks the pixel question as well as the
edge one, by calling the engine's own `pdfcer_render::will_composite_in_cmyk`
rather than by copying a byte figure into this crate. The band is closed
for every page that has been learned to ask for ink.

The line remains because the budget is an **operator setting**
(`Settings::max_cmyk_buffer_bytes`): set it below what a page needs and the
engine still falls back, correctly and silently. Then the operator is
**told**, which is rule 4's surviving half doing exactly its job: this is an inference the operator cannot see — a screenshot
of the page says nothing about which space it was composited in — so it owes
an off-canvas report. Nothing is marked on the canvas.

It names **zooming out** as the remedy, because that is the one that
works, is instant, and is the opposite of what an operator chasing a colour
difference would try.
