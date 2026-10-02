# `editmodel::reface` — tokenising a line for a mixed-face commit

`Reface` is what a commit carries when some of its characters must be set in
another face: the face's selector and label, the characters, and the
placeholder characters the run's own font *does* have.

`tokenise_any(replacement, reface, lines)` rewrites `replacement` so every
maximal stretch of refaced characters becomes a token — a run of one
placeholder character — and returns the tokenised text with the
`(token, original stretch)` pairs. A token must be unique on the page, so its
length is one more than the longest run of that placeholder anywhere on the
page's lines or in the replacement. A placeholder that sits next to a
refaced stretch, or is itself refaced, is unusable (the token would merge with
it); it tries each placeholder in turn and answers `None` when none is usable.
The chosen face must also carry the placeholder, because the token is
formatted into it before it is overwritten.

Pure: no engine, no egui. The engine verbs that consume the tokens are in
`pdfcer-gui::app::actions::reface`.
