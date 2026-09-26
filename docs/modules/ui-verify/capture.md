# `ui-verify/capture`

Capture the target window to an [`Image`], and refuse a capture that is not
evidence.

## The guard is the point

Grabbing pixels is four lines. The reason this is a module is the check at
the end of [`window`]: **a near-uniform capture is refused rather than
returned.**

A blank capture is indistinguishable from a real one at the call site. The
file exists, the call succeeded, and the only thing that says "this is not
evidence" is a human looking at it. pdfcer's predecessor recorded exactly
what happens without the guard: a run of blank screenshots got a
plausible-sounding cause invented for them (a compositor recomposite race),
the fix appeared to work because the real cause happened to go away at the
same time, and the actual cause — the operator's **display had powered
down**, and `CopyFromScreen` reads the composited desktop, which has
nothing to read from a sleeping display — was found later, by the operator.

The lesson kept from that, and the reason this function returns an error
instead of a picture: a change that appears to fix something is not
evidence for the story told about it, and the fix that mattered was not the
sleep that was tried first — it was refusing to treat a uniform capture as
evidence at all.

## Known causes of a uniform capture, in the order they have occurred

1. **The display is asleep or powered off.** Nothing to read.
2. **The window was not raised**, so the region belongs to another
   application. This usually looks like a screenshot *of that application*
   rather than a blank — which is why [`window`] raises first and why the
   check is for uniformity rather than for blankness specifically.
3. **The application died** before the shot. Check its trace.
