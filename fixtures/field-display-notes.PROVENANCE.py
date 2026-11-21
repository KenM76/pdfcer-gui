# -*- coding: utf-8 -*-
"""Build fixtures/field-display-notes.pdf.

Two fields whose drawing differs from what they store, for the Forms
panel's display notes (`a_field_drawn_otherwise_says_why_in_the_forms_panel`):
  Amount  a text field whose /AA /F format action is a script pdfcer does
          not reproduce, so its value is drawn unformatted
  Pick    a list box whose /TI 7 names no option of its two

Written from scratch here; rights-clean. Offsets are computed.
"""
import io

NL = chr(10).encode('ascii')

objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [4 0 R 5 0 R 6 0 R] '
       b'/DA (/Helv 12 Tf 0 g) /DR << /Font << /Helv 7 0 R >> >> >> >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
       b'/Annots [4 0 R 5 0 R 6 0 R] >>',
    4: b'<< /Type /Annot /Subtype /Widget /FT /Tx /T (Amount) /Rect [72 650 400 680] '
       b'/F 4 /P 3 0 R /DA (/Helv 12 Tf 0 g) /V (1234.5) '
       b'/AA << /F << /S /JavaScript /JS (event.value = "#" + event.value;) >> >> >>',
    5: b'<< /Type /Annot /Subtype /Widget /FT /Ch /T (Pick) /Rect [72 520 400 600] '
       b'/F 4 /P 3 0 R /DA (/Helv 12 Tf 0 g) /Opt [(One) (Two)] /TI 7 /V (One) >>',
    6: b'<< /Type /Annot /Subtype /Widget /FT /Tx /T (Plain) /Rect [72 420 400 450] '
       b'/F 4 /P 3 0 R /DA (/Helv 12 Tf 0 g) /V (Hello) >>',
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

dst = 'fixtures/field-display-notes.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
