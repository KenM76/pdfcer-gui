# `app::actions::handsign` — writing a hand signature into the page

## Drawn

`place` fits the mark into the box in **canvas space** (display-oriented, one
unit per point), then maps every point through `canvas_to_pdf_space`, so the
signature is upright on a rotated page. It writes one
`EditSession::add_markup_as_content` Ink call through the funnel: one undo
step, and content that renders exactly as it saves (R8b).

Why page content, not an Ink annotation or a `/Sig` value:

- it prints under every reader's default print scope, where an annotation
  depends on its print flag and the reader's settings;
- it can never be mistaken for a digital signature: no `/V`, no `/ByteRange`;
- the `/Sig` field is untouched, so a certificate signature can still be
  added later.

The ink is a fixed dark blue-black, a document colour rather than a theme role,
because it is written into the file. On success the field enters
`OpenDoc::hand_signed`, which hides its *sign here* tag (see the ledger in
`pdfcer-gui-base/handsign.md`).

## Typed

The name is fitted by `fit_typed` in canvas space, its baseline origin mapped
to page space, and written by one `EditSession::add_text` call with
`with_embedded_face` (the subset of the chosen handwriting face),
`FontProvenance::Supplied`, the same ink colour: one undo step, and the face
travels with the file. Success is read from the edit epoch moving.

`add_text` has no rotation, so on a page shown turned (`writes_along` false)
the name would run along the box's short side. The window greys the Type tab
there; the action refuses as its backstop (`hand-sign-refused reason=turned-page`).
