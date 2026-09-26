# `canvas::textedit::pin` — naming the exact show operator, and the exact
buffer it lives in

## What a pin is, and why nothing that edits text may go without one

`pdfcer-core`'s text verbs — [`EditSession::edit_text`] and
[`EditSession::format_text`] — locate their operand two ways. Given only a
search string they find the **first** show operator on the page whose
decoded text matches. Given a `pinned_span` they find the one whose byte
span in the decoded content buffer is exactly that.

The difference is not an optimisation. On a title-block sheet with two runs
reading `REV A`, the unpinned form edits the wrong one, silently, with no
error anywhere. This operator's own benchmark drawing carries **3,007
single-character show operators** in one page stream, so "the first match is
the one the operator meant" is false on the documents this program exists
for.

## Why this is its own module rather than a private detail of the caret

It was a private detail of the caret until 2026-08-27, inline in
[`super::plan`], and that was correct while exactly one thing edited text.
`format_text` is the second: restyling an existing run takes **the same
`pinned_span` and the same `EditTarget`** as replacing its text — the engine
shaped the two verbs that way deliberately, *"so a shell that has decided
which stream a caret is in does not have to translate that decision between
two verbs."*

A second copy of that decision is the thing to avoid. The `EditTarget` arm
below is nine lines of code and sixty of argument, and the argument is what
makes it right; a paraphrase of it beside the restyle verb would compile,
would look correct, and would drift.

## The extraction here is NOT the shared page-text cache

`crate::app::cache`'s extraction runs with `ExtractOptions::default()`, and
`capture_provenance` **defaults to off** — the engine's own words:
*"`None` unless the extraction set `ExtractOptions::capture_provenance`;
this keeps the default Pass 4 output byte-for-byte unchanged."*

With it off, `provenance()` answers `None` for every glyph, and a caller
built on the shared cache would get **no pin at all** while every line of it
kept compiling. That is this project's canonical failure shape: a correct
decision function wired to a value that is always the same.

Widening the shared cache is the other option and is worse. Every consumer
of `page_text()` — Find, both copy verbs, the text sweep — would then pay
for provenance on every page, and extraction is the expensive thing this
shell does (392 ms on the benchmark sheet). Paying it **once per edit**, in
[`resolve`], is the whole cost, and an edit is already an operation that
saves and re-rasters.

The run index is shared between the two extractions, which is safe and is
worth stating: `capture_provenance` populates a field and changes no
segmentation, so `runs[i]` names the same run under both options.

## Item notes

### `fn operator_is_inside`

The cheap half of the join ([`engine_places_in`] is the authority), kept
because it bounds the engine call to candidates. Two conditions, and dropping either one is a defect
with no symptom on the page it was written against:

1. **The buffer must be the page's own.** A byte offset is meaningless
   without the buffer it indexes, and a page that paints a form XObject has
   two buffers whose offsets overlap freely. [`target_of`]'s doc comment
   carries the same argument for the same reason, and names the case: on the
   operator's benchmark sheet the page stream holds 3,007 single-character
   show operators, so *"an arbitrary offset happens to name something in the
   wrong buffer"* is a dense field of near-misses rather than a theoretical
   collision.
2. **Containment, not overlap.** A show operator is wholly inside one text
   object or wholly outside it; a partial overlap would mean the
   decomposition and the extraction disagree about where an operator ends,
   which is a fault worth declining on rather than rounding into a hit.

### `struct Pinned`

The three fields travel together because they are **one measurement**. The
span alone is the defect this shell shipped first: it pinned the offset and
discarded the stream, and the engine correctly reported *"text not found"*
about text that was plainly there.

### `fn of_run`

`None` when the extraction did not capture provenance, or when the run has
no glyphs. Both mean the same thing to a caller — *this run cannot be
pinned* — and both must be treated as a refusal rather than as permission to
fall back to an unpinned request, for the reason the module header gives.

### `fn spans_one_operator`

## Why this question is worth its own function

Because the answer decides whether the run can be addressed as a *whole
operator* — `EditRequest::whole_operator`, `Pass 152.0` — and getting it
wrong is the difference between an edit that refuses and an edit that
duplicates text on the page.

[`of_run`] pins on **glyph 0's** operator. The engine's own measurement is
that **13% of runs over its corpus carry glyphs from more than one show
operator**, and on those runs:

| request | what happens |
|---|---|
| `find` = the run's text, pinned | `NoMatch` — the pinned operator holds only part of it |
| whole-operator (empty find), pinned | the pinned operator's text is replaced **with the whole replacement**, and the run's other operators keep their glyphs |

⇒ The second is worse. A refusal costs an operator a puzzled moment; the
other writes `Rev BEV A` onto a drawing and reports success. So the
whole-operator form is taken **only** when this answers `true`, and the
find-based form — with its clean refusal — is what a split run keeps.

It walks the glyphs rather than trusting a count, because
[`EditableTextModel::provenance`] answers `None` one past the end and that
is the same termination a length would give with one fewer thing to keep in
step.

`false` when there is no provenance at all. A caller with no pin is not
entitled to the whole-operator form in the first place — the engine refuses
an empty `find` without a pin, by name — and answering `true` here would
build a request it would then reject.

### `fn resolve`

The convenience form for a caller that does not already hold a model — the
restyle verbs, which start from a selection rather than from a caret. A
caller that has just recognised a model should use [`of_run`] and not pay
for a second extraction.

`None` when the page is absent, when the extraction fails, or when
[`of_run`] answers `None`.

### `struct RunStyle`

# Why this is separate from [`Pinned`] and returned beside it

[`Pinned`] is a *locator*: it names an operand, and every field on it is
consumed by the engine. This is a *reading*: every field on it is consumed
by a human. Merging them would mean the restyle verb carrying three fields
it never looks at, and — the part that matters — would make it possible to
pass a stale reading into an edit by passing the struct that also carries
the pin.

They come back from one call because they come from one `GlyphProvenance`,
and the extraction that produces it costs 392 ms on this operator's
benchmark sheet. Two calls would be two extractions for one question.

### `fn operators`

The form `crate::app::actions::textstyle` wants: a restyle acts on operators
and a selection names runs, and this is the hop between them. See
[`operators_in_run`] for why the two are not the same thing.

### `fn font_preflight`

The face combo was built from `fontinfo::FontInventory`, filtered to the
records naming this page, showing each `/BaseFont` with its §9.6.4 subset
tag stripped. The engine's own summary of that arrangement: *"a list built
from the first key is a superset of the second that is usually right, and
when it is wrong the operator finds out by pressing a button and getting a
refusal."*

Two failures, and the second is much worse than the first:

**1. Entries that cannot work.** `fontinfo` is keyed on the font
**dictionary**; `set_font` matches on `/BaseFont` and then asks whether the
face can encode *this run's characters*. A face that cannot — `Times-Bold`
with no code for `o` — was offered, pressed, and refused. A control whose
entries may not work is what this project spends its time removing.

**2. The wrong twin, silently.** One page can carry **two font
dictionaries sharing one `/BaseFont`** — two subsets of one face — which the
survey behind the Fonts panel found in **87 % of embedding files**. A name
match reaches exactly one of them, arbitrarily, and the operator is given no
hint that a choice was made on their behalf. That is not a refusal an
operator can see; it is the wrong font, applied.

`FontResourceEntry::selector` is the fix for both: it is *the string to pass
to `set_font` to reach THIS resource* — normally the stripped `/BaseFont`,
and the **resource key** instead when the page carries twins, with
`base_font_ambiguous` set so a chooser can say so.

# Why it takes the run and not just the page

Because acceptance is per-run. The same face is accepted for one line of a
page and refused for another, depending on which characters each contains —
so a page-scoped list would be back to being a superset. The `find` and the
pinned span are the same operands `format_text` takes, which is what makes
the preview and the commit incapable of disagreeing (the engine moved the
four conditions into one `accept_font_target` for exactly that reason, R221).

## Cost, and why this is not called per frame

It is `&self` and side-effect-free, and it runs one extraction plus one
acceptance test per page `/Font` resource. The callers hold it behind the
same `(page, run, epoch)` stamp their style read-back uses, so it is paid
once per selection change rather than sixty times a second.


The first version of this function called [`inspect`] itself, which reads
well and is a **doubling of the most expensive thing this shell does**: its
only caller is the properties draft's `sync`, which had just run `inspect`
to get the face, size and colour. Two extractions with provenance capture on
is **784 ms** on the operator's benchmark sheet where one is 392 — paid on
every selection change, to answer two halves of one question.


`None` coverage-tests the run's own characters, which is the right question
for a Properties panel describing a run as it stands. `Some(text)` tests the
characters in `text` instead, which is the right question — the **only**
right question — for a surface offering a way past a character the current
face refused.

⚠ **It is a required argument rather than an `Option` with a default, and
that is deliberate.** Adding a parameter that keeps the old behaviour when
omitted would have let both existing call sites silently decline the new
capability: the compiler goes quiet, the tests stay green, and the surface
that needed it most goes on asking the wrong question. Two call sites, two
forced decisions, one compile error each.

### `fn inspect`

The form a properties panel wants. [`resolve`] is this with the reading
dropped, kept as its own entry point so an edit path cannot accidentally
hold a stale style struct alongside a fresh pin.

### `fn operators_in_run`

# Why a run is not an operator, which is the thing this function exists to say

It is tempting — and this shell did it for one afternoon — to treat a
`TextRun` as a show operator: pin the first glyph's operator, pass the run's
text as `find`, and restyle. It works on most runs and fails on real
drawings, because `layout` closes a run on *geometry* and a producer closes a
show operator on *whatever its writer felt like*. A title-block cell reading
`FINISH` came back as one run spanning several `Tj`s, so the pin named the
first and the `find` named all of them, and `format_text` refused with *"text
to format ("FINISH ") was not found in an editable run on the page"* — on a
page where the very same string is found instantly by an UNpinned search.

That refusal is correct and is not a bug: `find` selects a contiguous code
range **within one string element**, and the shell was asking for a range
that spans several.

⇒ **The operator is the unit of a restyle**, so the operator is what this
answers with. A run of three `Tj`s is three entries, three `format_text`
calls and three undo entries, and every one of them restyles exactly what it
names.

# The `find` per entry

The glyphs that share that operator's span, sliced out of the run's text by
their own `text_start`/`text_len`. Every byte comes from a glyph, so no
**derived** character — a space the extraction synthesised from a `TJ` offset
— can get in, which is the second way the naive version failed.

# Order

Content order, ascending. A caller wanting the descending order that keeps
byte offsets stable across edits reverses it, and
`crate::app::actions::textstyle` does, with the argument.

### `struct ObjectText`

The reading behind the O89 object route. One value, from one extraction,
because the two questions have one answer: the walk that finds the runs is
the walk that reads their fills, and provenance capture is the expensive
thing this shell does (**392 ms** on the operator's benchmark sheet).

### `enum RunFill`

Three states, and collapsing the first two is a real defect with no
symptom on the page it is written against.

`GlyphProvenance::fill_color` is an `Option`, and its `None` means *"no
colour operator was in force, so §8.6.8's default — black DeviceGray 0 —
applies"*. A run that produced **no provenance at all** — a derived word
space, a line break, an `/ActualText` replacement — also yields `None` from
the same expression, and it means the opposite: *there is nothing here to
have a colour*.

⇒ Written as one `Option`, an object whose first run is explicitly red and
whose second run carries no colour operator (and is therefore **black**)
would read as *one colour, red* — and a control that opened on red would
propose flattening the black run to it, silently. That is exactly the
flattening `OPERATOR_REQUESTS.md` O89 refused to ship for multi-object
selections, arriving through the other door.

### `fn object_text`

`None` — and every one of these is a legitimate, silent, *not an error*
answer, which is why they are one return value rather than a `Result`:

* the page is not open, or the object model could not be built;
* `object` is not an index into this page's own paint order;
* the object is **not text** (a path, an image, a form);
* the extraction produced no run whose show operator lies inside the
  object's span — a text object whose every string failed to decode, which
  is a real state on a drawing with a broken font.

# The cost, stated because a caller must not put this in a paint loop

One `extract_page_view` with `capture_provenance` **on**, plus one block
recognition. That is 392 ms on the operator's benchmark sheet. Every caller
holds it behind a `(page, object, edit epoch)` stamp, exactly as
[`crate::panels::properties::text::TextStyleDraft`] holds [`inspect`], and
for the same measured reason.
