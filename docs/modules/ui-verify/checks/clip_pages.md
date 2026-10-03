# `the_clipboard_becomes_a_new_pdf_or_pages_after_this_one`

**Defect it guards:** a picture or text copied in another program cannot be
made into a new PDF or added as pages, as Acrobat's Create from Clipboard and
Insert from Clipboard do.

## What it drives

Off-screen, scripted pointer, on a copy of `four-pages.pdf`, page 0 on screen.
The clipboard is snapshotted and restored afterwards. The picture is a 64×32
CF_DIB at 96 pixels per inch (48×24 pt); the text is two lines split by a form
feed.

1. Ctrl+3 (Edit). Picture on the clipboard; Pages tab, Insert from clipboard.
2. Text on the clipboard; Pages tab, Insert from clipboard.
3. Picture on the clipboard; File tab, New from clipboard.
4. Text on the clipboard; File tab, New from clipboard.

## Oracles

| step | requires |
|---|---|
| 1 | a new `insert-pages page=1 n=1` line whose receipt reads "Inserted 1 page after page 1." |
| 2 | a new `import-text-applied pages=2 first=1` line |
| 3 | a new `pages-from-clipboard kind=image pages=1`, `w` and `h` 48×24 within 0.5, read from the new document's pages |
| 4 | a new `pages-from-clipboard kind=text pages=2` |

## Falsification

Inserting at the end instead of after the page on screen traces `page=5` and
fails step 1. Keeping the template's blank page in the new document traces
`pages=3` and fails step 4.
