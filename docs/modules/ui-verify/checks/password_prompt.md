# `ui-verify/checks/password_prompt`

`an_encrypted_document_can_be_opened_with_its_password` — the capability the
audit found missing, driven end to end.

# ★★★ The defect

`OPERATOR_REQUESTS.md` O108. Found 2026-09-03 by
`tools/security-coverage.py` — an instrument keyed on `pdfcer-core`'s own API
rather than on any document of ours — while answering the operator's
question *"can we get all of the encryption and signature features that have
been implemented in the engine under one new tab?"*

**An encrypted PDF could not be opened at all.** The shell detected the case
perfectly and had nowhere to type a password:
`Document::load_with_password` and `from_bytes_with_password` were named in
exactly one place in the crate — a doc comment listing the loading entry
points — and nothing called either.

★★ That doc comment is why the coverage tool strips comment-only lines
before it searches. Its first run reported `load_with_password` as
**reached**, on the strength of that one sentence, which would have recorded
the single most important missing capability in the area as already built —
in the instrument written to find exactly this.

# ★★★ Why a driven check and not the four unit tests

Because every link in this chain is a **call site**, and this project was
founded on the observation that a call site's effect is observable only in a
running process. The unit tests prove: an empty box raises no action, a typed
password becomes one action, the action cannot print the password, a
rejection clears the field. All four pass on a build where the prompt is
**never shown**, because nothing in them asks whether
`Status::NeedsPassword` reaches `DialogsState::ask_for_password`.

That link is the one that did not exist for as long as the detection did.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | launch on an encrypted fixture with **no** password | the document does NOT open, and `dialog:password` is declared — the prompt appeared unprompted, driven by document state |
| B | type a **wrong** password and press Open | the prompt is still up, and the trace says `password-rejected … reason=wrong` |
| C | type the right one | `password-accepted`, the prompt retires, and the canvas reports a page |
| D | read back the whole trace | **the password appears nowhere in it** |

★★★ **D is not decoration and it is the phase most worth having.** The
password travels through an `Action` in a queue, this crate traces liberally
to stderr under `PDFCER_DIAG`, and *this harness captures that stderr to a
file it keeps as evidence*. One `format!("{action:?}")` anywhere on the path
would write the operator's password into `target/ui-verify/`, in plain text,
in a directory whose whole purpose is to be kept and read — and it would fail
nothing and look like an ordinary diagnostic in review.

`crate::secret::Secret` makes it unrepresentable and two unit tests assert
the type. **This asserts the whole running program**, over the real captured
file, which is the only place the claim is actually about.
