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
