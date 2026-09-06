#!/usr/bin/env python3
"""check-engine-api-drift.py — EVERY PUBLIC ITEM THE ENGINE GAINS OWES AN ACT.

===========================================================================
★★★ WHY THIS GATE EXISTS, and the day two gates watched and both were blind
===========================================================================

On 2026-09-04 `pdfcer-core` shipped `pdfcer_core::text_edit::RefusalKind` — a
coarse, stable, exhaustively-matchable discriminant over `EditError` — **in
direct answer to a request this project filed**. It arrived inside the engine
revision this workspace then pinned, so it was callable from here the moment
`cargo update` ran.

It sat unconsumed for a day.

A function in this repository, `crate::text::status::edit_declined_by_engine`,
said **in its own doc comment** that it was "written to be deleted" the day
`EditError` gained a coarse kind. That day arrived. Nothing noticed. Not one
of the two gates aimed at exactly this question made a sound:

  * `tools/gates/check-verb-coverage.sh` reads `impl EditSession`'s `pub fn`s.
    `RefusalKind` is an **enum**, in a different module. Invisible.
  * `tools/gates/check-engine-backlog.sh` reads the engine's `docs/FEATURES.md`
    prose table. A discriminant type is not a *feature*; no row was written.
    Invisible.

⇒ ★★★ **BOTH GATES WERE KEYED ON `EditSession`'S VERBS.** A new public *type*,
  a new *variant*, a new *field* on an existing type, a new *free function* —
  the entire rest of the engine's API — could land, be pinned, and be shipped
  around without a single instrument on this side of the boundary saying a
  word. Two gates, one key, one blind spot.

This is the SIXTH recording of the same shape in this project:

  * `EDITABLE_SURFACES.md` §"The sweep found…" — three of the first four gaps
    were capabilities the engine shipped BECAUSE this shell asked, and then
    never consumed. *"A reply arriving is not a capability landing."*
  * `check-string-gaps.sh` — a catalogued string that reaches no rectangle.
  * `check-verb-coverage.sh` — a verb the engine implements that nothing names.
  * `check-engine-backlog.sh` — a capability the engine announced in prose.
  * O119's own row — *"it is the fourth time: this landed with no note and no
    announcement, and the only thing that made a noise was that gate."*
  * This.

★★ Each previous fix widened the key by one notch and left the next notch
uncovered. This one does not pick a notch: it enumerates **every `pub` item in
every source file of every engine crate this shell depends on**, and diffs.

---------------------------------------------------------------------------
★★★ AND ON ITS FIRST REAL RUN IT CAUGHT ITS OWN SIBLING BEING FOOLED
---------------------------------------------------------------------------

The engine shipped digital SIGNING the same day this file was written — a
capability this shell asked for by name — as `EditSession::sign` plus a
101-item `pdfcer_core::sign` module.

`check-verb-coverage.sh` scored `sign` as **consumed**. It greps
`crates/pdfcer-gui/src` for `\bsign\b`, and the word occurs 42 times in this
shell — the first two of them in `app/actions/bookmarks.rs`, in a documentation
table about the **arithmetic sign of `/Count`**:

    //! ## `/Count` is two different quantities, and the sign carries open/closed
    //! | sign | **cannot** be negative | **positive = open, negative = closed** |

⇒ A capability the operator asked for, shipped that morning, was reported as
  already wired **because a doc comment discusses positive and negative
  numbers.** That gate's own header says a hit is weak and a miss is strong;
  here is the weak hit costing a whole subsystem.

★★ It is why this gate tests a MODULE by its full qualified path and a VARIANT
by its `Owner::Variant` spelling, rather than by a bare identifier. A
three-letter engine name is not a search term.

===========================================================================
WHAT IT ASSERTS
===========================================================================

    For every public item the engine has gained since the snapshot, this
    repository must either NAME it in its own Rust sources, or SAY something
    about it in a root-level markdown register, or carry an `exempt` line for
    it in the snapshot with a written reason.

    — with ONE change of grain: when the item's MODULE is itself new, the unit
    of the finding is the shallowest new module, and one sentence naming that
    module discharges everything inside it. See "THE GRAIN OF A FINDING" below;
    it is the difference between one actionable line and ninety-eight.

That is the whole rule, and — exactly like its two blind siblings — it is
deliberately weak in one direction and strong in the other:

  * **Weak**: it does not judge the reason. A register sentence saying "not
    built, no plans" passes. A gate cannot read English and must not pretend
    to.
  * **Strong**: an item that appears in the engine and about which this
    repository is completely SILENT fails the build, on the first `cargo
    update` that pins it. Somebody has to look at it and write a sentence —
    which is the entire mechanism, and is exactly what did not happen when
    `RefusalKind` landed.

★★ The failure is therefore not "you have a gap". It is **"the engine grew
something and nobody here has said anything about it"**, which is a different
and much more actionable statement.

===========================================================================
★★★ WHICH REVISION IT MEASURES, AND WHY THE ANSWER IS "THE LOCK"
===========================================================================

`verb-coverage.py` measures the revision `Cargo.lock` pins, because its
question is *"could this shell CALL this?"* and the lock is the only honest
answer. `check-engine-backlog.sh` measures the engine's WORKING TREE, because
its question is *"has the engine SAID it has something?"* and a statement in a
document is made when it is written.

This gate's question is the first one, so it measures **the lock**, and it
fails only on the lock. An item in the engine's working tree that the lock does
not carry **cannot be called from here at all**; failing on it would redden
this build for a change nobody in this repository made and cannot undo without
moving the pin — which is an operator decision.

★★ But it is NOT silent about that half either. Items the engine's `HEAD` has
and the lock does not are printed under **COMING**, prominently, with a count
and a sample, exactly as `verb-coverage.py` does for verbs. Two facts, kept
separate and both said out loud:

    "nothing here names it"   and   "we could not name it if we wanted to."

===========================================================================
★★★ WHAT THE SNAPSHOT IS, AND — MORE IMPORTANTLY — WHAT IT IS NOT
===========================================================================

`tools/gates/engine-api-snapshot.txt` is a committed list of every public item
the engine carried at a stated baseline revision.

**It is a record of what has been LOOKED AT. It is not a claim that any of it
was reviewed, wanted, or consumed.** The baseline is the previous engine pin —
the last revision this project reconciled against with the gates it had. Every
item that existed then is in the snapshot; everything the engine has grown
since is the finding. That is a truthful and narrow claim, and it is written
into the snapshot's own header so nobody later reads it as an approval.

★★★ **THE RE-BASELINE TRAP, AND THE ONE DESIGN DECISION THAT CLOSES IT.**

The obvious way to silence a snapshot gate is to regenerate the snapshot. If
`--update` simply wrote the live API to disk, then the fix for a red gate would
be one command, the finding would vanish unread, and this instrument would join
the long list of things that measure nothing. `check-engine-backlog.sh`'s
header names the same hazard: *"a gate people re-baseline is a gate that has
stopped measuring."*

So **`--update` refuses to absorb an unaccounted item.** It folds in exactly
the new items that already pass the accounting rule above, prunes the ones the
engine deleted, and leaves every unaccounted item OUT — so the gate goes red
again on the very next run. There is no command that makes a finding disappear.
The only routes out are: consume it, write about it, or exempt it with a
reason.

===========================================================================
★★★ THE HAND-WRITTEN-LIST TRAP: WHAT IS DERIVED, AND WHY EACH ONE
===========================================================================

`RESUME.md` records, three times over, this project's most expensive recurring
defect: *"a hand-written list inside a completeness test is the gap."* A
completeness check that carries a typed list of things to check is blind to
exactly the new thing it was built to find. Every list this gate needs is
therefore derived:

  * **Which engine crates.** Read out of `crates/pdfcer-gui/Cargo.toml` — every
    dependency taking a `git = "file:///…"` URL. Not typed. The day this shell
    adds `pdfcer-fetch` as a fourth engine crate, this gate covers it without
    anybody remembering that this file exists. **Zero crates is a FAILURE**,
    never an empty clean scan.
  * **Which source files.** Every `*.rs` under each crate's `src/`, walked from
    the git tree object. Not a module-resolution walk from `lib.rs` — see the
    deliberate over-reporting note below.
  * **Where the engine is.** `tools/engine_path.locate()`, which reads the git
    URL out of the manifest Cargo itself builds from. **Never a literal.** On
    2026-09-03 a hard-coded `D:/Dev/pdfce` survived this project's rename,
    pointed at a directory that did not exist, and made
    `check-verb-coverage.sh` print `PASS: all 0 uncalled verb(s)` having
    examined nothing at all. That is the defect `engine_path.py` was written
    for, and this gate uses it rather than repeating it.
  * **Which registers count as "somebody wrote about it".** Every `*.md` at the
    repository root. Not a typed list of three filenames — a new register file
    is exactly the kind of thing a typed list goes blind to.

===========================================================================
★★★ WHAT IT ACTUALLY READS — the proxy question, asked of this gate
===========================================================================

*"A proxy condition survives one correction."* `RESUME.md` records the shim
tripwire that proudly caught itself testing `-d D:/Dev/pdfcer`, was fixed to
test the crate on disk, and was **still** a proxy. So, plainly:

  * It reads **the bytes of the engine's `.rs` files at a git revision**, via
    `git archive`. Not the working tree (which the engine session edits
    continuously), not `cargo metadata`, not rustdoc JSON, not a changelog.
  * It reads them with a **line-oriented, indentation-driven scanner**, not a
    Rust parser. `rustfmt` is what makes that sound: every engine crate is
    formatted, so an item at column 0 is a module-level item and an item at
    column 4 under an `impl` is a method. A hand-formatted file would be
    mis-read — and the direction of the error is toward reporting extra items
    (loud) rather than dropping them (silent).
  * It decides "does this repository name it" by a **word-boundary regex over
    this workspace's own `.rs` files and root `.md` files.** That is a grep.
    ★ A MISS is strong evidence: the identifier occurs nowhere, full stop. A
    HIT is weak: the name appears, which is not the same as a live call. That
    asymmetry is fine, because **this gate only fails on misses.**

===========================================================================
★★ DELIBERATE OVER-REPORTING: private modules are scanned too
===========================================================================

The scan walks every `.rs` file under `src/`, rather than following `pub mod`
declarations from `lib.rs`. That means a `pub` item inside a *private* module
— not reachable API — is recorded, and can be reported.

That is a decision, not an oversight, and it was measured. `RefusalKind` lives
in `crates/pdfcer-core/src/text_edit/refusal_kind.rs`. Had that module been
declared `mod refusal_kind;` with a `pub use refusal_kind::RefusalKind;` beside
it — a spelling used all over this engine — a module-visibility walk would have
skipped the file and **missed the exact item this gate was built for.** The
first draft of this scanner did precisely that, on a different module, and the
symptom was silence.

⇒ A handful of extra items from private modules is noise, and noise is
exemptible in one line. A missed item is invisible, and invisible is the
failure mode this whole file exists to remove. **Err toward reporting.**

===========================================================================
★★★ THE GRAIN OF A FINDING — measured on the first real run, and it changed
    the design twice
===========================================================================

**1. A whole new module is ONE finding.** The engine's digital-signing
subsystem — `pdfcer_core::sign`, with `apply`, `cms_build` and `pkcs12` under
it — landed in a single pin move carrying **101 public items**. Reported one by
one that is 101 lines about one event, and discharging it would mean typing 101
backticked symbol names into a register, which nobody will do and nobody
should. The thing a person must look at is the subsystem, once. So a module the
snapshot has never seen is reported as a module, with its item count and a
sample, and one register sentence naming its full path accounts for all of it.

★★ **And the rule stops at exactly that line.** It applies only to a module the
snapshot has NEVER SEEN. A new item in an EXISTING module is still reported as
an item — because that is the `RefusalKind` case, and `pdfcer_core::text_edit`
is named in half this project's documents. A module-level discharge that
reached old modules would have swallowed the very item this gate exists for.
*A new subsystem is one act of attention; a new item in a familiar one is
another.* The self-test asserts both directions, because either alone is
useless.

**2. A module path is NOT tested as a substring, and the first cut was.**
`pdfcer_core::sign` is a prefix of `pdfcer_core::signature` and
`pdfcer_core::signature_verify`, both of which this shell has consumed and
written about at length. A plain substring test found the prefix inside the
longer names and **printed the entire 101-item signing subsystem in the
accounted list, in green, as something somebody had looked at.** Nobody had.
The path must be followed by a character that cannot continue a Rust
identifier. One lookahead; without it this gate's largest finding was invisible.

**3. A VARIANT must be spelled `Owner::Variant`.** Also measured on the first
run: `pdfcer_core::edit::EncryptError::RedactionPending` arrived, `EncryptError`
appears in this shell and `RedactionPending` appears in this shell — as
`WriteError::RedactionPending`, a *different enum's* variant of the same name —
so an owner-and-leaf test found both names and called it consumed. It was not:
`protect/mod.rs` has no arm for it. The rule is applied to variants only,
because a method is called `x.method()` and a field read `x.field`; demanding
`Type::member` for those would report every correctly-consumed one. **The rule
follows the call syntax, not a taste.**

===========================================================================
★★ WHAT THIS GATE STILL CANNOT SEE — stated, because an unstated limit reads
   as coverage
===========================================================================

  * **A HIT IS WEAK.** "Named somewhere in the tree" is not "reached by a live
    operator route". A call site behind a condition nothing sets is a hit here
    and dead in the running program. `tools/ui-verify` is the only instrument
    for that question and this one cannot answer it. Only the MISS is strong.
  * **A CHANGED SIGNATURE is invisible.** The key is a name and a kind, so a
    `pub fn` that keeps its name and gains a parameter, changes its return
    type, or reverses the meaning of a bool reads as unchanged. The compiler
    catches that for anything this shell CALLS; nothing catches it for anything
    it does not.
  * **A CHANGED DOC-COMMENT or default is invisible**, and this project has
    been bitten by one before: the engine moved a default and a label here had
    it written down rather than asking (`RESUME.md`, 2026-08).
  * **`#[cfg]`-GATED items are counted once**, whatever features are enabled.
    An item behind a feature this shell does not take is reported as available.
  * **RE-EXPORTS are not followed.** The key is the DEFINITION site, so a
    `pub use` that changes an item's public path is not a finding, and a file
    move shows as one removal plus one addition (paired in the output as
    "probably MOVED" so a reader can tell a rename from a capability).
  * **The scanner assumes `rustfmt`.** It is indentation-driven. A
    hand-formatted engine file would be mis-read — toward reporting extra
    items, which is loud, rather than dropping them, which is silent.
  * **The COMING half never fails the build.** An item in the engine's HEAD
    that the lock does not carry is printed and not enforced, because pulling
    it is an operator decision.

===========================================================================
FAILURE MODES, EXACTLY
===========================================================================

FAIL (exit 1) when:
  1. the engine's locked revision carries a public item that is not in the
     snapshot, is named nowhere in this repository, and has no `exempt` line;
  2. an `exempt` line carries no reason, or a reason under 20 characters — an
     exemption without an argument is a re-baseline wearing a disguise;
  3. **the scan collapses**: the live enumeration returns fewer than half the
     items the snapshot holds, or zero items for a crate. A resolvable engine
     that yields nothing means the scanner went blind, and an empty scan MUST
     NOT read as a clean one. This is a FAIL and never a SKIP.
  4. no engine crate could be derived from the manifest — see (3)'s argument.
  5. `--self-test` did not detect its own plants.

SKIP (exit 2) when — and ONLY when:
  a. the engine checkout named by the manifest is not a directory on disk
     (a clean checkout on a machine that has no engine);
  b. `git` is not on PATH, or `git archive` refuses the locked revision (the
     object is genuinely absent from that clone);
  c. `Cargo.lock` pins no `git+file://` revision;
  d. the snapshot file is missing.
  A SKIP makes `run-all.sh` exit 3 — INCOMPLETE, not green.

★★ **THE SKIP SET IS ITSELF A HAZARD.** A skip is not red, so a gate can stop
running and nobody notices. The five conditions above are the *complete* list;
every one of them prints the word SKIP and the actual fact. If this gate ever
appears in `run-all.sh`'s SKIPPED block on a machine that has the engine, that
is a defect in this file, not an expected state.

PASS (exit 0) when every new item is accounted for. The summary line always
states how many items were measured and at which revision, so "it passed"
is never separable from "it measured something".

===========================================================================
USAGE
===========================================================================
    tools/gates/check-engine-api-drift.sh              measure
    tools/gates/check-engine-api-drift.sh --self-test  prove it can fail
    tools/gates/check-engine-api-drift.sh --update     fold accounted items in
    tools/gates/check-engine-api-drift.sh --list       print the live API
"""

from __future__ import annotations

import argparse
import io
import os
import pathlib
import re
import subprocess
import sys
import tarfile
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
import engine_path  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent

#: The committed baseline, overridable by `PDFCER_ENGINE_API_SNAPSHOT`.
#:
#: ★★ The override exists so that FALSIFYING this gate never touches the
#: committed file. `check-engine-backlog.sh` carries the same escape hatch
#: (`PDFCER_ENGINE_BACKLOG`) for the same reason, and the reason is specific to
#: how this repository is worked: several tracks edit this tree at once, and
#: `RESUME.md`'s standing rule is that a `git checkout` to undo an experiment
#: discards somebody else's uncommitted work. A gate that can only be proved
#: fallible by mutating a tracked file teaches people not to prove it.
SNAPSHOT = pathlib.Path(
    os.environ.get(
        "PDFCER_ENGINE_API_SNAPSHOT",
        str(ROOT / "tools" / "gates" / "engine-api-snapshot.txt"),
    )
)
MANIFEST = ROOT / "crates" / "pdfcer-gui" / "Cargo.toml"
LOCK = ROOT / "Cargo.lock"

#: Minimum characters of prose an `exempt` line must carry. An exemption with
#: no argument is a re-baseline in disguise, which is the one move this gate
#: is built to make impossible.
MIN_REASON = 20


def rel(p: pathlib.Path) -> str:
    """`p` relative to the repo root when it is inside it, else its full path.

    The snapshot can be relocated by `PDFCER_ENGINE_API_SNAPSHOT` -- that is
    how this gate is falsified without mutating a tracked file -- and a bare
    `relative_to` throws on a path outside the tree, which would turn a
    falsification run into a traceback instead of a verdict.
    """
    try:
        return str(p.relative_to(ROOT))
    except ValueError:
        return str(p)

#: How far the live enumeration may fall below the snapshot before the scan is
#: treated as collapsed rather than as a large deletion. Half is generous: the
#: engine has never removed more than a handful of public items in one pin
#: move, and a genuine halving is something a person should look at anyway.
COLLAPSE_RATIO = 0.5


# ===========================================================================
# THE SCANNER
# ===========================================================================
#
# Line-oriented and indentation-driven. See the header's "WHAT IT ACTUALLY
# READS" section for why that is sound here and where it is not.

_OPEN_STRUCTISH = re.compile(
    r"^pub\s+(?:struct|enum|trait|union)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
_KIND_STRUCTISH = re.compile(
    r"^pub\s+(struct|enum|trait|union)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
_TOPLEVEL_FN = re.compile(
    r"^pub\s+(?:(?:const|async|unsafe|extern(?:\s+\"[^\"]*\")?)\s+)*"
    r"(fn|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
_IMPL = re.compile(r"^(?:unsafe\s+)?impl(?:\s*<.*?>)?\s+(.+)$")
_METHOD = re.compile(
    r"^pub\s+(?:(?:const|async|unsafe|extern(?:\s+\"[^\"]*\")?)\s+)*"
    r"fn\s+([A-Za-z_][A-Za-z0-9_]*)"
)
_TRAIT_FN = re.compile(
    r"^(?:pub\s+)?(?:(?:const|async|unsafe|extern(?:\s+\"[^\"]*\")?)\s+)*"
    r"(?:fn|type)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
_FIELD = re.compile(r"^pub\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*:")
_VARIANT = re.compile(r"^([A-Z][A-Za-z0-9_]*)\s*(?:[,({=]|$)")
_INLINE_ENUM = re.compile(
    r"^pub\s+enum\s+([A-Za-z_][A-Za-z0-9_]*)[^{]*\{(.*)\}\s*$"
)


def module_path(crate: str, rel: pathlib.PurePosixPath) -> str:
    """`crates/pdfcer-core/src/text_edit/mod.rs` -> `pdfcer_core::text_edit`.

    The DEFINITION site, not the re-exported public path. A key has to be
    stable across the churn the engine actually produces, and `pub use`
    re-export lines are rewritten far more often than files are moved. When a
    file DOES move, the item leaves under one key and arrives under another —
    which is why [`report`] prints a "probably moved" note pairing an addition
    with a removal of the same leaf name, rather than presenting the arrival
    as a new capability.
    """
    parts = list(rel.parts)
    # rel is relative to `<crate>/src`
    if parts and parts[-1] in ("lib.rs", "mod.rs"):
        parts = parts[:-1]
    elif parts:
        parts[-1] = parts[-1][:-3] if parts[-1].endswith(".rs") else parts[-1]
    return "::".join([crate.replace("-", "_")] + parts)


def scan_file(text: str, mod: str) -> list[tuple[str, str]]:
    """Every `pub` item in one file, as `(kind, dotted::path)` pairs.

    The state machine is two-level, which is all rustfmt'd Rust needs:

      * column 0  — a module-level item, and possibly the opener of a block.
      * column 4  — a member of whatever the column-0 opener was: a method of
                    an `impl`, a field of a `struct`, a variant of an `enum`,
                    a `fn` of a `trait`.
      * deeper    — ignored. Nothing public to the crate's consumers lives
                    there that is not already named at one of the two levels
                    above.

    `impl Trait for Type` blocks are skipped entirely: their methods are the
    TRAIT's API, already recorded at the trait's own declaration, and counting
    them would add `Display::fmt` for four hundred types as "new capability".

    Block comments are tracked so that a `/* pub fn … */` cannot manufacture an
    item. Line comments and doc comments need no handling: every pattern here
    anchors on `pub`, `impl` or an uppercase identifier at column 0 or 4, and
    `///` matches none of them.
    """
    out: list[tuple[str, str]] = []
    opener_kind: str | None = None
    opener_name: str | None = None
    in_comment = False

    for raw in text.split("\n"):
        line = raw.rstrip()
        stripped = line.strip()

        if in_comment:
            if "*/" in stripped:
                in_comment = False
            continue
        if stripped.startswith("/*") and "*/" not in stripped:
            in_comment = True
            continue
        if not stripped or stripped.startswith("//") or stripped.startswith("#"):
            continue

        indent = len(line) - len(line.lstrip(" "))

        if indent == 0:
            if stripped.startswith("}"):
                opener_kind = opener_name = None
                continue

            m = _INLINE_ENUM.match(stripped)
            if m:
                # `pub enum Foo { A, B }` on one line. Rare in rustfmt'd source
                # but it costs three lines to be right about and would
                # otherwise drop every variant of such an enum silently.
                name = m.group(1)
                out.append(("enum", f"{mod}::{name}"))
                for piece in m.group(2).split(","):
                    v = _VARIANT.match(piece.strip())
                    if v:
                        out.append(("variant", f"{mod}::{name}::{v.group(1)}"))
                opener_kind = opener_name = None
                continue

            m = _KIND_STRUCTISH.match(stripped)
            if m:
                kind, name = m.group(1), m.group(2)
                out.append((kind, f"{mod}::{name}"))
                if stripped.endswith("{"):
                    opener_kind, opener_name = kind, name
                else:
                    opener_kind = opener_name = None
                continue

            m = _TOPLEVEL_FN.match(stripped)
            if m:
                kind, name = m.group(1), m.group(2)
                out.append(("fn" if kind == "fn" else kind, f"{mod}::{name}"))
                opener_kind = "opaque" if stripped.endswith("{") else None
                opener_name = None
                continue

            m = _IMPL.match(stripped)
            if m:
                rest = m.group(1)
                if re.search(r"\bfor\b", rest):
                    opener_kind, opener_name = "opaque", None
                else:
                    t = re.match(r"([A-Za-z_][A-Za-z0-9_]*)", rest.strip())
                    opener_kind = "impl" if t else "opaque"
                    opener_name = t.group(1) if t else None
                continue

            # Anything else at column 0 that opens a block is opaque: a private
            # item, a `mod tests {`, a macro invocation. Its contents are not
            # this gate's business, and treating it as opaque is what stops a
            # `bitflags! { ... }` body being read as a struct's fields.
            opener_kind = "opaque" if stripped.endswith("{") else opener_kind
            if stripped.endswith("{"):
                opener_name = None
            continue

        if indent == 4 and opener_kind and opener_name:
            if opener_kind == "impl":
                m = _METHOD.match(stripped)
                if m:
                    out.append(("method", f"{mod}::{opener_name}::{m.group(1)}"))
            elif opener_kind == "trait":
                m = _TRAIT_FN.match(stripped)
                if m:
                    out.append(("method", f"{mod}::{opener_name}::{m.group(1)}"))
            elif opener_kind in ("struct", "union"):
                m = _FIELD.match(stripped)
                if m:
                    out.append(("field", f"{mod}::{opener_name}::{m.group(1)}"))
            elif opener_kind == "enum":
                m = _VARIANT.match(stripped)
                if m:
                    out.append(("variant", f"{mod}::{opener_name}::{m.group(1)}"))

    return out


# ===========================================================================
# READING THE ENGINE
# ===========================================================================


def engine_crates() -> list[str]:
    """Every engine crate this shell depends on, DERIVED from the manifest.

    A typed list here would be the very defect this gate is built to find: the
    day a fourth engine crate is taken, a typed list makes that crate's whole
    API invisible to the instrument built to notice new API.

    Matched on the `git = "file:///…"` marker rather than on the `pdfcer-`
    prefix, because the prefix is exactly what a rename changes and the URL is
    what Cargo resolves — `engine_path.py`'s argument, applied one level up.
    """
    if not MANIFEST.is_file():
        return []
    names: list[str] = []
    for line in MANIFEST.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.lstrip().startswith("#"):
            continue
        m = re.match(r'^\s*([A-Za-z0-9_-]+)\s*=\s*\{[^}]*git\s*=\s*"file:///', line)
        if m and m.group(1) not in names:
            names.append(m.group(1))
    return names


def locked_revision() -> str | None:
    """The engine commit `Cargo.lock` pins, from the source URL's fragment."""
    if not LOCK.is_file():
        return None
    for line in LOCK.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith('source = "git+file:') and "#" in line:
            return line.rsplit("#", 1)[1].rstrip('"')
    return None


def api_at(repo: pathlib.Path, rev: str, crates: list[str]) -> dict[str, set[str]]:
    """`{crate: {"<kind> <path>", …}}` for the engine as of `rev`.

    One `git archive` for the whole set — 200-odd files through a single pipe
    rather than 200 `git show` spawns, which is the difference between this
    gate costing a second and costing a minute. It must run on every commit.

    Raises `LookupError` when git will not produce the tree, which the caller
    turns into a SKIP: a revision genuinely absent from the clone is a real
    state on a fresh machine, and is not evidence of anything about the API.
    """
    paths = [f"crates/{c}/src" for c in crates]
    try:
        p = subprocess.run(
            ["git", "-C", str(repo), "archive", rev, "--"] + paths,
            capture_output=True,
        )
    except OSError as exc:  # git not on PATH
        raise LookupError(f"git could not be run: {exc}") from exc
    if p.returncode != 0:
        raise LookupError(p.stderr.decode("utf-8", errors="replace").strip())

    found: dict[str, set[str]] = {c: set() for c in crates}
    with tarfile.open(fileobj=io.BytesIO(p.stdout)) as tf:
        for member in tf:
            if not member.isfile() or not member.name.endswith(".rs"):
                continue
            parts = pathlib.PurePosixPath(member.name).parts
            # crates/<crate>/src/<rel...>
            if len(parts) < 4 or parts[0] != "crates" or parts[2] != "src":
                continue
            crate = parts[1]
            if crate not in found:
                continue
            fh = tf.extractfile(member)
            if fh is None:
                continue
            text = fh.read().decode("utf-8", errors="replace")
            mod = module_path(crate, pathlib.PurePosixPath(*parts[3:]))
            for kind, path in scan_file(text, mod):
                found[crate].add(f"{kind} {path}")
    return found


def flatten(api: dict[str, set[str]]) -> set[str]:
    return {f"{c} {item}" for c, items in api.items() for item in items}


# ===========================================================================
# THE SNAPSHOT
# ===========================================================================


def read_snapshot(path: pathlib.Path) -> tuple[set[str], dict[str, str], list[str]]:
    """`(seen, exemptions, complaints)`.

    An `exempt` line whose reason is missing or under [`MIN_REASON`] characters
    is returned as a complaint rather than as an exemption, and the gate fails
    on it. A reason nobody had to write is a re-baseline with a keyword in
    front of it.
    """
    seen: set[str] = set()
    exempt: dict[str, str] = {}
    complaints: list[str] = []
    if not path.is_file():
        return seen, exempt, complaints
    for n, raw in enumerate(
        path.read_text(encoding="utf-8", errors="replace").splitlines(), 1
    ):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("exempt "):
            body = line[len("exempt "):]
            if "--" not in body:
                complaints.append(f"line {n}: `exempt` with no `--` reason: {line}")
                continue
            key, reason = body.split("--", 1)
            key, reason = key.strip(), reason.strip()
            if len(reason) < MIN_REASON:
                complaints.append(
                    f"line {n}: exemption reason is {len(reason)} characters, "
                    f"under the {MIN_REASON}-character floor: {line}"
                )
                continue
            if len(key.split(" ")) < 3:
                complaints.append(
                    f"line {n}: exemption key is not `<crate> <kind> <path>`: {line}"
                )
                continue
            exempt[key] = reason
            continue
        # * A line that is neither a comment nor three fields is MALFORMED, and
        # it is reported rather than tolerated. The first run of this gate hit
        # exactly this: three un-commented header lines parsed as items, and the
        # crash was better than the alternative, which would have been three
        # phantom "removed" items quietly widening the moved-item pairing.
        if len(line.split(" ")) < 3:
            complaints.append(f"line {n}: not `<crate> <kind> <path>`: {line}")
            continue
        seen.add(line)
    return seen, exempt, complaints


SNAPSHOT_HEADER = """\
# engine-api-snapshot.txt — the engine's public surface, as last looked at.
#
# GENERATED. Written by `tools/gates/check-engine-api-drift.py --update`, read
# by `tools/gates/check-engine-api-drift.sh`. Read that gate's header first;
# this file is its data and means nothing without it.
#
# ---------------------------------------------------------------------------
# ★★★ WHAT THIS FILE CLAIMS, AND WHAT IT DOES NOT
# ---------------------------------------------------------------------------
#
# Every line below is a public item that existed in an engine crate at the
# BASELINE revision named under `baseline:`. That is the entire claim.
#
# It is **NOT** a claim that any of these items was reviewed, wanted, judged,
# or consumed. It is a record of what has been LOOKED AT, so that what the
# engine grows NEXT can be told apart from what was already there. Reading a
# line here as approval would be reading it as the opposite of its purpose.
#
# ---------------------------------------------------------------------------
# ★★★ WHY YOU CANNOT SILENCE THE GATE BY REGENERATING THIS FILE
# ---------------------------------------------------------------------------
#
# `--update` folds in only the new items that ALREADY pass the gate's
# accounting rule — named in this repository's Rust sources, or written about
# in a root-level markdown register, or exempted below with a reason. An
# unaccounted item is deliberately left OUT, so the gate goes red again on the
# next run. There is no command that makes a finding disappear.
#
# ---------------------------------------------------------------------------
# LINE FORMAT
# ---------------------------------------------------------------------------
#
#   <crate> <kind> <definition path>
#       an item seen at the baseline. `kind` is one of struct, enum, trait,
#       union, fn, type, const, static, method, field, variant. The path is the
#       DEFINITION site, not the re-exported public path.
#
#   exempt <crate> <kind> <path> -- <reason>
#       an item the gate must not report, and WHY. The reason is mandatory and
#       must be at least %(min_reason)d characters: an exemption without an argument is
#       a re-baseline wearing a disguise.
#
# ---------------------------------------------------------------------------
# baseline: %(baseline)s
# crates: %(crates)s
# items: %(count)d
# ---------------------------------------------------------------------------
"""


def write_snapshot(path: pathlib.Path, baseline: str, crates: list[str],
                   items: set[str], exempt: dict[str, str]) -> None:
    body = sorted(items)
    head = SNAPSHOT_HEADER % {
        "min_reason": MIN_REASON,
        "baseline": baseline,
        "crates": " ".join(crates),
        "count": len(body),
    }
    lines = [head]
    for key in sorted(exempt):
        lines.append(f"exempt {key} -- {exempt[key]}")
    if exempt:
        lines.append("")
    lines.extend(body)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


# ===========================================================================
# THE ACCOUNTING RULE
# ===========================================================================


def repo_corpus() -> tuple[str, str]:
    """`(rust, markdown)` — everything this repository says, as two blobs.

    One pass over the tree holding it all in memory, rather than a grep per
    item: several thousand identifiers over several hundred files is a number
    of file reads that makes the naive shape unusable on Windows. This is
    `verb-coverage.py`'s measured lesson, reused.

    The markdown half is EVERY `*.md` at the repository root — derived by
    glob, never a typed list of register filenames. A new register file is
    exactly the kind of thing a typed list would go blind to.
    """
    rust_parts: list[str] = []
    for crate in sorted((ROOT / "crates").glob("*")):
        src = crate / "src"
        if src.is_dir():
            for f in src.rglob("*.rs"):
                rust_parts.append(f.read_text(encoding="utf-8", errors="replace"))
    md_parts = [
        f.read_text(encoding="utf-8", errors="replace")
        for f in sorted(ROOT.glob("*.md"))
    ]
    return "\n".join(rust_parts), "\n".join(md_parts)


#: Kinds whose path ends in the item name itself; everything else is a member
#: hanging off an owner, so its module is two segments up rather than one.
_TOPLEVEL_KINDS = {"struct", "enum", "trait", "union", "fn", "type", "const", "static"}


def module_of(key: str) -> str:
    """The module a key lives in. `enum a::b::C` -> `a::b`; `field a::b::C::d` -> `a::b`."""
    _, kind, path = key.split(" ", 2)
    segs = path.split("::")
    drop = 1 if kind in _TOPLEVEL_KINDS else 2
    return "::".join(segs[:-drop]) if len(segs) > drop else path


def module_index(keys) -> set[str]:
    """Every module path present in `keys`, plus all their ancestors.

    Ancestors are added because a module can hold only submodules and no items
    of its own; without them, `a::b` would look brand new the day `a::b::c`
    gains its first item, which is a false alarm about a module that has been
    there all along.
    """
    out: set[str] = set()
    for k in keys:
        segs = module_of(k).split("::")
        for i in range(1, len(segs) + 1):
            out.add("::".join(segs[:i]))
    return out


def new_module_root(key: str, seen_modules: set[str]) -> str | None:
    """The SHALLOWEST ancestor module of `key` that the snapshot has never seen.

    ★★★ WHY THE FINDING IS SOMETIMES A MODULE AND NOT AN ITEM.
    =========================================================

    Measured on this gate's first real run, and it changed the design.

    The engine's signing subsystem — `pdfcer_core::sign`, with `apply`,
    `cms_build` and `pkcs12` under it — landed in one pin move carrying **98
    public items**. Reported one by one, that is 98 lines about one event, and
    a reader would have to reconstruct "a whole new subsystem arrived" from a
    list of `SignReport::byte_range`-shaped fragments. Worse, discharging it
    would mean writing 98 backticked symbol names into a register, which
    nobody will do and which nobody should: the thing a person needs to look at
    is **the subsystem**, once.

    So when an item's module is itself new, the unit of the finding is the
    shallowest new module, and one register sentence naming that module
    discharges everything inside it.

    ⇒ ★★ AND THE RULE STOPS THERE, DELIBERATELY. It applies ONLY to a module
      the snapshot has never seen. An item arriving in an EXISTING module is
      still reported as an item, because that is the `RefusalKind` case: a
      module-level discharge for `pdfcer_core::text_edit` — which has existed
      for months and is named in half this project's documents — would have
      swallowed the very item this gate was built to catch. The distinction is
      the whole reason the rule is safe: **a new subsystem is one act of
      attention; a new item in a familiar one is another.**
    """
    segs = module_of(key).split("::")
    for i in range(1, len(segs) + 1):
        prefix = "::".join(segs[:i])
        if prefix not in seen_modules:
            return prefix
    return None


def names_of(key: str) -> tuple[str, str | None]:
    """`("<crate> <kind> <path>") -> (leaf identifier, owner identifier or None)`.

    For a member — a method, a field, a variant — the owner is the type it
    hangs off. Both must appear before the item counts as named, because a
    bare leaf like `Other`, `new` or `value` matches somewhere in any codebase
    of this size and would silently discharge a real finding.
    """
    path = key.split(" ", 2)[2]
    segs = path.split("::")
    leaf = segs[-1]
    kind = key.split(" ", 2)[1]
    owner = segs[-2] if kind in ("method", "field", "variant") and len(segs) >= 2 else None
    return leaf, owner


def make_accountant(rust: str, md: str):
    """A closure answering "has this repository said anything about `key`?".

    Two routes, and both are deliberately generous, because the gate's job is
    to catch SILENCE and not to audit quality:

      1. **Named in this workspace's own Rust.** Word-boundary. For a member,
         the owner must appear too.
      2. **Written about in a root-level markdown file, in backticks.** This is
         how every register in this project names an engine symbol, and it is
         the route `check-verb-coverage.sh` established. A sentence saying "not
         built, declined" passes -- a gate cannot read English and must not
         pretend to.
    """
    # ** TOKENISED ONCE, NOT GREPPED PER ITEM, and the difference is the whole
    # difference between a gate that runs on every commit and one that does not.
    #
    # The first cut ran `re.search(r"\bNAME\b", blob)` per identifier. Over a
    # ~13 MB Rust corpus and a few hundred candidate items that is several
    # hundred full scans of the corpus, and it measured **38 seconds** — for a
    # gate whose stated contract is seconds. Splitting each corpus into its
    # identifier set once and testing membership is exactly equivalent to a
    # word-boundary match (`[A-Za-z_][A-Za-z0-9_]*` IS the word-boundary token)
    # and turns every subsequent question into a hash lookup.
    _WORD = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
    rust_words = set(_WORD.findall(rust))
    md_words = set(_WORD.findall(md))
    md_ticked = set(_WORD.findall("\n".join(re.findall(r"`[^`\n]+`", md))))

    def accounted(key: str) -> str | None:
        leaf, owner = names_of(key)
        kind = key.split(" ", 2)[1]

        # *** A VARIANT MUST BE SPELLED IN FULL, and the first real run of this
        # gate is why.
        #
        # `pdfcer_core::edit::EncryptError::RedactionPending` arrived in the
        # v0.39.0 window. `EncryptError` appears in this shell (`protect/mod.rs`
        # flattens it arm by arm) and `RedactionPending` appears in this shell
        # (as `WriteError::RedactionPending`, a DIFFERENT enum's variant of the
        # same name) -- so an owner-and-leaf test found both names, called the
        # item consumed, and discharged it. It was not consumed: the flattening
        # match has no arm for it and it falls through the `#[non_exhaustive]`
        # catch-all, so the engine's most safety-critical refusal reaches the
        # operator as a generic one.
        #
        # A variant is essentially always written `Owner::Variant`, at the
        # construction site and at every match arm, so requiring the qualified
        # spelling costs almost nothing and closes that hole. A `use E as K;
        # K::Variant` aliasing would be reported rather than discharged -- which
        # is the safe direction, and one exemption line away.
        #
        # ** It is NOT applied to methods or fields, and that is a fact about
        # how Rust is written rather than a preference: a method is called
        # `x.method()` and a field read `x.field`, so demanding `Type::member`
        # would report every correctly-consumed one of them. The rule follows
        # the call syntax, not a taste.
        if kind == "variant" and owner is not None:
            qualified = f"{owner}::{leaf}"
            if qualified in rust:
                return "spelled in full in this workspace's Rust sources"
            if qualified in md:
                return "spelled in full in a root-level markdown register"
            return None

        if leaf in rust_words and (owner is None or owner in rust_words):
            return "named in this workspace's Rust sources"
        if leaf in md_ticked and (owner is None or owner in md_words):
            return "written about in a root-level markdown register"
        return None

    def module(path: str) -> str | None:
        """Has this repository written down a whole MODULE path?

        The FULL dotted path, never its last segment. A module called `sign`,
        `view`, `build` or `object` would be discharged by the bare word
        appearing in any sentence in any register, which is not somebody having
        looked at a new subsystem -- it is a coincidence. The qualified spelling
        is also this project's own convention: every register here names an
        engine symbol as `pdfcer_core::text_edit::RefusalKind`, not as
        `RefusalKind`.

        *** AND IT IS NOT A SUBSTRING TEST, because the first cut was and it
        silently discharged the largest finding this gate has ever produced.
        `pdfcer_core::sign` -- the whole digital-signing subsystem, 101 public
        items arriving in one pin move -- was reported as accounted for,
        because `pdfcer_core::sign` is a PREFIX of `pdfcer_core::signature` and
        `pdfcer_core::signature_verify`, which this shell has consumed and
        written about at length. A plain `in` found the prefix inside the
        longer name and called the subsystem discussed.

        ⇒ The path must be followed by something that cannot continue a Rust
        identifier. That is one lookahead, and without it this gate's headline
        finding printed as a green line in the accounted list.
        """
        pat = re.compile(re.escape(path) + r"(?![A-Za-z0-9_])")
        if pat.search(rust):
            return "its module path is named in this workspace's Rust sources"
        if pat.search(md):
            return "its module path is written down in a root-level register"
        return None

    accounted.module = module
    return accounted


# ===========================================================================
# THE REPORT
# ===========================================================================


def report(live: set[str], seen: set[str], exempt: dict[str, str],
           complaints: list[str], accounted,
           out=sys.stdout) -> tuple[int, list[str], set[str]]:
    """The whole comparison. `(exit code, unaccounted keys, foldable keys)`.

    Factored out so `--self-test` drives exactly the code path the real run
    drives. A self-test that exercises a parallel implementation is a test of
    the parallel implementation.

    ** `foldable` is computed HERE and handed to `--update`, rather than
    `--update` re-deriving it. Two places deciding "is this item accounted for"
    is two answers to one question, and this project has paid for that twice --
    `text_edit_focused` cost the Delete key and then the space bar because two
    sites each had their own idea of one predicate. The gate's verdict and what
    the snapshot absorbs are the same decision, so they are made once.
    """
    w = out.write
    added = sorted(live - seen)
    removed = sorted(seen - live)
    seen_modules = module_index(seen)

    unaccounted: list[str] = []
    consumed: list[tuple[str, str]] = []
    excused: list[str] = []
    foldable: set[str] = set(seen & live)
    # Items whose module is itself new, grouped under the shallowest new module.
    # See [`new_module_root`] for why the unit of the finding changes shape.
    by_new_module: dict[str, list[str]] = {}
    for key in added:
        if key in exempt:
            excused.append(key)
            continue
        crate = key.split(" ", 1)[0]
        root = new_module_root(key, seen_modules)
        if root is not None:
            by_new_module.setdefault(f"{crate} module {root}", []).append(key)
            continue
        why = accounted(key)
        if why:
            consumed.append((key, why))
            foldable.add(key)
        else:
            unaccounted.append(key)

    # A whole new module is ONE finding. It is discharged by naming the module
    # itself -- or by exempting it under the key `<crate> module <path>`, the
    # same `exempt` line every other item uses.
    new_modules: list[tuple[str, int]] = []
    for mkey, items in sorted(by_new_module.items()):
        if mkey in exempt:
            excused.append(mkey)
            continue
        path = mkey.split(" ", 2)[2]
        why = accounted.module(path)
        if why:
            consumed.append((mkey, f"{why}; the {len(items)} item(s) under it follow"))
            foldable.update(items)
        else:
            new_modules.append((mkey, len(items)))

    w(f"measured {len(live)} public item(s) against {len(seen)} in the snapshot\n")
    w(f"         {len(added)} added, {len(removed)} removed, "
      f"{len(consumed)} of the added already accounted for, "
      f"{len(excused)} exempt\n")

    # ** THE EXEMPTION SET IS PRINTED ON EVERY RUN, and that is the same
    # discipline `run-all.sh` applies to SKIPs: a thing that silences a check
    # must be visible every time the check runs, or it becomes permanent by
    # nobody looking at it. An exemption is a finding somebody decided not to
    # act on, and it stays on screen with its reason until it is deleted.
    if excused:
        w(f"\nexempt ({len(excused)}): live, NEW since the baseline, and silenced by a"
          f" written reason.\n         Read them; an exemption is a finding somebody"
          f" chose not to act on.\n\n")
        for key in excused:
            w(f"        {key}\n              -> {exempt[key]}\n")

    if complaints:
        w("\nFAIL: the snapshot carries exemption line(s) with no usable reason:\n\n")
        for c in complaints:
            w(f"        {c}\n")
        w("\n  An exemption without an argument is a re-baseline with a keyword in\n"
          "  front of it. Write the reason, or delete the line and account for the\n"
          "  item properly.\n")
        return 1, unaccounted, foldable

    if consumed:
        w("\nnote: these arrived and are already accounted for; `--update` folds them in:\n\n")
        for key, why in consumed[:20]:
            w(f"        {key}\n              -> {why}\n")
        if len(consumed) > 20:
            w(f"        ... and {len(consumed) - 20} more\n")

    if new_modules:
        w("\nFAIL: the engine grew whole MODULE(S) this repository has never named:\n\n")
        for mkey, n in new_modules:
            path = mkey.split(" ", 2)[2]
            w(f"        {path}  ({n} public item(s))\n")
            for k in sorted(by_new_module[mkey])[:6]:
                w(f"             {k.split(' ', 2)[2]}\n")
            if n > 6:
                w(f"             ... and {n - 6} more\n")
        w("""
  A whole new module is ONE act of attention, not N of them, which is why it
  is reported this way. It is also the shape that arrives when the engine ships
  a SUBSYSTEM -- and a subsystem is exactly the thing that gets shipped, noted
  in a reply nobody reads, and then forgotten. `check-engine-backlog.sh`'s
  header records that happening to PNG/JPEG/SVG export: the engine shipped all
  of it in a day, sent a note saying "here is what a shell wires", and this
  shell built none of it and filed no row for a day.

  TO DISCHARGE IT, do exactly one of:

    * **Write the row.** `ENGINE_BACKLOG.md`, naming the module by its full
      path in backticks, with a verdict -- `wanted`, `declined`, `blocked`, or
      an honest `unknown` -- and the argument. One sentence naming the module
      accounts for everything inside it, because the module is the finding.

    * **Consume it.** Naming the module path anywhere in this workspace's Rust
      does the same thing, for the same reason.

    * **Exempt it**, in `tools/gates/engine-api-snapshot.txt`, under the key
      printed above with `module` as its kind, with a written reason.

  What is NOT allowed is silence, because a subsystem that nobody has an
  opinion about is indistinguishable from one nobody noticed.
""")

    if not unaccounted:
        return (1 if new_modules else 0), unaccounted, foldable

    # * The "probably moved" note, which is `check-engine-backlog.sh`'s
    # near-miss idea applied to a different key. A file move takes an item out
    # under one path and puts it back under another, and a rename and a new
    # capability are IDENTICAL to a key while being opposite acts. Pairing an
    # addition with a removal of the same leaf name is the only thing that
    # tells a reader which one they are looking at.
    removed_leaves: dict[str, list[str]] = {}
    for key in removed:
        leaf, _ = names_of(key)
        removed_leaves.setdefault(leaf, []).append(key)

    w("\nFAIL: the engine gained public item(s) that this repository names nowhere\n"
      "      and says nothing about:\n\n")
    for key in unaccounted:
        w(f"        {key}\n")
        leaf, _ = names_of(key)
        for old in removed_leaves.get(leaf, [])[:2]:
            w(f"          -> probably MOVED, not new; the same leaf left: {old}\n")
    w("""
  An item in this list is one of FOUR things, and they are opposite acts:

    1. **An item that MOVED.** Where a "probably MOVED" line was printed above,
       read it FIRST: the engine relocated a file and the item arrived under a
       new definition path. FIX: nothing, except `--update` -- but read the pair
       before you believe it, because an innocent leaf name can collide.

    2. **A capability that landed and nobody noticed.** That is what this gate
       is for. `pdfcer_core::text_edit::RefusalKind` shipped in answer to this
       project's own request, was pinned here, and sat unconsumed for a day
       while a function in this repo said in its own doc comment that it was
       "written to be deleted" the day that type arrived. Two gates watched and
       both were keyed on `EditSession`'s verbs. Go and read the channel at
       `D:\\Dev\\FeatureRequests\\pdfce_FeatureRequests\\open\\`, then wire it.

    3. **A capability this shell should discuss but not build yet.** Fine -- say
       so, in `ENGINE_BACKLOG.md`, in backticks, with a verdict and an
       argument. That discharges it here, because a gate cannot read English
       and the written sentence is the whole mechanism.

    4. **Engine-internal noise.** A helper in a private module, an alternate
       spelling, a type this shell will never hold. Fine -- `exempt` it in
       `tools/gates/engine-api-snapshot.txt` with a reason of at least
""" + f"       {MIN_REASON} characters.\n" + """
  What is NOT allowed is silence, because silence is indistinguishable from
  (2) and reads as (4).
""")
    return 1, unaccounted, foldable


# ===========================================================================
# ** SELF-TEST -- a gate that has never been seen to fail is a rumour
# ===========================================================================
#
# Both halves are asserted, because only the pair is evidence:
#
#   * it CATCHES a planted item that nothing accounts for, and it catches an
#     exemption whose reason is too thin to be an argument;
#   * it PASSES the four shapes that are CORRECT -- an item already in the
#     snapshot, an item named in this repository's Rust, an item written about
#     in a register, and a properly-reasoned exemption -- because a gate that
#     reports the correct shape trains people to ignore it, which is worse than
#     not having the gate.
#
# * And the SCANNER is asserted separately, on a fixture carrying every shape
# that has a plausible way to go wrong: a brace inside a block comment, an
# `impl Trait for Type` whose methods must NOT be counted, a private inline
# `mod tests` whose `pub fn` must NOT be counted, a macro body whose lines look
# like fields, and a single-line enum. That is the one failure a "did it
# report?" check cannot see: a scanner that silently DROPPED half the file
# would produce a smaller live set, a smaller `added` set, and a clean PASS --
# silent blindness, which is the worst outcome available to a gate.
# ===========================================================================

_FIXTURE = '''\
//! A module doc comment.

/*
 * A block comment carrying a brace {  and a decoy:
 * pub struct NotAnItem {
 */

use std::fmt;

pub struct Kept {
    pub taken: u32,
    private: u32,
}

pub enum Coarse {
    First,
    Second { detail: u32 },
}

pub enum OneLine { Alpha, Beta }

impl Kept {
    pub fn wanted(&self) -> u32 { self.taken }
    fn unwanted(&self) -> u32 { self.private }
}

impl fmt::Display for Kept {
    pub fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { Ok(()) }
}

pub trait Classify {
    fn classify(&self) -> Coarse;
}

pub fn free_function(a: u32) -> u32 { a }

pub const LIMIT: u32 = 4;

bitflags! {
    pub struct Decoy: u32 {
        const NOTHING = 1;
    }
}

mod tests {
    pub fn helper_that_is_not_api() {}
}
'''

_FIXTURE_EXPECT = {
    "struct m::Kept",
    "field m::Kept::taken",
    "enum m::Coarse",
    "variant m::Coarse::First",
    "variant m::Coarse::Second",
    "enum m::OneLine",
    "variant m::OneLine::Alpha",
    "variant m::OneLine::Beta",
    "method m::Kept::wanted",
    "trait m::Classify",
    "method m::Classify::classify",
    "fn m::free_function",
    "const m::LIMIT",
}


def self_test() -> int:
    fail = 0

    # --- the scanner, on the fixture ---------------------------------------
    got = {f"{k} {p}" for k, p in scan_file(_FIXTURE, "m")}
    missing = _FIXTURE_EXPECT - got
    extra = got - _FIXTURE_EXPECT
    if missing:
        print("engine-api-drift --self-test: FAIL -- the scanner DROPPED item(s):")
        for k in sorted(missing):
            print(f"    {k}")
        print("  A scanner that drops items produces a smaller live set, a smaller")
        print("  added set, and a clean PASS. That is silent blindness and it is the")
        print("  worst outcome available to a gate.")
        fail = 1
    if extra:
        print("engine-api-drift --self-test: FAIL -- the scanner INVENTED item(s):")
        for k in sorted(extra):
            print(f"    {k}")
        print("  Expected none of: a decoy inside a block comment, an `impl Trait for`")
        print("  method (that is the trait's API, already recorded), a `pub fn` inside")
        print("  a private `mod tests`, or a macro body's contents read as fields.")
        fail = 1

    # --- the comparison, on planted data -----------------------------------
    live = {
        "pdfcer-core enum pdfcer_core::text_edit::RefusalKind",       # in snapshot
        "pdfcer-core enum pdfcer_core::sharpen::ReticulatingSplines",  # NEW, silent
        "pdfcer-core struct pdfcer_core::edit::EditSession",           # NEW, in Rust
        # * NEW MODULE whose path is a PREFIX of two modules this repository
        # HAS written about (`pdfcer_core::signature`, `..::signature_verify`).
        # It must still be REPORTED: that prefix collision silently discharged
        # a 101-item subsystem on this gate's first run with the module rule.
        "pdfcer-core fn pdfcer_core::sign::sign_document",
        # * NEW MODULE whose path IS written down. It must NOT be reported, or
        # the module rule is pure noise and a 101-item subsystem could only be
        # discharged by typing 101 backticked symbols.
        "pdfcer-core fn pdfcer_core::export::to_svg",
        "pdfcer-core fn pdfcer_core::hidden::internal_helper",         # NEW, exempt
        "pdfcer-core enum pdfcer_core::moved::RefusalKind",            # NEW, a move
        # * NEW, and the shape that discharged a real gap on this gate's first
        # run: both names present in the corpus, never together, on a
        # DIFFERENT enum. It must be REPORTED, not accounted.
        "pdfcer-core variant pdfcer_core::edit::EncryptError::RedactionPending",
        # * NEW, and the correct shape one notch over: a variant spelled in
        # full. It must NOT be reported, or the rule above is just noise.
        "pdfcer-core variant pdfcer_render::font::StrokeDisplay::Hairline",
        # * A WHOLE NEW MODULE nothing names. It must be reported ONCE, as a
        # module, with a count -- never as three separate item findings.
        "pdfcer-core struct pdfcer_core::reticulate::Splines",
        "pdfcer-core fn pdfcer_core::reticulate::reticulate",
        "pdfcer-core enum pdfcer_core::reticulate::deep::Mode",
    }
    seen = {
        "pdfcer-core enum pdfcer_core::text_edit::RefusalKind",
        "pdfcer-core enum pdfcer_core::old_home::RefusalKind",         # the move's origin
        # Anchors so that the modules above are NOT new; only `reticulate` and
        # `sign` are, and each tests one direction of the module rule.
        "pdfcer-core struct pdfcer_core::sharpen::Anchor",
        "pdfcer-core struct pdfcer_core::edit::Anchor",
        "pdfcer-core struct pdfcer_core::hidden::Anchor",
        "pdfcer-core struct pdfcer_core::moved::Anchor",
        "pdfcer-core struct pdfcer_render::font::Anchor",
    }
    exempt = {
        "pdfcer-core fn pdfcer_core::hidden::internal_helper":
            "engine-internal; this shell never holds one and never will",
    }
    accountant = make_accountant(
        # `EncryptError` and `RedactionPending` are BOTH here and never
        # together, which is the corpus that fooled the owner-and-leaf rule.
        # `StrokeDisplay::Hairline` is here in full, which must still pass.
        rust="let s: EditSession = todo!();\n"
             "match e { EncryptError::Write(_) => {} }\n"
             "if x == WriteError::RedactionPending { }\n"
             "opts.stroke_display = StrokeDisplay::Hairline;\n",
        # * `pdfcer_core::signature` is here and `pdfcer_core::sign` is NOT.
        # The first cut of the module rule tested the path as a plain
        # substring, so this corpus discharged the whole `pdfcer_core::sign`
        # subsystem on a prefix match. The planted `reticulate` module below is
        # absent from both corpora and must still be REPORTED.
        md="`pdfcer_core::signature` and `pdfcer_core::signature_verify` are wired; "
           "`pdfcer_core::export` is wanted and has a row",
    )
    buf = io.StringIO()
    rc, unacc, _fold = report(live, seen, exempt, [], accountant, out=buf)
    out = buf.getvalue()

    if rc != 1:
        print(f"engine-api-drift --self-test: FAIL -- the plants were not detected (rc={rc}).")
        fail = 1
    if "ReticulatingSplines" not in out:
        print("engine-api-drift --self-test: FAIL -- an unaccounted item was not reported.")
        fail = 1
    # * The variant rule, both directions. Only the pair is evidence: a rule
    # that reports every variant would pass the first half and be useless.
    if not any("EncryptError::RedactionPending" in k for k in unacc):
        print("engine-api-drift --self-test: FAIL -- a VARIANT whose owner and leaf")
        print("  both appear in the corpus, never together and on a different enum,")
        print("  was accounted for. That exact shape discharged a real gap on this")
        print("  gate's first run: `EncryptError::RedactionPending` fell through a")
        print("  `#[non_exhaustive]` catch-all while both its names sat in the tree.")
        fail = 1
    for ok, why in (
        ("EditSession", "it is named in this workspace's Rust"),
        ("to_svg", "its new module's path is written about in a register"),
        ("internal_helper", "it carries a reasoned exemption"),
        ("StrokeDisplay::Hairline", "it is spelled in full in the Rust corpus"),
        ("reticulate", "a new MODULE is reported as a module, never item by item"),
    ):
        if any(ok in k for k in unacc):
            print(f"engine-api-drift --self-test: FAIL -- {ok} was reported although {why}.")
            print("  A gate that reports the correct shape trains people to ignore it,")
            print("  which is worse than not having the gate.")
            fail = 1
    # * Asserted on the MARKER LINE, not on the word "MOVED": the FAIL
    # message's own explanatory prose contains "MOVED" too, so a grep for the
    # word passes against a gate that printed no pairing whatsoever. An
    # assertion satisfiable by the explanation of the assertion is not an
    # assertion -- `check-engine-backlog.sh` paid for that lesson first.
    if "-> probably MOVED, not new; the same leaf left:" not in out:
        print("engine-api-drift --self-test: FAIL -- a moved item was reported with no")
        print("  pairing to the removal it came from, so the message cannot tell a")
        print("  file move from a new capability. They are identical to a key and")
        print("  they are opposite acts.")
        fail = 1
    # * The MODULE rule, both directions, and only the pair is evidence.
    if "pdfcer_core::reticulate  (3 public item(s))" not in out:
        print("engine-api-drift --self-test: FAIL -- a whole new module with three")
        print("  items was not reported as ONE module finding with its count. The")
        print("  engine ships SUBSYSTEMS -- `pdfcer_core::sign` arrived with 98 public")
        print("  items in one pin move -- and 98 item lines about one event is not a")
        print("  finding anybody can act on.")
        fail = 1
    if any("reticulate" in k for k in unacc):
        print("engine-api-drift --self-test: FAIL -- items of a new module were ALSO")
        print("  reported individually, which is the noise the module rule removes.")
        fail = 1
    if "pdfcer_core::export  (" in out:
        print("engine-api-drift --self-test: FAIL -- a new module whose path IS written")
        print("  down in a register was reported anyway. If naming a module does not")
        print("  account for it, the only way to discharge a 101-item subsystem is to")
        print("  type 101 backticked symbols, which nobody will do and nobody should.")
        fail = 1
    # *** THE PREFIX-COLLISION REGRESSION, and it is the most important line in
    # this self-test. `pdfcer_core::sign` is a prefix of `pdfcer_core::signature`
    # and `pdfcer_core::signature_verify`, both of which the planted register
    # names. Tested as a plain substring -- which the first cut did -- the whole
    # signing subsystem printed in the ACCOUNTED list, in green, as a thing
    # somebody had looked at. Nobody had.
    if "pdfcer_core::sign  (1 public item(s))" not in out:
        print("engine-api-drift --self-test: FAIL -- a new module whose path is a")
        print("  PREFIX of a module the register does name was discharged by that")
        print("  prefix. On the real tree that exact collision hid 101 items of a")
        print("  digital-signing subsystem behind two sentences about signature")
        print("  VERIFICATION, which is a different capability entirely.")
        fail = 1
    # * And the rule must NOT reach into a module the snapshot already knows.
    # `pdfcer_core::text_edit` is named all over this project's registers; if a
    # module-level discharge applied to an EXISTING module, one such sentence
    # would swallow `RefusalKind` itself -- the exact item this gate was built
    # to catch. The anchored `sharpen` module is that case: it is old, its new
    # item is silent, and it must still be reported by name.
    if "sharpen::ReticulatingSplines" not in out:
        print("engine-api-drift --self-test: FAIL -- a new item in an EXISTING module")
        print("  was not reported by name. A module-level discharge that reached old")
        print("  modules would swallow the `RefusalKind` case this gate exists for.")
        fail = 1
    if "old_home::RefusalKind" not in out:
        print("engine-api-drift --self-test: FAIL -- the move pairing did not NAME the")
        print("  key the item left from, which is the only thing that makes it")
        print("  actionable.")
        fail = 1

    # --- a thin exemption reason must fail ---------------------------------
    thin = pathlib.Path(tempfile.mkdtemp()) / "snap.txt"
    thin.write_text(
        "# a snapshot\n"
        "exempt pdfcer-core fn pdfcer_core::a::b -- too short\n"
        "exempt pdfcer-core fn pdfcer_core::c::d\n"
        "pdfcer-core fn pdfcer_core::e::f\n",
        encoding="utf-8",
    )
    _, ex, comp = read_snapshot(thin)
    if len(comp) != 2 or ex:
        print("engine-api-drift --self-test: FAIL -- an exemption with a nine-character")
        print("  reason, and one with no reason at all, were both expected to be")
        print("  rejected. An exemption nobody had to argue for is a re-baseline with")
        print(f"  a keyword in front of it. Got {len(comp)} complaint(s), {len(ex)} exemption(s).")
        fail = 1
    buf2 = io.StringIO()
    rc2, _, _ = report(set(), set(), {}, comp, accountant, out=buf2)
    if rc2 != 1:
        print("engine-api-drift --self-test: FAIL -- a thin exemption reason did not")
        print("  fail the gate.")
        fail = 1

    # --- the derived crate list must never be silently empty ---------------
    if not engine_crates():
        print("engine-api-drift --self-test: FAIL -- no engine crate could be derived")
        print(f"  from {MANIFEST}. That is the empty-scan condition, and it must be a")
        print("  FAILURE rather than a clean run: a gate that measured nothing is not")
        print("  a gate that found nothing.")
        fail = 1

    if fail:
        return 1
    print("engine-api-drift --self-test: PASS -- the scanner reads all 13 shapes of the")
    print("  fixture and invents none of the four decoys; the comparison catches an")
    print("  unaccounted item, a variant whose two halves appear apart, an unnamed new")
    print("  MODULE (once, with its count) and a thin exemption; it names the removal")
    print("  a moved item came from; and it reports none of the five correct shapes,")
    print("  including a new item in an EXISTING module, which the module rule must")
    print("  never swallow.")
    return 0


# ===========================================================================
# THE REAL RUN
# ===========================================================================


def skip(msg: str) -> int:
    print("SKIP: " + msg)
    print()
    print("  Nothing was measured, and 'nothing measured' is not 'nothing wrong'.")
    print("  run-all.sh renders this in its own block and the whole run exits 3.")
    return 2


def main() -> int:
    ap = argparse.ArgumentParser(add_help=True, description=__doc__)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--update", action="store_true",
                    help="fold ACCOUNTED new items into the snapshot")
    ap.add_argument("--list", action="store_true",
                    help="print the live public API and stop")
    ap.add_argument("--bootstrap", metavar="REV",
                    help="write the snapshot as the API at REV, wholesale. This "
                         "is the ONE mode that absorbs items without accounting "
                         "for them, and it is only honest against a PAST "
                         "revision, where 'everything that existed then' is what "
                         "the snapshot is claiming. Never point it at the lock.")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    crates = engine_crates()
    if not crates:
        # * FAIL, not SKIP. An empty crate list means the derivation went blind,
        # and a blind derivation produces an empty scan that looks exactly like
        # a clean one. `engine_path.py`'s header is a defect report about
        # precisely this happening to check-verb-coverage during the rename.
        print(f"FAIL: no engine crate could be derived from {MANIFEST}.")
        print()
        print("  This gate reads every dependency taking a `git = \"file:///...\"` URL.")
        print("  Finding none means the manifest changed shape, and an empty crate")
        print("  list would make this gate scan nothing and print a clean PASS. On")
        print("  2026-09-03 exactly that happened to check-verb-coverage, through a")
        print("  rename rather than a code change, and it reported")
        print("  'PASS: all 0 uncalled verb(s)' having examined nothing at all.")
        return 1

    repo = engine_path.locate(ROOT)
    if repo is None or not repo.is_dir():
        return skip(
            f"the engine checkout is not on this machine.\n"
            f"      {MANIFEST.relative_to(ROOT)} names {repo or 'no file:/// dependency'},\n"
            f"      which is not a directory. The path is DERIVED from the manifest\n"
            f"      Cargo builds from, never hard-coded -- so this is a real absent\n"
            f"      engine and not a stale literal."
        )

    rev = locked_revision()
    if rev is None:
        return skip(f"{LOCK.name} pins no `git+file://` revision, so there is no\n"
                    f"      engine revision to measure.")

    # The one mode that absorbs items wholesale, and it runs before the
    # snapshot-exists check because its whole job is to create one. It is only
    # honest against a PAST revision: pointed at the lock it would write down
    # today's engine as "already looked at", which is the re-baseline this gate
    # is built to make impossible, so it refuses that.
    if args.bootstrap:
        if args.bootstrap.startswith(rev[:7]) or rev.startswith(args.bootstrap):
            print("FAIL: --bootstrap was pointed at the LOCKED revision.")
            print()
            print("  Bootstrapping at the lock writes today's engine down as 'already")
            print("  looked at', which absorbs every unconsumed item in one command --")
            print("  the exact re-baseline this gate exists to make impossible. Point")
            print("  it at the revision this project last reconciled against instead;")
            print(f"  `git log -p -- {LOCK.name}` in this repo names every prior pin.")
            return 1
        try:
            boot = api_at(repo, args.bootstrap, crates)
        except LookupError as exc:
            return skip(f"git could not produce the engine tree at "
                        f"{args.bootstrap}:\n      {exc}")
        items = flatten(boot)
        if not items or any(not v for v in boot.values()):
            print(f"FAIL: the bootstrap scan at {args.bootstrap} returned nothing for at")
            print("      least one crate, so there is nothing honest to write down.")
            return 1
        _, keep_exempt, _ = read_snapshot(SNAPSHOT)
        write_snapshot(SNAPSHOT, args.bootstrap, crates, items, keep_exempt)
        print(f"--bootstrap: wrote {len(items)} item(s) to "
              f"{rel(SNAPSHOT)} at baseline {args.bootstrap}.")
        print("  Everything the engine has gained SINCE that revision is now the")
        print("  gate's finding. Run it.")
        return 0

    if not SNAPSHOT.is_file():
        return skip(f"{rel(SNAPSHOT)} is missing. It is this gate's\n"
                    f"      baseline; without it every one of the engine's several\n"
                    f"      thousand public items is 'new' and the gate has no opinion\n"
                    f"      worth printing. Create it with --bootstrap REV, pointed at\n"
                    f"      the engine revision this project last reconciled against.")

    try:
        api = api_at(repo, rev, crates)
    except LookupError as exc:
        return skip(f"git could not produce the engine tree at {rev[:7]}:\n"
                    f"      {exc}\n"
                    f"      A revision absent from the clone is a real state on a fresh\n"
                    f"      machine and says nothing about the API.")

    live = flatten(api)
    seen, exempt, complaints = read_snapshot(SNAPSHOT)

    if args.list:
        for key in sorted(live):
            print(key)
        return 0

    # ** THE EMPTY-SCAN GUARD. A resolvable engine that yields nothing means
    # the scanner went blind -- a moved `src/`, a changed layout, a regex that
    # stopped matching. Every one of those prints exactly what a clean run
    # prints, which is why this is a FAIL and never a SKIP.
    empty = [c for c, items in api.items() if not items]
    if empty:
        print(f"FAIL: the scan of {', '.join(empty)} at {rev[:7]} returned NOTHING.")
        print()
        print("  A crate with zero public items is not a crate with nothing new; it")
        print("  is a scan that went blind. The likely causes are a moved `src/`, a")
        print("  crate renamed in the manifest but not on disk, or this file's own")
        print("  patterns no longer matching the engine's formatting.")
        print()
        print("  This is reported as a FAILURE rather than as an empty result,")
        print("  because a check that cannot fail is not evidence.")
        return 1
    if seen and len(live) < len(seen) * COLLAPSE_RATIO:
        print(f"FAIL: the scan found {len(live)} public item(s) where the snapshot holds")
        print(f"      {len(seen)}. That is a collapse, not a deletion.")
        print()
        print("  The engine has never removed more than a handful of public items in")
        print("  one pin move. Losing half of them means the scanner stopped reading")
        print("  something -- and a scanner that reads less produces a smaller `added`")
        print("  set and a cheerful PASS, which is the failure this gate must never")
        print("  have. Run --list and compare against the snapshot before believing")
        print("  any part of this.")
        return 1

    rust, md = repo_corpus()
    accountant = make_accountant(rust, md)

    print(f"engine {repo} at {rev[:7]} (the revision {LOCK.name} pins)")
    print(f"crates {' '.join(crates)}  (derived from {MANIFEST.relative_to(ROOT)})")
    rc, unaccounted, foldable = report(live, seen, exempt, complaints, accountant)

    # * COMING -- the other half of the truth, reported and never failed on.
    # An item the engine's HEAD carries and the lock does not CANNOT be called
    # from here; reddening this build for it would be reddening it for a change
    # nobody in this repository made. But staying silent about it is how a
    # whole shipped subsystem goes unnoticed, so it is printed loudly.
    try:
        head = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "HEAD"],
            capture_output=True, check=True,
        ).stdout.decode().strip()
    except (OSError, subprocess.CalledProcessError):
        head = ""
    if head and not head.startswith(rev[:7]):
        try:
            ahead = flatten(api_at(repo, head, crates)) - live
        except LookupError:
            ahead = set()
        if ahead:
            print()
            print(f"COMING ({len(ahead)}): public item(s) the engine's HEAD ({head[:7]}) has and")
            print(f"      {LOCK.name} does not, so they are NOT callable from here yet and")
            print("      are NOT a failure. This is the `cargo update` waiting to happen.")
            unnamed = sorted(k for k in ahead if accountant(k) is None)
            print(f"      {len(ahead) - len(unnamed)} of them are already named or written about here;"
                  f" {len(unnamed)} are not.")
            for key in unnamed[:12]:
                print(f"        {key}")
            if len(unnamed) > 12:
                print(f"        ... and {len(unnamed) - 12} more")

    if args.update:
        # * `foldable` comes from `report`, not from a second pass over the
        # accounting rule -- see its docstring. An EXEMPTED item is not in it,
        # deliberately: folding one would make it "seen", its exemption line
        # would attach to nothing, and the decision to silence it would stop
        # being printed on every run, which is precisely how a temporary
        # exemption becomes permanent by nobody looking at it.
        keep = foldable
        baseline = rev
        write_snapshot(SNAPSHOT, baseline, crates, keep, exempt)
        left = sorted(live - keep - set(exempt))
        print()
        print(f"--update: wrote {len(keep)} item(s) to "
              f"{rel(SNAPSHOT)} at baseline {baseline[:7]}.")
        if left:
            print(f"          {len(left)} unaccounted item(s) were deliberately NOT folded")
            print("          in, so this gate stays red until each is consumed, written")
            print("          about, or exempted with a reason. There is no command that")
            print("          makes a finding disappear.")
        return 0

    if rc == 0:
        print()
        print(f"PASS: every public item the engine has gained since the snapshot's")
        print(f"      baseline is accounted for.")
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
