# `pdfcer-gui/text/panels/properties/markup`

## Item notes

### `fn markup_subtype`

The file's own `/Subtype`, translated. An operator placed a *rectangle*
and the file calls it `Square`; they placed an *arrow* and the file calls it
`Line`. Showing the file's word would be correct and useless — the standing
rule in `text::commands` is that a label is the operator's vocabulary and an
id is the format's.

### `fn markup_line_style_label`

*"Line style"* is `RIBBON_IA.md` §5.8's own name for the row and is what
the Format tab's command is called, so the two surfaces agree. It sits
directly under *Line width*, and the shared first word is doing work: the two
rows are one subject and read as a pair.

Not *"Dash pattern"*. The chooser's first entry is **Solid**, and under a
label reading *Dash pattern* that entry would read as *no dash pattern* — the
absence of the thing the label names rather than one of its values. The entry
names themselves are `crate::text::markup`'s, because three surfaces show
them and only one of the three is this panel.

### `fn markup_opacity_suffix`

A percentage, because that is the unit every application an operator has
used states opacity in. `/CA`'s own `0.0..=1.0` is a file-format detail they
should never meet.

### `fn markup_clear`

*"Clear"*, not *"Reset"* or *"Default"*. It removes the key from the
annotation dictionary, and what happens then is that the **standard's**
default applies — which is not necessarily what the mark looked like when
the operator placed it. "Reset" would promise a return to a previous state
that pdfcer does not remember.

### `fn markup_note`

Two facts an operator cannot see and would otherwise discover from a
changed file:

1. **The appearance is regenerated.** `set_markup_style` redraws the mark
   from the geometry pdfcer models, so anything the original expressed
   *outside* that model — a border effect pdfcer does not author, a producer's
   own decoration — is gone from the new appearance even though its
   dictionary key survives. The engine reports each one, and those arrive
   verbatim on the status row; this sentence is the standing warning that
   such a report is possible at all.
2. **A wider line moves the box.** For every subtype except a rectangle and
   an ellipse, `/Rect` is derived from the geometry plus a margin that
   contains the stroke and any arrowheads — so widening the pen makes the
   annotation's rectangle bigger. That is the engine's own ⚠, and it is the
   difference between a mark that looks the same and a mark that occupies
   the same space.

### `fn markup_locked`

Names the standard, because an operator who meets this wants to know whether
pdfcer is refusing or the document is — and it is the document. It also names
the one thing that is still possible, which is the rule a refusal follows
everywhere in this shell.

### `fn markup_not_restylable`

The reachability test behind it, and why R9 wants a sentence here rather
than an empty space, are in `crate::panels::properties::markup`'s header.
This doc records the part that belongs to the **words**: each claim below
is checked against a named engine verb, because a limitation sentence
nobody checked is the one way this string can lie.

⚠ **This sentence is only reached where NEITHER style verb reaches.**
`set_text_annot_style` restyles subtypes `set_markup_style` refuses, and
[`crate::text::panels::textannotstyle`]'s header has that account. A claim
here that the mark cannot be restyled at all would be false for those.

- **move** — `move_annotation` refuses a ce dimension and a form widget by
  name, then works from `/Rect` and whatever geometry keys are present.
- **resize** — `resize_annotation` refuses the same two and otherwise
  **carries** a foreign appearance rather than rebuilding it; a uniform
  scale is exact. (A non-uniform scale of a foreign appearance is refused
  unless distortion is allowed — that refusal arrives from the engine with
  its own message and is not this sentence's subject.)
- **delete** — [`markup_locked`] already promises it, and
  `crate::panels::properties::annotdelete` speaks for any annotation: the
  verb is document-wide rather than per-subtype.
- **the note** — `set_markup_note` refuses a ce dimension and a widget, and
  nothing else.

### `fn markup_fill_label`

*"Fill"* rather than *"Interior"*: `/IC` is the format's word and every
drawing application an operator has used calls it fill. The standing rule in
`text::commands` is that a label is the operator's vocabulary and an id is
the format's.

### `fn markup_fill_none`

It exists because a swatch cannot show *absence*. With no `/IC` the swatch
falls back to black, and a black square beside the word "Fill" says "this
shape is filled black" — which is the opposite of the truth. Acrobat draws a
red diagonal through its no-colour swatch for exactly this reason; this
shell says the word instead, which survives a theme change and a screen
reader where a drawn diagonal does not.

### `fn markup_line_ending_name`

Table 176 names ten endings and pdfcer authors three of them —
`annot_author::LineEnding` has exactly `None`, `OpenArrow` and `ClosedArrow`,
and its own doc comment calls the rest "a documented not-yet-authored
remainder". The chooser offers what the engine can draw, because a
fourth entry that produced a `/Butt` the appearance did not show would be
the inert control this project forbids, one level down.

The words are the operator's rather than the file's: *"Open arrow"*, not
`/OpenArrow`.

### `fn markup_line_ending_note`

It is owed because the readback is **lossy in one direction and silent
about it**: `annot_author::read_line_endings` degrades any Table 176 ending
pdfcer does not author down to `None`, so a `/Line` a foreign producer gave
a `/Butt` or a `/Diamond` end reads here as *No end* — and the mark on the
page plainly has one. Without this sentence the operator's conclusion is
that the chooser is broken.

### `fn markup_endings_clear`

# Two states, one picture, and why the operator is offered both

`MarkupStyle::endings` is a `StyleEdit` rather than a plain value: `Set`
writes `/LE`, and `Clear` **removes the key**. Both are needed because
*"draw no arrowheads"* and *"have no `/LE` at all"* are different
documents. With only the first, an operator who turned an arrow's heads off
would get a file differing from the one they opened in a key neither this
panel nor the Format tab shows.

Table 176 makes `/None` the default for both ends, so `/LE [/None /None]`
and an absent `/LE` draw the same line. The difference is **bytes**, and
this project does not treat different bytes as the same document. The
engine's reply names the argument that decided it over a cheaper
doc-comment fix: a **signed** drawing, and the question *"is this
byte-identical to what my client sent me"* — a question undo does not
answer, because undo covers the session and not the round trip.

# Why these words

*"Clear"* is the verb this panel already uses for `/C`, `/IC` and `/CA`, and
[`markup_clear`] argues it: the word is honest about the **act** — the key
goes, and what applies afterwards is the standard's default rather than
anything pdfcer remembers — where *"Reset"* or *"Default"* would promise a
return to a previous state pdfcer never recorded.

It carries a noun where the other three do not. A bare *"Clear"* in a list
whose first entry is already *"No arrowheads"* reads as a second name for
that entry, which is the one misreading this control cannot afford: those
two produce identical pictures and different files, so an operator who
confuses them cannot discover the mistake by looking.

⚠ It deliberately does **not** say `/LE`, *key*, or *dictionary*. The
operator is a drafter; the fact they need is *the file goes back out the way
it came in*, and [`markup_endings_clear_hint`] says exactly that.

### `fn markup_endings_clear_hint`

The hover carries the whole distinction, because the control cannot: the
two states are indistinguishable on the page and the difference only shows
up in a byte comparison of the saved file. A caveat below the thing it
qualifies arrives after the operator has drawn their conclusion, which is
why it is a hover on the control rather than a note underneath it.

It names the **consequence** an operator has met (a drawing that comes
back different from the one that went out) rather than the mechanism (a
dictionary key). Both surfaces read this one string, so the Format tab and
this panel cannot come to explain the same act two different ways.

### `fn markup_colour_narrowed`

§12.5.2 lets `/C` and `/IC` be a 0-, 1-, 3- or **4**-component array, and the
four-component case is CMYK, which is not rare on a CAD sheet where the
producer is plotter-bound. The swatches convert one for display and a change
made through them writes RGB in its place, which is a real narrowing of the
colour space and is disclosed rather than performed quietly — the engine's
own posture on every conversion it makes.

The full argument, including the refuse-to-show behaviour this replaced and
why that was worse than an approximation, is on
`crate::panels::properties::markup`'s `swatch_of`.

### `fn markup_dropped`

`set_markup_style` redraws a mark from the geometry pdfcer models, so
anything the original expressed *outside* that model is gone from the new
appearance even though its dictionary key survives. The engine names each
one; this is the sentence that reaches the operator, and it is owed under
rule 4's surviving half — **an inference the operator cannot see still owes
an off-canvas report.**

Every sentence says **what they will see**, not what a key is called. An
operator who is told *"the `/BE` border effect was dropped"* has been told
nothing; one who is told *"its cloudy edge is now a plain outline"* can look
at the page and decide whether they mind.

# ⚠ THESE VARIANTS FIRE NARROWLY, AND A SENTENCE WRITTEN FOR THE WIDE CASE
# IS A FALSE DISCLOSURE

`DroppedProperty::BorderStyle` and `::DashPattern` fire far less often than
their names suggest, because `pdfcer-core` reads a dashed border back and
re-authors it rather than solidifying it. **A disclosure whose wording was
written for a wider case than the one that fires is a false disclosure**,
which rule 4 forbids in exactly the direction it forbids silence — so what
each variant means is read out of the emission site rather than assumed:

| variant | fires when |
|---|---|
| `BorderStyle` | `/BS` `/S` names a style pdfcer does not redraw — `/B`, `/I`, `/U` — **or `/S /D` whose dash was not carried** |
| `DashPattern` | `/BS` `/D` is present and the dash was not carried: an array §8.4.3.6 does not admit, or a caller that **cleared** it |

⚠ **The `BorderStyle` string must name the dash case and not only a bevel,
an inset and an underline.** An operator who presses **Solid** on a dashed
mark clears the dash, the original dictionary still says `/S /D`, and both
variants fire; a sentence naming only the three redraw failures would tell
that operator their mark had a *bevel*. Both sentences are therefore
phrased as facts about the new outline rather than about a cause, so each
is true whether the change was asked for or merely disclosed.

The redundancy on a requested clear is accepted rather than engineered
away. Suppressing a disclosure when the shell believes the operator asked for
it would put the decision *"was this loss requested?"* into
`app::actions::apply`'s routing arm, which that arm's own note forbids — it
routes and does not compute — and a suppression rule that got it wrong would
hide a real loss. A true sentence twice beats a missing one once.
