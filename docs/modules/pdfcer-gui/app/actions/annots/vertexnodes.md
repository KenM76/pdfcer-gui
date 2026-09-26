# `pdfcer-gui/app/actions/annots/vertexnodes`

**A markup shape's vertex verbs** — `move_node`, `insert_node` and
`remove_node`, one body (`reshape`) behind all three, over
`EditSession::reshape_annotation`.

The seam this file sits on is the one `inknodes.rs` draws for `/Ink`: one
file per geometry family, each calling its own engine verb through the
shared `vector_edit` wrapper. `/Polygon`, `/PolyLine` and `/Line` are
addressed by a flat vertex index (`VertexEdit`); `/Ink` by
`(stroke, point)` (`InkEdit`) — two files because they are two index
spaces, which is also the engine's argument for two verbs.

Nothing here decides whether a node may move — `canvas::annotnodes` asks
`reshape_annotation_preview` on every frame of the drag, so a release that
reaches these functions is one the engine already said yes to.


> *"I also can't edit or delete nodes of a markup shape once it is drawn."*

Three engine wrappers over one planner. They share everything except which
`VertexEdit` they build, which is why they share [`reshape`] rather than
each spelling the funnel out — the disclosure obligation is identical for
all three and stating it once is what stops the third one growing up
without it.

**`reshape_annotation` and not the three wrappers**, and that is a
deliberate reversal of the obvious call. The wrappers are one-liners that
pass `modified: None`, so they can never stamp `/M`; this shell knows the
time and the engine reads no clock, on purpose:

> pdfcer reads no clock (determinism — the same edit on the same file
> produces the same bytes), so the three convenience wrappers leave `/M`
> exactly as it was and say so.

A reviewer's comment whose shape changed and whose modification date did
not is a comment that lies about when it was last touched, and §12.5.2
admits any string for `/M`. So this shell supplies one, in the ASN.1 form
§7.9.4 defines, and `AnnotationReshape::mod_date_written` reports whether
it landed.

**The date comes from [`crate::app::clock::pdf_date_utc`]**, and nothing
here writes a second civil-from-days conversion of its own. **Two copies
of a calendar are two calendars**, and the one nobody looks at is the one
that claims 30 February.

Its `None` case is the one [`reshape`] cares about: a clock before the Unix
epoch yields no stamp, `/M` is left exactly as it was, and the annotation's
date is unchanged rather than false. `AnnotationReshape::mod_date_written`
reports which happened.

**An extraction moves the doc comment with the function**, and a
free-floating `//` banner is the shape most likely to be left behind,
because nothing in the language binds it to anything.
