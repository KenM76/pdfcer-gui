"""Author `word-gap.pdf` — a paragraph whose words are separated by TJ
displacements instead of space glyphs, so Edit > Content's Auto word gap can
measure it.

    python fixtures/word-gap.PROVENANCE.py

Deterministic: no timestamps, no randomness, no producer string.

Driven by `tools/ui-verify/src/checks/word_gap_band.rs` (OPERATOR_REQUESTS
O312).

# Why `paragraph.pdf` will not do

The engine's automatic threshold (`pdfcer-text` `text_extract::word_gap`) takes
every same-line gap between consecutive glyphs, in ems, and looks for an empty
interval of at least 0.05 em whose midpoint lies in 0.06..0.60, with at least
12 gaps and at least 3 on each side. A page that writes its words with space
glyphs has only near-zero gaps — the space glyph's own advance closes each one —
so it has no valley and keeps the fixed ratio, reported `measured=false`. That
is indistinguishable from Auto never having been applied, which is the defect
the check exists to catch.

# What it carries

Six lines of Helvetica 12 at the same place as `paragraph.pdf` (left 72 pt,
first baseline 700 pt, leading 16 pt), so the check's click point carries over.
Each line is one `TJ` array: every word is a string, and between words a
displacement of `GAP` thousandths of an em. Letters inside a word sit at their
font advance (gap 0 em); words sit `GAP / 1000` em apart. The valley is
therefore (0, 0.3) and the measured threshold its midpoint, 0.15 em — inside
the search band, and different from both the fixed default 0.2 and any width
the check's slider press produces, so a measured page cannot be mistaken for a
fixed one.
"""

import io
import os

NL = b"\n"

LEFT, TOP, LEADING, SIZE = 72.0, 700.0, 16.0, 12
# Thousandths of an em between words; a TJ number is subtracted from the
# advance, so the array carries it negated.
GAP = 300

LINES = [
    "The quick brown fox jumps over the lazy dog",
    "Pack my box with five dozen liquor jugs now",
    "How vexingly quick daft zebras jump today",
    "Sphinx of black quartz judge my vow again",
    "Bright vixens jump while dozy fowl quack",
    "Jackdaws love my big sphinx of quartz too",
]

pairs = sum(len(l.replace(" ", "")) - 1 for l in LINES)
words = sum(len(l.split()) - 1 for l in LINES)
# The engine's minimums: 12 gaps in all, 3 on each side of the valley.
assert pairs >= 12 and words >= 3 and pairs - words >= 3, (pairs, words)
assert 0.06 < GAP / 2000 < 0.60 and GAP / 2000 != 0.2


def tj(line):
    parts = [f"({w})" for w in line.split()]
    return "[" + f"-{GAP}".join(parts) + "] TJ"


content = [f"BT /F1 {SIZE} Tf"]
for i, line in enumerate(LINES):
    content.append(f"1 0 0 1 {LEFT:.2f} {TOP - i * LEADING:.2f} Tm {tj(line)}")
content.append("ET")
CONTENT = "\n".join(content).encode("ascii") + NL

OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
    b"/Resources << /Font << /F1 << /Type /Font /Subtype /Type1 "
    b"/BaseFont /Helvetica >> >> >> /Contents 4 0 R >>",
    b"<< /Length " + str(len(CONTENT)).encode() + b" >>" + NL
    + b"stream" + NL + CONTENT + b"endstream",
]

buf = io.BytesIO()
buf.write(b"%PDF-1.7" + NL)
buf.write(b"%" + bytes([0xE2, 0xE3, 0xCF, 0xD3]) + NL)
offsets = []
for n, body in enumerate(OBJECTS, start=1):
    offsets.append(buf.tell())
    buf.write(str(n).encode() + b" 0 obj" + NL + body + NL + b"endobj" + NL)
startxref = buf.tell()
buf.write(b"xref" + NL + b"0 " + str(len(OBJECTS) + 1).encode() + NL)
buf.write(b"0000000000 65535 f " + NL)
for off in offsets:
    buf.write(str(off).zfill(10).encode() + b" 00000 n " + NL)
buf.write(b"trailer" + NL)
buf.write(b"<< /Size " + str(len(OBJECTS) + 1).encode() + b" /Root 1 0 R >>" + NL)
buf.write(b"startxref" + NL + str(startxref).encode() + NL + b"%%EOF" + NL)
raw = buf.getvalue()

for n, off in enumerate(offsets, start=1):
    header = str(n).encode() + b" 0 obj"
    assert raw[off:off + len(header)] == header, (n, off)
assert raw[startxref:startxref + 4] == b"xref"

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "word-gap.pdf")
io.open(out, "wb").write(raw)
print("wrote", out, len(raw), "bytes; xref at", startxref)
