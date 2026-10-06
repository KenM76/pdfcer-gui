# -*- coding: utf-8 -*-
"""Build fixtures/ocr-two-columns.pdf: one image-only Letter page, a scan of
two columns of two paragraphs each, with no text on the page.

Each paragraph opens with a word no other paragraph holds, in reading order
Apples, Bridges (left column), Candles, Dolphins (right column). The two
columns' lines sit at the same heights, so a writer laying the recognised
words out line by line across the page interleaves the columns, and one
laying them out block by block in reading order does not.

Rendered with Arial at 200 dpi, 11 pt, greyscale, Flate-compressed.
Run from the repository root with Pillow installed.
"""
import io
import zlib

from PIL import Image, ImageDraw, ImageFont

DPI = 200
W_PT, H_PT = 612, 792
W, H = W_PT * DPI // 72, H_PT * DPI // 72
FONT = ImageFont.truetype('C:/Windows/Fonts/arial.ttf', 11 * DPI // 72)
LEADING = 16 * DPI // 72
COLUMN_WIDTH = int(3.1 * DPI)
COLUMNS_X = (int(1.0 * DPI), int(4.4 * DPI))
TOP = int(1.2 * DPI)

PARAGRAPHS = [
    [
        'Apples ripen slowly in the cool orchard behind the old farmhouse, and '
        'the family gathers them by hand every autumn before the first frost.',
        'Bridges over the river were rebuilt after the flood, with stronger '
        'piers of stone and a wider deck for the heavy trucks that cross daily.',
    ],
    [
        'Candles burned in every window of the small village during the long '
        'winter evenings, when the roads were closed by snow for many weeks.',
        'Dolphins followed the fishing boats out of the harbour each morning, '
        'leaping through the bright waves while the crews prepared their nets.',
    ],
]


def wrap(draw, text):
    lines, line = [], ''
    for word in text.split():
        trial = f'{line} {word}'.strip()
        if draw.textlength(trial, font=FONT) <= COLUMN_WIDTH:
            line = trial
        else:
            lines.append(line)
            line = word
    lines.append(line)
    return lines


image = Image.new('L', (W, H), 255)
draw = ImageDraw.Draw(image)
for x, column in zip(COLUMNS_X, PARAGRAPHS):
    y = TOP
    for paragraph in column:
        for line in wrap(draw, paragraph):
            draw.text((x, y), line, font=FONT, fill=0)
            y += LEADING
        y += LEADING

pixels = zlib.compress(image.tobytes(), 9)
content = f'q {W_PT} 0 0 {H_PT} 0 0 cm /Im0 Do Q'.encode()
objs = [
    b'<< /Type /Catalog /Pages 2 0 R >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {W_PT} {H_PT}] '
    f'/Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>'.encode(),
    (f'<< /Type /XObject /Subtype /Image /Width {W} /Height {H} /ColorSpace /DeviceGray '
     f'/BitsPerComponent 8 /Filter /FlateDecode /Length {len(pixels)} >>\nstream\n').encode()
    + pixels + b'\nendstream',
    f'<< /Length {len(content)} >>\nstream\n'.encode() + content + b'\nendstream',
]

out = io.BytesIO()
out.write(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
offsets = []
for number, body in enumerate(objs, 1):
    offsets.append(out.tell())
    out.write(f'{number} 0 obj\n'.encode() + body + b'\nendobj\n')
xref = out.tell()
out.write(f'xref\n0 {len(objs) + 1}\n0000000000 65535 f \n'.encode())
for offset in offsets:
    out.write(f'{offset:010d} 00000 n \n'.encode())
out.write(f'trailer\n<< /Size {len(objs) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode())
with open('fixtures/ocr-two-columns.pdf', 'wb') as f:
    f.write(out.getvalue())
print(len(out.getvalue()), 'bytes')
