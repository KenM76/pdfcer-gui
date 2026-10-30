# -*- coding: utf-8 -*-
"""Build `fixtures/certified-no-changes.pdf`: `certified-comments.pdf` with its
DocMDP permission changed from `/P 2` to `/P 1` (no changes permitted).

The two strings are the same length, so every xref offset still holds. The
signature's `/Contents` is filler, as in the source fixture: the document
counts as certified for every census, and carries no certificate of its own.

Used by `validation_evidence_added_without_the_mouse` to drive the
"add it anyway" path of Security > Add validation evidence.

Run: python fixtures/certified-no-changes.PROVENANCE.py
"""
import pathlib

here = pathlib.Path(__file__).parent
src = (here / "certified-comments.pdf").read_bytes()
assert src.count(b"/TransformParams /P 2 ") == 1
out = src.replace(b"/TransformParams /P 2 ", b"/TransformParams /P 1 ")
(here / "certified-no-changes.pdf").write_bytes(out)
