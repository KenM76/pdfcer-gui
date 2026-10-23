# -*- coding: utf-8 -*-
"""Build fixtures/print-shop-pdfx4.pdf.

fixtures/print-shop.pdf with one change: the catalog carries an XMP metadata
stream claiming PDF/X-4 (`pdfxid:GTS_PDFXVersion`), the way PDF/X-4 states
its level, and no `/OutputIntents` and no Info dictionary. The overprint
default "Only for PDF/X files" must read this as PDF/X: a reading that looks
only at the output intent and the Info key says it is not.
"""
import io

XMP = (
    '<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>\n'
    '<x:xmpmeta xmlns:x="adobe:ns:meta/">\n'
    '<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">\n'
    '<rdf:Description rdf:about="" '
    'xmlns:pdfxid="http://www.npes.org/pdfx/ns/id/" '
    'pdfxid:GTS_PDFXVersion="PDF/X-4"/>\n'
    '</rdf:RDF>\n'
    '</x:xmpmeta>\n'
    '<?xpacket end="w"?>'
)

src = io.open('fixtures/print-shop.PROVENANCE.py', encoding='utf-8').read()
old_cat = "b'<< /Type /Catalog /Pages 2 0 R >>'"
new_cat = "b'<< /Type /Catalog /Pages 2 0 R /Metadata 6 0 R >>'"
old_form = "    % len(form) + form + b'\\nendstream',\n]"
new_form = ("    % len(form) + form + b'\\nendstream',\n"
            "    b'<< /Type /Metadata /Subtype /XML /Length %d >>\\nstream\\n' % len(XMP_BYTES)"
            " + XMP_BYTES + b'\\nendstream',\n]")
old_dst = "DST = 'fixtures/print-shop.pdf'"
new_dst = "DST = 'fixtures/print-shop-pdfx4.pdf'"
assert src.count(old_cat) == 1, 'the catalog line moved in the sibling generator'
assert src.count(old_form) == 1, 'the object list moved in the sibling generator'
assert src.count(old_dst) == 1, 'the output path moved in the sibling generator'
XMP_BYTES = XMP.encode('utf-8')
exec(src.replace(old_cat, new_cat).replace(old_form, new_form).replace(old_dst, new_dst))
