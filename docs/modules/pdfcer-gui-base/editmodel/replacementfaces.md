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
- `candidates(chars)` reads every font file in the folders
  (`installedfaces::font_files`, at most `fontlibrary::MAX_FONT_FILE_BYTES`
  each), asks `pdfcer_render::font::InstalledFaces` to describe each file's
  faces, renumbers the candidates into one id space, and drops the bytes. It
  remembers which file and which id within the file each candidate came from,
  and traces `replacement-faces folders= faces= ms=`.
- `plan(candidate, chars)` reads the picked file again and asks it to cut the
  subset. A candidate from an earlier call, or a file that has since gone,
  answers a sentence (`text::fonts::face_no_longer_offered`,
  `face_unreadable`); the engine discloses it and tries the next face.
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

`pdfcer_render::font::InstalledFaces` holds every font's bytes for as long as
it lives; this computer's font folder is several hundred megabytes. The ladder
runs once per press, so reading the folders per press (well under a second
here) is cheaper than holding them for the session. Engine request G111 asks
for a provider that keeps names, not bytes; when it lands this module goes.

## What it does not decide

Which face wins is the engine's ranking. Among faces that tie, it currently
takes the first in file order (engine request G110); the shell does not
re-rank.
