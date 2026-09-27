# `markuppen` — the colour and width the next markup is authored with

## What this closes

`RIBBON_IA.md` §5.5 specifies a **Style** group on the Markup tab —
*"Colour · Line width · Fill · Opacity"* — and marks it `partial G`,
*"colour only"*, describing the **old** shell. This shell had none of it:
`MarkupKind::rgb()` returned a hard-coded red, `PEN_WIDTH_PTS` was a
hard-coded `2.0`, and the manifest's `colour_swatch` item was declared and
never built, so the Style group rendered an empty caption.

§5.5's own note on it is the operator's complaint restated in advance:

> The `Style` group sets defaults for the next markup. … Both must exist;
> today only the first does, **which is why a placed markup feels final**.

## The seam was already named, and this took it

`MarkupKind::rgb`'s doc comment predicted this module almost exactly:

> The old shell carried `markup_color`/`markup_width` on the application
> and a swatch in its ribbon. This shell has neither … So the pen is a
> **default**, stated once, in the one place that builds a spec, and the
> seam for a real pen control is exactly this function: **give it a colour
> and a width from the document's markup state and nothing else in the
> module changes.**

That is what happened: `spec` and `action` gained a `Pen` parameter and
nothing else in `canvas::markup` moved. The prediction was right down to
the shape of the change, which is worth recording — a doc comment that
names its own seam is the cheapest refactoring aid this project has.

## Why the pen lives on the application and not on the document

A pen is a **tool setting**, not document content. An operator who picks
green expects the next rectangle to be green in *whatever* file they draw
it in, exactly as a pencil does not change colour when you turn the page.
Putting it on `OpenDoc` would reset it on every open, which is the
behaviour of a program that has forgotten what you told it.

It is deliberately **not persisted** to the settings file, and that is a
narrower statement than it sounds. `pdfcer_core::settings` is for choices the
*standard* leaves open — its window says so in its own first paragraph —
and a pen colour is not an ambiguity, it is a preference. Persisting it
belongs with the ribbon layout and the keymap, under the same `userdata/`
roof and in their own file, which is `SHELL_FRAMEWORK.md`'s subject rather
than this one's.

## EIGHT PENS, NOT TWO — and the argument that said otherwise is kept
## here, superseded rather than deleted


> **Two pens, not one, and it is not a per-kind palette.** … What this is
> **not** is a colour per markup kind. `MarkupKind::rgb`'s own note argued
> that down when three kinds were added: they are comment linework, they have
> to be seen against a drawing that is already black on white, and *"a
> per-kind palette would be a style decision made in code where the Style
> group is the surface that owns it."* One pen for every geometric kind is
> the same answer, now made choosable.

### Why that was a good argument

It is worth saying plainly, because the correction is only useful if the
thing being corrected was reasonable. The argument had two halves and both
were sound at the time:

1. **A colour chosen in code is a style decision made in the wrong place.**
   True, and still true. A shell that hard-codes a green underline because
   somebody liked green has put a preference in a source file where no
   operator can reach it.
2. **Comment linework has one job — to be seen over black-on-white CAD.**
   Also true. Nothing about a per-kind palette makes an underline more
   legible over a drawing than the pen colour would.

### Why it is superseded anyway

Because it answered the wrong question. It asked *"can this shell justify
inventing eight colours?"* — and the answer to that is still no. The
operator's ask of 2026-09-06 asks something else:

> *"Also make sure you've used the same default colours and style look for
> these things as Adobe."*

⇒ The values are no longer this shell's to choose. [`super::palette`] reads
them out of **Acrobat's own tool-defaults store**, and Acrobat does not use
one colour for everything: its highlighter is orange, its underline is blue,
its strikeout is a light red that is not the shape red, and its sticky note
is violet. A single pen cannot express that table, so the table is the shape
the pen has to have. The style decision is not being made in code — it is
being **transcribed from the program the operator compares against**, which
is this project's standing tie-breaker for anything of this kind.

Half of the old argument survives intact and is worth keeping: the values
must still be the operator's to override, and every slot below is. What
changed is only where the *shipped* value comes from.

### The slots, and why they are named rather than an array

[`PenSlot`] has one variant per **key Acrobat keeps a separate default
under** — not per [`MarkupKind`] variant, and the difference matters in both
directions:

* The seven geometric kinds share `cSquare`/`cCircle`/`cLine`/… , all holding
  the identical red, so they share [`Pen::ink`]. Giving each its own field
  would be seven copies of one number and seven places for it to drift.
* Squiggly and Stamp hold that same red **under their own registry keys**, so
  they get their own slots even though the shipped values agree today. An
  operator who recolours the shape pen has not thereby asked for a recoloured
  squiggly — Acrobat's do not move together, and collapsing two slots that
  happen to agree is how a per-kind palette quietly becomes a single pen
  again.

[`Pen::colour_for`] is the total function from a kind to its slot's colour;
[`Pen::colour_of`] and [`Pen::set_colour`] are the slot-addressed pair the
swatch uses. Both matches are exhaustive, so a ninth slot fails to compile
rather than silently landing on the shape pen.

## Item notes

### `fn default`

# Every colour here is measured, and none of it is chosen


# THIS DELIBERATELY BREAKS THE STANDING "OMITS NOTHING" RULE, and
# the rule is not being ignored — it is being answered

[`Self::opacity`]'s doc comment states the rule this project applies when
a capability becomes choosable:

> **a build which omits nothing must behave as it did before the choice
> existed**, byte for byte.

That rule is about a *capability arriving*: an operator who never touches
a new control must not discover that the new control changed their output
anyway. It is a rule against **silent** change, and it is a good one.

⇒ It does not bind here, and the reason is that this change is not
silent — **it is the thing the operator asked for by name**:

> *"Also make sure you've used the same default colours and style look
> for these things as Adobe."*

A markup authored by this build is therefore a different colour from one
authored by the build before it. That is a deliberate, requested,
operator-visible change, and pretending otherwise by keeping the old
values *"for compatibility"* would be answering a request with a refusal
dressed as a principle. Saying so here, rather than quietly departing
from a rule written four lines above, is the whole reason this paragraph
exists.


# 2 pt is KEPT, and Adobe's number is not being ignored — there
# isn't one

The brief for this change said to weigh Acrobat's default line width
against this shell's 2 pt and decide, recording both sides. The weighing
found only one side, and the finding is the decision:

**There is no measurable Acrobat width to match.** `cAnnots` holds a
colour key, a fill key, a text key, an opacity key and an icon name for
every subtype, and **no width, thickness or border key at all** — the
whole `…\Adobe Acrobat\DC` tree was searched for `width`, `thick` and
`border` and the only hits were print-N-up and multimedia settings. So an
"Acrobat default of 1 pt" could only have come from memory or from a
web page, and this project's **claim-bearing copy** rule is explicit that
a plausible number from a marketplace convention is not a source. A line
width is written into `/BS /W` and reaches the operator's file; it is a
claim.

The two sides, since both were asked for:

| for adopting Adobe's | for keeping 2 pt |
|---|---|
| the operator asked for parity with Adobe, and asked for it about style | **the number is not sourced** — the ask was for Adobe's value, not for a guess at it |
| a thinner default is easier to thicken than a thick one is to find | the operator's own drawings are dense CAD exports whose linework is 0.25 pt, and *"a hairline vanishes among the drawing's own linework"* is his use case, argued here since the constant existed |
| | 2 pt is what every markup this shell has authored is drawn at, so an old and a new comment on one sheet match |

⇒ **2 pt wins**, on the first row of the right-hand column alone. If a
measurement of Acrobat's width later turns up — its Properties dialog
shows a `Thickness` field, so the number exists somewhere this search did
not reach — this decision should be revisited **with that measurement**,
not with a recollection of it.

### `fn rgb_of`

Alpha is dropped — see [`Pen::set_ink`]. The division is by `255.0` rather
than by `256.0`: the component range is *inclusive* of both ends, so `255`
must map to exactly `1.0` or a "pure red" chosen in the picker would be
written as `0.996` and round-trip to a slightly different swatch.

### `fn color32_of`

The inverse of [`rgb_of`], and **opaque**: the swatch shows the colour the
annotation will be, and an annotation has no alpha in `/C`. Rounding rather
than truncating, so the round trip through the picker is stable — truncation
would walk a colour down by one unit per visit.

### `fn every_slot_ships_at_the_acrobat_value_it_was_measured_from`

The successor to `the_default_pen_is_the_constants_it_replaced`, which
pinned `(0.85, 0.16, 0.16)` and `(1.0, 1.0, 0.0)` — this shell's own
invented red and yellow — under the *"a build that omits nothing behaves
as it did before"* rule. That test was **deleted deliberately**, not
renamed: it asserted the exact values the operator asked to have
replaced, so leaving it would have made the requested change fail the
suite. `Pen::default`'s own doc comment carries the argument for why the
rule does not bind here.

What replaces it is stronger, because it is checkable against something
outside this file: every slot must equal the [`super::palette`] constant
whose doc comment names the Acrobat registry key it was read from. A
hand-typed drift in either place fails here.

Falsified by changing `ink` to `palette::NOTE_PURPLE`: the assertion
fired naming `Shape`. Restored.

### `fn the_highlighter_is_acrobats_orange_and_not_the_old_yellow`

Stated as its own test because it is the single value most likely to be
"corrected" back to yellow by somebody who knows that PDF highlighters
are yellow. They are not, in the program the operator compares against:
`cHighlight\cstrokeColor` reads `1.0, 0.384308, 0.0`.

The assertion is written as *"not the yellow it used to be"* rather than
only as *"is the orange"*, so the failure message says what happened.

### `fn every_kind_takes_the_slot_it_is_documented_to_take`

The successor to `only_the_highlight_kind_uses_the_highlighter`, over
the whole `MarkupKind::ALL` list rather than a hand-written subset — so a
ninth kind is covered without anybody remembering to add it here.

What it asserts is now the *routing*, not the colour: eight
distinguishable values are planted, one per slot, so a kind that took the
wrong pen names itself. Planting real colours would let a wrong answer
pass whenever two slots happened to ship the same red — which three of
them do.

### `fn setting_one_slot_leaves_the_other_seven_alone`

This is the *"once they set a colour for a kind, it sticks for that
kind"* half of the operator's ask, and it is the property a collapsed
slot would silently lose: if `Squiggly` were folded into `Shape` because
they ship the same red, recolouring the shape pen would recolour every
squiggly on every future page and nothing would say so.

Falsified by making `set_colour`'s `Squiggly` arm write `self.ink`: the
assertion fired on the `Shape` slot while setting `Squiggly`. Restored.

### `fn the_three_text_annotation_kinds_do_not_share_a_pen`

The mapping `app::actions::apply` depends on. Before 2026-09-06 all
three took `pen.ink`, so a sticky note came out shape-red where
Acrobat's `cText` is violet — this is the assertion that would have
caught that, stated as *"three kinds, three slots"* rather than as three
hard-coded colours, because the colours may legitimately be edited and
the separation may not.

### `fn the_tolerance_follows_the_width`

The rule [`Pen::simplify_tolerance_pts`] carries at length: ε must be a
quarter of the stroke width, because that is half of the half-width and
therefore the bound that keeps a simplified centreline strictly inside
the stroke the operator drew.

It is asserted by **varying the width**, which is the only form of the
rule a stale constant cannot pass: `ink::SIMPLIFY_TOLERANCE_PTS` was
`PEN_WIDTH_PTS / 4.0` frozen at 2 pt, and it satisfied every test that
used the default pen while being wrong by 4× at the thin end.

⚠ **If a width ever goes per-slot, this test must gain the slot too.**
See the function's own note; the rule has been broken once already by a
session that had just read it.

### `fn planted`

Named rather than spelled out at three call sites, and built from the
slot's own *index* so it cannot fall out of step with [`PenSlot::ALL`]:
a ninth slot gets a ninth distinct value with no edit here.

### `fn a_colour_round_trips_through_the_swatch`

The property that makes the swatch usable rather than merely present.
A conversion that truncated would walk a colour down by one unit every
time the operator opened the picker and closed it without choosing —
a slow, silent drift with no event to attach a bug report to.

The endpoints are the ones that catch an off-by-one scale factor:
dividing by 256 rather than 255 makes pure white round-trip to 254.

### `fn an_impossible_component_clamps_instead_of_wrapping`

Not reachable from the picker, which cannot produce one — reachable from
a hand-edited file the day the pen is persisted, and from any future
loader. `as u8` on a NaN is 0, a silent black; the clamp states the
intent instead of inheriting that.

### `fn the_shipped_width_is_reachable_on_its_own_control`

# Why the range's own bounds are asserted in a `const` block

`MIN_WIDTH_PTS > 0.0` is a relationship between two literals, so an
ordinary `assert!` is a statement the compiler folds away and clippy
rightly refuses. It is still worth stating — **zero is legal PDF** and
means *"the thinnest line the device can draw"*, a width whose
appearance depends on the output device and therefore exactly what a
comment annotation must not be — so it is stated where a compile-time
claim belongs, and a future edit that lowered the floor to zero would
fail to build rather than fail to run.

The runtime half is the one that can actually change: the shipped
default is a value, and a default outside its own control's bounds
would be silently rewritten the first time anybody touched the swatch.

### `enum PenSlot`

# Why this exists rather than the swatch naming a field

Because there are now eight colours and one control. A swatch that wrote to
`pen.underline` by naming the field would need a second swatch for every
slot, and the slot a control edits is a *runtime* choice — which is the
definition of a value rather than a field name. [`Pen::colour_of`] and
[`Pen::set_colour`] take one of these and are total over it, so the control
is written once and a ninth slot is a compile error rather than a silent
fallback to the shape pen.

# The mapping to Acrobat's store

| slot | Acrobat key | shipped colour |
|---|---|---|
| [`Self::Shape`] | `cSquare`, `cCircle`, `cLine`, `cLine:LineArrow`, `cPolyLine`, `cPolygon`, `cPolygon:PolygonCloud`, `cInk` | [`palette::MARKUP_RED`] |
| [`Self::Highlighter`] | `cHighlight`, `cInk:InkHighlight` | [`palette::HIGHLIGHTER_ORANGE`] |
| [`Self::Underline`] | `cUnderline` | [`palette::UNDERLINE_BLUE`] |
| [`Self::StrikeOut`] | `cStrikeOut` | [`palette::STRIKEOUT_PINK`] |
| [`Self::Squiggly`] | `cSquiggly` | [`palette::MARKUP_RED`] |
| [`Self::Note`] | `cText` | [`palette::NOTE_PURPLE`] |
| [`Self::TextBox`] | `cFreeText` | [`palette::MARKUP_RED`] |
| [`Self::Stamp`] | `cStamp` | [`palette::MARKUP_RED`] |

See [`super::palette`]'s header for where those readings come from and for
the evidence that they are Adobe's factory values rather than this machine's
history.

### `const ALL`

Exists for the reason `MarkupKind::ALL` does: it is what lets a test
sweep every slot rather than a hand-written subset, so a ninth slot is
covered without anybody remembering to add it.

### `struct Pen`

`Copy`, and small enough that it is passed by value everywhere. That is
deliberate: a pen borrowed from the application while a gesture is being
committed would conflict with the `&mut self` the commit needs, and the
alternative — cloning at each call — would be the same bytes with a
question about whether the copy is stale.

⚠ **Keeping it `Copy` is what decided [`super::linestyle::LineStyle`]'s
shape.** The engine's `BorderDash` owns a `Vec<f64>` and would have taken
`Copy` away from every caller in `canvas::markup` and `app::actions`; see
that type's header.

### `const MIN_WIDTH_PTS`

**A quarter point, not zero.** Zero is a legal PDF border width and means
*"the thinnest line the device can draw"*, which on a 2400 dpi plotter is
invisible and on screen is one pixel — so it is a width whose appearance
depends on the output device, which is precisely the property a comment
annotation must not have. A quarter point is the thinnest value that means
the same thing everywhere.

### `const MIN_OPACITY`

A tenth, not zero. An annotation at `/CA 0` is **invisible** — it is in the
file, it is selectable, it prints as nothing, and nothing on screen says it
is there. A control whose bottom end authors an invisible mark is a control
whose bottom end is a defect report waiting to be filed, and the operator
would have no way to tell it from a markup that failed to author at all.

A tenth is faint enough to be a wash over dense linework and still visible
as a mark.

### `const MAX_WIDTH_PTS`

Twelve points is about a sixth of an inch — a marker rather than a pen, and
already heavy enough to obscure the drawing underneath, which is the failure
a comment on an engineering sheet has to avoid. Beyond it the annotation
stops being linework and starts being a fill, and `MarkupSpec` has a
separate concept for that which this shell deliberately does not offer.

### `fn simplify_tolerance_pts`

# This function exists because the constant it replaced went stale
the day the pen became a control

`ink::SIMPLIFY_TOLERANCE_PTS` was `PEN_WIDTH_PTS / 4.0`, a `const`
derived from the pen's **shipped** width of 2 pt. That was exactly
right while the width was a constant, and `canvas::markup::ink`'s §3.2
wrote down what to do when it stopped being one:

> it is a *rule* — **if the pen ever becomes an operator control, the
> tolerance follows it** rather than being re-tuned by eye.


# Why the stale value was wrong in a way an operator would see

The derivation is not arbitrary. Ramer–Douglas–Peucker guarantees that
no removed point lay further than ε from the line replacing it, so ε
bounds how far the **drawn centreline** can move. Setting ε to half of
the stroke's *half*-width means the simplified centreline stays strictly
inside the body of the stroke the raw trail would have drawn: **no pixel
of the mark can move outside the mark.** That is the strongest statement
available about a lossy simplification, and it is the whole reason the
number is defensible rather than tuned.

A fixed 0.5 pt breaks it in one direction and merely wastes work in the
other:

| pen width | half-width | fixed ε = 0.5 | verdict |
|---:|---:|---:|---|
| 0.25 pt | 0.125 pt | **4× the half-width** | ⛔ the centreline can move well outside the stroke. An operator drawing a fine detail line gets a visibly different curve from the one they drew |
| 2 pt | 1 pt | 0.5 = half the half-width | ✅ the shipped case, and the only one that was ever right |
| 12 pt | 6 pt | 0.17× the half-width | ⚠ correct but pointlessly tight — keeps far more points than the guarantee needs |

The thin-pen row is the one that matters: it is the direction that
**changes what is authored**, it is silent, and it is worst on exactly
the drawings this shell is for, where a 0.25 pt pen exists to match a
CAD sheet's own linework.

# It lives here rather than in `ink`

Because the rule is *"the tolerance follows the pen"*, and a derivation
kept beside the value it derives from cannot be forgotten when that
value changes — which is precisely what happened when it lived
elsewhere as a `const`.



Two things follow, and both are load-bearing:

1. **Today**, `self.width_pts` is the only width, so this signature is
   correct and needs no kind. A `Pen` is one thickness and eight colours.
2. **The day a width goes per-slot**, this function must gain the slot
   and every caller must pass it. `ink::simplify` is the only consumer
   and it already knows the kind it is simplifying, so the change is
   mechanical — but it is *not optional*: an ink stroke simplified at the
   shape pen's tolerance while drawn at a per-slot width would move the
   centreline outside the stroke on exactly the thin-pen row of the table
   above, silently, on the dense drawings this shell is for.

[`tests::the_tolerance_follows_the_width`] is what enforces this rather
than the prose — it varies the width and asserts the tolerance moves,
which is the only form of the rule a future edit cannot read past.

### `fn colour_for`

Two total functions composed: [`PenSlot::of`] says which pen draws the
kind, [`Self::colour_of`] says what colour that pen is. Neither has a
catch-all arm, so a ninth [`MarkupKind`] or a ninth [`PenSlot`] is a
compile error rather than a silent landing on the shape pen — which is
what the old two-arm `match` with its `_ => self.ink` did, and was right
to do while there were exactly two pens.

### `fn colour_of`

Exhaustive over [`PenSlot`], deliberately and without a `_` arm: this is
the function a new slot must be taught about, and a catch-all here would
let a new slot compile while authoring the shape pen's red — a defect
with no symptom except a colour nobody chose.

### `fn set_colour`

The write half of [`Self::colour_of`] and the one place the operator's
override lands. *"Once they set a colour for a kind, it sticks for that
kind"* is this function plus the fact that [`Pen`] lives on the
application rather than on the document — see the module header's own
section on why a pencil does not change colour when you turn the page.

Alpha is dropped; see [`Self::set_ink`] for the argument, which is about
`/C` having three components and no fourth.

### `fn text_annot_colour`

Its one caller is `app::actions::apply`'s `CommitTextAnnot` arm, and
the line it replaced read `self.pen.ink` — one colour for all three,
which made a pdfcer sticky note shape-red where Acrobat's is violet.
Routing through [`PenSlot::of_text_annot`] rather than exposing the three
fields keeps the mapping total and keeps it here, beside the table it is
derived from.

### `fn set_ink`

# Alpha is dropped, deliberately, and this is not a limitation

`egui`'s colour picker offers an alpha channel; a PDF annotation's `/C`
entry is **three components and no more** (§12.5.2 Table 164: an array
of 0, 1, 3 or 4 numbers in the annotation's colour space, with
transparency carried by `/CA` instead). Feeding the picker's alpha into
`/C` would be a value with nowhere to go.

Opacity is therefore a **separate control**, and it is one that **now
exists** — [`Self::opacity`], drawn beside the swatches in
[`super::swatch::show`] since 2026-08-28. This paragraph used to end
*"it is not built yet: `/CA` support was filed against `pdfcer-core` and
is accepted-and-scheduled rather than shipped"*, which was true when
written and stopped being true when `Pass 81.1` landed
`MarkupOptions::opacity`. Corrected rather than deleted, because a stale
blocker is this project's most-repeated defect and the shape of it is the
useful part.

⇒ The alpha channel is *still* not offered **on the colour picker**, and
that is unchanged and correct: `/C` has three components, and a picker
alpha would be a fourth with nowhere to go. Transparency is `/CA` and it
has its own control.

### `fn opacity_option`

`None` at fully opaque. See [`Self::opacity`] for why that is not the
same as `Some(1.0)` even though it is the same on screen, and why the
difference is worth a method rather than a comparison at each call site
— there are three, and the one that forgot would be the one that made a
pdfcer annotation textually unlike everybody else's.

### `fn dash_option`

⚠ **Not passed on the text-markup route.** `app::actions::apply`'s
`CommitMarkup` arm sends it; `CommitTextMarkup` does not, because a
highlight, underline, strikeout and squiggly draw no `/BS` border and
the engine ignores the field for all four. Sending a value that is
documented as ignored would be this shell asking for something and
calling the silence success — which is the exact failure shape
`MarkupStyleSupport` was shipped to end.
