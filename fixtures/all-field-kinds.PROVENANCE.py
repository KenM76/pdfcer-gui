# -*- coding: utf-8 -*-
"""Build fixtures/all-field-kinds.pdf -- one widget of every field kind.

## Why this fixture exists

O205: the form tools are each missing a different part of themselves, and no
fixture in this repository carries more than one kind at a time. The corpus has
text fields (`three-text-fields.pdf`, `text-field-with-appearance.pdf`,
`autosize-field.pdf`), a push button (`submit-button.pdf`) and a signature
(`signed-two-pages.pdf`) -- and **no choice field at all**, which is why "a
drop-down cannot be filled in on the canvas" was never going to be caught by a
driven check. A parity table needs a document that can exercise every cell.

## What it carries

| field | kind | what it is for |
|---|---|---|
| `TextOne` | `/Tx` | the baseline; a value and an appearance |
| `CheckOne` | `/Btn` check box | two appearance substates, `/AS /Off`, `/MK /CA (4)` -- Acrobat's check |
| `RadioGroup` | `/Btn` radio | ONE field, TWO widget kids, export names `/A` and `/B` |
| `ComboOne` | `/Ch` combo | closed drop-down, three plain `/Opt` strings |
| `ComboEdit` | `/Ch` combo + edit | the editable variant, which is a different `/Ff` bit |
| `ListOne` | `/Ch` list | `/Opt` as **export/display pairs**, `/TI` top index |
| `ListMulti` | `/Ch` list + multiselect | `/I` selected indices, which only a multi-select list has |
| `PushOne` | `/Btn` push button | a caption in `/MK /CA`, and deliberately NO action |
| `SigOne` | `/Sig` | unsigned, so the box is empty and the kind still has properties |

Nine fields, ten widgets. `/Annots` lists ten entries; `/AcroForm /Fields`
lists nine, because a radio group is one field.

## Two properties that are load-bearing, and are easy to lose on a rewrite

**Every widget carries an `/AP`.** `canvas::forms::boxes` admits a widget to
the on-canvas census only when it can draw it, and an `/AP`-less field is
routed to the properties panel instead -- so a fixture without appearance
streams cannot be clicked on the page, which is the gesture under test. The
check box and the radio kids carry the two-substate form (`/N << /Yes ... /Off
... >>`), because a substate dictionary is what `/AS` selects from and a check
box drawn from a single stream cannot be toggled by a reader at all.

**`/NeedAppearances` is absent, deliberately.** With it true a viewer is
entitled to rebuild every appearance from `/DA` and `/V`, which would mask a
shell that writes a value and forgets to regenerate. Its absence makes the
appearance streams the only thing on screen, so a stale one shows.

Offsets are computed here rather than written out, and object numbers are
allocated rather than hand-counted, so a field can be added in the middle
without renumbering the rest.
"""
import io
import os

NL = chr(10).encode('ascii')

# --- object allocator -------------------------------------------------------
#
# `emit` returns the number it assigned, so a dictionary can reference an
# object allocated two lines above it by name rather than by a number the
# author has to keep in their head. Objects come out in allocation order,
# which is also numeric order, which is what the xref table wants.
_objects = []


def emit(body):
    _objects.append(body)
    return len(_objects)


def ref(num):
    return '%d 0 R' % num


def stream(dict_head, body):
    """A stream object: dictionary with the /Length filled in, then the body."""
    return (
        (dict_head % len(body)).encode('ascii') + NL
        + b'stream' + NL + body + NL + b'endstream'
    )


# --- fixed objects ----------------------------------------------------------
#
# The catalog, page tree, page and content stream are patched after the fields
# are built, because they have to name objects that do not exist yet. They are
# allocated first anyway so that the low numbers stay readable in a hex dump.
CATALOG = emit(b'PLACEHOLDER')
PAGES = emit(b'PLACEHOLDER')
PAGE = emit(b'PLACEHOLDER')
CONTENTS = emit(b'PLACEHOLDER')
HELV = emit(
    b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
    b'/Encoding /WinAnsiEncoding >>'
)
ZADB = emit(b'<< /Type /Font /Subtype /Type1 /BaseFont /ZapfDingbats >>')

PAGE_W, PAGE_H = 420, 620
BOX_W, BOX_H = 200, 22
LEFT = 170


def appearance(text, w=BOX_W, h=BOX_H, font='/Helv', size=10, plate=None):
    """A widget appearance: optional plate, optional border, one line of text.

    `plate` is a grey level for the background; a push button gets one and a
    text field does not, which is the visible difference between the two kinds
    before either is touched.
    """
    ops = []
    if plate is not None:
        ops.append('q %s g 0 0 %d %d re f Q' % (plate, w, h))
    ops.append('q 0.5 G 0.5 w 0.25 0.25 %.1f %.1f re S Q'
               % (w - 0.5, h - 0.5))
    if text:
        ops.append('/Tx BMC q BT %s %d Tf 0 g 3 %d Td (%s) Tj ET Q EMC'
                   % (font, size, (h - size) // 2 + 1, text))
    body = (chr(10).join(ops)).encode('ascii')
    head = (
        '<< /Type /XObject /Subtype /Form /BBox [0 0 %d %d] '
        '/Resources << /Font << /Helv %d 0 R /ZaDb %d 0 R >> >> /Length %%d >>'
        % (w, h, HELV, ZADB)
    )
    return stream(head, body)


def check_appearance(on, size=16):
    """The two substates of a check box or radio button.

    `on` is the ZapfDingbats character the /MK /CA style names: `4` is
    Acrobat's check, `8` its cross, `l` (lowercase L) the filled circle a radio
    button uses. The OFF substate is an empty stream with the border only --
    not an absent one, because /AS has to be able to name it.
    """
    border = 'q 0.5 G 0.7 w 0.35 0.35 %.1f %.1f re S Q' % (size - 0.7, size - 0.7)
    on_body = (border + chr(10) +
               'q BT /ZaDb %d Tf 0 g 3 4 Td (%s) Tj ET Q' % (size - 5, on)
               ).encode('ascii')
    off_body = border.encode('ascii')
    head = (
        '<< /Type /XObject /Subtype /Form /BBox [0 0 %d %d] '
        '/Resources << /Font << /ZaDb %d 0 R >> >> /Length %%d >>'
        % (size, size, ZADB)
    )
    return stream(head, on_body), stream(head, off_body)


# --- the fields -------------------------------------------------------------
#
# Laid out top to bottom in the order the parity table lists them, each with a
# caption drawn into the page content so a screenshot of either application is
# self-describing.
rows = []          # (label, y, height) for the page content stream
fields = []        # /AcroForm /Fields entries (one per FIELD)
annots = []        # /Annots entries (one per WIDGET)

y = PAGE_H - 60


def place(label, height=BOX_H):
    global y
    y -= height + 18
    rows.append((label, y, height))
    return y


# 1. text field
ty = place('Text field')
ap = emit(appearance('Ada Lovelace'))
num = emit((
    '<< /Type /Annot /Subtype /Widget /FT /Tx /T (TextOne) '
    '/TU (A single-line text field) /Rect [%d %d %d %d] /F 4 /P %s '
    '/DA (/Helv 10 Tf 0 g) /V (Ada Lovelace) /MaxLen 40 /Q 0 '
    '/MK << /BC [0 0 0] /BG [1 1 1] >> /BS << /W 1 /S /S >> '
    '/AP << /N %s >> >>'
    % (LEFT, ty, LEFT + BOX_W, ty + BOX_H, ref(PAGE), ref(ap))
).encode('ascii'))
fields.append(num)
annots.append(num)

# 2. check box
cy = place('Check box', 16)
c_on, c_off = check_appearance('4')
c_on = emit(c_on)
c_off = emit(c_off)
num = emit((
    '<< /Type /Annot /Subtype /Widget /FT /Btn /T (CheckOne) '
    '/TU (A check box, currently off) /Rect [%d %d %d %d] /F 4 /P %s '
    '/DA (/ZaDb 0 Tf 0 g) /V /Off /AS /Off '
    '/MK << /CA (4) /BC [0 0 0] /BG [1 1 1] >> /BS << /W 1 /S /S >> '
    '/AP << /N << /Yes %s /Off %s >> >> >>'
    % (LEFT, cy, LEFT + 16, cy + 16, ref(PAGE), ref(c_on), ref(c_off))
).encode('ascii'))
fields.append(num)
annots.append(num)

# 3. radio group -- one field, two widget kids
ry = place('Radio buttons', 16)
r_on, r_off = check_appearance('l')
kid_aps = [(emit(r_on), emit(r_off)), None]
r_on2, r_off2 = check_appearance('l')
kid_aps[1] = (emit(r_on2), emit(r_off2))
RADIO_PARENT = len(_objects) + 3   # two kids are allocated before the parent
kids = []
for i, (export, dx) in enumerate((('A', 0), ('B', 90))):
    on_ap, off_ap = kid_aps[i]
    kids.append(emit((
        '<< /Type /Annot /Subtype /Widget /Parent %s /Rect [%d %d %d %d] '
        '/F 4 /P %s /AS /Off /MK << /CA (l) /BC [0 0 0] /BG [1 1 1] >> '
        '/BS << /W 1 /S /S >> /AP << /N << /%s %s /Off %s >> >> >>'
        % (ref(RADIO_PARENT), LEFT + dx, ry, LEFT + dx + 16, ry + 16,
           ref(PAGE), export, ref(on_ap), ref(off_ap))
    ).encode('ascii')))
parent = emit((
    '<< /FT /Btn /T (RadioGroup) /TU (Pick one) /Ff 32768 /V /Off '
    '/DA (/ZaDb 0 Tf 0 g) /Kids [%s %s] >>'
    % (ref(kids[0]), ref(kids[1]))
).encode('ascii'))
assert parent == RADIO_PARENT, 'radio parent landed at %d, kids expect %d' % (
    parent, RADIO_PARENT)
fields.append(parent)
annots.extend(kids)

# 4. combo box, closed
by = place('Drop-down')
ap = emit(appearance('Green'))
num = emit((
    '<< /Type /Annot /Subtype /Widget /FT /Ch /T (ComboOne) '
    '/TU (A closed drop-down) /Rect [%d %d %d %d] /F 4 /P %s '
    '/DA (/Helv 10 Tf 0 g) /Ff 131072 /Opt [(Red) (Green) (Blue)] '
    '/V (Green) /Q 0 /MK << /BC [0 0 0] /BG [1 1 1] >> /BS << /W 1 /S /S >> '
    '/AP << /N %s >> >>'
    % (LEFT, by, LEFT + BOX_W, by + BOX_H, ref(PAGE), ref(ap))
).encode('ascii'))
fields.append(num)
annots.append(num)

# 5. combo box, editable -- /Ff adds the Edit bit (1 << 18)
ey = place('Drop-down, editable')
ap = emit(appearance('Other'))
num = emit((
    '<< /Type /Annot /Subtype /Widget /FT /Ch /T (ComboEdit) '
    '/TU (An editable drop-down) /Rect [%d %d %d %d] /F 4 /P %s '
    '/DA (/Helv 10 Tf 0 g) /Ff 393216 /Opt [(Small) (Medium) (Large)] '
    '/V (Other) /MK << /BC [0 0 0] /BG [1 1 1] >> /BS << /W 1 /S /S >> '
    '/AP << /N %s >> >>'
    % (LEFT, ey, LEFT + BOX_W, ey + BOX_H, ref(PAGE), ref(ap))
).encode('ascii'))
fields.append(num)
annots.append(num)

# 6. list box, single selection, /Opt as export/display PAIRS
ly = place('List box', 46)
ap = emit(appearance('Toronto', h=46))
num = emit((
    '<< /Type /Annot /Subtype /Widget /FT /Ch /T (ListOne) '
    '/TU (A list box) /Rect [%d %d %d %d] /F 4 /P %s '
    '/DA (/Helv 10 Tf 0 g) '
    '/Opt [[(tor) (Toronto)] [(mtl) (Montreal)] [(van) (Vancouver)]] '
    '/V (tor) /TI 0 /MK << /BC [0 0 0] /BG [1 1 1] >> /BS << /W 1 /S /S >> '
    '/AP << /N %s >> >>'
    % (LEFT, ly, LEFT + BOX_W, ly + 46, ref(PAGE), ref(ap))
).encode('ascii'))
fields.append(num)
annots.append(num)

# 7. list box, multi-select -- /I is the selected-index array
my = place('List box, multi-select', 46)
ap = emit(appearance('Mon, Wed', h=46))
num = emit((
    '<< /Type /Annot /Subtype /Widget /FT /Ch /T (ListMulti) '
    '/TU (Choose any number) /Rect [%d %d %d %d] /F 4 /P %s '
    '/DA (/Helv 10 Tf 0 g) /Ff 2097152 '
    '/Opt [(Mon) (Tue) (Wed) (Thu) (Fri)] /V [(Mon) (Wed)] /I [0 2] '
    '/MK << /BC [0 0 0] /BG [1 1 1] >> /BS << /W 1 /S /S >> '
    '/AP << /N %s >> >>'
    % (LEFT, my, LEFT + BOX_W, my + 46, ref(PAGE), ref(ap))
).encode('ascii'))
fields.append(num)
annots.append(num)

# 8. push button -- a caption, a plate, and NO /A, which is pdfcer policy
py = place('Push button')
ap = emit(appearance('Press me', plate='0.75'))
num = emit((
    '<< /Type /Annot /Subtype /Widget /FT /Btn /T (PushOne) '
    '/TU (A push button that does nothing) /Rect [%d %d %d %d] /F 4 /P %s '
    '/DA (/Helv 10 Tf 0 g) /Ff 65536 '
    '/MK << /CA (Press me) /BC [0 0 0] /BG [0.75 0.75 0.75] >> '
    '/BS << /W 1 /S /B >> /AP << /N %s >> >>'
    % (LEFT, py, LEFT + BOX_W, py + BOX_H, ref(PAGE), ref(ap))
).encode('ascii'))
fields.append(num)
annots.append(num)

# 9. signature field, unsigned
sy = place('Signature', 40)
ap = emit(appearance('', h=40))
num = emit((
    '<< /Type /Annot /Subtype /Widget /FT /Sig /T (SigOne) '
    '/TU (Sign here) /Rect [%d %d %d %d] /F 4 /P %s '
    '/MK << /BC [0 0 0] >> /BS << /W 1 /S /S >> /AP << /N %s >> >>'
    % (LEFT, sy, LEFT + BOX_W, sy + 40, ref(PAGE), ref(ap))
).encode('ascii'))
fields.append(num)
annots.append(num)

# --- page content: one caption per row, plus a title ------------------------
ops = [
    'BT /Helv 13 Tf 20 %d Td (All field kinds) Tj ET' % (PAGE_H - 35),
]
for label, ry_, rh in rows:
    ops.append('BT /Helv 9 Tf 20 %d Td (%s) Tj ET' % (ry_ + rh - 11, label))
CONTENT = (chr(10).join(ops)).encode('ascii')

_objects[CATALOG - 1] = (
    '<< /Type /Catalog /Pages %s /AcroForm << /Fields [%s] '
    '/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv %s /ZaDb %s >> >> >> >>'
    % (ref(PAGES), ' '.join(ref(n) for n in fields), ref(HELV), ref(ZADB))
).encode('ascii')
_objects[PAGES - 1] = (
    '<< /Type /Pages /Kids [%s] /Count 1 >>' % ref(PAGE)
).encode('ascii')
_objects[PAGE - 1] = (
    '<< /Type /Page /Parent %s /MediaBox [0 0 %d %d] '
    '/Resources << /Font << /Helv %s /ZaDb %s >> >> /Contents %s '
    '/Annots [%s] >>'
    % (ref(PAGES), PAGE_W, PAGE_H, ref(HELV), ref(ZADB), ref(CONTENTS),
       ' '.join(ref(n) for n in annots))
).encode('ascii')
_objects[CONTENTS - 1] = stream('<< /Length %d >>', CONTENT)

# --- serialise --------------------------------------------------------------
out = bytearray()
out += b'%PDF-1.7' + NL
out += b'%' + bytes(bytearray([0xE2, 0xE3, 0xCF, 0xD3])) + NL

offsets = {}
for i, body in enumerate(_objects, start=1):
    assert body != b'PLACEHOLDER', 'object %d was never filled in' % i
    offsets[i] = len(out)
    out += ('%d 0 obj' % i).encode('ascii') + NL
    out += body + NL
    out += b'endobj' + NL

startxref = len(out)
count = len(_objects) + 1
out += b'xref' + NL
out += ('0 %d' % count).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for i in range(1, count):
    out += ('%010d 00000 n ' % offsets[i]).encode('ascii') + NL
out += b'trailer' + NL
out += ('<< /Size %d /Root %s >>' % (count, ref(CATALOG))).encode('ascii') + NL
out += b'startxref' + NL
out += ('%d' % startxref).encode('ascii') + NL
out += b'%%EOF' + NL

dest = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    'all-field-kinds.pdf')
with io.open(dest, 'wb') as f:
    f.write(bytes(out))

print('wrote %s: %d bytes, %d objects, %d fields, %d widgets'
      % (dest, len(out), len(_objects), len(fields), len(annots)))
