# text::ribbon — the ribbon's *structural* strings

Tab labels, the one-line question each tab exists to answer, group
captions, the three mode labels, and the words inside the band's own
non-button controls. Everything a person reads on the
ribbon that is **not** a command; command labels and tooltips live in
[`crate::text::commands`], which is a much longer file for a reason
that is worth stating: there are eight tabs and thirty-seven groups, and
there are a hundred and twenty commands. Splitting the catalog along that seam
keeps both halves navigable and both files inside the project's
1,500-line ceiling.

## Why the "question" is a first-class string

`RIBBON_IA.md` §4 keeps an idiom from the salvage source: every tab
carries a one-line question it exists to answer — *"What is on my
screen, and how is the page laid out?"*. In the old crate that
sentence was the tab's hover tooltip. It is kept here for the same
reason it was written, which is not decoration:

> A tab whose question cannot be written in one line is a tab carrying
> two unrelated jobs.

That test is what split six tabs into seven in `RIBBON_IA.md` — the
old File tab could not answer one question, because it held Properties,
text copying, DXF export, print, panel-layout reset, settings and the
shortcut list. Keeping the sentence in the catalog keeps the test
visible to whoever adds the next command.

## Voice

The questions are written in the **operator's** first person — "What do
*I* do with the file" — exactly as `RIBBON_IA.md` §4 writes them, and
not in the old crate's third-person descriptive voice ("What you do
with the file as a whole, and with pdfcer itself: open, save a copy,
copy text out, …"). The old form drifted into an enumeration of the
tab's contents, which stops being true the moment a command moves and
is a second place to maintain the ribbon.

Group captions are **sentence case**, per the catalog convention in
[`crate::text`]: `Page display`, not `Page Display`. That is a change
from the salvage source, which mixed the two within one tab (`Across
files` beside `Build Form`).
