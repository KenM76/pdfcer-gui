# `editmodel::fallbackface`

`EditOptions::with_fallback` takes a `&'static FallbackFace` so the options
stay `Copy`. `named(selector)` interns one leaked `FallbackFace::Named` per
distinct selector in a process-wide list and answers the same reference on
every later call; the leak is bounded by the faces a session falls back to.

The selector is the one the face picker sends (`FaceChoice::selector`): a page
`/Font` key, a `/BaseFont` on the page, or a standard-14 name, which is what
`FallbackFace::Named` accepts.

`source_token` names a `FallbackSource` for the `text-edit-fallback` trace —
`page`, `standard14`, `embedded`, `same-program` — spelled out rather than Debug-formatted
because a check reads it.
