# `text::markup` — the words the Markup ▸ Style group shows

Five tooltips, one suffix, **ten colour names** and the **five names a
line style goes by**, which is the whole operator-visible surface of
`canvas::markup::swatch` and of `canvas::markup::linestyle`. Most of the
controls are colour chips and numbers: none of those can carry a label
without doubling the width of a ribbon group, so **the tooltip is the only
place they say what they are** — which makes these strings load-bearing
rather than supplementary.

The line-style names are the exception, and they are here rather than in
`text::ribbon` or `text::panels::properties` for a reason worth stating:
**three surfaces show them** — the pen that authors, the Format ▸ Markup band
that restyles, and the Properties panel that restyles — and a name that lived
on one surface would be re-spelled on the other two. `canvas::markup::linestyle`
is the one module all three read, and this is the one place its words live.

⚠ The count in that first sentence has been wrong before. It read *"Three
tooltips and one suffix"* while the opacity tooltip and the percent suffix
sat forty lines below it, added on 2026-08-28 without the header being told.
A count in prose is a claim nothing checks;
[`tests::the_header_counts_what_this_module_actually_holds`] now does.

## Each one answers "what will this change, and when?"

Because that is the question a swatch in a ribbon cannot answer by looking
like a swatch. Every tooltip here says two things: which markup the setting
applies to, and — the half an operator is most likely to get wrong — that it
applies to the **next** one rather than to anything already on the page.

`RIBBON_IA.md` §5.5 is explicit that these are two different surfaces:

> The `Style` group sets defaults for the next markup. Changing an
> *existing* markup's style happens on the contextual **Format** tab.

The Format tab's property editors are not built yet, so an operator who
recolours the swatch expecting the rectangle they just drew to change will
be disappointed — and the tooltip is the only thing standing between them
and concluding the control is broken. Saying "the next one" is therefore a
disclosure and not a nicety.

## Item notes

### `fn every_style_tooltip_says_it_applies_to_the_next_mark`

The disclosure this module exists for. `RIBBON_IA.md` §5.5 puts
"restyle what is already there" on the contextual Format tab, whose
property editors are not built — so an operator who recolours the swatch
expecting the rectangle they just drew to change has no other way to
learn otherwise, and would reasonably report the control as broken.

A test rather than a convention, because the natural edit when a tooltip
reads long is to cut its second sentence.

### `fn every_palette_cell_has_its_own_word`

A cell's name is its whole accessible label — see this module's palette
section — so two cells reading "Purple" would be two controls an operator
cannot tell apart by any means the program offers, hover included.

It also asserts each is non-empty, which is the failure a `const fn`
returning `""` produces: a cell with no tooltip at all, silently, on a
control that has nothing else to say what it is.

### `fn the_palette_says_where_its_colours_came_from`

Two disclosures that a shortening edit would take out first, and both are
the kind this project does not leave to convention:

* the heading is the only place in the running program where the
  provenance of these ten values is visible — the operator asked for
  *Adobe's* colours and is entitled to see the claim being made;
* white is invisible on a white page, and an operator who picks it sees a
  tool that has stopped working rather than a colour they chose.

### `fn the_header_counts_what_this_module_actually_holds`

It read *"Three tooltips and one suffix"* for four months after a fourth
tooltip and a second suffix were added. Nothing was broken by it and
nobody could have noticed, which is exactly the class of statement that
rots — a count in prose is a claim with no reader that verifies it.

Falsified by changing the header to say "five tooltips": the assertion
fired. Restored.

### `fn the_line_style_names_are_words_rather_than_arrays`

The first half is the ordinary anti-collision assertion: a combo whose
two entries read the same is a control an operator cannot use.

The second half is the one worth having. These names are the whole
reason `LineStyle::pattern`'s run lengths never reach an operator, and
the cheap way to add a fifth style is to name it after its array. This
asserts no name contains a digit — which is what a `[8 4]` or an
`8, 4` creeping into the list would trip.

Falsified by renaming *Long dash* to `"Dashed 8 4"`, which turned the
digit assertion red.

### `fn the_two_swatches_are_told_apart_by_their_words`

They are two controls sitting side by side with no labels, so identical
or near-identical hover text would make them indistinguishable — which
is the state the operator is already in before they hover.

### `fn highlighter_colour_tooltip`

A separate control and a separate sentence, because they are separate pens
— see `canvas::markup::pen`'s header. An operator who sets the ink to green
does not thereby want a green highlight, and a tooltip that said "the
markup colour" for both would suggest they had.

### `fn pen_width_tooltip`

Names the **unit** as well as the effect, because "2" on a ribbon is a
number without a scale — and points are what the PDF stores, so it is also
the number the operator would see if they opened the file in another
program.

### `fn pen_opacity_tooltip`

# Why this sentence names the CAD case rather than describing the slider

Because the reason to reach for it is specific and is not obvious from a
percentage: a comment sits on top of the thing it is about, and on a dense
drawing an opaque cloud hides the dimension it is drawing attention to. An
operator who has never used annotation transparency has no reason to guess
that, and a tooltip reading *"the opacity of the next mark"* would restate
the label.

# It says the mark stays selectable, because faint is not gone

The bottom of the range is a tenth, deliberately (`canvas::markup::pen`'s
`MIN_OPACITY` carries the argument), and at a tenth over dark linework a
mark can be hard to find with the eye. Saying it is still there and still
listed is the disclosure that stops a faint mark reading as a failed one.

### `fn opacity_suffix`

A percent sign, because opacity is the one property in this group an
operator already thinks about as a percentage — every other program that
offers it says 40%, not 0.4. The value written into `/CA` is the fraction;
the conversion happens at the control and nowhere else.

### `fn pen_dash_tooltip`

# Why this sentence is about the DRAWING and not about the dash

"Choose a dash pattern" tells an operator what the widget obviously is. What
they cannot see from the control is *when* it applies — this is the pen, so
it governs the **next** mark and not the one they are looking at — and that
is the half every tooltip in this module leads with, for the reason its
header gives.

It also names the one subtype family the setting does nothing for.
`MarkupOptions::dash` is *"ignored by the text-markup family"*: a highlight
is a colour wash and an underline is its own line, and neither draws a
`/BS` border for a dash to be in. The chooser is on the Style group beside
the pen colour, which serves the highlighter too, so an operator who set it
and then drew a highlight would otherwise be owed an explanation nobody
gave them.

### `fn line_style_solid`

*Solid*, not *None*. "None" is the word this shell uses for the **absence
of a property** — `markup_fill_none`, the arrowhead chooser's first position
— and a solid line is not an absence, it is a line. Table 166 agrees: `/S`
is a named border style, not a missing one.

### `fn line_style_dashed`

The plain word, because it is the plain case: an operator who wants "a dashed
line" and does not care which dash should find the entry they would have
named, and it should be the one the standard itself would have given them.

### `fn line_style_long_dash`

Named by its **appearance**, not by what it is conventionally used for. The
tempting name was "Hidden" — the draughting convention this pattern echoes —
and it was rejected for `text::markup`'s standing reason about the palette
cells: a mark drawn in it is not thereby hidden, and a name that describes a
convention rather than the thing on screen makes a claim about the operator's
drawing that the annotation does not make.

### `fn line_style_dash_dot`

Named for what it draws, for [`line_style_long_dash`]'s reason — the centre-
line convention it echoes is in `LineStyle`'s doc comment, where a reader who
wants the rationale is.

### `fn line_style_foreign`

# It names the FILE, and that is the whole job of this string

The engine preserves a foreign dash through a restyle that does not mention
one, so this state is not a defect and is not going to be corrected by
anything the operator does — it is simply what their producer wrote.
Showing *Dashed* for it would be the quiet lie the colour swatch's CMYK arm
was rewritten to stop telling: a control claiming a value that is not the
file's, which the operator would discover by pressing something else and
watching the pattern change.

The parenthetical is what keeps it from reading as an error. *"Dashed (the
file's own pattern)"* says **this is fine and it is theirs**; a bare
*"Unknown dash"* would read as damage and would send an operator looking for
a repair that is not needed.

### `fn colour_violet`

**Violet, not purple**, and the difference is worth the thought it took.
`#9643FC` sits on the blue side of purple, and the two neighbouring cells are
Blue and Magenta — so an operator scanning for "the purple one" between a
blue and a magenta gets no help from a word that could mean either. Violet
names the position in the spectrum, which is how the cell is found.

### `fn colour_pink`

Acrobat's strikeout colour, which is a light desaturated red. "Light red"
would be the accurate description and is the wrong label: it puts two cells
called Red and Light red side by side in a grid, which is a distinction the
eye has to make twice. Pink is the word for it.

### `fn colour_white`

The one cell whose tooltip earns a second clause. A white mark on a
black-on-white CAD sheet is invisible everywhere except over the drawing's
own linework, so an operator who picks it by accident sees a tool that has
stopped working. Saying so at the moment of choosing is cheaper than the
support question.

### `fn palette_heading`

It names **Adobe**, deliberately and once. The operator's ask was for
Acrobat's colours specifically, and a grid captioned "Colours" would look
like ten colours somebody liked. This is the one place the provenance of the
values is visible from inside the program.

### `fn more_colours`

The trailing ellipsis is the platform convention for *"this opens
something"* and is load-bearing here: every other cell in the popup applies
immediately, and this one does not.
