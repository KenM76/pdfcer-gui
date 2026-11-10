# -*- coding: utf-8 -*-
"""Build `fixtures/transparency-cmyk-a1.pdf`: `transparency-cmyk.pdf` with its
MediaBox widened from A4 to A1 portrait.

The page keeps its subtractive transparency group, and at 300 DPI an A1 raster
is about 70 million pixels, past the engine's default ink buffer ceiling
(`RenderOptions::max_cmyk_buffer_bytes`), so the engine composites it in row
bands (`Diagnostics::cmyk_bands` > 1) rather than in one pass. A4 at 300 DPI
fits in one pass, which is why the A4 fixture cannot exercise banding.

The replacement box has the same byte length as the original, so every xref
offset stays valid and the file needs no rewrite.

Run from the repository root: `python fixtures/transparency-cmyk-a1.PROVENANCE.py`.
"""
from pathlib import Path

SRC = Path("fixtures/transparency-cmyk.pdf")
DST = Path("fixtures/transparency-cmyk-a1.pdf")
OLD = b"/MediaBox [0 0 595.2760 841.8900]"
NEW = b"/MediaBox [0 0 1683.780 2383.937]"

assert len(OLD) == len(NEW)
data = SRC.read_bytes()
assert data.count(OLD) == 1, "the source fixture's MediaBox changed"
DST.write_bytes(data.replace(OLD, NEW))
