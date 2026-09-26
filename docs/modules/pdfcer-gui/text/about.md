# `text::about` — the attribution surface, in the operator's own window

Every word the About dialog shows, plus the **structured attribution
catalog** it draws from. Consumed by [`crate::dialogs::about`], and by the
test that pins this catalog against the shipped
`THIRD_PARTY_LICENSES.md`.

## Why an attribution surface exists at all, when `LICENSE` already ships

Permissively-licensed *code* does not need one. MIT and Apache-2.0 notices
are satisfied by the `LICENSE` file in the package and by the licence
metadata in `Cargo.toml`, which a reader of the source tree can check:
nobody is handed a file whose licence obliges pdfcer to *tell them*
something they could not already look up.

Redistributed **assets** are the case that breaks it. The `ocrs` OCR model
weights — the operator's ruling was *"yes ship that model in the mit repo
with proper credit"* — are **CC-BY-SA-4.0**, and that licence's **BY**
clause requires attribution to reach the **recipient of the work**, not
merely a reader of the repository it was built from. A `PROVENANCE.md` in a
source tree discharges nothing for someone who was handed
`pdfcer-gui.exe` in a folder.

The same obligation already runs on three works this program redistributes
today: the bundled substitute faces and the two Adobe data tables below.
They arrive through the engine (`pdfcer-core` and `pdfcer-render` are path
dependencies and Rust links them statically), so they are inside
`pdfcer-gui.exe` whether or not anyone here thought about them — which is
why an attribution catalog cannot be deferred until an OCR build.

## The two surfaces, and why neither replaces the other

| Surface | What it carries | Who reaches it |
|---|---|---|
| `THIRD_PARTY_LICENSES.md`, in the package | every licence **text**, in full — hundreds of kilobytes of it | someone who opens the folder |
| this dialog | the **attribution**: who made it, what it is, under what terms, and whether pdfcer changed it | someone who runs the program |

Neither is a substitute for the other and the split is not arbitrary. A
dialog cannot reasonably render that much licence text, and a `.md` file
in a folder is invisible to an operator who launched pdfcer from a shortcut
and is one `del` away from an operator who tidied up. So the dialog names
the works and the terms and points at the file, and the file carries the
texts. `tools/gates/check-shipped-assets.py` requires **both**: a
redistributed asset directory must be cited in `about.hbs` *and* in this
module, because a change that remembers one and forgets the other is the
commonest way an obligation half-lands.

## What this module is NOT

It is not a second copy of `THIRD_PARTY_LICENSES.md`, and it must not grow
into one. It carries no licence text — [`Attribution`] has no field for
any — precisely so that the two surfaces cannot disagree about what a
licence says. Where a licence's own terms require a **link** rather than a
reproduction (Creative Commons licences do; BSD-style ones do not),
[`Attribution::licence_url`] carries it.

## Conventions

The catalog convention of [`crate::text`] applies unchanged: sentence
case, no trailing period on a label, full sentences with punctuation for
prose. One addition specific to this module — **every field of every
[`Attribution`] is lifted from a source that was read, never reconstructed
from what a licence of that family usually says.** The sources are named
per entry. A wrong attribution is worse than an absent one.

## Item notes

### `fn the_build_stamp_is_populated`

`build.rs` sets `PDFCER_BUILD_TIME` from `PDFCER_BUILD_STAMP` when the
packager supplies one and from a computed UTC clock otherwise. Both
paths must produce something an operator can read; an empty string would
render as "built  from abc1234", which looks like a layout bug rather
than a missing value.

### `fn the_engine_reports_a_version_and_a_revision`

A `Cargo.lock` this build script could not read would leave the version
empty and the About box would say "pdfcer - not in this build" about a
program that is nothing but pdfcer. That reads as a far worse claim than
a missing date, so it is asserted rather than left to be noticed.

### `fn a_component_line_survives_a_missing_commit_date`

The date is optional because a dependency taken from a source this build
cannot run `git` in - crates.io, or an `https://` remote - still has a
version and a revision worth printing. What it must not do is print a
dangling "committed" with nothing after it.

### `fn every_attribution_says_all_five_things`

A half-filled entry is worse than none: it looks like an attribution
has been made while leaving out the part the licence actually asked
for. `licence_url` is excluded because `None` is a real answer for the
licence families here — see its own documentation.

### `fn the_shipped_notice_carries_every_attribution_this_dialog_makes`

This is the property that makes two surfaces safe rather than twice
the maintenance. The dialog names a work and its licence; the file
carries that licence's text. If a work is named in the program and
absent from the file, the operator is told about terms they have no
way to read — which is a worse state than either surface alone.

It fails LOUDLY, which is the point. A generated artefact whose only
check is somebody else running a round-trip rots unnoticed; this
assertion runs in every `cargo test`.

### `fn the_shipped_notice_is_a_real_notice_and_not_a_stub`

Guards against the regeneration having produced a stub — a truncated
or failed `cargo about generate` still writes a file, and a file that
exists is exactly what the assertion above would be satisfied by.

### `fn the_licence_line_matches_the_shipped_licence_file`

Claim-bearing copy: the About box states the terms the operator grants
to everyone who receives this program. It is verified against
`LICENSE` rather than trusted, because the cost of the two disagreeing
is borne by somebody relying on the wrong one.

### `struct Attribution`

# Why these five fields and not others

They are the union of what CC-BY-SA-4.0 §3(a)(1) requires — identification
of the creator, a notice of the licence, a link to it, and an indication
of whether the material was modified — with what a BSD-style notice
requires, which is the copyright line and the licence text. The text
itself is deliberately absent; see the module header.

`origin` is not required by any licence here. It is carried because an
attribution that names a creator but not *which* artefact is unverifiable
by the person reading it, and this project's rule about claim-bearing copy
is that a claim a reader cannot check is a claim nobody should make.

### `fn product`

Lower-case, like the window title and every ribbon caption. The product is
written `pdfcer` everywhere, and a title-cased About box would be the one
place in the program that disagreed.

### `fn version_line`

# Where `version` comes from, and why it is not the crate manifest

From the **git tag**, through `PDFCER_RELEASE_VERSION` — see
`crates/pdfcer-gui/build.rs`'s `release()`, which derives it, and
[`crate::dialogs::about::version_label`], which decides between this
function and the two below.

**`CARGO_PKG_VERSION` is not the release version and must never be
substituted for it.** `Cargo.toml` says `0.1.0` on purpose: the crate is
versioned by the pdfcer workspace it folds into, not by this staging one,
and `OPERATOR_REQUESTS.md` O109 and O110 both record the decision not to
bump it. Reading the manifest here therefore shows an operator a number
that has nothing to do with the build they were handed, and the two are
indistinguishable on screen.

The word in front is the part that is copy. Nothing here is ever
hand-written: a version literal in this file would be a second place to
bump and the first to be forgotten.

### `fn version_line_after`

# Why a development build is not allowed to name a release bare

`Version 0.5.0` on a build twenty-three commits past `v0.5.0` is the same
class of untruth this whole row is about, one step smaller: an operator
comparing their build against the released one would be told they match. So
the distance is stated, and so is the conclusion — *not the released build*
— because a reader should not have to know what "plus 23 commits" implies.

`modified` is the working tree having had uncommitted changes at compile
time. It is reported here as well as in the `-dirty` suffix on the revision
three lines below, and that is not duplication: the revision line answers
*what was this built from*, and this line answers *is this the release*,
and a clean tree sitting exactly on the tag is the only state where the
answer to the second is yes.

Reads *"Version 0.5.0, plus 23 commits — not the released build"*.

### `fn version_unreleased`

# Why this says something rather than showing nothing

The no-placeholders rule (R9) forbids rendering a stub, and it would be
satisfied by drawing no line at all. This says a sentence instead, for the
reason [`component_absent`] already records for `iccce`: **that rule governs
controls, and this is a provenance report.** An operator asking what they
are running is owed *"this is not a released build"*, which is a real answer
and a more useful one than a gap where a version used to be — and a gap is
also indistinguishable from a layout fault.

It contains **no number**, and that is load-bearing rather than incidental.
The states that reach it — a tarball with no `.git`, a machine with no
`git`, a clone with no tags — are exactly the states in which any number
shown would be invented. `dialogs::about::tests::the_unavailable_case_invents_no_number`
asserts the absence of digits directly, so the day someone "improves" this
by falling back to the crate manifest, a test says why not.

It points at the Build block rather than ending on the bad news, because
that block *does* identify this executable — a timestamp and a commit — and
an operator who came here to tell two builds apart can still do it.

### `fn build_heading`

The operator asked for this: *"when I go to about pdfcer in pdfcer-gui, can
you include the date and time of the build. Also the date and time of the
builds of the used pdfcer and iccce"*.

### `fn build_line`

`stamp` and `rev` come from `build.rs` through `env!`, so they cannot drift
from what was actually compiled. `rev` carries a `-dirty` suffix when the
tree had uncommitted changes, which is the fact an operator most needs when
a build does something a commit does not explain.

### `fn component_line`

The wording distinguishes **committed** from **built**, and the
distinction is the whole reason this is not simply three build times.
`pdfcer-core` and its siblings have no build of their own — they were
compiled by the same `cargo build` that produced everything else here, so
their "build time" is this binary's build time restated, which answers
nothing. What identifies the engine in a given executable is the revision
and when that revision was committed.

Reads *"pdfcer 0.7.0 — revision 6af5655, committed 2026-08-18 14:02"*.

### `fn component_absent`

Reported rather than omitted, and the judgement is worth writing down
because it looks like a breach of the no-placeholders rule and is not. That
rule governs **controls**: an unavailable capability must offer no button,
because a button that does nothing is a lie about what the program can do.
This is a provenance report. An operator asking what is inside their
executable is owed *"no colour management in this one"* — which is a
different and more useful answer than silence, and is what tells them why
a colour-managed file looks the way it does.

It fills itself in the day the dependency is added: `build.rs` reads the
workspace `Cargo.lock`, and a transitive dependency appears there without
anyone editing anything.

### `fn summary`

Present because an About box that gives a version and no identity tells an
operator with two pdfcer builds installed nothing they did not know. Names
the engine explicitly: the single most common question about this binary
is whether it contains pdfcer or merely talks to it.

### `fn licence_line`

Lifted verbatim from the repository's `LICENSE` file — the SPDX name on
its first line and the copyright line on its third. Not reconstructed, and
not softened: this is claim-bearing copy about the terms the operator
grants, and the source of truth is the file that ships beside the binary.

### `fn full_texts_note`

Names the file **and** where it is, because "see the licence file" is
advice an operator cannot act on. `THIRD_PARTY_LICENSES.md` is copied into
the portable folder by `tools/package-portable.py`; the gate asserts that
it is, so this sentence cannot become false without something failing.

### `fn attributions`

# What is deliberately absent

- **Rust crates.** `cargo-about` harvests all of them from `Cargo.lock`
  into `THIRD_PARTY_LICENSES.md` mechanically. Restating a subset of them
  here by hand would create a second list that drifts, and the drift would
  be invisible — the crate list runs to well over a hundred entries and
  nobody re-reads it.
- **The pdfcer icon set** (`crates/pdfcer-gui/src/icons/assets/`). It is the
  operator's own art under the project's own MIT licence, confirmed by him;
  the `LICENSE` file already covers it and there is no third-party grant to
  reproduce. Listing it under a heading that says "third-party" would make
  this dialog say something untrue. See that directory's `PROVENANCE.md`.

**An entry may not be added before the files are actually in the package.**
An attribution for something that is not shipped is a false statement in
the other direction, and it is the failure mode this list is most exposed
to, because the notice is easy to write and the packaging is not. The
`ocrs` weights satisfy it two ways at once:
`tools/package-portable.py`'s `PAYLOAD_ASSET_DIRS` copies them to
`models/ocrs/` beside the executable, and
`tools/gates/check-shipped-assets.py` declares them `how="copied"`, which
makes the two facts fail together rather than separately.

**They are also the one entry here that is not compiled into the binary.**
The other three are `include_bytes!` payloads and static tables; these are
loose files the program opens at run time. That difference does not change
what CC-BY-SA-4.0's BY clause asks — the weights are redistributed either
way, and the recipient is the same person — which is exactly why this
dialog says *what* each work is rather than *how it got here*.

# The engineering constraint that travels with those weights

Written here rather than only in the engine's provenance note, because
this is the file someone will have open when the idea occurs to them.

**CC-BY-SA's share-alike clause binds adaptations, not collections.**
Shipping the weights **unmodified** alongside MIT code is distribution of
a verbatim work in a collection, and pdfcer's own licence is unaffected —
that is the reading the operator was shown and accepted. **Modifying them
creates Adapted Material, and the adapted weights must then be released
under CC-BY-SA-4.0 or a compatible licence.** That includes fine-tuning
them for CAD drawings, **quantizing them to shrink the 12,240,008-byte
download**, retraining them on any corpus, and converting them into
another runtime's format.

It would bind the derived model, not pdfcer's source. But it means
*"we'll just quantize it"* is a decision with a licence attached and needs
its own operator decision at the time — which is precisely the thought a
future reader will have while looking at a 12 MB file in a portable folder
and wondering what it costs to halve it.
