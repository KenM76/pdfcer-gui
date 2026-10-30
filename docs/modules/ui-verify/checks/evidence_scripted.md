# `ui-verify/checks/evidence_scripted`

`validation_evidence_added_without_the_mouse`: Security ▸ Add
validation evidence… opens an options window whose *Choose files…* embeds the
chosen files in a signed document; clearing *the signers' certificates* leaves
those out; on a document certified with no changes permitted the button does
nothing until *Add it anyway* is ticked; an unsigned one is refused. The window is off the desktop and driven only through
`ScriptedPointer`, so the check runs under `--no-input`.

## Fixtures

- `fixtures/signed-revocation-urls.pdf` — one approval signature by one
  self-signed certificate (see its `PROVENANCE.md`).
- `fixtures/evidence-ca.cer` and `fixtures/evidence-crl.crl` — DER copies of
  the engine's `fixtures/synthetic/crl/ca.cer` and `crl-good.crl`: a throwaway
  test CA and a CRL it issued. Unrelated to the signer, deliberately — the
  command embeds what it is given and does not build a chain.
- `fixtures/certified-no-changes.pdf` — certified at DocMDP P=1 (see its
  `.PROVENANCE.py`).
- `fixtures/four-pages.pdf` — unsigned.

`PDFCER_DIAG_EVIDENCE_FILES` (paths separated by `;`) answers the file picker.

## Steps

1. Open the signed document; click `ribbon.tab.file`, the collapsed Security
   group if the band is narrow, then `ribbon.item.file.add_validation_evidence`,
   then `evidence.commit`.
2. An `evidence-applied` line must carry `certs=2 crls=1 own=1`: the supplied
   CA plus the signer's own certificate, the supplied CRL, and one certificate
   taken from the signature.
3. Again, clearing `evidence.own-certs` first: `certs=1 own=0`.
4. Open `certified-no-changes.pdf` and click `evidence.commit` alone: no
   `evidence-options` line may follow. Again with `evidence.allow` ticked
   first: `evidence-applied` with `crls=1`.
5. Open the unsigned document and click the same item. No `evidence-applied`
   line may appear, and an `evidence-refused` line must carry
   `reason=unsigned`.

## Falsification

- A build that dropped every CRL before calling the engine fails step 2:
  `certs=2 crls=0 own=1`.
- A build passing `own_certs: true` regardless of the checkbox fails step 3.
- A build whose button ignores *Add it anyway* fails step 4's first half.
- A build without the unsigned pre-check fails step 5 with "traced nothing":
  the engine still refuses (`evidence-added-refused … no signature`), but
  under the edit funnel's generic line, not the command's own reason.

## What it does not cover

- a damaged PEM file or a blob the engine cannot parse;
- the saved file's `/DSS` bytes, which the engine's own tests read.
