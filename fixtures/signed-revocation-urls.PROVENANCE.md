# signed-revocation-urls.pdf

`three-boxes.pdf` signed once (PAdES B-B, RSA-2048, SHA-256) by a self-signed
throwaway certificate whose extensions name where its revocation status lives:

- CRL distribution point `http://crl.example.test/ca.crl`
- OCSP responder `http://ocsp.example.test`
- CA issuers `http://ca.example.test/ca.crt`

The `.test` hosts are reserved (RFC 2606) and never resolve. Used to drive the
Signatures panel's revocation-location lines: one certificate, three URLs.

Rebuild (Git Bash; openssl 1.1.1 and the engine's `pdfcer` CLI):

```sh
cat > ext.cnf <<'CNF'
[req]
distinguished_name=dn
prompt=no
x509_extensions=v3
[dn]
CN=Revocation URL Test Signer
[v3]
basicConstraints=CA:FALSE
keyUsage=digitalSignature,nonRepudiation
crlDistributionPoints=URI:http://crl.example.test/ca.crl
authorityInfoAccess=OCSP;URI:http://ocsp.example.test,caIssuers;URI:http://ca.example.test/ca.crt
CNF
MSYS_NO_PATHCONV=1 openssl req -x509 -newkey rsa:2048 -nodes -keyout k.pem -out c.pem \
  -days 36500 -config ext.cnf -set_serial 71
openssl pkcs12 -export -inkey k.pem -in c.pem -out id.p12 -passout pass:test
pdfcer sign --cert id.p12 --password test --signing-time D:20260930000000Z \
  -o fixtures/signed-revocation-urls.pdf fixtures/three-boxes.pdf
```

The key is generated fresh each time, so a rebuild is not byte-identical; the
URLs and the signature's shape are.
