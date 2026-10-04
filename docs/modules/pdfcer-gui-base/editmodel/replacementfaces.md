# `editmodel::replacementfaces`

When the engine retypes a run (`Workaround::Retype`) and the run's own font
cannot encode the new text, it sets the text in a face its replacement-face
ladder picks (`EditOptions::with_replacement_faces`, trait
`ReplacementFaces`). The engine reads no files; this module offers it the
faces in the operator's font folders.

## Contract

- `for_folders(folders)` returns one `'static` source per distinct folder
  list (the engine holds a `&'static dyn ReplacementFaces`), leaked once and
  reused.
- `candidates(chars)` answers from a `pdfcer_render::font::FaceCatalog`
  holding every face in the folders (`installedfaces::font_files`, at most
  `fontlibrary::MAX_FONT_FILE_BYTES` each): descriptions and coverage, never
  font bytes. The catalogue is built on the first call and rebuilt whenever
  the folders' listing (path, size, modified time) differs from the one it was
  built from, so a font installed mid-session is offered. It traces
  `replacement-faces folders= faces= ms=`.
- `plan(candidate, chars)` asks the catalogue the last `candidates` call
  answered from, whose loader reads only the picked file. Only `candidates`
  rebuilds, so a candidate's id is always read in the catalogue that numbered
  it. A file that has since gone or grown too large answers
  `text::fonts::face_unreadable`; no catalogue at all answers
  `face_no_longer_offered`. The engine discloses the sentence and tries the
  next face.
- `laddered(options, faces)` adds the source to an edit's options.

## Why it is applied only on the press

With `replacement_faces` set and no `fallback`, the engine uses the ladder
for any character the run's font cannot encode. The keystroke query, the
preview and the commit must agree on what a key does, and only the commit can
afford to read the folders, so the keystroke still declines such a letter and
the ladder is added only to a commit made by an explicit press
(`app::actions::textcommit`, `workarounds: true`):

- *Type "Ω" in an installed font* under a letter refused at the keystroke
  (`panels::properties::installedletter`), which commits the open draft with
  the letter at the caret;
- *Make the edit this way* on a workaround offer
  (`panels::properties::workaround`), so a retype can set a letter its font
  lacks.

The engine names the face it picked on the status line; nothing marks it on
the canvas.

## Why no bytes are kept

This computer's font folder is several hundred megabytes. The catalogue keeps
what ranking needs and reads one file when a face is picked, so holding it for
the session costs descriptions, not fonts. Listing the folders on each press
is what keeps it current; it reads metadata only.

## What it does not decide

Which face wins is the engine's ranking, including its metric-equivalent rung
(a Helvetica run gets Arial). The shell does not re-rank.
