# -*- coding: utf-8 -*-
"""Build `fixtures/signed-revoked-dss.pdf`: `three-boxes.pdf` signed once by a
certificate its issuer has since revoked, with the issuer's CRL carried in the
document's `/DSS /CRLs` (ETSI EN 319 142-1 §5.4.2.2).

Used by `signature_says_its_revocation_verdict`: opening it must report the
signer REVOKED (reason keyCompromise, after the signing time) with no CRL
supplied by hand, because the evidence is inside the file.

Inputs, made with openssl 1.1.1 in Git Bash (MSYS_NO_PATHCONV=1):

    root CA  CN=CRL Test Root, CA:TRUE, keyCertSign+cRLSign, self-signed
    signer   CN=Revoked Test Signer, serial 4660, issued by the root,
             CDP http://crl.example.test/root.crl
    id.p12   signer key + signer cert + root cert, password "test"
    signed   pdfcer sign --cert id.p12 --password test
               --signing-time D:20260930000000Z -o signed.pdf three-boxes.pdf
    root.crl openssl ca -revoke s.pem -crl_reason keyCompromise, then
             -gencrl, converted to DER

The revocation date is the moment `openssl ca -revoke` ran, so it falls after
the fixed signing time. Keys are generated fresh, so a rebuild is not
byte-identical; the verdict is.

This script appends one incremental update to `signed.pdf`: the CRL as a
stream, a `/DSS` dictionary naming it, and the catalog rewritten to point at
the `/DSS`. The signature's byte range is untouched, so it still verifies and
still covers everything but the appended update.

Run: python fixtures/signed-revoked-dss.PROVENANCE.py signed.pdf root.crl fixtures/signed-revoked-dss.pdf
"""
import re
import sys

signed_path, crl_path, out_path = sys.argv[1:4]
pdf = open(signed_path, "rb").read()
crl = open(crl_path, "rb").read()

prev = int(re.findall(rb"startxref\s+(\d+)", pdf)[-1])
size = int(re.findall(rb"/Size (\d+)", pdf)[-1])
root = int(re.findall(rb"/Root (\d+) 0 R", pdf)[-1])
# The newest body of the catalog object is the last one in the file.
catalog = re.findall(rb"\n%d 0 obj\s*(<<.*?>>)\s*endobj" % root, pdf, re.S)[-1]
assert catalog.endswith(b">>") and b"/DSS" not in catalog
crl_obj, dss_obj = size, size + 1
new_catalog = catalog[:-2] + b"/DSS %d 0 R>>" % dss_obj

out = bytearray(pdf)
if not out.endswith(b"\n"):
    out += b"\n"
offsets = {}
for num, body in (
    (crl_obj, b"<</Length %d>>\nstream\n" % len(crl) + crl + b"\nendstream"),
    (dss_obj, b"<</CRLs [%d 0 R]>>" % crl_obj),
    (root, new_catalog),
):
    offsets[num] = len(out)
    out += b"%d 0 obj\n" % num + body + b"\nendobj\n"

xref = len(out)
out += b"xref\n"
for num in sorted(offsets):
    out += b"%d 1\n%010d 00000 n \n" % (num, offsets[num])
out += b"trailer\n<</Size %d/Root %d 0 R/Prev %d>>\nstartxref\n%d\n%%%%EOF\n" % (
    dss_obj + 1,
    root,
    prev,
    xref,
)
open(out_path, "wb").write(out)
print(f"wrote {out_path}: {len(out)} bytes, CRL {len(crl)} bytes as object {crl_obj}")
