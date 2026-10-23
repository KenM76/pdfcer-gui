# -*- coding: utf-8 -*-
"""Build fixtures/print-shop-pdfx.pdf.

fixtures/print-shop.pdf with one change: the catalog carries an
/OutputIntents entry with /S /GTS_PDFX, the claim a PDF/X file makes. The
intent names a condition and carries no profile, so the page draws exactly as
print-shop.pdf does; only the claim differs, which is what the overprint
default "Only for PDF/X files" reads.
"""
import io

src = io.open('fixtures/print-shop.PROVENANCE.py', encoding='utf-8').read()
old_cat = "b'<< /Type /Catalog /Pages 2 0 R >>'"
new_cat = ("b'<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent "
           "/S /GTS_PDFX /OutputConditionIdentifier (CGATS TR 001) >>] >>'")
old_dst = "DST = 'fixtures/print-shop.pdf'"
new_dst = "DST = 'fixtures/print-shop-pdfx.pdf'"
assert src.count(old_cat) == 1, 'the catalog line moved in the sibling generator'
assert src.count(old_dst) == 1, 'the output path moved in the sibling generator'
exec(src.replace(old_cat, new_cat).replace(old_dst, new_dst))
