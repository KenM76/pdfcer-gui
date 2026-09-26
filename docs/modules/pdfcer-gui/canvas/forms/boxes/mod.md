# `canvas::forms::boxes` — where a form's widgets are, and what a click on
one would mean

The **pure half** of filling a form on the page. Everything here is a
function of the document and a rectangle; nothing here needs an
`egui::Ui`, a laid-out scroll area or a live pointer, and every rule this
surface is judged on is therefore something a unit test can hold rather
than something a running window has to be trusted to demonstrate.

That split is the seam `canvas/mod.rs` and `canvas/keys.rs` already draw
between themselves — one side is drivable by a headless `egui::Context`,
the other needs a window — applied one level down. It is a subject and not
a line count: [`super`] contains no decision at all, only the wiring that
spends the decisions below.

## "Redraw appearances" is only half the remedy for an undrawn field

[`super`]'s §5 reason 1 offers `RegenerateAppearances` to an operator whose
field draws nothing. `EditSession::regenerate_appearances` writes an `/AP`
for a text field only when that field holds a `/V`, so it does nothing at
all for an **empty** undrawn one. The remedy that always works is a fill:
`fill_text_field` writes the value and regenerates every widget's `/AP`, so
filling once in the panel makes the field clickable on the page from then
on. [`crate::text::forms::forms_canvas_undrawn_note`] is where the panel
says that to the operator.

Read [`super`]'s header first. It carries the whole argument — why this is
not a [`CanvasTool`] variant, why the panel is not replaced, what the
editor cannot promise, how input layers, why the hit test takes no
tolerance, and the four reasons a field is routed to the panel instead.
This file is where those four reasons are actually decided
([`classify`]), where the geometry is done
([`crate::canvas::mapping::annot_canvas_rect`], which serves annotation
selection too) and
where the hit test lives ([`hit`]).

## Item notes

### `const MIN_EDITOR`

A form field is whatever size its author made it, and at 25 % zoom a
perfectly ordinary 12 pt field is three pixels tall. An editor that small
is an editor nobody can read what they typed in, so the box is grown about
its own centre until it reaches this — which means it can overhang the
field it is editing.

That overhang is the deliberate half. The alternative is an editor that
sits exactly on a field the operator cannot see into, which trades a
visible, self-explaining imprecision for an invisible, silent one. It also
has an obvious operator-side remedy that needs no code: zoom in.

### `const EDITOR_TEXT_RATIO`

A glyph box is taller than its letters, and a font size equal to the box
height clips descenders. 0.62 is the ratio at which an ascender-plus-
descender line fits inside the box with the padding `egui` adds, measured
against the theme's own text style rather than derived.

### `const EDITOR_TEXT_RANGE`

The lower bound is legibility; the upper bound stops a full-page field —
a signature block, a comment box — from being typed into at 40 pt, which
reads as a bug rather than as fidelity.

### `enum BoxKind`

One variant per gesture, and no others: everything in
[`NotOnCanvas::NotOffered`] has no canvas gesture at all, so it is absent
here rather than present-and-inert. The "no placeholders" invariant applies
to enums as much as to labels — a variant nothing can raise is dead code
wearing a design pattern.

### `struct Routing`

Produced by the same walk that produces the boxes ([`place`]) rather than
by a second pass, so the count and the behaviour cannot disagree — a panel
promising "3 fields can only be filled here" over a canvas that declined
four is worse than no count at all.

Counted **per field, not per widget**: one clickable widget is enough for
the field to be reachable on the page, and a per-widget count would report a
two-page field as unreachable because one of its two boxes sits on a
rotated sheet.

### `struct FieldTarget`

## Why this is not [`WidgetBox`], and why it comes from the same walk

A `WidgetBox` is a widget a click can **fill**, and five conditions narrow
the set: no appearance, a rotated page, an unlisted widget, a kind with no
canvas gesture (a drop-down, a button), or a field type this shell does not
type into. Every one of those is right for filling and **wrong for
selecting**. An operator who has just placed a drop-down and wants to look
at its properties must be able to click the thing they can plainly see.

So the authoring surface needs a wider set. It is produced by the **same
walk** ([`place`]) rather than by a second one, which is this module's
standing rule stated in [`Placed`]'s own doc: two walks are two statements
of the placement rule, and the drift between them is a click that selects a
field the canvas is not drawing.

The only condition that still excludes a widget here is the one that is not
a policy: **no canvas rectangle**, which means a non-invertible page
transform or a degenerate `/Rect` — a widget with no area to click.

### `struct Placed`

One type, because they are one walk. The alternative — a `boxes_for` and a
separate `routing_for` — is two statements of the five-reason rule in §5,
which is exactly the drift this module exists to prevent.

### `struct WidgetBox`

**`rect` is CANVAS space**, which is what makes this cacheable across zooms
and scrolls: canvas space is the frame `PageMapping` converts *to*, and a
canvas coordinate does not move when the view does (`mapping`'s
`a_canvas_point_survives_every_zoom_and_scroll_position`). A screen-space
cache would have to be rebuilt on every wheel notch, and a stale one would
focus the wrong field.

### `fn offered_in`

[`CanvasTool::Select`] and nothing else — see the module header §1. The
hand tool's click means "nothing" everywhere else in this canvas and must
go on meaning it here; a markup tool's press is claimed unconditionally by
`gesture::press_kind`, and a pen that filled a field as well as drawing
would be two gestures on one press.

A held space bar borrows the hand out of Select before this is asked
(`tool::resolve`), so filling is suspended for exactly as long as the bar
is down and returns with nothing stored — the same property the space
override buys everywhere else.

### `fn editor_fill`

# What this is for, and why it is not a facsimile

The in-canvas field editor lays a live `egui` text box over the raster for
the duration of a keystroke. With no fill that box is `extreme_bg_color` —
near-white under the light presets — so a pale-yellow or shaded form field
**turns grey the moment the operator touches it** and turns back a gesture
later: a visible change to content nobody asked for, which is the thing
pdfcer's rule 4 forbids.

It is not a fidelity claim, and the module header's §3 is the reason it
does not have to be one. §3 refuses to make this box a facsimile because a
substituted font cannot promise the document font's glyph advances — an
**arithmetic** argument, and one that reaches exactly as far as the
arithmetic does. The test a property must pass to be honoured here is
therefore *does honouring it make a claim about where a particular glyph
will land*. A fill does not, exactly as `/Q` does not — and the engine's
own `Widget::background` doc names this editor as the intended consumer.

# The three-state `/BG`, which is why this takes the widget and not a colour

Table 189 distinguishes **absent** (`background == None`) from an **empty
array** ([`MkColor::None`] — *explicitly no colour, transparent*). Both
answer `None` here, because both mean *keep the theme's box*, but they are
different facts about the file and a caller that collapsed them upstream
would have lost one. This function is where they are allowed to merge, and
it merges them at the point of use rather than at the point of reading.

# DeviceCMYK IS converted here, and elsewhere in this shell it is not

`app::markupband::rgb_of` returns `None` for a CMYK mark and
`app::fontband` greys the swatch, on a rule this project holds firmly: *a
swatch showing a converted colour is a control whose readback is a
conversion the operator never asked for — pick it up, put it down
unchanged, and the file now says something different.*

That argument is about a **round trip**, and this is not one. Nothing here
is written back; the value is a tint on a transient overlay that is gone
the moment the edit commits. And the conversion is
[`pdfcer_core::color::cmyk_to_srgb`] — the engine's own calibrated one, the
same conversion that produced **the raster pixels immediately around the
box**. Refusing it would not avoid a conversion; it would make the editor
disagree with the page it is sitting on.

The match is exhaustive with no wildcard, so a new [`MkColor`] variant
stops the build here rather than silently taking the `None` arm.

### `fn classify`

Pure, and deliberately takes the page's `/Rotate` as a number rather than a
`&Page`: the rotation is the only thing about the page this decision
depends on, and passing the page would make the rule untestable without
building one.

# The order of the questions is the rule

Appearance first, because a field nothing draws cannot be pointed at
whatever else is true of it. Then the panel's own
[`block_reason`](crate::panels::forms::rows::block_reason), **asked rather
than re-derived** — a read-only field must be refused here for the same
reason and in the same words it is refused there, and two statements of one
rule is how the two surfaces come to disagree about which fields are
fillable.

Rotation is asked **last, and only for text**, which is the whole of the
rotated-page decision: the box is placed correctly at every rotation, and
it is only the editor that cannot be.

# It asks nothing about geometry

The geometry a click is tested against does not come from `Widget::rect`
— [`place`] takes it from each page's `/Annots` instead. Asking about it
here as well would be a second source of truth for where a widget is, and
the one that is *not* the one being hit-tested.

### `fn place`

Built once per `(document, edit epoch)` and cached — see
[`super::placed`] — rather than per frame, which is what makes an I-beam
cursor over a form affordable. The whole document rather than the visible
pages, because the cache key has no room for a scroll position and a form is
small: `pdfcer-core`'s corpus has nothing over a thousand fields.

`annots[i]` is page `i`'s `EditSession::widget_rects(i)` — every `/Widget`
annotation that page's `/Annots` lists, with its `/Rect` already normalised.

# Which page a widget is on is answered by `/Annots`, never by `/P`

The obvious implementation reads `pdfcer_core::forms::Widget::page` — the
widget's `/P` entry — and looks the page object up by id. It is
**silently wrong on a large class of real files**.

`/P` is *Optional* (§12.5.2 Table 164). A widget that omits it is perfectly
conformant and is common in the wild, and `pdfcer-core` additionally reads
the key **without resolving through the graph**, so a direct rather than
indirect `/P` also reads as absent. Either way a `/P`-keyed placement
returns *nothing at all* for such a form: no error, no refusal, no trace —
a form on which clicking a field simply does not work, with the panel
cheerfully reporting every field as fillable.

**No test written against the fixture corpus can catch this.** The form
fixtures in `D:\Dev\pdfcer\fixtures\synthetic\forms\` write `/P` on every
widget, so the failing case is unreachable from them, and *a test that
cannot reach the case is satisfied by any implementation* — sabotage the
implementation and the suite stays green. That is why
[`tests::a_widget_with_no_p_entry_is_still_placed`] builds its input by hand
rather than opening a fixture.

So the direction is inverted: rather than asking each widget which page it
claims, each **page** is asked which widgets it lists, and `/P` is not
consulted anywhere in this module. A widget no page lists is
[`NotOnCanvas::NotPlaced`], which is the honest statement of the same fact
and is *true* rather than merely defaulted.

# Ordering

Within a page, `/Annots` order — **paint order**, and absent `/Tabs` also
tab order. Deliberately not the panel's order, which is `/AcroForm`
`/Fields` order: the two commonly differ, and they answer different
questions. [`hit`] depends on this one (a widget painted over another wins
the click); the panel's list depends on its own. Making either match the
other would break the surface that needed it.

### `fn hit`

**Containment, no tolerance** — see the module header §4 for why this is
the one hit test in `canvas/` that takes none.

Later boxes win. `/Annots` order is paint order, so a widget drawn over
another is the one the operator can see, and the one they can see is the
one they meant.

### `fn hit_target`

[`hit`]'s twin over the wider set. Containment with no tolerance and later
boxes winning, for the identical reasons — a widget drawn over another is
the one the operator can see, and the one they can see is the one they
meant.

### `fn editor_rect`

Grown about the **centre** rather than from the top-left, so a field that
is already wide enough and only too short does not slide sideways under the
operator's pointer between one zoom and the next.

### `fn editor_font_size`

Derived from the box rather than from the field's `/DA`, and that is the
honest choice rather than the lazy one: the `/DA` size is stated in *page*
units for a *document* font, and this editor draws a *substituted* font at
*screen* scale. Honouring the `/DA` number would produce a box whose text
is the right nominal size and the wrong physical one, which looks like a
fidelity claim and is not one. See the module header §3.

### `fn editor_align`

One line of arithmetic-free translation, given a function of its own for
two reasons that are both about evidence rather than about tidiness:

1. **It is the whole of the `/Q` decision**, and this file is the half of
   the surface a unit test can hold ([the module header](self)). Applying
   the mapping inline in [`super::editor`] would put the only statement of
   §12.7.4.3's three codes inside a function that needs a laid-out
   `egui::Ui`, a live pointer and a page raster to run at all — which is to
   say, inside the half that can only be checked by looking.
2. **The failure it guards is a silent transposition.** `1` is centre and
   `2` is right; swapping them compiles, draws, passes every other test in
   this file and is visible only as a right-aligned form typed into
   centred. [`Quadding`] has already turned the integers into names, and
   this keeps the names paired with `egui`'s in one place.

[`Quadding::from_code`] has already applied Table 222's own tolerance — any
`/Q` that is not `1` or `2` is left — so there is no malformed case left
for this to decide.

### `fn truncate`

Live rather than at commit, and by character rather than by byte — the
panel's rule, restated as a function so the two surfaces cannot enforce
different limits. A byte index would both split a multi-byte character and
refuse an accented name three letters early.
