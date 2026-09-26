# `egui-shell/ribbon/render`

## Item notes

### `fn band_holds_keyboard_focus`

The `holds_focus` keep-term of [`crate::peek::Peek::resolve`]. Without it a
keyboard user who tabs into a revealed band loses it on the next frame,
because the pointer is nowhere near — a control that is drawn and then
withdrawn from under the focus is the same defect class as one that is drawn
and unclickable.

# Why it is answered geometrically rather than by id

The obvious implementation asks whether the focused `egui::Id` is one the
ribbon derived from [`super::ctx::Ctx::id`]. It cannot: an `Id` is a hash
and does not decompose, so there is no way to ask "did this come from my
salt". What *is* available is the focused widget's own rectangle —
`Context::read_response` — and whether it lies inside the rectangle the
overlay occupied last frame. The overlay is the only thing drawn there, so
containment answers the question exactly.

It reads **last frame's** overlay, which is the only one that exists at
the moment the question is asked, and that is not a staleness bug: it is the
same rectangle this frame will draw unless the theme changed, and the term
is a *keep* rather than a *start*, so the worst a stale rectangle can do is
hold a band open for one extra frame. It cannot open one.

Returns `false` when nothing has focus, when the focused widget has no
recorded response yet (its first frame), and when auto-hide has never drawn
an overlay — all three being "the keyboard is not in there".
