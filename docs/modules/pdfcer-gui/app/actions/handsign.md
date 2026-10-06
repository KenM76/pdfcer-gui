# `app::actions::handsign` — writing a hand signature into the page

Every kind lands in the rectangle `handsign::place::ink_rect` gives: the
operator's chosen placement clamped to the allowed region, or the fit rule.

## Drawn

`place` stretches the mark into its ink rectangle in **canvas space** (display-oriented, one
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
because it is written into the file. Every route tags what it writes with the
field's name (`MarkupOptions::hand_signature`, `AddTextRequest::with_hand_signature`,
`NewImage::as_hand_signature`),
and the box counts as signed because the document says so (see *Which boxes
are signed* in `pdfcer-gui-base/handsign.md`).

## Adjusted

`adjust` moves or resizes a placed signature, any kind, so what spans `from`
spans `to` (PDF user space; `canvas::forms::sigadjust` has already clamped
`to`). One `EditSession::transform_objects` call through the funnel with the
scale-and-translate matrix between the two: one undo step. The objects are
those of `EditSession::page_objects(page)` whose byte spans lie inside a
`BDC … EMC` sequence tagged `/pdfc_HandSig` with this field's name, found by
walking `ContentStream::from_page` — the stream `page_objects` was built
from. The transform rewrites inside the sequence, so the tag survives;
`hand-sign-adjusted tagged=` re-reads it to say so. The engine's mark carries
no object list (G127); when it does, this reader goes.

A rectangle under half a point a side, or a mark whose objects cannot be
found, is refused with `hand-sign-adjust-refused reason=`.

## Typed

The name is set by `typed_in` so the face's full height fills the ink
rectangle's height, in canvas space; its baseline origin mapped
to page space, and written by one `EditSession::add_text` call with
`with_embedded_face` (the subset of the chosen handwriting face),
`FontProvenance::Supplied`, the same ink colour: one undo step, and the face
travels with the file. Success is read from the edit epoch moving.

`add_text` has no rotation, so on a page shown turned (`writes_along` false)
the name would run along the box's short side. The window greys the Type tab
there; the action refuses as its backstop (`hand-sign-refused reason=turned-page`).

## Picture

One `EditSession::add_image` call with `NewImage::stretching` and
`as_hand_signature(field)`: the ink rectangle's two corners are mapped to page
space and the picture fills the rectangle they span. Stretching rather than
the default aspect-keeping fit, because the rectangle already has the
picture's proportions unless the operator chose others with Shift, and then
his choice is the one to honour. A PNG's alpha and the clear-white colour key
are carried by the `ImportedImage` (`handsign::picture`). The placement
disclosures are the Insert Image command's, so a low effective resolution is
reported the same way.

`add_image` places along the page's own axes; on a page shown turned the
window greys the Picture tab and the action refuses as its backstop
(`hand-sign-refused via=picture reason=turned-page`).
