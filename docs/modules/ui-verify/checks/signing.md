# `ui-verify/checks/signing`

`checks::signing` — **a document is signed, and the signature is read back
out of the file by a different subsystem in a different process**


# THE ONE THING THIS CHECK EXISTS FOR, AND WHY A TRACE LINE IS NOT IT

`pdfcer-gui` has shipped features that traced perfectly and did nothing.
`EditSession::sign` emits `sign-written path=… bytes=… field=Signature1
self_verified=1`, and every word of that line can be true of a build that
wrote a PDF with no signature in it — because the line is written by the
same code that would have written the signature, in the same process, from
the same beliefs.

⇒ So the verdict of this check is **phase D**, which launches a **fresh
process** on the file phase A wrote, opens the **Signatures panel** — the
verification side that shipped as `Pass 10.5`, a different subsystem written
months earlier for a different purpose — and requires:

```text
signature-row field="Signature1" covered=… pairs=1 well_formed=1 integrity=verified …
```

`integrity=verified` is `pdfcer_core::signature_verify` re-parsing the file
from disk, recomputing the digest over the byte ranges, and checking the CMS
against the embedded certificate. A build that wrote a plausible-looking
file would produce `digest-mismatch` or `unverifiable`, and a build that
wrote no signature at all would produce **no row at all**, which this check
also fails on.

# The eight phases, and what each is for

| phase | document | what it proves |
|---|---|---|
| **A** | `four-pages.pdf` | THE NEGATIVE CONTROL — a document that signs. Also: the gate on the confirm control opens only after the certificate is opened, which is the dynamic range the two refusals are measured against |
| **B** | `encrypted-aes-128.pdf` | the **encrypted** refusal, stated instead of a form |
| **C** | `four-pages.pdf` with a redaction armed | the **pending-redaction** refusal, stated instead of a form |
| **D** | phase A's output, fresh process | THE VERDICT — the signature is in the file |
| **E** | `sig-field-empty.pdf` | `Pass 10.13` — a box the SENDER placed is listed, chosen, and the placement controls RETIRE |
| **F** | phase E's output, fresh process | THE SECOND VERDICT — the signature went INTO that box |
| **G** | `four-pages.pdf`, certifying | `Pass 10.12` — the operator can sign as the document's AUTHOR |
| **H** | phase G's output, fresh process | THE THIRD VERDICT — the `/DocMDP` is in the file |

# PHASE H'S ORACLE IS THE DOCUMENT CENSUS, WHICH IS NOT THE SIGNING CODE

There is no signature-panel row that reports a certification, so phase F's
trick — read the name back through the verification side — has no equivalent
here. What does exist is `EditSession::signature_census`, which parses
`/Reference … /TransformMethod /DocMDP` and the catalog's `/Perms` out of the
bytes on disk. It shipped months before the signing verb, for a different
purpose (deciding whether a save would break somebody else's signature), and
this shell reads it **when the Sign window opens**.

⇒ So phase H launches a fresh process on the certified file and presses
`Sign…` again. `sign-opened certification=2` is the census finding a
`/DocMDP` transform where the document had none — asked of a subsystem that
knows nothing about how the file was produced.

# WHY PHASE F IS A SEPARATE VERDICT AND NOT A REPEAT OF PHASE D

Phase D asks *"is there a signature in the file?"* Phase F asks a question
phase D cannot distinguish: **which box did it go into?**

Signing into a pre-placed field and signing beside one produce outcomes that
are identical in every respect this check could otherwise measure — a file
exists, the bytes grew, `self_verified=1`, the Signatures panel shows one
row, `integrity=verified`. A build that quietly ignored `field_name` and
created its own field would pass every assertion in phase D.

⇒ The discriminator is **the field's name**. A signature written into the
author's box carries the author's own `/T` — `SignHere` on this fixture — and
one written into a field pdfcer invented carries `Signature1`, Acrobat's
convention. That name is read back **in a fresh process, by the verification
side**, from `signature-row field=…`, so it is not the signing code's account
of its own behaviour.

And the same phase measures the thing that has no in-process oracle at
all: `sign-written field_reused=1`. That line is written by the same beliefs
as the signing, so it is reported as a **note**, never as the verdict — the
verdict is the name, read by somebody else.

**Phase A is not a formality and it is not there for coverage.** A probe
whose baseline has no dynamic range cannot produce a verdict: without a
document that signs, phases B and C would pass identically on a build where
`file.sign` opened a window that refused *everything*, or on one where the
confirm control was never drawn under any circumstances. Every absence
phases B and C assert is a presence phase A measured, in the same build,
over the same region names, minutes apart.

# THE CERTIFICATE: read from the engine's corpus, never committed here

`D:\Dev\pdfcer\fixtures\synthetic\signing\rsa2048-modern.pfx`, with the
passphrase `pdfcer` that its own `PROVENANCE.md` publishes. That file is
**category (a) wholly synthetic key material, minted by a committed script
with OpenSSL**; its subject says *"(test fixture, trust nothing)"* in its own
`CN`; its validity is ~100 years, so this check acquires no expiry date.

⚠ **No certificate is copied into this repository, and none is generated
here.** A committed `.pfx` is either somebody's real identity, which must
never enter a git history, or a throwaway that expires and starts failing a
suite on a date nobody chose. The engine's corpus is READ-ONLY to this
project, which is exactly the relationship this needs: read it, write
nowhere near it.

Missing corpus is a hard error naming the path, not a SKIP. A SKIP reads
as *"this build does not have the feature"*, and this is a fact about the
checkout.

# ⚠ The passphrase is TYPED, never passed in the environment

`crate::app::files::pick_certificate` has a `PDFCER_DIAG_CERTIFICATE_PATH`
seam and there is deliberately **no** `…_PASSPHRASE` beside it: this harness
captures the child's stderr into an evidence directory it keeps, and
`crate::sign`'s §5 forbids a private key's passphrase reaching any file that
outlives the session. So phase A clicks into the field and types it, the way
an operator does — which is also the only way to prove the field works.
