# `dialogs::compact_version` — *Make compatible with*

One list in File ▸ Save a compacted copy: **Keep its version** (the default),
then PDF 1.4, 1.5, 1.6 and 1.7. Choosing a version runs
`pdfcer_core::optimize::lower_version` on the bytes the window would
otherwise save — after recompression and every other switch — on a thread
(`compact-lower`). Save is greyed while it runs.

**Keep by default.** ISO 32000-2 Annex I.2: a writer should not change a
document to an older version. Lowering happens only when asked.

**1.3 is not offered.** The engine's version floor starts at 1.3, so nothing
below it can be proven.

**Signed files are lowered with `allow_signed`.** The compacted copy already
drops every signature and the window says so; refusing here would only repeat
that.

## What the window says

| outcome | lines |
|---|---|
| lowered | the old and new version; the catalog `/Version` removed, when it was; *rebuilt* when the engine had to rewrite rather than patch the header; up to three constructs that set the floor |
| below the floor | the version the content needs and up to three constructs that need it; Save writes the unlowered copy |
| not lower | the copy is already at or below the target |
| unproven | something in the file could not be read, so no floor is proven |
| encrypted / failed | the engine's reason |

Any change to the other switches, or to the list, drops the result and starts
again from the plain copy.

## Trace

`compact-lowered to=<x.y> outcome=<lowered|below-floor|not-lower|unproven|encrypted|failed> bytes=<n> ms=<n>`
— `bytes` is `0` when nothing was lowered.

Regions: `compact.version` (the list), `compact.version.<i>` (each entry while
open, `0` = Keep), `compact.version.said` (each line).

Driven: `a_compacted_copy_is_made_compatible_with_an_older_version`.
