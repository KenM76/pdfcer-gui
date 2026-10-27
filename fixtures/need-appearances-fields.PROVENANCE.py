# -*- coding: utf-8 -*-
"""Build fixtures/need-appearances-fields.pdf.

A form that asks the viewer to draw its values: /NeedAppearances true and no
/AP on any widget. Acrobat draws such fields; O327's driven check
(`ui-verify` `a_form_without_drawn_values_draws_and_clicks`) asserts this
program does too, without modifying the document.

Three /Tx widgets, each with a value and a blue /MK /BC border:
  Tall30   30 pt tall, /DA 0 Tf (auto-size)
  Tall60   60 pt tall, /DA 0 Tf (auto-size)
  Fixed12  60 pt tall, /DA 12 Tf

Written from scratch here; rights-clean. Offsets are computed.
"""
import io

NL = chr(10).encode('ascii')


def widget(name, y0, y1, size):
    return (
        '<< /Type /Annot /Subtype /Widget /FT /Tx /T (%s) /Rect [72 %d 400 %d] '
        '/F 4 /P 3 0 R /DA (/Helv %d Tf 0 g) /V (Hello) /MK << /BC [0 0 1] >> >>'
        % (name, y0, y1, size)
    ).encode('ascii')


objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [4 0 R 5 0 R 6 0 R] '
       b'/NeedAppearances true /DA (/Helv 0 Tf 0 g) '
       b'/DR << /Font << /Helv 7 0 R >> >> >> >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
       b'/Annots [4 0 R 5 0 R 6 0 R] >>',
    4: widget('Tall30', 650, 680, 0),
    5: widget('Tall60', 520, 580, 0),
    6: widget('Fixed12', 420, 480, 12),
    7: b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
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

startxref = len(out)
out += ('xref' + chr(10) + '0 %d' % (len(objects) + 1)).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for num in sorted(objects):
    out += ('%010d 00000 n ' % offsets[num]).encode('ascii') + NL
out += b'trailer' + NL
out += ('<< /Size %d /Root 1 0 R >>' % (len(objects) + 1)).encode('ascii') + NL
out += b'startxref' + NL
out += ('%d' % startxref).encode('ascii') + NL
out += b'%%EOF' + NL

dst = 'fixtures/need-appearances-fields.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
