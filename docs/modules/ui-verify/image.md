# `ui-verify/image`

## Item notes

### `fn temp_path`

Uniqueness comes from the process id plus a monotonic counter, which is
enough for a harness that never runs two conversions concurrently and does
not warrant a uuid dependency.

### `fn from_bgra`

# Errors

If the buffer is not exactly `width * height * 4` bytes. Checked rather
than trusted because every later index derives from these numbers, and
a mismatch would read a neighbouring row — producing an image that is
subtly sheared rather than obviously broken.

### `fn pixel`

`None` rather than a clamp or a panic: a region that runs off the edge
is a calibration error, and the oracles report how many pixels they
actually sampled so that error is visible in the output instead of
being papered over by edge pixels repeated a thousand times.

### `fn from_bmp`

Handles only what the converter above emits, and says so when handed
anything else rather than guessing. BMP rows are **bottom-up** unless
the height is negative, which is the one detail worth reading twice:
getting it wrong mirrors the image vertically, and a mirrored
screenshot still looks like a screenshot.
