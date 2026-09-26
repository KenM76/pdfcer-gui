# `text::import_text` — every word the **Import text as pages** window says


> *"also the engine can export PDFs as text. we should have export/import
> for that."*

## The window's job, and it is not the same as the export window's

`text::export_text`'s header sets out a two-part shape: **standing** losses
stated before the press, **counted** ones reported afterwards. That shape is
inherited here, and one thing about it inverts.

Exporting *loses* things — layout, fonts, position — and the window's job is
to name the losses. Importing **invents** things: a sheet size, four
margins, a typeface, a size, a leading, an alignment. None of that is in the
text file, and every one of those is a decision somebody has to make.

⇒ So this window is a **chooser**, not a warning. Its controls are the
decisions, its defaults are answers to *"what would he have picked?"*, and
the only sentence that warns about anything is the one about the font,
because that is the invention an operator is least likely to expect.

## The two things it says that a file picker cannot

**New pages are created.** This does not put words onto the sheet he is
looking at. The label says *"as pages"* for that reason and the window says
it again, because a File ▸ Import that quietly replaced a page would be the
worst outcome available to this feature.

**The words are set in one standard font.** `PageTemplate::face` is a
`Std14` — no embedding, R79 — so a text file full of Polish or Greek arrives
with characters the face cannot write. The engine **refuses the import** and
names every one of them by default (`Unmappable::Refuse`), which is the
right default and is the one thing here an operator would call a bug if it
were silent.

## What is deliberately NOT offered

**A font picker beyond the Standard 14.** `place_text` embeds nothing, so
offering a face this program cannot write with would be a control that
declines on press. The five the chooser offers are the ones a plain text
file plausibly wants: two serif, two sans, one monospace.

**A preview.** It would mean running the whole import to draw a window that
offers to run the import, and the report afterwards answers the same
questions with real numbers rather than provisional ones.

## Item notes

### `fn window_title`

*"as pages"* in the title as well as on the command, because a window that
has been open for a minute is the only thing on screen and its title is the
last statement of what is about to happen.

### `fn margin_label`

**One margin, not four.** `PageTemplate` carries four and this window
offers one, which is a deliberate narrowing: an operator importing a text
file wants a readable page, and four spinners is a form to fill in rather
than a decision to make. The engine's four are still set — all to this
number — so nothing is lost that a later ask could not add.

### `fn face_note`

The one warning in the window, and it earns its place: `place_text`
embeds nothing (R79), so a file containing a character none of the Standard
14 can write is **refused entirely** rather than imported with gaps. That is
the correct behaviour and it is also the one an operator will not predict,
because every other program on his machine would have substituted a font.

### `fn face_name`

The engine's `/BaseFont` name with its hyphen opened out — `Times-Roman`
becomes *Times Roman* — and nothing else. These are the names printed in
every PDF reader's font panel and written into the `/BaseFont` key, so an
operator checking what a page uses meets the same word pdfcer wrote.
`new_document::size_name` makes the same choice about sheet names.

⚠ **The `_` arm answers Helvetica, and that is a real hazard rather than a
tidy default**: a face added to `dialogs::import_text`'s `FACES` without an
arm here would silently show *Helvetica* in the chooser beside the real
Helvetica. `dialogs::import_text::tests::every_offered_face_has_its_own_label`
is what notices — it asserts the labels are distinct rather than that they
are correct, which is the property a test can actually hold.

### `fn window_title_for`

The file's name after an em dash, which is this program's title
convention — the window says what it does first, because that is what an
operator alt-tabbing back to it needs, and names the subject second.
