#!/usr/bin/env python3
"""Generate ``fixtures/mixed-paragraph.pdf`` — one paragraph of two looks.

The six lines of ``fixtures/paragraph.pdf`` (``gen-reflow-fixture.py``), with
the word ``notes`` on line 2 set in Helvetica-Bold and filled red. A driven
check opens the paragraph from a click, edits it, and re-opens it to see
that the rewrite kept both looks: a build that declines a mixed paragraph,
or a rewrite that falls back to the first run's look, both fail it.

The look change is inside line 2 (``Tf`` and ``rg`` between show operators
on one line), never between lines, so the recogniser has no new reason to
split the block. Uncompressed and deterministic, like its sibling; run

    python tools/gen-mixed-paragraph-fixture.py

It prints the paragraph's first baseline, for the check's click.
"""

import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "gen_reflow_fixture", os.path.join(HERE, "gen-reflow-fixture.py")
)
reflow = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(reflow)

# Line 2 of the sibling's paragraph, split around the word set apart.
MIXED_LINE = 1
BEFORE, WORD, AFTER = "in one file, and the ", "notes", " beside"
assert BEFORE + WORD + AFTER == reflow.LINES[MIXED_LINE]


def build_content() -> bytes:
    """One ``BT`` / ``ET``; every line placed by ``Tm``; one word bold red."""
    size = reflow.SIZE
    out = ["BT\n", f"/F1 {size:g} Tf\n", "0 g\n"]
    for i, text in enumerate(reflow.LINES):
        y = reflow.TOP - i * reflow.LEADING
        out.append(f"1 0 0 1 {reflow.LEFT:.2f} {y:.2f} Tm\n")
        if i == MIXED_LINE:
            out.append(reflow.show(BEFORE))
            out.append(f"/F2 {size:g} Tf\n1 0 0 rg\n")
            out.append(reflow.show(WORD))
            out.append(f"/F1 {size:g} Tf\n0 g\n")
            out.append(reflow.show(AFTER))
        else:
            out.append(reflow.show(text))
    out.append("ET\n")
    return "".join(out).encode("ascii")


def build_pdf() -> bytes:
    """Six objects and a classic cross-reference table, offsets measured."""
    content = build_content()
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /Font << /F1 5 0 R /F2 6 0 R >> >> /Contents 4 0 R >>",
        b"<< /Length " + str(len(content)).encode() + b" >>\nstream\n"
        + content + b"endstream",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
        b"/Encoding /WinAnsiEncoding >>",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold "
        b"/Encoding /WinAnsiEncoding >>",
    ]
    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = []
    for i, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += f"{i} 0 obj\n".encode() + body + b"\nendobj\n"
    xref_at = len(out)
    out += f"xref\n0 {len(objects) + 1}\n".encode()
    out += b"0000000000 65535 f \n"
    for off in offsets:
        out += f"{off:010d} 00000 n \n".encode()
    out += (
        f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\n"
        f"startxref\n{xref_at}\n%%EOF\n"
    ).encode()
    return bytes(out)


def main() -> None:
    target = os.path.join(os.path.dirname(HERE), "fixtures", "mixed-paragraph.pdf")
    data = build_pdf()
    with open(target, "wb") as fh:
        fh.write(data)
    print(f"wrote {target}: {len(data)} bytes")
    print(f"first baseline: x={reflow.LEFT} y={reflow.TOP}, size {reflow.SIZE}")


if __name__ == "__main__":
    main()
