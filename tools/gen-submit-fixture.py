"""Generate ``fixtures/submit-button.pdf`` — a document that phones home.

WHY THIS FIXTURE EXISTS
======================================================================

`a_document_that_phones_home_says_so` needs a document whose push button
carries a ``/SubmitForm`` action pointing at a URL. Nothing in the engine's
synthetic corpus has one — checked, 2026-08-30, every ``fixtures/synthetic``
PDF reports ``js_network_actions=0`` — and the shell's own corpus is CAD
drawings.

That absence is itself worth stating: **the one document shape the disclosure
exists for is the one nobody had a copy of.** A check written against a
document with no submit action would have asserted that a silent program stayed
silent, which is the green-result-reporting-nothing this harness is built to
remove.

WHAT IT CONTAINS, AND WHY EACH PART
======================================================================

* one page, 200 × 200 pt, with a **one-line content stream** and a
  ``/Resources`` dictionary.

  ★ Both were absent in the first version and the GUI refused the file with
  ``open failed`` while ``pdfcer inspect`` read it perfectly. The CLI asks
  *"is this a valid PDF?"*; the shell additionally wants a page it can
  RASTERISE, and a page with no ``/Contents`` and no ``/Resources`` is valid and
  not drawable. **A fixture that only one of the two tools accepts is a fixture
  that will waste somebody's afternoon**, so this one draws a line;

* one **push-button widget** whose ``/A`` is::

      << /Type /Action /S /SubmitForm /F (http://example.invalid/collect) /Flags 4 >>

  ``/S /SubmitForm`` is what ``forms::scan_javascript`` counts as a network
  action. ``/Flags 4`` is ExportFormat (§12.7.5.2 Table 237) — an ordinary,
  entirely conventional submit, so the fixture represents the case an operator
  actually meets rather than an exotic one;

* the URL host is ``example.invalid``. **RFC 2606 §2 reserves ``.invalid`` to
  be guaranteed not to resolve**, so a fixture that somehow reached a network
  stack cannot contact anything. ``example.com`` resolves and is somebody's
  server; ``localhost`` is the operator's own machine. Neither belongs in a
  file that exists to be opened by a test.

  ★ pdfcer never follows it — actions are recognised and round-tripped, never
  executed (NF4) — so this is belt and braces. It is belt and braces because
  the fixture will outlive that guarantee's current wording.

* an ``/AcroForm`` listing the field, because a widget unreachable from
  ``/AcroForm /Fields`` is an orphan and the shell's own forms panel would
  report it as one — a second, unrelated finding in a fixture that should test
  exactly one thing.

WHAT IT DELIBERATELY DOES NOT CONTAIN
======================================================================

No JavaScript, no ``/OpenAction``, no ``/Launch``. The disclosure under test
joins its findings into one sentence, and a fixture that triggered three of
them would let a build that had lost two still pass.

RUN
======================================================================

    python tools/gen-submit-fixture.py

Idempotent: writes the same bytes every time, so a regenerated fixture is not
a diff.
"""

from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "fixtures" / "submit-button.pdf"

# The page's ink: one horizontal rule. Named so `/Length` can be computed.
CONTENT = b"1 w 0 G 20 170 m 180 170 l S"

# Object bodies, 1-based; index 0 is the free head.
OBJECTS = [
    b"<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [4 0 R] >> >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Annots [4 0 R]"
    b" /Resources << >> /Contents 5 0 R >>",
    # The widget and the field are ONE dictionary (the merged Shape A form,
    # 12.5.6.19), which is what producers overwhelmingly write and what the
    # shell's own authoring path produces.
    b"<< /Type /Annot /Subtype /Widget /FT /Btn /Ff 65536 /T (Send)"
    b" /Rect [20 20 120 50] /F 4 /P 3 0 R"
    b" /MK << /CA (Send) >>"
    b" /A << /Type /Action /S /SubmitForm"
    b" /F (http://example.invalid/collect) /Flags 4 >> >>",
    # A single stroked line, so the page has ink. See the module docstring:
    # the shell will not open a page it cannot rasterise.
    # A single stroked line, so the page has ink. See the module docstring:
    # the shell will not open a page it cannot rasterise.
    #
    # ★ /Length is COMPUTED rather than written by hand: a wrong one is a
    # silently truncated stream, and hand-counting a byte string is the
    # classic way to produce a fixture that is subtly wrong.
    b"<< /Length %d >>\nstream\n%s\nendstream" % (len(CONTENT), CONTENT),
]


def build() -> bytes:
    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    for number, body in enumerate(OBJECTS, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % number
        out += body
        out += b"\nendobj\n"

    startxref = len(out)
    count = len(OBJECTS) + 1
    out += b"xref\n0 %d\n" % count
    out += b"0000000000 65535 f \n"
    for offset in offsets[1:]:
        out += b"%010d 00000 n \n" % offset
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\n" % count
    out += b"startxref\n%d\n%%%%EOF\n" % startxref
    return bytes(out)


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes(build())
    print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
