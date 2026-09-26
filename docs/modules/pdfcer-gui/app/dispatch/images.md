# `app::dispatch::images` — choosing a picture, reading it, and refusing it
by name

## Why this is a module and not a match arm

Because it is a **sequence**, not a verb. `edit.insert_image` is several
steps before anything is drawn — pick a file, read it, import it, and then
either refuse it or open a window — and a sequence that long inside a
`match` arm is unreadable among arms that are one line each.

It sits beside [`super::pages`], which is here for the same reason: that
module holds the Pages tab's ids, whose bodies share an operand rule, and
this one holds a single id whose body is longer than most tabs.

## ★ Why the import happens BEFORE the window opens

So a file that cannot be placed is refused **at the moment it is chosen**,
naming the file's own problem — *"pdfcer does not place GIF images — it
places PNG, JPEG, BMP and TIFF"*, *"this image uses {feature}, which pdfcer
cannot place"* — rather than opening a window full of controls over a
picture that was never going to go in.

It is also what lets the window state the picture's real facts: its format,
its pixel size **as displayed** (an EXIF-rotated photograph is transposed by
the importer, and the stored shape is not on screen anywhere), and whether
its resolution is one the file declared or one pdfcer assumed.

## ★ The refusal is the ENGINE's, passed through

Unlike a `TwoLineRefusal`, whose wording lives in
[`crate::text::measure`]. The difference is what the message is **about**:
an `ImageImportError` names the operator's own file and the specific thing
wrong with it, so a catalog sentence would have to discard the half that is
the whole answer. `check-ui-strings.sh`'s exclusion 3 covers this shape, and
`crate::text::images::import_failed` is the wrapper that keeps the sentence
pdfcer's own while the detail stays the file's.

## Why the decode runs on this thread

A drawing's logo is a few kilobytes and a site photograph is a few
megabytes; the decode is milliseconds either way. A worker would need a
channel, a pending state and a way to say the operator changed their mind —
machinery for a wait nobody notices. A **scan at 600 dpi** is the case that
would justify it, and it is the case to re-measure before building for
rather than the case to assume.
