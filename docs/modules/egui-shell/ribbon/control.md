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
