# -*- coding: utf-8 -*-
"""Build `fixtures/xfa-choices.pdf`: a dynamic XFA form for the Form preview
window's Reading choices.

One letter page, one positioned subform:

- `shade`, a rectangle filled mid-grey (160,160,160) with a hidden edge;
- `unfilled`, a text field over the middle of `shade` whose `border` has an
  edge and no `fill`, so `MissingFill::None` lets the grey show through and
  `MissingFill::White` paints its box white;
- `screen_only`, a draw with `relevant="-print"`, so it is drawn for the
  form's own views (the file names none) and left out when the print view is
  chosen.

The file has no `config` packet and no data. Its only PDF page is a
"Please wait..." stand-in, as a LiveCycle-authored form ships.

Run: python fixtures/xfa-choices.PROVENANCE.py
"""
import pathlib

XDP = (
    '<xdp:xdp xmlns:xdp="http://ns.adobe.com/xdp/"><template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">'
    '<subform name="form1" layout="tb"><pageSet><pageArea name="P1"><contentArea x="0.5in" y="0.5in" w="7.5in" h="10in"/>'
    '<medium stock="letter" short="8.5in" long="11in"/></pageArea></pageSet>\n'
    '<subform name="body" w="7.5in" layout="position" h="4in">\n'
    '<draw name="shade" x="0in" y="0in" w="5in" h="1.5in"><value><rectangle><edge presence="hidden"/>'
    '<fill><color value="160,160,160"/></fill></rectangle></value></draw>\n'
    '<field name="unfilled" x="1in" y="0.5in" w="3in" h="0.5in"><ui><textEdit/></ui><border><edge/></border></field>\n'
    '<draw name="screen_only" relevant="-print" x="0in" y="2in" w="5in" h="0.5in"><value><text>Shown on screen only</text></value>'
    '<font typeface="Arial" size="14pt"/></draw>\n'
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
pathlib.Path(__file__).with_name("xfa-choices.pdf").write_bytes(bytes(out))
