# `ui-verify/checks/trust_store`

`signature_trust_is_reported_as_its_own_fact` — **the Signatures panel
reports three facts and never merges them, and `not checked` says so.**


Said here, in the module's own words, rather than left for an absent result
to imply. The operator may be at his keyboard, and this harness drives a
real cursor and a real keyboard over the whole desktop — a run while he is
working is a run whose verdict is noise that looks like a finding.

`export_text` and `copy_as_vector` carry the same disclosure for the same
reason. **An unrun check is not a passing check**, and this project's
standing rule is that no UI change is done until it has been verified by
driving the running binary. This one has not been.

# What it detects, and why nothing else can

`pdfcer-core`'s `SignatureVerdict` carries **three facts that never collapse
into one bool**: whether the signed bytes are intact, what the signature
covers, and whether the signer chains to a trusted anchor. The design exists
because a reader that says *"valid ✓"* about a chain it did not validate is
the one failure worse than saying nothing.

`crate::panels::signatures`' unit tests pin the *sentences* and the
*mapping* — that four unchecked states produce four explanations, that a
`Trusted` line discloses what it did not check. What no unit test can reach
is the six links between a signed file on disk and three lines on screen:

1. the panel is reachable at all, from the View tab, in a real layout;
2. its dock tab is selected, so its body actually executes — a docked pane
   that is not in front publishes nothing, which is indistinguishable from a
   panel with nothing to say (`RESUME.md` records that exact misdiagnosis);
3. `crate::trust::cached_report` reads the file **from disk** and gets a
   verdict list back, rather than failing silently and leaving the panel
   showing coverage alone;
4. the row loop pairs coverage `i` with verdict `i`;
5. every row publishes an `integrity=` token AND a `trust=` token — the two
   facts that did not exist before this work;
6. with the setting at its shipped default, `trust=not-checked` is what
   comes out, rather than nothing.

Link 6 is the assertion the whole feature stands on. A build that dropped
the trust line entirely when there were no anchors would pass every other
assertion here and would be the exact defect this feature was written to
prevent: on screen, "we did not look" and "we looked and it was fine" would
be the same picture.

# ★★ The oracle is the TRACE TOKEN, not the sentence

`signature-row … integrity=<token> trust=<token>` carries diagnostic words —
`verified`, `not-checked`, `untrusted` — that are deliberately **not** in
the copy catalog. A check that matched operator prose would go red the day
somebody improved a sentence, which trains people to re-baseline checks; and
a check re-baselined is a check that has stopped measuring.

The **rectangle** is asserted separately, through `panel:signatures`,
because a trace line proves the code ran and says nothing about whether an
operator can see it. This project has recorded that distinction more than
once.

# ★★★ What this deliberately does NOT assert

**That anything is ever `trusted`.** It cannot, and pretending otherwise
would be the worst check in the suite. A `Trusted` verdict needs a signature
whose signer chains to a real AATL or EUTL anchor, which needs (a) somebody's
real certificate committed to this repository and (b) the operator's own
Acrobat trust store present on whatever machine runs the suite. The first is
a certificate that expires; the second makes the verdict a report about the
machine rather than about the code.

So the positive trust path is **unverified by this harness**, and saying so
is the whole of the honesty available here. What covers it instead:

* `pdfcer-core`'s own `trust_chain` tests, against synthetic chains it
  builds and signs itself;
* `pdfcer trust-store-list` at the command line, which the engine ran
  against this operator's real 1,780-anchor store;
* the Settings group's *Show what is in it* button, which is the operator's
  own route to the same reading and reports the count and the store's date.

⇒ **If one thing in this feature is to be driven next, it is the Settings
group on the operator's own machine**, where a store really exists: turn the
setting on, press *Show what is in it*, and read the counts back. That needs
his machine and his file, which is why it is named here rather than written
as a check that would SKIP everywhere else.

# The fixture

`fixtures/signed-two-pages.pdf`, **pinned**, and `--pdf` is ignored. The
check's subject is *"what does the panel say about a signature"*, and on a
document with no signature fields the panel correctly draws one sentence and
publishes no rows — so an arbitrary fixture would make this unable to fail.
It says so in its notes when a `--pdf` was supplied and thrown away, because
a run that silently ignored a flag is indistinguishable from one that
honoured it.
