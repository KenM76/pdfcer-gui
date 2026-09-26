# `panels::properties::markup` — restyling a markup that is already on the
page

The panel half of `FEATURES.md`'s Phase 1 row *"Format tab contents —
colour, width, style, opacity for a **placed** markup"*: the surface that
reaches `pdfcer-core`'s `set_markup_style`.

## Why the PANEL and not the Format tab

`RIBBON_IA.md` §5.8 settles it, in the operator's own words:

> The division of labour: the **tab** carries what a user changes *while
> working* — colour, width, style, align, delete. The **panel** carries
> everything, including the read-only facts … The panel is also where the
> **editable geometry** lives.

So the panel is where the complete set goes, and it is also the cheaper
surface by a wide margin: a ribbon band cannot hold a colour picker or a
slider without a new `Item::custom` kind and a renderer for it, which is
shell work in a crate that must never learn what a PDF is (R7). The tab's
slice is a smaller job that reads the same actions.

## Every control is `None` unless the operator touched it

`MarkupStyle`'s own doc comment is the rule and the reason:

> Every field is `None` by default … That shape is deliberate: a Format tab
> whose colour picker also had to restate the current width would overwrite
> whatever the operator had set from the other control.

So this section raises **one action per control that changed**, carrying one
field, and never a whole style struct assembled from what the widgets happen
to show. The failure that prevents is specific: two controls drawn from the
same annotation, one of them stale by a frame, and a colour change that
silently reverts a width the operator set a moment earlier.

## Where the style verb can reach, and how that is decided

`set_markup_style` begins by calling `annot_author::spec_from_dict`, whose
`match` reads a fixed set of `/Subtype`s — `Square`, `Circle`, `Line`,
`Ink`, `Polygon`, `PolyLine`, `Highlight`, `Underline`, `StrikeOut`,
`Squiggly` — with every other name answering
`SpecReadError::UnsupportedSubtype`. `AnnotKind::Markup` is wider than that:
its own doc covers *"a shape, a note, a stamp, a text markup"*. So the kind
check alone would draw a live colour swatch and a live opacity spinner over
a `/Text`, a `/FreeText` or a `/Stamp`, and every press would be refused
with `EditError::MarkupSpec` — the *visible control, silently inert* class
this project forbids by name.

### Reachability is asked of `spec_from_dict`, never of a subtype list

A `matches!(subtype, "Square" | "Circle" | …)` beside the kind check goes
stale the day the engine learns an eleventh subtype, and it goes stale
**silently**, in the direction that withholds a control that would have
worked. [`Current::read`] already calls `spec_from_dict`, so
[`Current::reach`] carries that verdict forward: this section and the verbs
answer the same question through the same function, and an engine that grows
a subtype grows this panel with it and no shell change.

### Two verbs, one routing decision

A `/Text`'s icon and colour and a `/Stamp`'s colour are changeable, but
through a **second engine verb with a second reader and a second style
struct** — `set_text_annot_style`. [`textannot`] is that route and its
header carries the account, including why `/FreeText` is refused and on what
grounds.

⇒ Nothing here goes through `set_markup_style` on their behalf. The two are
separated by [`Reach`], an enum whose arms the compiler makes exhaustive,
because a routing decision that can be got wrong silently is exactly the
defect this seam exists to prevent.

### The refusal SAYS something

R9 makes an unavailable capability render nothing. It does not make the
panel go silent: the heading and the subtype line still draw, because
something *is* selected, and a heading over an empty space reads as a bug.
[`t::markup_not_restylable`] names what is still possible — move, resize,
delete, edit the note — and its doc comment records the engine verb each of
those four claims is checked against.

## Two properties this panel offers that the author-time path does not

- **Fill (`/IC`) is offered on restyle and never at author time.**
  `canvas::markup::spec` writes `interior: None` on purpose — *"a filled
  comment shape hides the drawing it is a comment about, which on a CAD
  sheet is the whole content under it"* — and this module does not go near
  that. A default and a prohibition are different acts, and letting one
  stand in for the other is how a sensible default becomes a capability
  nobody can reach. Acrobat's shape tools all offer fill and all default it
  to none; that is the shape matched here.

- **Line endings (`/LE`) are offered on a `/Line`.** A `/Line` with
  `/LE [/None /None]` is the same `/Subtype`, with the same geometry,
  reached by the same verb, and §12.5.6.7 treats its endings as *style* in
  exactly the way `/C` and `/BS` `/W` are style — which is why
  `MarkupStyle::endings` sits beside them in one struct rather than in a
  reshape. A leader with a head at one end only is among the commonest
  annotations on a drawing sheet.

- **A ce dimension is NOT restyled here.** [`super::dimension`] owns those,
  through `set_dimension_style` — a different verb with a different model,
  and `AnnotKind` carries the distinction **in the type** so this section's
  guard is a `match` the compiler checks. Restyling a ce dimension as
  ordinary markup regenerates it as a bare line with its label and witness
  lines gone. Rule 15 in one sentence: never write a bare dimension.

## WHICH subtype takes WHICH property is the ENGINE's question

This shell holds no copy of that list:

> *"That list is the engine's to know. The first subtype that gains or loses
> a border is the day our copy is wrong and nothing tells us."*

`pdfcer-core`'s `edit::MarkupStyleSupport::for_subtype` answers it, with
`takes_border`, `takes_interior` and `takes_endings`. [`Current::support`]
holds the answer and the rows read it. **A comment saying a list was checked
against the engine source is a comment that ages; a call cannot.**

⚠ **What this does NOT cover.** *"What IS this mark's width?"* is read off
the `MarkupSpec` arm, because only `MarkupSpec::Square` has a `border_width`
field and the engine publishes no API that would answer it. A **value** read
and a **capability** question are different questions with different owners;
`canvas::annotnodes`' header draws the same line for painting.

### The refusal, which is the other half of the same contract

`EditError::StylePropertyNotApplicable { id, subtype, property }` is raised
by `set_markup_style` **before** anything is regenerated. So the predicate
above shapes this panel and the refusal catches a shell that drifted anyway
— belt and braces, and the reason [`Current::reach`]'s neighbours are not
enough on their own. It reaches the operator through the channel every
engine refusal uses, `app::actions::funnel::vector_edit`'s `Err` arm: the
decline sentence on screen, the engine's own words into `PDFCER_DIAG`.
Nothing here builds a second route, because `check-ui-strings.sh`'s
exclusion 3 forbids one in as many words.

## Item notes

### `const MIN_WIDTH_PT`

Zero is excluded and it is a decision rather than an oversight: §8.4.3.2
gives `0` a defined meaning — *the thinnest line the device can render* —
which on a 600 dpi plot is a hairline and on screen at 25 % is invisible.
An operator who wants a mark they cannot see has the visibility toggle;
what they must not get is a mark whose weight depends on the output device
without being told.

### `const MAX_WIDTH_PT`

Beyond about twelve points a border stops reading as a border and starts
reading as a filled shape, which is the thing this shell deliberately does
not author. The same ceiling `canvas::markup::pen` uses, for the same
reason and from the same argument.

### `const DASH_WIDTH`

Wider than the Format band's `DASH_WIDTH` (88), and deliberately so: this
is the surface with room for the whole of
[`crate::text::markup::line_style_foreign`] — *"Dashed (the file's own
pattern)"* — which the band clips. §5.8's division of labour is that the tab
carries what an operator changes while working and the panel carries
everything; a reading that needs a sentence belongs on the second.

### `fn markup_rows`

Extracted rather than left inline purely so the routing `match` above
reads as four one-line arms. A `match` whose first arm is forty lines and
whose others are three is a `match` a reader stops seeing as a routing
decision, which is the one thing this one has to remain.

### `struct Current`

Five terms — colour, fill, width, opacity and the two line endings — plus a
sixth field, [`Self::reach`], which is not a term at all but the answer to
WHICH VERB the other five are reachable through.

# Why it is read through `spec_from_dict` and not from `annot::Annotation`

`pdfcer_core::annot::Annotation` is the **reader's** view — id, subtype,
rect, flags, `/CA`, appearance — and it deliberately carries no `/C` and no
`/BS /W`, because nothing that renders a page needs them: the picture comes
from the baked `/AP`.

`annot_author::spec_from_dict` is the **author's** view, and it exists for
exactly this: *"so an existing annotation can be restyled by regenerating
its appearance from its own declared geometry"*. Reading through it means
the values these controls show are the values `set_markup_style` will read
when it plans — one derivation, not two.

Its refusals are `None` here rather than an error, and that is honest
rather than lax. `SpecReadError`'s own doc says every variant is *"a refusal
to guess"* — an unsupported `/Subtype`, or geometry that is missing or is
not something pdfcer models.

⚠ **A refusal here means the mark cannot be given a colour either**, not
merely that the one it has cannot be shown. `set_markup_style` opens by
calling this same function and propagating its error with `?`. So a swatch
drawn over a refused mark would not be uninformative — it would be unable to
commit. See [`Self::reach`] and the module header.
⚠ **`Clone`, not `Copy`** — [`Self::reach`] carries a
[`textannot::Reading`] on its `TextAnnot` arm, which carries a `StickyIcon`,
which has an owning `Other(Vec<u8>)` variant. The frame reads one of these
and hands it out by reference.

### `struct Swatch`

The second field is the whole reason this is a struct rather than an
`Option<[u8; 3]>`. `/C` and `/IC` may be grey, RGB **or CMYK** (§12.5.2), and
the three are not equally showable: grey is the same ink as its equal-
component RGB and converts losslessly in both directions, where CMYK does
not. Carrying *whether a conversion happened* beside the converted value is
what lets [`section`] disclose it rather than the operator discovering it
from a changed file. See [`t::markup_colour_narrowed`] for the argument that
replaced the old refuse-to-show behaviour.

### `impl Default`

`MarkupStyleSupport` is `#[non_exhaustive]` and has no `Default`, so the
derive had to go — and that is worth keeping rather than working around.
The honest default for *"the dictionary could not be read"* is **not** a
hand-written all-`false` literal; it is what the engine answers for a
subtype it does not recognise, which `for_subtype`'s own doc calls *"the
conservative direction: a caller is told a property is unavailable rather
than being told one is available on a shape pdfcer cannot restyle at all."*
Asking for it removes the last place a `false` about a subtype could have
been written by hand in this module.

### `fn from_spec`

# Why it is split out from [`Self::read`]

Because it is the part with the decisions in it, and it is the part a
test can reach. `read` needs an `OpenDoc`, a session and a real
annotation dictionary; `from_spec` needs a `MarkupSpec`, which is a value
a test constructs in one expression. The tests at the foot of this module
assert the reachability verdict, the interior slot and the endings slot
through this function, and each of them was falsified by breaking the arm
it guards and watching it go red.

`None` means the read refused — an unsupported `/Subtype`, or geometry
pdfcer does not model. Both produce the same answer here for the same
reason: `set_markup_style` would refuse the same call.

### `fn offers_fill`

Purely the engine's answer: *no fill* is a legitimate current state and
[`fill_row`] shows it as a default swatch with [`t::markup_fill_none`]
beside it, so there is no value whose absence should withhold the row.

### `fn offers_width`

Two terms meaning two different things. `takes_border` false is *this
subtype has no border* — the engine's, and permanent. A `None` width
under a `takes_border` that is true is *this build cannot read this
arm's width*, which `MarkupSpec` being `#[non_exhaustive]` makes
reachable; a spinner with no value to show is what R9 and
`app::markupband::placeholder` both forbid.

### `fn offers_dash`

**Purely the engine's answer, with no second term** — unlike
[`Self::offers_width`], which also asks whether a width was read. The
asymmetry is real: a width has to be *shown* in a spinner, so a mark
whose width this build could not read has nothing to put in one; a line
style always has a value, because *solid* is a state rather than an
absence and [`crate::canvas::markup::linestyle::read`] is total — every
dictionary answers it, including one carrying no `/BS`.

⇒ So the only question left is the engine's *does this subtype have a
border?*, which is the same predicate `set_markup_style` guards
`style.dash` with. A row drawn here cannot produce that refusal.

### `fn swatch_of`

# Why CMYK gets a converted swatch rather than none

§12.5.2 lets `/C` be a 0-, 1-, 3- or 4-component array, so the tempting
answer is `None` for anything that is not RGB — on the grounds that showing
a CMYK mark's *converted* colour is a readback the operator never asked for.

**That answer is worse than the thing it avoids.** A CMYK mark is not rare
on a CAD sheet, where a plotter-bound producer writes process colour, and
`None` gives it a **default black swatch and no Clear**: the panel would
tell the operator their coloured mark has no colour, which is not a smaller
misstatement than an approximate one, and it would withhold Clear, the one
operation on a CMYK `/C` that loses nothing at all.

**And the feared round trip is not a thing this control can do.** egui's
colour button reports `changed()` only when the value actually moves, so
*pick it up and put it down unchanged* raises no action and writes no byte.

# ⚠ The narrowing, disclosed rather than hidden

A restyle raised from a swatch fed by this function writes
`Color::Rgb`, because `color_edit_button_srgb` produces sRGB and nothing
else. **On a mark whose `/C` or `/IC` was CMYK that NARROWS the colour
space** — four components in the file become three, and a colour-managed
consumer downstream will separate the result differently than the original
process values. The engine's own posture is that a narrowing conversion is
disclosed rather than performed quietly, so [`Swatch::narrowed`] carries the
fact up to [`section`] and [`t::markup_colour_narrowed`] is the sentence the
operator reads **before** they pick, not after.

Grey is **not** narrowing and is not flagged: `Gray(v)` and `Rgb(v, v, v)`
are the same ink, exactly, in both directions.

# The conversion itself

The naïve `1 - min(1, x + k)` per channel — the same one §8.6.4.4 states as
the default `DeviceCMYK` → `DeviceRGB` transform when no colour management is
in play. It is an approximation and this shell says so; it is not a place to
invent an ICC pipeline for a 16-pixel square.
