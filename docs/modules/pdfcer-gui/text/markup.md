# `text::markup` — the words the Markup ▸ Style group shows

Five tooltips, two suffixes, **ten colour names** and the **five names a
line style goes by**, which is the whole operator-visible surface of
`canvas::markup::swatch` and of `canvas::markup::linestyle`. Most of the
controls are colour chips and numbers: none of those can carry a label
without doubling the width of a ribbon group, so **the tooltip is the only
place they say what they are** — which makes these strings load-bearing
rather than supplementary.

★ The line-style names are the exception, and they are here rather than in
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
