# `text::markup` — the words the Markup ▸ Style group shows

Five tooltips, two suffixes, **ten colour names** and the **five names a
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
