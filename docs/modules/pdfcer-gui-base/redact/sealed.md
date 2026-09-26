# `redact::sealed` — the call-site monopoly, read from the syntax tree

[`super`] §2.4. One property, asserted over **every `.rs` file in this
crate**:

> Every engine verb that stages, performs or disarms a content removal is
> *called* from exactly one FILE — `redact/mod.rs` — and exactly the number
> of times that module accounts for by name.

**The subject is a TABLE, not one identifier.** The engine's removal is
more than one surface: `apply_redactions` itself, plus
`apply_redactions_deferred` (stage), `save_applying_redaction` (perform, at
save) and `cancel_pending_redaction` (disarm).

⇒ **A monopoly pinned to one identifier watches the feature walk out of the
module the day the engine splits the verb.** So [`SUBJECTS`] is a table of
(identifier, expected count) rather than a constant, and the argument for
each row is in [`super`] §2.4 beside the function that owns it.

And each row is an exact **count**, never a ceiling. A route that is
deleted lowers the number, and an exact count makes that deletion an edit
somebody has to write down rather than a figure that quietly still fits.

`cancel_pending_redaction` is pinned even though it removes nothing. It
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

## Why the syntax tree rather than a grep, and rather than a gate script

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

## Item notes

### `fn crate_src`

`CARGO_MANIFEST_DIR` rather than a relative path from the working directory:
`cargo test` and a test run from an IDE do not agree about the latter, and a
path that resolved to nothing would be the "looked at nothing" failure this
module's header is about.

### `const OWNER`

Compared as a path suffix rather than as a string, so the separator is the
platform's rather than this constant's — the same file is `redact\mod.rs` on
the machine this is built on and `redact/mod.rs` elsewhere.

### `struct Counter`

`syn::visit::Visit` rather than a hand-written recursion: "anywhere" has
to include a closure body, a nested `fn`, a `match` arm, an `if let`
scrutinee and every other place an expression can hide, and the variant
a hand-written walker forgot would be a silent hole rather than a
compile error.

### `const MIN_FILES_SWEPT`

A floor rather than an exact count, deliberately: an exact number is a
figure in prose, and figures in prose drift, while a floor only ever
fails for the reason it exists — a walker that stopped early. The crate
holds around 150 source files.

### `const FIXTURE_SUBJECT`

[`SUBJECTS`]'s first row rather than a fourth string literal, so the
fixtures cannot drift away from a name the real check uses. Which of
the four it is does not matter — the self-tests prove the *reader* and
the *walker*, not the table — and taking it from the table means a
rename of that row updates them for free.

### `fn every_removal_verb_is_called_from_exactly_one_place`

The assertion this module exists to make. A failure here means one of
three things, and the message says which: a second path to the engine's
removal has appeared — an unverified redaction that will not know it is
one; a legitimate call has moved out of the
proving file; or a route has been added or deleted and this table has
not been told.

It sweeps once per subject rather than once, and pays four directory
walks for it. That is deliberate: a single sweep counting four
identifiers together would report *"seven calls in one file"* and be
satisfied by three of one and none of another, which is precisely the
arithmetic that lets a deletion hide behind an addition.

### `fn the_apply_pipeline_never_reaches_for_the_incremental_writer`

[`super::super`] §1.1's *"there is no parameter anywhere that could make
an apply write incrementally"*, restated as a property of the directory
rather than of a reader's care.

The hazard is specific to this shell and did not exist in the salvage
source's world: `crate::app::save` is built on `to_incremental_bytes`
and `file.save_copy`'s shipped tooltip promises it, so the verb is
idiomatic here, well documented, and one autocompletion away from the
one directory where it would leave the un-redacted content in a prior
revision of a file the operator has been told is redacted.

# The exception

`redact/tests/` **does** call the forbidden verb, deliberately and
repeatedly, and it must. It performs exactly the save the ban forbids
and asserts it is **refused by name** (`WriteError::RedactionPending`),
because the un-redacted content is still live in the staged session, so
the guarantee is a refusal rather than a property of the bytes.

The shape survives a change of engine contract. Were a removal to
collapse into the session instead of staying pending, the same call
would be made and the assertion would become *the removed text is not in
the result* — the measurement is a save that is actually made and looked
at, whichever guarantee the engine offers.

⇒ Either way, *"the guarantee is the engine's; the measurement is ours"*,
and a ban that also forbade the measurement would leave the whole
deferred route resting on a doc comment. So the ban is scoped to the
**production** files of the directory, and the exception is pinned
rather than merely allowed:

1. no production file under `redact/` calls it — unchanged, and it is
   the assertion that was always the point;
2. **the `tests/` suite calls it at least twice**, because if the measurement is
   ever deleted this test starts passing for the wrong reason and the
   only evidence for the deferred route's safety goes with it.

Without (2) the narrowing would be a hole. With it, the file is either
proving the property or failing.

### `fn fixture`

**Built from [`FIXTURE_SUBJECT`] rather than spelling the verb out.**
It was a `const` with the name written in, and the day [`SUBJECTS`]'
first row named a different verb every assertion below went looking for
a call the fixture no longer contained. A falsification that plants
nothing reports *nothing found*, which is indistinguishable from a
passing check; these four survived only because they assert an exact
count rather than an absence, which is luck rather than design.
Deriving the text removes the possibility.

### `fn the_reader_finds_a_real_call`

Without this, assertion B below could pass by finding nothing at all,
which is the failure mode the module header's "fail closed" section is
about, arriving inside the self-test instead.

### `fn the_reader_counts_only_calls`

The four false positives a grep produces, and the reason the count in A
is `1` rather than `5`. The `use` line matters most: every module that
imports the function without calling it would otherwise be reported as a
breach, and a check that cries wolf gets its allow-list widened until it
says nothing.

### `fn a_planted_second_call_is_reported`

The real defect, in the two shapes it would actually take: a plain call
in a function, and one inside a closure — which is where a hand-written
AST walk would most plausibly have stopped, and which is why this uses
`syn::visit` rather than a bespoke recursion.

### `fn a_missing_tree_is_an_error`

The "looked at nothing" state, closed at the level of the tree. In the
real check it cannot arise — [`crate_src`] is built from
`CARGO_MANIFEST_DIR` — and it is asserted anyway, because the reason it
cannot arise is a property of one line that a refactor could change.

### `fn the_walker_descends_and_reports_a_planted_file`

A and C prove the *reader* bites; this proves the *walker* does. A
fixture tree is built under the OS temporary directory with the
violation two levels down, because a walker that only read its top
directory would pass every test above and report the real crate clean —
the crate's own offender would have to be in `src/` itself to be seen.

### `fn calls_in`

Both a free call (`path::to::subject(..)`) and a method call
(`receiver.subject(..)`) count. The engine's is a free function, so only the
first can occur today; the second is counted because a future engine that
moved it onto a type would otherwise slip the monopoly silently, and because
counting one shape and not the other is the kind of narrowness that makes a
check answer a question nobody asked.

# Errors

The source did not parse as Rust. **Fails closed**: an unreadable file
stops the sweep rather than contributing a reassuring zero.

### `fn sweep`

`root` is a parameter rather than a reach for [`crate_src`] for
[`calls_in`]'s reason, one level up: the self-test below points it at a
temporary tree containing a planted violation, and a sweep that could only
be aimed at the real crate could not be shown to report one.

# Errors

The directory could not be read, or a file in it did not parse. Both fail
closed.
