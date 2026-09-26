# `pdfcer-gui/canvas/forms/selecting`

**Selecting a form field, rather than filling it** — the Edit-mode half of
[`super`].

# The two surfaces, and why they are genuinely different subjects

The parent module fills a field: a click opens a live `egui` text box over
the raster, keystrokes go into a draft, and a commit turns the draft into
one `Action`. **Nothing here does any of that.** This half answers a
different question — *which field is the properties panel talking about* —
and its output is a selection, an outline, eight grips and a cursor.

They are separated by **mode**, not by taste: filling is the Read/Review
reading of a click on a widget, selecting is the Edit reading, and
[`super::surface`] chooses between them once per frame. A reader debugging
"my click did the wrong thing" needs to know which of the two ran; a reader
debugging "the outline is in the wrong place" needs only this file.

# What every function here has in common

**None of them mutate.** Each reads `boxes::FieldTarget`s — the memoised
placement the parent's [`super::placed`] owns — and either paints, sets a
cursor, or pushes an [`Action`]. The selection itself lives on the
document and is applied by the action queue at the end of the frame, which
is why a hit test rather than a state read is the right question to ask
during one (see `canvas::rightclick`'s table).

# Visibility contract

Everything here is `pub(super)` except [`right_click_hits_a_field`], which
is `pub` and re-exported by the parent so that
`canvas::forms::right_click_hits_a_field` resolves for `canvas::rightclick`.
Narrowing it breaks that caller.
