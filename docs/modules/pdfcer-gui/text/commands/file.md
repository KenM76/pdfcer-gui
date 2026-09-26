# `pdfcer-gui/text/commands/file`

## Item notes

### `fn file_export_image`

# The label is `RIBBON_IA.md` §5.1's, and the tooltip is not the label again

§5.1's Export band already carried the row — *Export image… (PNG/JPEG/TIFF,
DPI picker)* — so the label is settled rather than chosen. What the tooltip
has to add is the thing a label cannot say and an operator cannot find out
by pressing: **which formats are actually offered**, because that is the
decision they are about to make and one of the three names in the IA row is
not one that shipped. TIFF has no encoder in the engine; SVG arrived after
the row was written and is the one that makes his second sentence — *"copy
and paste vector graphics into word or inkscape"* — possible at all.

**And it names transparency**, because that is the half of the request he
put in a parenthesis and an exclamation mark — *"(including transparency
where supported!)"* — and a tooltip that omitted it would leave him pressing
the control to find out whether the thing he asked for by name is there.

### `fn file_export_text`

# The tooltip's job is the difference from its two NEIGHBOURS

This control sits in a band that already contains `Copy page text` and
`Copy document text`, and an operator scanning it is entitled to know why
there are three ways to get words out of a drawing. The answer is one word —
**a file** — so the tooltip leads with it.

# And it names the one thing that makes the export empty

A scanned drawing has no text layer, so this export finds nothing on it. The
receipt says so afterwards and names `Recognise text`, but a tooltip that
warned nobody would let an operator press the control, wait for an
extraction, and be told no. The sentence is short and it is the one an
operator with a plotted sheet needs before pressing.

⇒ **The tooltip deliberately says nothing about importing text.**

⚠⚠ **THAT PARAGRAPH WAS TRUE FOR TWO DAYS AND IS NOW FALSE.** It read:

> *"…because nothing imports text: `pdfcer-core` offers no route from a text
> file back into a PDF. See `crate::app::actions::exporttext`'s header. A
> tooltip that mentioned a round trip would be the promise R9 forbids."*


The tooltip below still does not mention it, and that is now a *wording*
decision rather than an honesty one: a tooltip's job is its own control, and
the pair is expressed by the two commands sitting next to each other in the
band — which is how `file.export_form_data` and `file.import_form_data`
already say it.

### `fn file_import_text`

# The half of the operator's ask that was missing for two days

> *"also the engine can export PDFs as text. we should have export/import
> for that."* — 2026-09-04


# Why the label says *pages* and not *text*

Because that is the consequence the operator is choosing. This does not put
words onto an existing sheet — it **makes new sheets** and adds them to the
document, and a label reading *"Import text…"* invites the reading that a
selected page is about to be overwritten. The engine offers only the
paginating shape; it declined to build page-level replacement, and this
label is where that decision meets the operator.

# What the tooltip must say before the press, and what it cannot

It says the two things a chooser cannot: **new pages are created** and **the
text is set in one standard font**, neither of which is visible in a file
picker. Everything countable — how many pages it became, paragraphs split
across a page break, characters the font could not write — is a fact about
*this* file and arrives afterwards, off-canvas, from the report.

That is `file_export_text`'s own two-part shape, mirrored: standing losses
in the window, counted ones in the receipt.

### `fn file_save_as`

The tooltip's job is the **difference from its neighbour**: the two labels
are one word apart and the acts are not. It says so in the tooltip rather
than only in the receipt, because the receipt arrives after the bytes are
written, and a control whose consequence is explained only afterwards is one
the operator learns by being surprised.
