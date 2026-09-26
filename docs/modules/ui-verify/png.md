# `ui-verify/png`

A minimal PNG encoder — enough to write an evidence file, and nothing more.

## Why hand-rolled

The alternative is `png` plus a deflate implementation, and the trade is
about what this crate is *for*. `ui-verify` is reached for when the
application is misbehaving; every dependency it carries is one more way for
it to fail to build on that day. A hundred and twenty lines of well-
understood format code is cheaper to own than two crates, and it has no
version to reconcile with the workspace's pinned set.

## Why the output is uncompressed

PNG's image data is a zlib stream, and zlib streams may consist entirely of
**stored** (uncompressed) deflate blocks. That is a legal, universally
readable PNG that costs a few lines instead of a compressor. The price is
file size — a 1920×1080 screenshot lands around 8 MB instead of around 1 MB.

That is the right trade for this crate. These files are written to a
scratch directory during a check run, read by a human once when something
failed, and deleted. Nothing ships them, nothing stores them long-term, and
nothing transfers them over a network. Paying a compressor's build cost and
a compressor's dependency risk to make a temporary diagnostic file smaller
would be optimising the wrong thing.

## Format notes for whoever maintains this

A PNG is an 8-byte signature followed by length-prefixed, CRC-suffixed
chunks. This writer emits exactly three:

* `IHDR` — width, height, bit depth 8, colour type 2 (truecolour RGB),
  compression 0, filter 0, interlace 0.
* `IDAT` — a zlib stream: a 2-byte header, then deflate stored blocks, then
  a 4-byte Adler-32 of the *uncompressed* data.
* `IEND` — empty.

The uncompressed data is the scanlines, each prefixed by a filter-type byte
of 0 (None). That per-row prefix byte is the classic thing to forget; the
symptom is an image that shears diagonally, because every row is offset one
byte further than the last.
