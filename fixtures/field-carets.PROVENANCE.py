# -*- coding: utf-8 -*-
"""Build fixtures/field-carets.pdf.

Three empty text fields whose own layout differs from a plain left-set line
at the box's height, so an editor that sets the typed text in its own font
puts glyphs and caret somewhere the saved appearance does not:

  Comb   /Ff Comb, /MaxLen 6, /DA /Helv 18   rect [72 600 372 640]  50 pt cells
  Right  /Q 2,                /DA /Helv 12   rect [72 520 372 560]
  Para   /Ff Multiline,       /DA /Helv 14   rect [72 360 232 480]

Each widget carries an empty /AP /N so the canvas offers it for filling.
`ui-verify` `a_field_editor_sets_text_where_the_field_draws_it` types into
each and compares the editing picture with the committed one. Written from
scratch here; rights-clean. Offsets are computed.
"""
import io

NL = chr(10).encode('ascii')


def widget(name, rect, extra, da, ap_ref):
    return (
        '<< /Type /Annot /Subtype /Widget /FT /Tx /T (%s) /Rect [%s] /F 4 /P 3 0 R '
        '/DA (%s) %s /AP << /N %d 0 R >> >>' % (name, rect, da, extra, ap_ref)
    ).encode('ascii')


def blank(w, h):
    body = b'/Tx BMC EMC'
    head = ('<< /Type /XObject /Subtype /Form /BBox [0 0 %d %d] /Length %d >>'
            % (w, h, len(body))).encode('ascii')
    return head + NL + b'stream' + NL + body + NL + b'endstream'


objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [4 0 R 5 0 R 6 0 R] '
       b'/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv 10 0 R >> >> >> >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
       b'/Annots [4 0 R 5 0 R 6 0 R] >>',
    4: widget('Comb', '72 600 372 640', '/Ff 16777216 /MaxLen 6', '/Helv 18 Tf 0 g', 7),
    5: widget('Right', '72 520 372 560', '/Q 2', '/Helv 12 Tf 0 g', 8),
    6: widget('Para', '72 360 232 480', '/Ff 4096', '/Helv 14 Tf 0 g', 9),
    7: blank(300, 40),
    8: blank(300, 40),
    9: blank(160, 120),
    10: b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
        b'/Encoding /WinAnsiEncoding >>',
}

out = bytearray()
out += b'%PDF-1.7' + NL
out += b'%' + bytes(bytearray([0xE2, 0xE3, 0xCF, 0xD3])) + NL
offsets = {}
for num in sorted(objects):
    offsets[num] = len(out)
    out += ('%d 0 obj' % num).encode('ascii') + NL
    out += objects[num] + NL
    out += b'endobj' + NL
size = max(objects) + 1
startxref = len(out)
out += ('xref' + chr(10) + '0 %d' % size).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for num in range(1, size):
    out += ('%010d 00000 n ' % offsets[num]).encode('ascii') + NL
out += b'trailer' + NL
out += ('<< /Size %d /Root 1 0 R >>' % size).encode('ascii') + NL
out += b'startxref' + NL
out += ('%d' % startxref).encode('ascii') + NL
out += b'%%EOF' + NL

dst = 'fixtures/field-carets.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
