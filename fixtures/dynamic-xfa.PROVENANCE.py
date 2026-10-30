# -*- coding: utf-8 -*-
"""Build `fixtures/dynamic-xfa.pdf`: a dynamic XFA form whose only PDF page is
a "Please wait..." stand-in, as a LiveCycle-authored form ships.

The catalog says `/NeedsRendering true` and `/AcroForm /XFA` is a single XDP
stream holding a template: a title draw, two captioned fields (text and
numeric) and a rectangle. The engine lays it out as one page of five items
with no warnings. No `datasets` packet: filled-in values are not merged yet.

Run: python fixtures/dynamic-xfa.PROVENANCE.py
"""
import pathlib

XDP = (
    '<xdp:xdp xmlns:xdp="http://ns.adobe.com/xdp/"><template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">'
    '<subform name="form1" layout="tb"><pageSet><pageArea name="P1"><contentArea x="0.5in" y="0.5in" w="7.5in" h="10in"/>'
    '<medium stock="letter" short="8.5in" long="11in"/></pageArea></pageSet>\n'
    '<subform name="body" w="7.5in" layout="position" h="4in">\n'
    '<draw name="title" x="0in" y="0in" w="7.5in" h="0.5in"><value><text>Synthetic Rental Form</text></value><font typeface="Arial" size="18pt"/></draw>\n'
    '<field name="tenant" x="0in" y="0.8in" w="5in" h="0.35in"><caption reserve="1.5in"><value><text>Tenant name</text></value></caption><ui><textEdit/></ui><border><edge/></border></field>\n'
    '<field name="rent" x="0in" y="1.4in" w="3in" h="0.35in"><caption reserve="1.5in"><value><text>Monthly rent</text></value></caption><ui><numericEdit/></ui><border><edge/></border></field>\n'
    '<draw name="box" x="0in" y="2.2in" w="7in" h="1.2in"><value><rectangle><edge thickness="1pt"/></rectangle></value></draw>\n'
    '</subform></subform></template></xdp:xdp>'
).encode()
STAND_IN = b"BT /F1 14 Tf 72 700 Td (Please wait... this is the stand-in page.) Tj ET"

objects = [
    b"<< /Type /Catalog /Pages 2 0 R /NeedsRendering true /AcroForm << /Fields [] /XFA 4 0 R >> >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 5 0 R /Resources << /Font << /F1 6 0 R >> >> >>",
    b"<< /Length %d >>\nstream\n" % len(XDP) + XDP + b"\nendstream",
    b"<< /Length %d >>\nstream\n" % len(STAND_IN) + STAND_IN + b"\nendstream",
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
]
out = bytearray(b"%PDF-1.7\n")
offsets = []
for n, body in enumerate(objects, 1):
    offsets.append(len(out))
    out += b"%d 0 obj\n" % n + body + b"\nendobj\n"
xref = len(out)
out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objects) + 1)
for o in offsets:
    out += b"%010d 00000 n \n" % o
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objects) + 1, xref)
pathlib.Path(__file__).with_name("dynamic-xfa.pdf").write_bytes(bytes(out))
