# `editmodel::installedfaces`

A Word document embeds only the letters it uses. When the operator types a
letter the subset has no outline for, the engine can append that glyph to the
subset, but only from the face the subset was cut from and only after proving
the two are the same font (`EditOptions::with_subset_augment`,
`pdfcer_render::font::InstalledFaceAugmenter`). This module supplies those
faces from the font folders.

## Contract

- `for_folders(folders)` returns one `'static` source per distinct folder
  list. The engine's `SubsetAugment` holds a `&'static dyn SubsetAugmenter`,
  so a source is leaked once per list and reused for the life of the process.
- The first call starts a background thread that reads each font file in the
  folders (sorted, at most `fontlibrary::MAX_FONT_FILE_BYTES` each), records
  the names it advertises, and drops the bytes. It traces
  `installed-faces-indexed folders= files= names= ms=`.
- A request is answered by the subset's `/BaseFont` without its tag
  (`FontEnvironment::subset_stem`). The files advertising that name are read
  on the first request for it and kept, so a second keystroke reads nothing.
- Until the index is built, `addable` is empty and `augment` refuses with
  `text::fonts::folders_still_indexing`. The keystroke query and the commit
  both see that state, so a key is never accepted that the commit then
  refuses.
- `augmenting(options, faces)` adds the source at the engine's defaults:
  every shared glyph compared, hinting stripped when the programs differ.

## Why the index and not the font library

`fontlibrary::Library` keeps every file's bytes. This computer's font folder
holds hundreds of megabytes, and a subset only ever asks for the one face it
names, so the index keeps names and paths and reads a face when it is asked
for.

## Why the bytes are labelled by file name

The label is the engine's name for the face in its disclosure ("from
installed font 'calibri.ttf'"). The file name is what the operator can find
in the folder; the full path would bury it.
