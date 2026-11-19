# -*- coding: utf-8 -*-
"""Build `fixtures/xfa-widgets.pdf`: a dynamic XFA form whose fields take every
entry Form preview offers other than text, behind a "Please wait..." stand-in.

One page, laid out by position, every field bound by `match="dataRef"`:

- `agree`, a check button whose `items` are Y / N; the data holds N (off).
- `pets`, a check button with no `items`, so 1 / 0; the data holds nothing.
- `plan`, an exclusion group of two round buttons, `monthly` (M) and
  `yearly` (A); the group binds `$.Plan`, which holds M.
- `city`, a drop-down list (`open="userControl"`) of three cities whose
  display and save lists are the same; the data holds Ottawa.
- `colour`, a list always shown (`open="always"`) whose display names
  differ from its saved codes (r / g / b); the data holds g.
- `note`, a multi-line text field with `maxChars="40"`.

Run: python fixtures/xfa-widgets.PROVENANCE.py
"""
import pathlib


def items(*values, save=False):
    body = "".join("<text>%s</text>" % v for v in values)
    return '<items%s>%s</items>' % (' save="1"' if save else "", body)


def caption(text):
    return '<caption reserve="1.4in"><value><text>%s</text></value></caption>' % text


XDP = (
    '<xdp:xdp xmlns:xdp="http://ns.adobe.com/xdp/"><template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">'
    '<subform name="form1" layout="tb"><pageSet><pageArea name="P1"><contentArea x="0.5in" y="0.5in" w="7.5in" h="10in"/>'
    '<medium stock="letter" short="8.5in" long="11in"/></pageArea></pageSet>\n'
    '<subform name="body" w="7.5in" layout="position" h="6in">\n'
    '<draw name="title" x="0in" y="0in" w="7.5in" h="0.5in"><value><text>Synthetic Widget Form</text></value><font typeface="Arial" size="18pt"/></draw>\n'
    '<field name="agree" x="0in" y="0.8in" w="3in" h="0.35in">' + caption("I agree")
    + '<ui><checkButton shape="square" size="10pt"/></ui>' + items("Y", "N")
    + '<border><edge/></border><bind match="dataRef" ref="$.Agree"/></field>\n'
    '<field name="pets" x="3.5in" y="0.8in" w="3in" h="0.35in">' + caption("Pets")
    + '<ui><checkButton shape="square" size="10pt"/></ui>'
    '<border><edge/></border><bind match="dataRef" ref="$.Pets"/></field>\n'
    '<exclGroup name="plan" x="0in" y="1.4in" w="7in" h="0.35in" layout="position"><bind match="dataRef" ref="$.Plan"/>'
    '<field name="monthly" x="0in" y="0in" w="3in" h="0.35in">' + caption("Monthly")
    + '<ui><checkButton shape="round" size="10pt"/></ui>' + items("M") + '</field>'
    '<field name="yearly" x="3.5in" y="0in" w="3in" h="0.35in">' + caption("Yearly")
    + '<ui><checkButton shape="round" size="10pt"/></ui>' + items("A") + '</field>'
    '</exclGroup>\n'
    '<field name="city" x="0in" y="2.0in" w="4in" h="0.35in">' + caption("City")
    + '<ui><choiceList open="userControl"/></ui>' + items("Toronto", "Ottawa", "Calgary")
    + '<border><edge/></border><bind match="dataRef" ref="$.City"/></field>\n'
    '<field name="colour" x="0in" y="2.6in" w="4in" h="0.9in">' + caption("Colour")
    + '<ui><choiceList open="always"/></ui>' + items("Red", "Green", "Blue")
    + items("r", "g", "b", save=True)
    + '<border><edge/></border><bind match="dataRef" ref="$.Colour"/></field>\n'
    '<field name="note" x="0in" y="3.8in" w="7in" h="0.8in">' + caption("Note")
    + '<ui><textEdit multiLine="1"/></ui><value><text maxChars="40"/></value>'
    '<border><edge/></border><bind match="dataRef" ref="$.Note"/></field>\n'
    '</subform></subform></template>'
    '<xfa:datasets xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/"><xfa:data><form1>'
    '<Agree>N</Agree><Pets/><Plan>M</Plan><City>Ottawa</City><Colour>g</Colour><Note/>'
    '</form1></xfa:data></xfa:datasets></xdp:xdp>'
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
pathlib.Path(__file__).with_name("xfa-widgets.pdf").write_bytes(bytes(out))
