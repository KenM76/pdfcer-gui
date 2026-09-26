# `app::markupband` — the five Format ▸ Markup controls the ribbon cannot
draw itself

`RIBBON_IA.md` §5.8's *Markup annotation* row, and the operator's ask of
2026-09-06: *"getting full editing working for the Markup tools."*

## What this module is

The Markup group has five controls and **not one of them is a button**: two
colours that must also *show* the mark's current one, a width the operator
drags, a percentage, and a four-way choice of arrowheads.
`egui_shell::manifest::Item::Custom` is the extension point for exactly that
— it hands the application a `Ui` and gets out of the way — and this module
is what goes in that `Ui`.

[`crate::app::fontband`] is the precedent and this file is deliberately its
twin: same shape, same four obligations, same park-and-report contract. Read
that module's header first; what is below is what differs, and every
difference is argued rather than inherited.

## The shell reserves the slot; everything else is ours, including the
greying

`egui_shell::ribbon::control::render_command` does four things for a command
item: it evaluates `enable`, it draws the control greyed when the predicate
is false, it shows the tooltip through `on_hover_text` **or**
`on_disabled_hover_text`, and it publishes the control's rect under
`ribbon.item.<id>`. For a custom item it does **none** of them — it cannot,
because it does not know what is being drawn.

So all four are done here, in the same shapes and under the same names. The
rect matters as much as the greying: it is published under
`egui_shell::ribbon::report::band_item(id)`, the same name the shell builds
for a command control, so a driven check finds the fill swatch the way it
finds a Delete button. A second naming scheme for *the same kind of thing,
drawn by the other half of the program* is how a harness comes to have two
lookup paths, which is the defect `driving::declared_or_in_overflow` was
written to end.

## It reports; it does not dispatch

Every control here parks a [`MarkupEdit`] and returns the command's
`HandlerToken`. It raises no `Action` and touches no document.

That is `egui-shell`'s contract — *"the shell reports, the application
dispatches"* — and R8's dispatch choke point besides: a capability that
edits the document is a registered command, and a registered command is
invoked through `PdfcerApp::dispatch_command`, which is the same point a
chord and a context-menu row reach.

⚠ **`panels::properties::markup` pushes an `Action` straight into the queue
and this must not copy it.** That is the panel's established path and
`app::surfaces` states the rule at the ribbon's own custom-item closure: the
three Font controls *"DO return a token, where the pen's swatch above does
not, and the difference is what the control acts on."* A ribbon control that
built its own `Action` would put the operand derivation — *which annotation,
on which page* — in the renderer, where a chord never reaches it, and the
copy in `app::dispatch::format` would be the one that went stale.

## One field per raised action, never a struct reassembled from widgets

[`MarkupEdit`] carries exactly one property and [`MarkupEdit::into_style`]
sets exactly one field of `MarkupStyle`. `MarkupStyle`'s own doc comment is
the rule and the reason:

> Every field is `None` by default … That shape is deliberate: a Format tab
> whose colour picker also had to restate the current width would overwrite
> whatever the operator had set from the other control.

The failure that prevents is specific and this surface is where it would
have happened: five controls drawn from one annotation, one of them stale by
a frame, and a colour change that silently reverts a width set a moment
earlier. `panels::properties::markup` §"Every control is `None` unless the
operator touched it" argues it at length; this module obeys the same rule
through a type rather than through care.

## Absence and greying, and why this group's answers differ from Font's

| state | Font group | Markup group |
|---|---|---|
| the mode cannot author | **absent** (`mode.edit_content`) | **absent** (`selection.markup_restylable`) |
| no operand | **greyed**, and the tooltip says how to get one | **absent** — same condition |
| the operand refuses | the run is CMYK: greyed swatch, own sentence | the mark is **locked**: greyed, own sentence |

The middle row is the whole difference. A greyed Font control is the one
surface in this application that can say *sweep with the Text tool first*
(O37); a greyed Markup control could only say *select a mark*, which the
operator has already done or the contextual tab would not be on screen. R9
then requires absence, and `manifest::format`'s `MARKUP_VISIBLE_WHEN`
carries the argument in full.

⇒ The bottom row is what greying is left for, and it is R9's textbook case:
§12.5.3 Table 165 bit 8 is a fact about **this annotation** rather than about
the build or the mode — click a different mark and the controls work. The
sentence is `text::panels::properties::markup_locked`, which is the string
the Properties panel shows, so the two surfaces cannot refuse for different
reasons.

## WHICH subtype takes WHICH property is the ENGINE's question, and this
module no longer holds an answer to it


> *"That list is the engine's to know. The first subtype that gains or loses
> a border is the day our copy is wrong and nothing tells us."*

⇒ The engine answers it. **`pdfcer_core::edit::MarkupStyleSupport`** carries
`takes_border`, `takes_interior` and `takes_endings`, and
`MarkupStyleSupport::for_subtype` derives them from a `/Subtype`; that
sentence is quoted into the type's own doc comment as its justification.
[`Current::support`] is the answer, asked once per frame off the
annotation's `/Subtype`; the three controls read it and **nothing here
re-derives it**.

### ⚠ The distinction that must not be collapsed

| question | whose | where it is answered |
|---|---|---|
| *"does this subtype take a border width?"* | the **engine's** | [`Current::support`] |
| *"what IS this mark's border width?"* | this module's | the `MarkupSpec` arm |
| *"which anchors do I paint for this shape?"* | this module's | `canvas::annotnodes` |

The middle row is why the `match` on `MarkupSpec` survives: only
`MarkupSpec::Square` has a `border_width` field, so reading the **value**
out of an arm is something the compiler checks and something the engine has
no API for. What is gone is the arm deciding whether the control exists.
`canvas::annotnodes`' header draws the third distinction and is right; this
module does not touch it.

### Belt and braces — a predicate to ask, and a refusal if you ask anyway

The engine also refuses: `set_markup_style` raises
`EditError::StylePropertyNotApplicable { id, subtype, property }` **before**
anything is regenerated, so a width sent to a highlight leaves the file
untouched. Both mechanisms are in use
here and neither replaces the other: the predicate shapes the UI so the
refusal is unreachable, and the refusal is what makes a drifted shell loud
instead of silent.

Surfacing it costs this module nothing, and that is by design rather than
by omission: `app::actions::funnel::vector_edit`'s `Err` arm
already routes every `EditError` to the decline channel —
`crate::text::status::edit_declined_by_engine` on screen, the engine's own
sentence into `PDFCER_DIAG` — and `check-ui-strings.sh`'s exclusion 3 says
in as many words that an error's `Display` is *"not permission to route UI
text through an error type"*. So the refusal arrives; it arrives through the
one channel every refusal uses, and this module does not build a second.

## The genuine fifth state of the arrowhead chooser

`MarkupStyle::endings` became `Option<StyleEdit<(LineEnding, LineEnding)>>`
on the same day: `Set` writes `/LE`, **`Clear` removes it**. The four
positions this chooser offers all *write* the key, so *"no arrowheads"* —
`/LE [/None /None]` — and *"no line-ending entry at all"* are two different
files that draw the same line, and only the first was reachable.

### It is an ACTION below a separator, not a fifth peer — and here is why

A fifth position in the list would be a fifth answer to the question the
list asks (*which ends carry a head?*) that **draws identically to the
first**. Two entries a drafter cannot tell apart by looking is the one thing
this control cannot afford: the mistake is undiscoverable on screen and
shows up in a byte comparison of a drawing that has already gone out. It
would also make the combo's `selected_text` ambiguous, because a mark with
no `/LE` and a mark with `/LE [/None /None]` would both have a claim on it.

⇒ So the four positions stay four, and the removal sits under a separator as
what it is — an act, not a state. Three consequences fall out of that shape
and each of them is the reason:

- **It costs no band width.** `ribbon::plan::CUSTOM_ITEM_WIDTH` is 96 and
  this group is already sized to fit inside it; a button beside the combo
  would have spent the budget on the rarest control in the group.
- **It is ABSENT when there is no `/LE` to remove**, which is the rule
  [`fill`]'s Clear and `panels::properties::markup`'s three Clears already
  obey: *a Clear beside a mark that has nothing to clear is a control whose
  only possible effect is an undo entry the operator did not earn.* That is
  what [`Current::endings_key_present`] is for, and it is read off the
  dictionary rather than off the spec, because `spec_from_dict` supplies
  Table 176's default and so cannot tell an absent key from a written one.
- **It is where the operator already is.** They opened the chooser to change
  their mind about arrowheads; the way back to the file they were given is
  in the same popup, not on a second control they must go looking for.

The wording is a drafter's and lives in the catalog, once, read by **both**
surfaces: `text::panels::properties::markup_endings_clear` and its hover.
One string, so the tab and the panel cannot come to describe one act two
ways.

## Rule 15 — a ce dimension is a different verb and must not appear here

**ce dimensions** are the ones pdfcer authors; **pdf dimensions** are CAD
page content. A ce dimension is `panels::properties::dimension`'s and uses
`set_dimension_style`; handing one to `set_markup_style` regenerates it as a
bare line with its label and witness lines gone, and the engine refuses it
by name.

The guard is a `match` on `AnnotKind` that the **compiler** checks, in two
places that cannot disagree: [`resolved`] here and `app::conditions`'
publication of the condition. It is deliberately not a comparison of
`/Subtype` strings, because a ce dimension's `/Subtype` **is** `/Line`,
exactly like an arrow's — a string test would restyle the operator's
dimensions into bare lines and would look correct while doing it.

## Why this module reads the session itself rather than sharing the
panel's reader

[`Current::read`] is a twenty-line dictionary read that
`panels::properties::markup::Current::read` also performs. Duplicating it is
the deliberate choice: that function is private to its module, five tracks
were writing in this tree on the day this landed, and making it
`pub(crate)` would have been an edit to another author's file to save
twenty lines.

⚠ **Corrected 2026-09-06.** This paragraph used to end: *"The two are
allowed to differ, and one of them does — this one also reads `/LE` and the
interior, which the panel offers no control for."* That was true when it was
written and stopped being true the same day: `panels::properties::markup`
gained `fill_row` and `endings_row` in the session that added this band, so
**both surfaces now read all five terms**. The permission still stands — the
two readers are allowed to differ — but the example of a difference is gone,
and a header that keeps a stale example teaches the next reader a fact about
the panel that is no longer so.

⚠ What is **not** duplicated is the derivation that matters: both read
through `annot_author::spec_from_dict`, the author's view, which is what
`set_markup_style` reads when it plans. One derivation of *what this mark
currently is*, two readers of it.

## Rule 4 / R8b — fuzzy, never sneaky

Nothing here marks the canvas and nothing here can. The restyled mark
renders exactly as the saved file will render it; what the engine could not
reproduce is raised by `app::actions::annots` into the status bar, and what
a wider pen does to the annotation's `/Rect` is disclosed in this group's
tooltips and in the Properties panel's standing note.

## Item notes

### `const MIN_WIDTH_PT`

The same floor `panels::properties::markup` and `canvas::markup::pen` use,
and the reason is theirs: §8.4.3.2 gives `0` a defined meaning — *the
thinnest line the device can render* — which on a 600 dpi plot is a hairline
and on screen at 25 % is invisible. An operator who wants a mark they cannot
see has the visibility toggle; what they must not get is a mark whose weight
depends on the output device without being told.

### `const MAX_WIDTH_PT`

Beyond about twelve points a border stops reading as a border and starts
reading as a filled shape. Same ceiling as the pen that authors, so an
operator cannot set a width here that no gesture could have produced.

### `const FIELD_WIDTH`

Chosen against `egui_shell::ribbon::plan::CUSTOM_ITEM_WIDTH`, which is
**96** and is what the band budgets for a custom item it cannot measure.
That module is explicit about the asymmetry — *"an estimate that is too
small costs a clipped group; it cannot cost the overflow control"* — so
every control here is sized to fit **inside** the budget rather than to look
comfortable. `app::fontband`'s size field is 46 for the same reason and
these match it, because two drag fields on one tab that are different widths
read as a layout accident.

### `const ENDINGS_WIDTH`

Wider than the fields because its longest entry is a phrase rather than a
number, and still inside the 96-point budget with room for the combo's own
frame and arrow.

### `const DASH_WIDTH`

Narrower than [`ENDINGS_WIDTH`] even though its list has a longer entry,
and that is a deliberate acceptance of one clipped reading rather than a
measurement mistake. The four *entries* are short — the longest is
*"Dash-dot"* — and the only long string this combo can show is
`linestyle::DashReading::Foreign`'s *"Dashed (the file's own pattern)"*,
which appears on a producer's mark and not on anything this shell drew. A
group of six custom items has to fit inside
`egui_shell::ribbon::plan::CUSTOM_ITEM_WIDTH` apiece or it costs a clipped
band, and spending the extra points on the rarer string would be the wrong
trade.

⇒ The full reading is still reachable: the Properties panel's Line style row
draws the same chooser with the panel's width, which is where a reader who
wants the whole sentence goes. That is §5.8's division of labour working as
intended rather than a gap.

### `fn command_for`

One place, so that the kind → id mapping cannot be spelled one way in the
renderer and another in `manifest::CUSTOM_BACKED` — which is the register
that keeps these five from looking like orphaned commands to the
reachability check, and which is asserted against the manifest rather than
against this function.

### `enum Operand`

Three variants and not an `Option`, because the two failure states get
different hovers and an `Option` would force the caller to re-derive which
one it was. That re-derivation is exactly the shape of `fontband::colour`'s
recorded defect, where one arm answered two reasons with one sentence for
eight days.

### `fn resolved`

# Read from the SESSION every frame, never from a cache

The verb these controls raise rewrites the very values they display, and an
action is applied *after* the frame that raised it — so a cached copy would
be stale for exactly the frame the operator is looking at, which is the
frame they judge the result on. `panels::properties::markup` states the same
rule for the same reason.

# The `AnnotKind` guard is here AND in `app::conditions`, deliberately

The condition already refuses a ce dimension, so this looks redundant. It is
not: a condition is a hint published for the ribbon's benefit and is
evaluated a frame's worth of state earlier than this draw, and — the case
that actually bites — a **chord** consults no condition at all. Rule 15's
guard has to be where the operand is built, and the `match` on `AnnotKind`
is what makes routing one to the wrong verb a compile error rather than a
wrong `/Subtype` string comparison.

### `fn placeholder`

**A greyed field shows the PLACEHOLDER, not a number.**
`fontband::size` records what the alternative costs: with nothing swept the
draft held its `Default` — zero — and `DragValue`'s own range clamped it up,
so the greyed control read `1.0 pt`, which is a claim about the operator's
document and a false one. A driven check was right to pass; it asserted that
the control was drawn, and it was.

A `Button` rather than an inert `DragValue`, for that module's reason: the
shape of the thing an operator is looking at should say *there is no value
here*, not *here is a number you may scrub*. It is disabled, so it takes no
clicks and reports nothing.

### `struct Current`

# Why it is read through `spec_from_dict` and not from `annot::Annotation`

`pdfcer_core::annot::Annotation` is the **reader's** view — id, subtype,
rect, flags, `/CA`, appearance — and it deliberately carries no `/C`, no
`/IC`, no `/BS /W` and no `/LE`, because nothing that renders a page needs
them: the picture comes from the baked `/AP`.

`annot_author::spec_from_dict` is the **author's** view, and it exists for
exactly this — *"so an existing annotation can be restyled by regenerating
its appearance from its own declared geometry"*. Reading through it means
the values these controls show are the values `set_markup_style` will read
when it plans: one derivation, not two.

Its refusals are absent values here rather than an error, and that is
honest rather than lax. `SpecReadError`'s own doc says every variant is *"a
refusal to guess"* — geometry that is missing, or is not something pdfcer
models. A mark like that can still be **given** a colour; what cannot be
done is show the one it has.

### `impl Default`

`MarkupStyleSupport` is `#[non_exhaustive]` and has no `Default`, so this
impl is written out — and that is a small piece of luck worth keeping,
because the honest default for *"the dictionary could not be read"* is
**not** a hand-written all-`false` literal. It is what the engine answers
for a subtype it does not recognise, which `for_subtype`'s own doc calls
*"the conservative direction"*. Asking for it costs one call and removes the
last place a `false` could have been written here by hand.

### `fn offers_fill`

Purely the engine's answer: `/IC` needs no readback to be *offerable* —
*no fill* is a legitimate current state and [`fill`] shows it as the
swatch's white default with no Clear beside it.

### `fn offers_width`

Both terms, and they mean different things — see [`width`]'s doc. The
first is the engine's (*this subtype has no border*), the second is this
build's (*this arm's width is not one I can read*).

### `fn offers_dash`

**Purely the engine's answer, with no second term** — unlike
[`Self::offers_width`], which also asks whether a width was read. The
asymmetry is real rather than an oversight: a width has to be *shown* in
its field, so a mark whose width this build could not read has nothing to
put in one; a line style always has a value, because *solid* is a state
and not an absence, and [`crate::canvas::markup::linestyle::read`] is
total — every dictionary answers it, including one with no `/BS` at all.

⇒ So the only question left is the engine's *does this subtype have a
border?*, and `takes_border` is it. That is the same predicate
`set_markup_style` guards `style.dash` with
(`pdfcer_core::edit::EditSession::set_markup_style`), so a chooser drawn
here cannot
produce the `StylePropertyNotApplicable` refusal — which is the belt this
module's header describes, with the engine's braces behind it.

### `fn stroke`

# A swatch alone, where the Properties panel has a swatch and a Clear

`StyleEdit` has two arms and they mean different things in the file: `Set`
writes `/C` and `Clear` removes it, restoring the standard's default. The
panel offers both, and §5.8's rule is that *"the tab's contents are a
**subset**"* of the panel's — so dropping one is permitted where adding one
would not be.

It is dropped rather than kept because a `/C` an operator wants gone is a
deliberate, rare act, and the ribbon is the surface for the frequent one:
two controls per colour would take two of the group's five slots for the
stroke alone. **Fill is the exception**, and [`fill`] argues why — *no fill*
is not a rare act there, it is the state every mark starts in.

### `fn fill`

# *No fill* is a first-class state, and it is why this control is two
widgets where [`stroke`] is one

`canvas::markup::spec` authors every shape with `interior: None`, and its
reason is quoted in `panels::properties::markup`'s header: *"a filled
comment shape hides the drawing it is a comment about, which on a CAD sheet
is the whole content under it."* Acrobat's default is the same for every
shape.

⇒ So **no fill is where every mark starts**, and a control that could only
ever set one would be a one-way door: try a fill on a drawing, decide against
it, and there is no way back to the mark you had. `StyleEdit::Clear` is the
way back and [`crate::text::ribbon::markup_no_fill`] is what it is called.

⚠ **This does not change what NEW markup is authored with.** The pen is
`canvas::markup::pen`'s and is untouched; `NO_SURFACE.md` records that
reversing the authoring default is the operator's call, and offering a
restyle control does not make it.

The clear button is **absent** when there is nothing to clear rather than
greyed — the panel's rule, and its reason: a Clear beside a mark that has no
`/IC` is a control whose only possible effect is an undo entry the operator
did not earn.

### `fn width`

⚠ **This moves `/Rect` for every subtype except `Square` and `Circle`** —
the rectangle is derived from the geometry plus a margin that contains the
stroke and any arrowheads, so a wider pen needs a bigger box. That is the
engine's own ⚠ and it is disclosed in the command's tooltip
(`text::commands::markupstyle::format_line_width`) rather than here, because
a ribbon band has no room for a sentence and the hover is the surface that
does.

Committed on `drag_stopped` or `lost_focus`, **never** on `.changed()`. A
`DragValue` reports a change on every pixel of a drag, and each one here
regenerates the appearance and is one undo entry — so a single drag across
the control would leave forty entries on a `Ctrl+Z` stack the operator could
not get back through, and would re-plan the annotation forty times.

**Absent** rather than greyed when the mark has no border to widen: a
highlight is `/QuadPoints` and has nothing to stroke. R9, and the same
answer `panels::properties::markup::width_row` gives.


⇒ [`Current::offers_width`] therefore carries two terms and both are needed.
`takes_border` false means *this subtype has no border* — nothing renders,
forever. A `None` width under a `takes_border` that is true means *this
build cannot read this arm's width*, which `MarkupSpec` being
`#[non_exhaustive]` makes possible: a control drawn there would have no
value to show, which `placeholder`'s own argument forbids. The `let else`
below is the extraction, not a third guard.

### `fn opacity`

Shown as a **percentage**, because that is the unit every other
application an operator has used states opacity in, and `/CA`'s own
`0.0..=1.0` is a file-format detail they should never meet. The Properties
panel's twin makes the same choice and this reads the same suffix from the
same catalog entry, so the two surfaces cannot come to call it different
things.

Release-not-change, for [`width`]'s reason exactly.

No Clear. `/CA` absent and `/CA` at 100 % render identically, so removing
the key is a change with no visible consequence — and the panel, which has
room for a control whose effect is invisible, is where that belongs. Here it
would spend a slot on a button an operator could not tell had worked.

### `fn dash`

# What this closes


# The preserve half is why this control is SAFE, and it is why it
# shipped at all

Before that Pass, a dashed mark in the operator's file was **silently
converted to a solid one the first time anything about it changed** — and
the engine's reply records that the defect was wider than this shell
reported it: the recolour path was named, and `resize_annotation`,
`reshape_annotation` and authoring solidified a dash too. So dragging a
resize handle or a vertex destroyed it, not only pressing the colour swatch.
All four carry it now.

⇒ That is the precondition a *Line style* control needs. Offering one over
an engine that dropped every dash it did not author would have been a control
whose neighbours undid it.

# There is no Clear beside it, and the reason is Table 166

The other two `StyleEdit` controls in this group put `Clear` on its own
button — `fill`'s *No fill*, `endings`' *Clear the setting* — because in both
cases the cleared state is the **absence of a key** and has no name in the
list. Here it does: `Clear` makes the border solid, and *Solid* is Table
166's own `/S` and the chooser's first entry. A separate button would be a
second spelling of one act, and one of the two would eventually be pressed
expecting something different from the other.

# Absent for a subtype with no border

[`Current::offers_dash`], which is `MarkupStyleSupport::takes_border` and
nothing else. R9, and the same answer [`width`] gives for a highlight.

### `fn endings`

# Four positions, and the SHAPE is preserved rather than chosen

`/LE` is two independent endings over three shapes each (§12.5.6.7, Table
176) — nine combinations, which is not a list anybody reads on a ribbon
band. This offers the four *positions* an operator means and carries the
mark's existing arrowhead shape through unchanged, so a closed arrowhead
stays closed and an open one stays open.

⇒ That is the difference between a control that answers the question asked
(*which ends?*) and one that quietly answers a second question nobody asked
(*and what shape?*). A chooser that normalised every arrow to `/OpenArrow`
would silently rewrite a `/ClosedArrow` the operator's producer had set, and
the change would be visible in another viewer.

# …and a fifth state that is not a fifth position

The four positions all **write** `/LE`. `StyleEdit::Clear` **removes** it,
which is a different file drawing the same line, and it is offered here as
an **action below a separator** rather than as a fifth entry in the list.
The header carries the argument in full; the two sentences that matter are
that a fifth entry drawing identically to the first is a distinction a
drafter cannot check by looking, and that it would leave the combo's
`selected_text` with two equal claimants when `/LE` is absent.

It is **absent unless `/LE` is in the dictionary**
([`Current::endings_key_present`]), which is [`fill`]'s Clear rule: a
removal offered where there is nothing to remove has no possible effect but
an undo entry the operator did not earn.

### `fn arrow_shape`

`ClosedArrow` wins when the two ends disagree, and `OpenArrow` is the
answer for a line that has no head at all. Both choices are about not
destroying information: a mark with one closed head is a mark whose author
chose closed, so adding a second head should match it rather than convert
it; and `OpenArrow` is what pdfcer's own pen authors and what
`LineEnding::OpenArrow`'s doc records as *"Acrobat's default at both ends"*,
so a plain line given its first head gets the head this program draws.

### `fn rgb_of`

`None` for anything that is not RGB or grey, and that is honest rather
than lossy: §12.5.2 lets `/C` be a 0-, 1-, 3- or 4-component array, and a
swatch showing a CMYK mark's *converted* colour would be a control whose
readback is a conversion the operator never asked for — pick it up, put it
down unchanged, and the file now says something different, on a drawing
heading for a printer that cares.

Grey is included rather than refused because it is **lossless** in both
directions: `Gray(v)` and `Rgb(v, v, v)` are the same ink.

### `fn srgb_to_colour`

Always `DeviceRGB`, never a grey collapsed from three equal channels. The
swatch is an sRGB picker, so what the operator chose *is* an RGB triple; a
mark silently written as `DeviceGray` because its channels happened to match
would be pdfcer inferring a colour space the operator did not ask for, which
is Rule 4's whole subject.

### `enum MarkupEdit`

The type is the enforcement of this module's central rule. Each variant
carries exactly one field of `MarkupStyle`, so there is no expressible value
that restates a property the operator did not touch — which is what
`MarkupStyle`'s own doc requires and what a `MarkupStyle` parked directly
would have made merely a matter of care.

It is this module's own type rather than a reuse of `MarkupStyle`, and
that is worth one sentence: `MarkupStyle` is an **input struct** the engine
deliberately left non-`#[non_exhaustive]` so callers can build it, and it is
perfectly buildable here. What it cannot express is *"exactly one field, and
the caller chose which"*, which is the invariant the dispatcher relies on.

### `fn into_style`

Every arm sets **one** field and leaves the rest at `Default` — which is
`None`, which is *"do not touch this property"*. That is the whole
contract, and it is asserted from the outside by
`only_one_field_is_ever_set` below rather than trusted.

### `fn draw`

Returns the command's handler token when the operator changed something, in
which case `parked` holds the change and `target` holds the annotation it is
about. `None` means *nothing was invoked*, which is what the shell expects
for a frame in which the operator merely looked at the control.

# `kind` is matched, not asserted

An unrecognised kind returns `None` and draws nothing, exactly as
`PdfcerApp::ribbon_band`'s renderer does for one it does not know. A
manifest is data; the honest response to a kind nobody implements is a gap,
not a panic in the paint loop.
