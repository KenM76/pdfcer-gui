# `text::import_text` — every word the **Import text as pages** window says


> *"also the engine can export PDFs as text. we should have export/import
> for that."*

## ★★★ The window's job, and it is not the same as the export window's

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

## ★★ The two things it says that a file picker cannot

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

## ★ What is deliberately NOT offered

**A font picker beyond the Standard 14.** `place_text` embeds nothing, so
offering a face this program cannot write with would be a control that
declines on press. The five the chooser offers are the ones a plain text
file plausibly wants: two serif, two sans, one monospace.

**A preview.** It would mean running the whole import to draw a window that
offers to run the import, and the report afterwards answers the same
questions with real numbers rather than provisional ones.
