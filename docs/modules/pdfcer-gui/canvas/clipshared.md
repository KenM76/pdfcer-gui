# `pdfcer-gui/canvas/clipshared`

A selection copied in one pdfcer-gui window, pasted in another.

## The format

`clipimage::publish` places three entries in one clipboard transaction: the
picture (`CF_DIBV5`, when the clip has page content), the clip's own bytes
under the registered format `clipimage::OBJECT_CLIP_FORMAT`
(`ObjectClip::to_bytes`, unframed), and a sentence of text. Other programs take
the picture or the text; only pdfcer-gui reads the private format.

## When a paste adopts

`adopt(ctx, has_clip)` reads the private format when this window holds no clip
of its own, or when `clipseq::changed` says the OS clipboard has been written
since this window last copied. Otherwise this window's own clip is newer and
is pasted as before.

Bytes that are absent, or that `ObjectClip::from_bytes` refuses, return `None`,
and the paste falls through to `ospaste::newer`, which takes whatever another
program placed.

## What an adopted clip is

A `Clipped::Selection` whose `page` is `FOREIGN_PAGE`, a value no page index
equals, so a paste without a pointer position lands in place rather than
offset (the offset exists to keep a same-page paste from covering its source).
`annot_ids` and `left_behind` are empty: they name annotations in the source
document, which this window does not have. The anchor is the clip's bounding
box centre.

The adopted clip is stored with `canvas::clipboard::store`, which marks the
clipboard sequence. That makes `ospaste::newer` see nothing newer, so the
lossless `paste_objects` path runs, and a second paste reuses the stored clip
without reading the clipboard again.

## Driven check

`ui-verify` `a_selection_copied_in_one_window_pastes_as_objects_in_another`,
through `clipboard::place::DIAG_CLIPBOARD_DIR`: under it `publish` writes the
entries to a folder and `clippaste::own_clip` reads them back from it.
