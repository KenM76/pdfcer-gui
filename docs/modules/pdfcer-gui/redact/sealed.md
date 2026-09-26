# `redact::sealed` — the call-site monopoly, read from the syntax tree

[`super`] §2.4. One property, asserted over **every `.rs` file in this
crate**:

> Every engine verb that stages, performs or disarms a content removal is
> *called* from exactly one FILE — `redact/mod.rs` — and exactly the number
> of times that module accounts for by name.

★★★ **The subject is a TABLE, not one identifier.** The engine's removal is
more than one surface: `apply_redactions` itself, plus
`apply_redactions_deferred` (stage), `save_applying_redaction` (perform, at
save) and `cancel_pending_redaction` (disarm).

⇒ **A monopoly pinned to one identifier watches the feature walk out of the
module the day the engine splits the verb.** So [`SUBJECTS`] is a table of
(identifier, expected count) rather than a constant, and the argument for
each row is in [`super`] §2.4 beside the function that owns it.

★ And each row is an exact **count**, never a ceiling. A route that is
deleted lowers the number, and an exact count makes that deletion an edit
somebody has to write down rather than a figure that quietly still fits.

★ `cancel_pending_redaction` is pinned even though it removes nothing. It
*disarms* a removal, which is the same surface seen from behind: a second
caller that un-staged a redaction the operator had confirmed would be the
quietest possible way to hand him a file he believes is redacted, and it
would be four characters in a `match` arm.

## Why this exists at all, when the bytes are already private

Because the two mechanisms fail in opposite directions and neither covers
the other.

[`super::PreparedRedaction`]'s private `bytes` field stops anybody
**exfiltrating the proven buffer**. It says nothing about a module that
bypasses the type entirely: `let (bytes, _report) =
pdfcer_core::redact::apply_redactions(&doc, &opts)?; std::fs::write(p,
&bytes)?;` is four lines, needs nothing from this module, and produces
exactly the artefact this module exists to prevent: **a shell that calls
`redact::apply_redactions` directly and writes the bytes ships an
unverified redaction and will not know.**

It is not a hypothetical. `pdfcer`'s `redact-apply` does precisely that
at the engine's HEAD and exits `SUCCESS` on a file it never verified. The
failure has a worked example living in the same repository, written by
people who knew about the proof.

## ★ Why the syntax tree rather than a grep, and rather than a gate script

`crate::shell::commands::reach`'s header makes the general argument at
length; this is the same one aimed at a narrower question, and the specific
false pass is easy to name. **[`super`]'s own module documentation contains
the identifier `apply_redactions` seven times**, in prose, explaining why it
must not be called twice. A text scan counts eight call sites in a crate
that has one — and, worse, the same scan run against a build where the real
call had been *moved* would still find seven and report the monopoly intact.
Comments are not in the tree.

It is a **test** rather than a `tools/gates/` script for
`reach.rs`'s reason: what a gate script contributes over a test is a
precondition guarantee, and this has a stronger one than a script can offer.
[`CRATE_SRC`] is built from `CARGO_MANIFEST_DIR`, a compile-time constant, so
the directory is the one this crate is compiled from and not a path somebody
typed; and the sweep **fails closed** on two independent counts (below).

## Failing closed, twice

`run-all.sh`'s three-state model exists because *"found nothing"* and
*"looked at nothing"* print the same thing. Both are closed here:

1. **A sweep that reads implausibly few files fails**, so a walker that
   silently stopped at the first directory cannot report a clean monopoly
   over the one file it managed to read.
2. **A sweep that finds ZERO call sites fails**, and that is the more
   interesting of the two. Zero would mean the proof pipeline no longer calls
   the engine at all — which is either a rename this check has not been told
   about, or a redaction feature that has quietly stopped redacting. Reading
   zero as "the monopoly holds" is the exact shape of the vacuous pass this
   project has now shipped twice.

## What it does not claim

It is scoped to **this crate**. Another crate in the workspace could call
the engine directly and this would not see it — which is not a gap so much
as a boundary: `tools/ui-verify` drives the binary and `egui-shell` is
forbidden from knowing what a PDF is (`check-shell-purity.sh`), so the only
other Rust in this workspace that could reach `pdfcer-core` is a harness that
does not ship. See [`super`] §2.5.
