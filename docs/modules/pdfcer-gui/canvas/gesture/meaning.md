# `canvas::gesture::meaning` — what a press MEANS, decided once and then remembered

One pure function, [`press_kind`], and the two enums it decides between:
[`DragKind`], which says what a drag is going to *do*, and [`MarqueeIntent`],
which says what a rubber band does when it is released. Nothing in this file
holds state, touches egui, or knows that frames exist — a press is
`(tool, grip, zoom_armed, capabilities)` in and one meaning, or `None`, out.
That is what makes the whole precedence testable as a table.

The state machine that *carries* a meaning across a press, a drag and a
release — [`PointerFrame`](super::PointerFrame),
[`GestureState`](super::GestureState) and
[`GestureOutcome`] — is the parent module, [`super`].
It calls this function on exactly one frame per gesture, the press frame,
and then never asks again.

The precedence itself — which meaning wins when two are available, and which
presses a mode refuses outright — is documented on [`press_kind`], because it
*is* the rule rather than a note about it.

[`press_kind`] deliberately has no case for the hand tool, and the absence is
load-bearing: `canvas::interact` hands the state machine a **blank** frame
while the hand is active, so no press ever arrives here to be classified.
That rule — *one state machine, one meaning per frame* — is stated in full in
[`super`]'s header, under "Marquee versus pan".

## Marquee-select versus marquee-zoom: one rubber band, two releases

Phase 3.4 adds a marquee that *zooms* to what it encloses. It is
deliberately **the same gesture**: same press, same in-flight rect, same
pixels on screen ([`crate::canvas::overlay::draw_marquee`] is not
duplicated), same normalisation, same Escape. What differs is one thing —
*what happens on release* — so what is carried is one value, [`MarqueeIntent`].

It is sampled **at the press**, exactly as `shift` is, and for the identical
reason: the one-shot arming is retired when the drag completes, and an
intent re-read at release would be read after something else had already
consumed it. A gesture means what it meant when it started.
