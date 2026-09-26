# `swatch` — one colour control, three honest states

`OPERATOR_REQUESTS.md` **O89**, both pieces. It is the widget behind the
clicked-text colour ([`super::textobject`]) and behind the multi-object
fill/line colour ([`super::paint`]), and it exists because those two
surfaces have to answer the same three questions the same way:

| state | drawn as | picking a colour |
|---|---|---|
| the selection agrees | a swatch of that colour | sets it |
| the selection **disagrees** | the indeterminate plate and a dash | sets **all** of them |
| the colour cannot be shown | *nothing — the caller draws a sentence* | — |

The third row is a caller's job on purpose. What to say about an ink
pdfcer will not overwrite differs between a path (whose `/Separation` name
the file records and `pdfcer_core::vector::PathPaint::Other` carries) and a
text run (whose `pdfcer_core::text_extract::TextColor::Other` carries **no
name at all**), and a widget that tried to word both would word one of them
wrongly. This widget refuses to draw a control; the sentence that stands in
its place belongs to whoever knows what the ink is.

## Why "mixed" exists at all, and why it is not an invention

O89 recorded multi-object recolouring as *not offered*, with a reason:

> *"when the objects disagree there is no honest colour to open on and
> picking the first one's would quietly propose flattening the rest to it."*

That reasoning is right about the danger and wrong about the conclusion.
**Every editor in this product class already solved it** — Illustrator,
Inkscape, Figma and Word all show an *indeterminate* control over a
disagreeing selection, and applying a value sets every member. This
project's standing rule is that *the convergence of the product class IS the
specification, and an invented interaction is a defect even when it works*,
so the mixed state is the answer rather than one of several.

And the em dash is not a new marker either: this shell already writes one
for *no value*, in `crate::text::panels::properties::text_value_absent`,
whose own doc comment makes precisely this argument — *"every property grid
in this class shows a blank or a dash for no value and for mixed values,
which are the same state as far as a single field is concerned."*

## ONE UNDO STEP PER GESTURE, and why this widget could not be
## `ui.color_edit_button_srgb`

This is the load-bearing reason the widget is hand-built.

`egui`'s own colour button marks its response **changed on every frame of a
drag inside the picker** (`color_edit_button_hsva` calls
`button_response.mark_changed()` from inside the popup body, on every frame
`color_picker_hsva_2d` returns `true`). A caller that acts on `.changed()`
therefore authors **one document edit per frame** while the operator drags
across the saturation square — sixty content-stream rewrites a second, sixty
undo entries, and a `Ctrl+Z` stack the operator cannot get back through.

That is the same defect `super::text`'s size field already avoids by
committing on `drag_stopped`/`lost_focus` and never on `.changed()`, with
the same stated reason. A colour popup has no `drag_stopped` to hang it on,
so the equivalent event has to be *the popup closing* — which means owning
the popup id, which means owning the button. Hence this module.

⇒ [`show`] returns `Some` **exactly once**, on the frame the picker closes,
and only if the operator actually moved it. Open-and-close-without-touching
returns `None`, so idly inspecting a colour never writes to the document.

## Rule 4 — nothing here reaches the canvas

This widget draws in a dock panel. It marks no page, tints nothing, and
renders no preview: the document changes only when the caller's action is
applied, and from that instant the canvas shows exactly what the saved file
will show. What was skipped, and why, is disclosed off-canvas by the caller.

## Theme

The two colours the indeterminate state needs come from
`egui_shell::Theme::indeterminate_pair`, as a **pair**, for the reason that
function's own doc comment gives: `tools/gates/check-theme-colors.sh`
forbids invented values and cannot forbid a wrong role, and picking two
roles that happen to look right is how this project shipped defect D2 three
times. The swatch's own fill is not a theme colour at all — it is the
operator's document content — and says so on the line.

## Item notes

### `const ASPECT`

Wider than tall, which is what a colour *swatch* looks like everywhere
this operator works — Word's font-colour button, Illustrator's fill chip,
SolidWorks' line-colour control. A square reads as a button with a coloured
glyph; a bar reads as a sample of the colour itself.

### `fn disc`

Hand-drawn rather than a `Button` with a rounded corner radius, because a
rounded rectangle at this size reads as a button with a tint and the point
of the shape is that the operator sees a **disc** — the thing a radio
button's background will actually be. It keeps the button's own interaction
(a click opens the popup) by allocating an interactive rect of the same size
and painting into it.

### `struct Editing`

In `egui`'s temp data rather than on a draft struct, and that is a
deliberate difference from [`super::text::TextStyleDraft`]. A draft holds a
*reading of the document*, which has to be invalidated when the document
moves; this holds *where the operator's finger is*, which is meaningless the
moment the popup closes and must never outlive it. Storing it beside the
document reading would invite a stale finger position to be read as a value.

### `fn mixed_carries_no_colour`

The one assertion this type exists for. If [`Value`] ever gained a way
to represent "mixed, and here is a colour anyway", the next caller would
pass the first member's — which is precisely the flattening O89 refused
to ship, and it would look completely normal while it happened.

### `enum Value`

Two variants and not three: *"there is no colour to show"* is not a value
this widget can draw, so it is not a value this widget accepts. See the
module header on why the sentence for that case belongs to the caller.

### `fn show`

`id_salt` must be unique within the `Ui` — the fill and the line swatches on
one panel are two controls and must not share a popup. `region` is published
for a driven check.

# `mixed_hint` is a PARAMETER, and it was a hard-coded string for about
# twenty minutes

The sentence shown at the top of the picker over a disagreeing selection
names its subject — *"These **words** are not all one colour"* — and this
widget serves two subjects. The first draft read
`crate::text::panels::textobject::mixed_hint()` inline, which put a sentence
about words over a selection of **paths** on the vector row. It was caught
by reading the call sites rather than by any test, and no test could have
caught it: both strings compile, both render, and the wrong one is grammatical.

⇒ Same rule this module's header already states for the ink refusal: **the
sentence belongs to whoever knows what the selection is made of.** The widget
draws controls; it does not name subjects.

### `enum MkValue`

Deliberately **not** [`Value`], and the difference is the subject.
[`Value`] models a *selection of document objects*, which has exactly two
states — they agree or they do not. One widget's `/MK` `/BG` is one key on
one dictionary, and Table 189 gives it four: absent, the empty array that
states *no colour*, a DeviceGray or DeviceRGB value, and a DeviceCMYK
separation. Two of those four are colours no swatch can draw, and one of
them is not a colour at all.

So this enum carries the one state a swatch CAN draw and defers the rest to
the caller, exactly as the module header requires: *"the sentence that
stands in its place belongs to whoever knows what the ink is."* Which of the
three unshowable states a widget is in is a fact about `/MK`, and `/MK` is
not a thing this file knows about.

### `fn show_mk`

Returns `Some` **only** on the frame a choice was made: the picker closed
after the operator moved it, or the *no colour* entry was pressed. The
commit rule is [`show`]'s, for [`show`]'s reason — the module header's
sixty-content-stream-rewrites-a-second defect — and the two functions share
[`Editing`] so there is one implementation of it rather than two that drift.

The *no colour* entry commits **immediately** rather than on close, and
that is not an inconsistency: it is a discrete press, not a drag, so there
is no run of intermediate values for a close-edge to collapse. It closes the
popup itself, because `PopupCloseBehavior::CloseOnClickOutside` would
otherwise leave a picker open over a widget that no longer has a colour.
