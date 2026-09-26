# `ui-verify/checks/reach_out`

`a_document_that_phones_home_says_so` — the driven proof of
`OPERATOR_REQUESTS.md` **O61**.

# What this is about

**Ken, 2026-08-30:** *"I think pdfcer added support for several button
features and protections for outgoing submits."*

Half right. pdfcer cannot yet **author** a button action — still planned, and
a policy decision rather than a missing verb. What it did ship is the
**detection**, and this shell was not asking for it.

`Pass 133.0`, the engine's own words about why that gap was urgent rather
than merely incomplete:

> a push button that submits a form to a web server reported
> `js_network_actions=0` … three surfaces, none disclosing it. **A check
> that under-reports reads as a clean bill of health**, because silence and
> safety are indistinguishable to the reader.

⇒ They fixed their scanner. This shell never called it, so the whole finding
stopped at the crate boundary: pdfcer could tell an operator that the drawing
somebody just sent them will post data to a web server, and nothing on
screen said so.

# Why this needs its own fixture, and why that is the finding


That absence is worth stating rather than routing around: **the one document
shape this disclosure exists for is the one nobody had a copy of.** A check
written against a document with no submit action would have asserted that a
silent program stayed silent — a green result reporting nothing, which is
precisely what this harness exists to remove.

So `tools/gen-submit-fixture.py` builds one: 622 bytes, one push button, one
`/SubmitForm` pointing at `example.invalid` — a host **RFC 2606 §2
guarantees will never resolve**, because a fixture that exists to be opened
by a test must not be able to contact anything even if every guarantee above
it fails.

# The oracle, and the negative half that matters more

| assertion | what it catches |
|---|---|
| the status row carries the disclosure | the shell asked and said so |
| the sentence names **submitting**, not just "actions" | a disclosure too vague to act on |
| it says pdfcer **does not** do it | an alarm about something that cannot happen here |
| a **clean** document says **nothing** | the failure that costs every future disclosure |

The last one is a second launch, on an ordinary drawing, and it is the
assertion this check would be worthless without. A build that warned on
every document would pass every positive assertion above and train the
operator to ignore the status row — after which the sentence that matters is
one they have learned not to read.

## Item notes

### `const NOTE`

`record_note` puts prose on the status bar and traces nothing, so the
shell emits this beside it. Without it a check could prove the scan ran and
could not prove the operator was TOLD — which is the whole subject.
