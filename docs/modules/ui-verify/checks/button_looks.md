# `ui-verify/checks/button_looks`

`a_buttons_rollover_and_down_looks_are_drawn` — **while filling, a button
under the pointer is drawn from its `/AP /R` look, and from its `/AP /D` look
while pressed; moving off it puts the normal look back.**

# The fixture

`fixtures/button-looks.pdf` (`button-looks.PROVENANCE.py`): a push button
`Go` and a check box `Tick`, each with flat-fill appearances — `/N` grey, `/R`
blue, `/D` red — so one screenshot names the look on screen.

# The assertions, per button, in order

The pointer is scripted (`move`, `down`, `up`); the document opens in Review,
where a click fills.

1. **Away.** Pointer off the button: the box is grey.
2. **Hover.** A `canvas-widget-look` line reports `kind=rollover` with
   `drawn=1` (the texture on screen was rendered with that look, read off its
   render key), and the box is blue.
3. **Press.** Primary held inside it: `kind=down drawn=1`, the box red.
4. **Leave.** Released and moved away: `kind=none drawn=1`, the box grey
   again — no look is left behind.

The box is sampled 6 pt inside its edges; a colour must hold 80 % of the
pixels.

Print, export and save never carry a look: only `OpenDoc::looks_for` fills
`RenderRequest::appearance_states`, and only the canvas request builder calls
it.

# Falsification

`render_on_worker` not copying `appearance_states` into the render options:
FAIL at 2 (the key still says the look was drawn; the pixels stay grey).
