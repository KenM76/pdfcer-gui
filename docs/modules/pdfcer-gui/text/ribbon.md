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

## Item notes

### `fn every_tab_question_is_one_question`

Not a tautology: the question is the coherence test `RIBBON_IA.md`
§4 applies to a tab, and a "question" that is really a description
("Things you can do with files") passes a non-empty check and fails
the test it exists to be. Requiring the question mark is the
cheapest mechanical proxy for "this is one question".

### `fn tab_labels_are_distinct`

Two tabs with one label is a navigational dead end that no test in
`egui-shell` can catch — the manifest's uniqueness rules are about
*ids*, and two tabs may legally carry the same label as far as the
framework is concerned.

### `fn the_three_modes_are_named_in_capability_order`

Pinned because the *order* is the feature: the selector renders
them left to right and "slide left to calm the interface down" is
only an obvious gesture if Read is on the left.

### `fn question_format`

`RIBBON_IA.md` §5.8 describes the contextual tab's purpose but does not
give it a one-line question, because the table in §4 lists only the
seven ordinary tabs. This sentence is written to the same test: the tab
appears while something is selected and carries what the operator
changes *about that thing*, so the question names the selection.

### `fn group_file_file`

A group whose caption repeats its tab's label reads oddly in isolation
and correctly in place: the band under the File tab that holds the
file-level verbs (open, close) as distinct from saving, exporting and
printing, which are their own bands. `RIBBON_IA.md` §5.1 names it this
way and the alternative — `Document` — is already taken by the group
that holds Properties and Fonts.

### `fn group_file_recognise`

`RIBBON_IA.md` §5.7's own caption, kept rather than improved on. Two things
were considered and rejected: **`OCR`**, which is the acronym the operator
would search for but is jargon in a caption position where every other band
on this tab is a plain English noun; and **`Text`**, which already means
something else in this program — the text tool, the text markup band, text
editing — and would be the third meaning of one word on one ribbon.

`Recognise` is the verb the operation actually is, and the command's own
label carries `text` so the word an operator scans for is still on the
control.

British spelling, matching every other operator-visible string in this
crate (`Recognise`, `Colour`, `Centre`).

### `fn group_file_pdfcer`

Lower-case, because that is how the product's name is written
everywhere else, including the window title. A caption is not a
sentence and does not get a capital it would not otherwise have.

### `fn group_view_navigate`

Separate from Zoom because a tool is a *mode* the page is in, not an
action taken on it: pressing Hand changes what every subsequent drag
means, while pressing Fit page happens once and is over. Putting the
two in one group would caption a radio and three buttons with one
word that is true of neither.

### `fn group_rail_select`

*"the navigate selectors and some other related selection controls (lasso
tool when we implement one, etc)"*. His word for the group, and the group
it captions is the one the lasso joins when the lasso exists.

Drawn only at the rail's widest rung; see
`egui_shell::dock::rail::Rung::Tight` on why the word is the first thing
the strip gives up.

### `fn group_pages_clipboard`

The same caption as Edit ▸ Clipboard, deliberately. Two bands on two tabs
with one name is normally a smell; here it is the point — an operator
looking for a clipboard finds a band called Clipboard on whichever tab they
happen to be on, and the tab tells them what it acts on.

### `fn group_pages_organise`

British spelling, matching `RIBBON_IA.md` §5.3 and the rest of this
project's prose ("Sanitise", "Optimise", "Recognise"). A catalog that
mixes spellings is a catalog nobody can search.

### `fn group_edit_clipboard`

**Back on 2026-08-19**, and the note below — which explains why it was
deleted — is kept verbatim because its reasoning was right and only its
premise expired. It ends *"the next author of an object clipboard needs the
word — which is right here"*, and that is exactly what happened: the word was
read out of that comment and put back into this function.

It is a small vindication of a rule this project applies everywhere and that
is easy to feel silly about at the time: **delete the code, keep the
reasoning.** A dead `pub fn` would have compiled for five days and quietly
invited the group back before there was anything to put in it; a comment
could not, and was still there when there was.

### `fn group_edit_forms`

One band, where the salvage source had two (`Forms` and `Build Form`).
Splitting filling from authoring put `Fill Form` and `Create Field`
under different captions on the same tab, which is a distinction the
operator has to already understand to use.

### `fn group_markup_style`

Changing an existing markup's style happens on the contextual Format
tab; `RIBBON_IA.md` §5.5 is explicit that both surfaces must exist and
that having only this one is why a placed markup currently feels final.

### `fn group_tools_batch`

The salvage source called this band `Across files`, which described it
accurately and sorted badly: a caption that begins with a preposition
cannot be scanned down a column of captions. `Batch` is the word
`RIBBON_IA.md` §5.7 uses and the word the operation is called
everywhere else.

### `fn group_format_font`

**"Font", not "Text" and not "Type"**, and the choice is the operator's
rather than this file's. Word calls this group *Font*; so does every office
suite that copied Word, which is all of them. `RIBBON_IA.md` §5.8 lists the
controls individually — *Font · Size · Colour · Spacing · Alignment* —
without naming the band they sit in, so the name is ours to pick, and the
convention of the product class is the specification wherever the
specification is silent.

It is deliberately **not** disambiguated to something like *"Text style"*
on the grounds that the first control inside it is also called Font. Word
has exactly that repetition, has had it since 2007, and nobody has ever
been confused by it: the caption names the subject and the control names
the property.

### `fn group_format_selection`

`RIBBON_IA.md` §5.8 varies the Format tab's groups by *selection type* —
a markup gets Colour/Fill/Line width/…, a dimension gets
Group/Scale/Precision/…. Most of those property editors still do not exist
(§5.8: "build order: panel first, tab second"), and the ones that never
will until `EditSession` grows a verb are named in `manifest::PLANNED`. So
this band carries what can be done to any selection, whatever it is.

It stopped being the tab's **only** band on 2026-08-27, when the text
run's row of §5.8's table shipped as [`group_format_font`]. That is worth
noting here rather than only there, because this caption's own doc comment
used to assert *"the tab ships with the one band whose content is real"* —
a sentence that was true, that nothing would have failed if it had been
left, and that a reader would have believed.

### `fn group_format_markup`

`RIBBON_IA.md` §5.8's *Markup annotation* row: *Colour · Fill · Line width ·
Line style · Opacity · Arrowheads · Note text · Delete*. Five of those eight
are built ([`crate::app::markupband`]); Line style and Note text stay in
`manifest::PLANNED`, and Delete is the [`group_format_selection`] band's,
which is where §5.8 puts it for **every** selection type.

# "Markup", not "Mark", "Annotation", "Shape" or "Style"

- **"Annotation"** is the PDF word. `crate::text::paint`'s rule for this
  catalog — *"'Fill' and 'Line', not 'fill' and 'stroke'; stroke is the PDF
  word"* — refuses it for the same reason, and the Markup **tab** already
  made this exact choice: `RIBBON_IA.md` renamed §5.5's tab from *Review* to
  *Markup* because *"`Markup` is also the word this project's audience
  uses."*
- **"Shape"** is narrower than the group. A highlight and a sticky note are
  markup and are not shapes, and the colour and opacity controls act on
  both.
- **"Style"** is what the Markup tab's own pen band is called, and it edits
  the pen the **next** gesture will use. Two groups called Style, one
  changing what you are about to draw and one changing what you already
  drew, is a distinction an operator would have to learn from the
  consequences.

⇒ It matches the tab the marks were placed from, which is the association
worth having: the band that made this mark is called Markup, and so is the
band that changes it.

### `fn markup_no_fill`

# Why this is a named state and not an absent control

`canvas::markup::spec` authors every shape with `interior: None`, and its
reason is quoted in `panels::properties::markup`'s header: *"a filled
comment shape hides the drawing it is a comment about, which on a CAD sheet
is the whole content under it."* Acrobat's default is the same. So **no
fill is where every mark starts**, and a fill control that could only ever
set one would be a one-way door: try a fill on a drawing, decide against it,
and there is no way back to the mark you had.

*"No fill"*, not [`crate::text::panels::properties::markup_clear`]'s
*"Clear"*, although both raise `StyleEdit::Clear`. "Clear" is honest about
the **act** — it removes the key, and what applies afterwards is the
standard's default rather than anything pdfcer remembers — and that is the
right word beside a colour the operator set. Beside a fill it is the wrong
word twice over: *clear* also means *transparent*, which is what the result
looks like, and the state has a name every drawing program already uses.
It is what Word, Illustrator and Acrobat all call it.

### `fn markup_endings_none`

# Positions, not pairs

`/LE` is two independent endings (§12.5.6.7, Table 176) over three shapes
each — nine combinations, which is not a list anybody reads on a ribbon
band. [`crate::app::markupband`] offers the four *positions* an operator
means and **preserves the shape** the mark already carries, so a closed
arrowhead stays closed and an open one stays open. These are the names of
those four positions.

The order is *fewest endings first*, which is also increasing commitment
and is the same reading order §5.8's menu rule gives a group. It puts the
state pdfcer authors — a head at the end only, `(None, OpenArrow)` in
`canvas::markup` — third rather than first, and that is correct: the list is
ordered by what it does, not by what is common, because an operator scanning
four entries for "both" should find it at the end every time.

*"Start"* and *"end"*, not *"first point"* and *"last point"* and not
*"tail"* and *"head"*. The first pair is the file's vocabulary (`/L` is
`[x1 y1 x2 y2]`), the second is a draughtsman's, and the operator's is the
direction they dragged: an arrow points where the drag finished.
