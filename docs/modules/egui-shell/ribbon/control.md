# `egui-shell/ribbon/control`

## Item notes

### `fn report_enablement`

**On change, never per frame.** An unconditional emit is forty-odd lines
a frame at sixty frames a second, which is not a log anybody reads; it is
also the difference between a diagnostic left permanently on and one that
gets switched off. The previous answer is kept in `egui`'s own temp data
under an id derived from the command's, so it lives exactly as long as the
context does and costs nothing when tracing is off.

**The first frame always emits**, because there is no previous answer to
match — which is what a harness needs, since a check that clicks and then
greps cannot rely on having been present for a transition it did not cause.

This function knows nothing about what any id MEANS, which is R7: it
reports that a control registered under some id was drawn pressable or not.
Whether `format.bold` should have been is the application's business.

### `fn command_button`

Shared with [`super::qat`], which is why it lives here and takes
`shows_label`.

# `truncate`

Whether the label may lose characters rather than the button losing
its place. `true` on the tab-strip row, `false` in the band, and the
asymmetry is deliberate:

- A **band** control that does not fit is in a group the plan has
  already decided is visible, inside a `Ui` whose `max_rect` stops
  before the overflow affordance. Truncating it would hide a command's
  name to save a few points that the reservation has already accounted
  for.
- A **strip** control has nowhere to go. The QAT is a fixed cost with
  no menu behind it, and the active tab is pinned out of the strip's
  own menu ([`plan::plan_tab_strip`]). When either is wider than the
  room the row can give it, the only alternatives are "truncate" and
  "draw off the edge of the window", and the second one is the defect.
