"""Author `ink-manager.pdf`: one Letter page painted in three spot inks, for
File > Print > Ink Manager.

    python fixtures/ink-manager.PROVENANCE.py

Deterministic: no timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/ink_manager.rs`.

- A page group whose blending space is DeviceCMYK, so Output Preview
  composites in ink and lists one plate per spot.
- `/CS0` is `/Separation /PANTONE#20185#20C`, `/CS1` is `/Separation /PMS185`:
  the same red under two names, the case an alias merges. `/CS2` is
  `/Separation /Gold`, the case Convert to process removes. All three have a
  DeviceCMYK alternate and a Type 2 tint function, so both verbs accept them.
- Three filled patches, one per space at tint 1, and a 40% cyan control in
  plain DeviceCMYK that neither verb touches.
"""
import io
import os

HERE = os.path.dirname(os.path.abspath(__file__))
DST = os.path.join(HERE, 'ink-manager.pdf')

content = '\n'.join(
    [
        '/CS0 cs 1 scn 72 500 200 150 re f',
        '/CS1 cs 1 scn 340 500 200 150 re f',
        '/CS2 cs 1 scn 72 300 200 150 re f',
        '0.4 0 0 0 k 340 300 200 150 re f',
    ]
).encode('ascii')


def separation(name, cmyk):
    c1 = ' '.join(str(v) for v in cmyk).encode('ascii')
    return (
        b'[/Separation /' + name + b' /DeviceCMYK << /FunctionType 2 /Domain [0 1] '
        b'/C0 [0 0 0 0] /C1 [' + c1 + b'] /N 1 >>]'
    )


objects = [
    b'<< /Type /Catalog /Pages 2 0 R >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    b'/Group << /S /Transparency /CS /DeviceCMYK >> '
    b'/Resources << /ColorSpace << /CS0 5 0 R /CS1 6 0 R /CS2 7 0 R >> >> '
    b'/Contents 4 0 R >>',
    b'<< /Length %d >>\nstream\n' % len(content) + content + b'\nendstream',
    separation(b'PANTONE#20185#20C', [0, 0.9, 0.8, 0]),
    separation(b'PMS185', [0, 0.9, 0.8, 0]),
    separation(b'Gold', [0, 0.2, 0.6, 0.2]),
]

out = io.BytesIO()
out.write(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
offsets = []
for n, body in enumerate(objects, start=1):
    offsets.append(out.tell())
    out.write(b'%d 0 obj\n' % n + body + b'\nendobj\n')
xref = out.tell()
out.write(b'xref\n0 %d\n0000000000 65535 f \n' % (len(objects) + 1))
for off in offsets:
    out.write(b'%010d 00000 n \n' % off)
out.write(b'trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n' % (len(objects) + 1, xref))
io.open(DST, 'wb').write(out.getvalue())
print('wrote', DST, len(out.getvalue()), 'bytes')
