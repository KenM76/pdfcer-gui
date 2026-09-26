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

## Item notes

### `const RAISE_SETTLE_MS`

700 ms, measured rather than guessed — and the number matters less than the
note attached to it in the predecessor script, which briefly read 2500 ms
with an invented explanation. Three consecutive captures at 700 ms produced
identical non-blank content; the longer sleep bought nothing and cost 1.8 s
per capture.

### `fn window`

Raises the window, waits for it to paint, reads the desktop region it
occupies, and refuses the result if it is near-uniform.

# Errors

* The window cannot be measured (it has closed).
* The screen grab failed.
* The capture is near-uniform — see the module docs. This is reported as an
  error, not as a picture, because the caller would otherwise assert on it
  and produce a confident verdict about nothing.

### `fn frame_capture`

That is the worst available failure: a measurement of the wrong surface is
indistinguishable from a measurement of a broken one. `settings_headings_legible`
produced exactly that, naming two headings that render perfectly well.

`raise` is the caller's choice here rather than unconditional, because the
caller may already have raised the dialog to click something in it and a
second raise of the MAIN window would put it behind again — which is the
same z-order trap `Driver::window_owning` exists for.

### `fn window_to_png`

Every check that looks at pixels saves its evidence, pass or fail. On a
failure it is what the reader needs; on a pass it is what makes the *next*
failure diagnosable, because there is something to compare against.
