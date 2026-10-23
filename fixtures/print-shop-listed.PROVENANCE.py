# -*- coding: utf-8 -*-
"""Build fixtures/print-shop-listed.pdf.

fixtures/print-shop.pdf with one change: the magenta patch is painted without
`/GS0 gs`. The page's resources still list `/GS0` (`/OP true /op true`), but
nothing invokes it, so nothing on the page asks for overprint. The overprint
default "Automatic" must read this as "no": a reading that counts a listed
graphics state instead of an invoked one says "yes".
"""
import io

src = io.open('fixtures/print-shop.PROVENANCE.py', encoding='utf-8').read()
old_paint = "q /GS0 gs 0 1 0 0 k"
new_paint = "q 0 1 0 0 k"
old_dst = "DST = 'fixtures/print-shop.pdf'"
new_dst = "DST = 'fixtures/print-shop-listed.pdf'"
assert src.count(old_paint) == 1, 'the overprinting patch moved in the sibling generator'
assert src.count(old_dst) == 1, 'the output path moved in the sibling generator'
exec(src.replace(old_paint, new_paint).replace(old_dst, new_dst))
