# `a_key_the_subset_lacks_comes_from_its_installed_face`

A Word subset carries only the letters the document used. A letter it never
carried is appended to the subset from the face in the font folders whose
PostScript name is the subset's, once the engine has proved they are the
same font. The line then keeps its own font; without this the shell sets the
letter in the nearest other face (`canvas::textedit::reface`).

## The drive

`fixtures/augment-subset.pdf` shows `ABC` in `ABCDEF+pdfcerAugFace`, a simple
TrueType subset that outlines only those three letters.
`fixtures/augment-fonts/face.ttf` is the full face it was cut from. The check
points `PDFCER_DIAG_FONT_DIR` at that folder, waits for the
`installed-faces-indexed` line, arms Edit Text, clicks into `ABC`, presses
End, types `D` and presses Escape.

With `--pdf` and `--doc-point` it drives the operator's drawing instead,
copied first, with no extra font folder: the face comes from this computer's
own. It types `z`, the letter SW41177's `AQHZBV+CenturyGothic` subset lacks,
and drives two routes, each on a fresh copy and a fresh launch:

- **paragraph**, the default: the click opens the whole paragraph and the
  commit goes through the block verb (`edit-block-text`).
- **line**, with `edit.paragraphs` turned off: the click opens the line alone,
  and the keystroke asks the run's own repertoire. A composite (`/Type0`)
  subset is asked through `SubsetAugmenter::candidates`, so this is the route
  that set the letter in Helvetica when the forward was missing.

```
ui-verify --no-input --check a_key_the_subset_lacks_comes_from_its_installed_face   --pdf D:/Dev/pdfTests/SW41177/SW41177.pdf --doc-point 0,1215,595
```

## What it asserts

- No `refused-char` line names the letter.
- No `text-edit-refused-keys` line names its code point: the caret did not
  say it will be set in another face.
- No `text-edit-reface-planned` line: the letter was not sent to another face.
- The last `edit-text` or `edit-block-text` line's disclosures say one glyph
  was added to the subset; on the fixture, from `face.ttf`.
- An `edit-text-left-edge` line says `committed=yes`, or an
  `edit-block-text-applied` line has `glyphs_added` above zero.

## What makes it fail

Returning the options unchanged from `installedfaces::augmenting`. The
keystroke query then has no glyph for `D`, the caret plans it into another
face, and the second assertion fails.

On SW41177, making `InstalledFaces::candidates` answer nothing fails the line
route with `text-edit-refused-keys … characters=U+007A … face=Helvetica
state=planned`, the notice the operator reported. The paragraph route still
passes with it missing: the block verb does not ask `candidates`.
