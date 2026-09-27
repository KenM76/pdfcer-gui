# `markupswatch` — the Markup ▸ Style group's one control

The `colour_swatch` custom item the manifest has declared since S2 and
nothing ever drew, so the Style group rendered a caption over an empty band.

## Why it is a Custom item and not three commands

`egui-shell`'s `Item::Custom` is the extension point for a control that is
*not a button* — its own documentation names *"a split button with a
gallery"* — and this is three of those. A `Command` item can only render as
a button, and a button cannot ask *which colour* any more than the Recent
item's button could ask *which document*. The manifest declares
`Item::custom("colour_swatch")` and the application supplies the renderer,
exactly as it already does for Recent.

That is also why this is not three registered commands. A command is a verb
the operator invokes and the shell dispatches; setting a pen colour is not
a verb, it has no undo, it raises no `Action`, and giving it a handler token
would put a no-op through the dispatch `match` for every click of a colour
picker.

## What it sets, and the one thing it deliberately does not

`RIBBON_IA.md` §5.5's Style group is *"Colour · Line width · Fill ·
Opacity"*. Two of the four ship here and two do not, and the two absences
are different in kind:

| control | state | why |
|---|---|---|
| **Colour** | ✅ | two swatches, each opening [`super::palette`]'s grid of Acrobat's own colours — see [`super::pen`] on why there are eight slots and two controls |
| **Line width** | ✅ | a drag value in points, over the pen's own range |
| **Fill** | ⬜ | **a design decision about the PEN, and only about the pen** — see the row's own correction below |
| **Opacity** | ✅ | **shipped 2026-08-28** — a percentage drag value writing `/CA`. This row said *"blocked on the engine"* for four months after it stopped being true; see below |
| **Line style** | ✅ | **shipped 2026-09-06** — a four-entry chooser writing `/BS` `/S` and `/D`. §5.5 does not list it; it is here because it is a property of the next mark exactly as the four above are, and because the engine shipped the author-time half (`MarkupOptions::dash`) beside the restyle half on the day the Format tab got its own copy. See [`super::linestyle`] |

⚠ The table is now **five rows over a four-item specification**, which is a
table that has outgrown its source rather than one that is wrong. §5.5 was
written before a dash was expressible at all; the *Line style* row is the
one entry here that this shell added rather than answered. It is called out
so that the next reader does not spend a minute looking for a fifth bullet in
`RIBBON_IA.md` §5.5 that is not there — the entry they will find is §5.8's,
about the **other** surface.

### The Fill row, corrected: it was describing ONE of two surfaces and
### did not say which

It used to read *"a design decision, not a gap"*, followed by [`super::spec`]'s
argument that a filled comment shape hides the drawing it is a comment about.
That argument is **still correct and still in force** — for this control. What
the row failed to say is that there are *two* fills in this program and it was
only ever talking about one of them:

| which fill | whose surface | state |
|---|---|---|
| the fill the **pen** authors a NEW mark with | this module, the Markup ▸ Style group | ⬜ **deliberately absent**, and not a gap. `spec` passes `interior: None` for every shape, and on a CAD sheet an unfilled comment shape is the only kind that does not hide the content it is about |
| the fill of a mark **already on the page** | `panels::properties::markup`, the contextual Format tab | ✅ being built — an operator who wants a filled shape places one and fills it, which is a decision they made about a specific mark rather than a default applied to every future one |

⇒ The distinction is the whole answer to *"why can I fill that rectangle and
not this pen?"*, and a row that named neither surface could not give it.

### The Opacity row, corrected: it was FALSE, and false in the direction
### this project has been wrong in before

It used to read *"blocked on the engine. Annotation transparency is `/CA`,
which `pdfcer-core` does not write yet — filed, accepted, not started."* That
was true when written and stopped being true on **2026-08-27**, when
`Pass 81.1` landed `MarkupOptions::opacity` in answer to a request this shell
filed itself. `set_markup_style` writes `/CA`; `add_markup_with` authors a
translucent mark in one verb and one undo entry; [`Pen::opacity`] and
`MIN_OPACITY` have existed since the day after.

⇒ The control was **already drawn** in [`show`] below, with its own trace
region and its own tooltip, while this table three screens above it said it
could not exist. Both were read by everyone who opened the file and only the
table was believed, because a table is what a reader trusts.

That is the **eighth** stale blocker this project has found and it is the
second one *in this file*. The standing rule it produced — **a backlog row is
a record, not evidence** — has a corollary that this instance adds: *a
capability table in a module header is a claim about the module, and the
module is right there.* The correction is written rather than deleted because
the shape of the mistake is the useful part.

## THE PALETTE POPUP — what the operator asked for by name

> *"Also make sure you've used the same default colours and style look for
> these things as Adobe."*

The **colours** half is [`super::palette`] and [`super::pen::Pen::default`].
The **style look** half is this module, and it is a specific, nameable change:

| before | after |
|---|---|
| `egui`'s `color_edit_button_srgba` — a generic HSV wheel with hue, saturation and value sliders | a **grid of named preset swatches**, with the full picker one click below it |

Acrobat does not open a colour wheel when you press its comment colour chip.
It shows a small grid of colour cells, each with a name, and *More colours…*
underneath for anything else. That shape is not decoration — it is what makes
the control usable at ribbon speed: an operator marking up a drawing wants
*red*, and a wheel makes them navigate to it and produces a slightly different
red every time. A grid gives them the same red as last time, and — because
every cell is a value measured out of Acrobat — the same red Acrobat would
have given them.

⇒ The full picker is kept, underneath, because *"the ten Adobe uses"* is not
*"the ten that exist"* and an operator with a company standard colour must not
be told no. It expands **in place** rather than opening a second popup: a
popup inside a popup is two dismissal rules the operator has to learn.

## Why the swatch shows the colour rather than naming it

Because the operator is choosing a colour and the only useful preview of a
colour is the colour. The chip is a filled rectangle carrying the pen's
current value; its accessible name comes from the hover text, which is why
both swatches carry one.

**The alpha channel is not offered**, and [`super::pen::Pen::set_ink`]
carries the argument: a PDF annotation's `/C` is three components, and
feeding a picker's alpha into it would be a value with nowhere to go. The
`Opaque` variant of the picker is what says so, and it is why the grid's
cells are opaque too.

## Item notes

### `const REGION_MORE_COLOURS`

A salt rather than a region: it is `egui`'s persistence key for whether the
full picker is expanded, and it must be stable across frames or the section
would collapse itself every time the popup reopened.

### `const DASH_WIDTH`

Wide enough for its longest entry — *"Dashed (the file's own pattern)"* is
longer still but is a **reading** and never appears on this surface, because
the pen always holds one of the four. Sized against the two `DragValue`s
beside it rather than to look comfortable on its own: the Style group already
carries two chips and two numbers, and a fifth control that took a third of
the band would push the group into the overflow on a narrow window.

### `const CELL_PTS`

Sixteen, which is a little under the height of a ribbon button and a little
over the smallest target a pointer hits reliably. It is one constant for both
because a preview that is a different size from the cells it previews reads
as a different kind of thing — the chip *is* the cell that is currently
chosen, and the grid is where the other nine live.

### `fn chip`

# It is drawn rather than assembled from a `Button`

A `Button` fills with the *theme's* widget colour and paints its content on
top; what is wanted here is a rectangle of the **document's** colour, at a
known size, with a frame that keeps a white or near-white pen visible against
a light ribbon. `Button::fill` could be forced to the document colour, but
then hover and press would restyle the operator's ink — an application state
changing the apparent colour of an annotation, which is exactly what
`check-theme-colors.sh`'s document-colour rule exists to prevent.

So: the fill is the document's and never moves; the **frame** is the widget
visual and carries hover and focus. Two colours with two owners, which is the
distinction this project has already shipped wrong once.

# `CloseOnClickOutside`, not `CloseOnClick`

The default menu behaviour closes on any click inside, which would shut the
popup the instant the operator touched the *More colours…* disclosure or
dragged inside the picker — a picker that vanishes on its first drag is a
picker that cannot be used at all.

Picking a **cell** does close it, explicitly, via [`Ui::close`]: that is one
completed decision, and Acrobat's grid closes on a pick. The two behaviours
are therefore not inconsistent — clicking a cell is choosing, clicking the
disclosure is navigating, and only the first is finished.

# Returns

The **chip's** response, not the popup's. [`show`] discards it; the test that
asserts the popup opens needs `Popup::default_response_id` of exactly this
response, and there is no other way to name the flag the popup's open state
lives under — `Memory::any_popup_open` is `pub(crate)` to egui.

That is the same return, for the same reason, that `app::status::filter`'s
own `show` carries, and its header records why it is not a wasted one: the
alternative was a test that could only assert the chip exists, which is
precisely the claim that stayed true throughout the week a Select button did
nothing at all.

### `fn popup`

# Why the heading names Adobe

Once, at the top, and [`crate::text::markup::palette_heading`] carries the
argument: the values are Acrobat's, measured, and a grid captioned *Colours*
would look like ten colours somebody liked. It is the only place in the
running program where the provenance of these values is visible.

# Deliberately NOT `.strong()`

`tools/gates/check-strong-text.sh` rejects it and defect D11 is why: egui has
no role for emphasised text, so `.strong()` resolves to the accent-filled
widget state — pale text on a pale ground. The hierarchy here is carried by
the separator underneath, as it is in every other popup in this shell.

### `fn grid`

# The chosen cell is marked, and marking it is not decoration

A grid of ten colours with no indication of which one is current answers
*"what can I pick"* and not *"what did I pick"*, and the second is the
question an operator opening a colour popup usually has. The mark is a
**heavier ring in the theme's accent** — not a colour of this module's own,
because a fixed-colour marker would be invisible on the cell whose colour it
happened to match, which on a ten-colour grid is a one-in-ten chance of a
control that looks broken.


The ring was first drawn with `ui.visuals().selection.stroke`, which reads
like the obvious answer — *this cell is selected, use the selection stroke*
— and `tools/gates/check-selection-channel.sh` refused it by name. It is
right to: `egui::Visuals::selection` is how egui styles a **selected
widget**, it supplies the fill and text colour of every
`Button::selected(true)` in the application, and it is not a
general-purpose emphasis. Defect T2 is what happens when content code
borrows it — the theme repoints the channel to satisfy the borrowers and
every selected chrome control in the program is then painted with canvas
ink, with every gate still green because every colour involved was
correctly sourced from the palette.

⇒ **Correctly sourced, wrong role** — the same sentence this project has
already had to write about a colour that passed every check. What this cell
actually is is *chrome, an emphasised mark*, and the theme's name for that
is [`egui_shell::theme::Theme::accent_pair`]. Only the accent half is used:
the ring is a stroke, not a plate, so there is no `on_accent` to place on it
and `check-plate-colour.sh` has nothing to ask for.

# A cell counts as chosen only on an EXACT match

A pen the operator set through the full picker to a near-red is not the
palette's red, and marking the nearest cell would tell them they had picked
something they had not.

### `fn trace`

The whole pen rather than the field that moved, because what a harness needs
to assert is *what the next markup will be authored with* — and a line
carrying one field would need the reader to accumulate state across lines to
answer that. It is a handful of numbers.

### `fn every_control_publishes_a_distinct_region`

They exist so a harness can aim at one control out of five, and two that
shared a name would send it to whichever the application declared last —
a click on the wrong control, reported as the right one failing.

The list grew from three to five and the test name grew with it, on
purpose: `the_three_controls_publish_distinct_regions` would have gone on
passing while checking three of five, which is the shape of gate that
reports clean having looked at almost nothing.

### `fn click_at`

A press AND a release, because egui raises `clicked()` on the release and
a press-only frame would assert nothing about a click. Borrowed verbatim
from `app::status::filter`'s harness, which is where this shell learned
that a control can be perfectly laid out and completely inert.

### `fn pressing_the_swatch_opens_the_palette`

The whole of the *"style look"* half of the operator's ask, reduced to the
one thing that can be false about it. Everything else in this module —
the grid's geometry, its names, its measured values — is worth nothing if
the popup never opens, and *"the chip is drawn in the right place and
does nothing"* is a state this project has shipped before and describes in
`app::status::filter`'s header at length: 1,628 tests, 17 gates and an
off-screen launch all confirmed a Select button's rect while the button
itself was inert for a week.

It asserts on `Popup::is_id_open` — the flag a duplicated
`Popup::toggle_id` would fight over — rather than on anything downstream.

Falsified by deleting the `Popup::menu(…)` block from [`chip`]: the
assertion fired. Restored.

### `fn pressing_the_swatch_again_closes_the_palette`

The other half of a toggle, and the half a careless fix breaks: deleting
a duplicate toggle could as easily be deleting *the* toggle, leaving a
popup that opens and cannot be dismissed from the control that opened it.

### `fn clicking_a_cell_sets_that_slots_colour`

The claim the whole module rests on, and the one a screenshot cannot
make: a grid that renders ten beautiful squares and writes nothing is
indistinguishable from a working one until an annotation comes out the
wrong colour in a saved file.

# Why the FIRST and the LAST cell specifically

Because they are the two whose position can be derived from the returned
bounds without re-deriving the layout: the first cell's top-left **is**
`bounds.min` and the last cell's bottom-right **is** `bounds.max`, since
[`grid`] unions exactly the cell rects and the grid is rectangular
(`palette::tests::the_grid_is_rectangular`). Aiming at a middle cell would
mean this test computing the spacing, which is the layout asserting
itself.

They are also the two that matter: an off-by-one in the row loop puts the
last cell somewhere else entirely, and a reversed iteration swaps them.

Falsified by making [`grid`]'s click arm write `PenSlot::Shape` instead
of `slot`: the highlighter half of the assertion fired. Restored.

### `fn the_opacity_control_covers_the_whole_range_the_pen_allows`

Written because this module's header claimed for four months that it
could not exist — *"blocked on the engine … `/CA`, which `pdfcer-core`
does not write yet"* — while the widget was drawn thirty lines below the
claim. Nothing checked either statement, so the false one survived.

The assertion is deliberately about the **range** rather than about a
drag: it pins that the control's bounds are the pen's own
(`MIN_OPACITY..=1.0`, expressed as a percentage), which is the property
that would silently rewrite the operator's value if the widget's range
were narrower than what the pen may legally hold. A control narrower than
its value is the defect the settings window's own sliders document.

### `const REGION_INK`

One per part rather than one for the group: a harness proving that a colour
can be *changed* has to click the swatch, and a rect covering all three
controls would give it the wrong target two times in three.

### `const REGION_DASH`

It doubles as the combo's `id_salt`, which is deliberate and is the one
place in this module where a region name is load-bearing twice: a driven
check finds the control by this name, and `egui` remembers the popup's open
state under it. Two spellings of one control would give the harness a rect
for a widget whose popup lives under a different key.

### `const REGION_PALETTE`

Published only while a popup is open, which is the point: a driven check
asking *"did pressing the swatch show Acrobat's colours"* gets no rect at all
on the frame before the press, and a rect afterwards. A region that were
always present would answer the question the same way whether the popup had
opened or not — the exact failure `app::status::filter`'s own header records
from the day a Select button did nothing for a week.

### `fn show`

# It edits in place and raises nothing

No `Action`, no `HandlerToken`, no return value. The funnel's invariant is
that no code path runs from a widget to a **document**, and this touches no
document: it sets the pen the *next* gesture will use, which is application
state with no undo log to order against and nothing to alias. The same
argument `crate::dialogs::print` makes about spooling, one size down.

# Horizontal, and narrow on purpose

A ribbon group is a band about 70 points tall, and three stacked rows would
not fit. More usefully: these three are read together — *what colour, how
thick* — so a row is what an operator scans, and the ribbon's own group
caption underneath says which group they are in.
