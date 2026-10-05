"""Author `security-notes.pdf`: a §7.6.7 wrapper whose cover page carries an
action on every carrier the engine's action walk visits.

    python fixtures/security-notes.PROVENANCE.py

Deterministic: no timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/security_notes.rs`.

- The catalog's `/AF` names one file specification with `/AFRelationship
  /EncryptedPayload` and `/UF (drawing.pdf)`, so `wrapper::detect` reports a
  wrapper with one named payload.
- The page's `/AA /O` is a JavaScript action: a page trigger and a script.
- The one outline item's `/A` is a `/URI` action: an outline action that
  reaches the network.
- A link annotation's `/A` is a `/GoTo` whose `/Next` is a `/Launch`: an
  annotation action, a chained action and a launch, the launch visible only
  by following the chain.
"""
import io

DST = 'fixtures/security-notes.pdf'

content = b'2 w 36 36 540 720 re S 100 600 200 40 re S'
payload = b'not really encrypted; only the relationship is read'

objects = [
    b'<< /Type /Catalog /Pages 2 0 R /Outlines 7 0 R /AF [9 0 R] >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R '
    b'/Annots [5 0 R] /AA << /O << /S /JavaScript /JS (app.alert\\(1\\);) >> >> >>',
    b'<< /Length %d >>\nstream\n' % len(content) + content + b'\nendstream',
    b'<< /Type /Annot /Subtype /Link /Rect [100 600 300 640] /Border [0 0 0] '
    b'/A << /S /GoTo /D [3 0 R /Fit] /Next << /S /Launch /F (calc.exe) >> >> >>',
    b'<< /Type /EmbeddedFile /Length %d >>\nstream\n' % len(payload) + payload + b'\nendstream',
    b'<< /Type /Outlines /First 8 0 R /Last 8 0 R /Count 1 >>',
    b'<< /Title (Website) /Parent 7 0 R /A << /S /URI /URI (http://example.com/) >> >>',
    b'<< /Type /Filespec /F (drawing.pdf) /UF (drawing.pdf) '
    b'/AFRelationship /EncryptedPayload /EF << /F 6 0 R >> >>',
]

out = io.BytesIO()
out.write(b'%PDF-2.0\n%\xe2\xe3\xcf\xd3\n')
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
