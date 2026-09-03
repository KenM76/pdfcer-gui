#!/usr/bin/env python3
"""security-coverage.py — every ENCRYPTION and SIGNATURE thing `pdfcer-core`
exposes, and whether this shell reaches it.

WHY THIS EXISTS
===============

The operator, 2026-09-03 (`OPERATOR_REQUESTS.md` O108):

    "can we get all of the encryption and signature features that have been
     implemented in the engine under one new tab in the ribbon?"

★★★ **"All of the features that have been implemented in the engine" is a
completeness claim, and this project has a standing rule about those: a
completeness question needs an INSTRUMENT, not a document.** `FEATURES.md`
describes what the GUI does. `NO_SURFACE.md` lists tunables with no control.
`GUI_ROADMAP.md` is a plan. **None of the three is keyed on `pdfcer-core`'s
API**, so none of them can answer *"is there an encryption or signature thing
the engine implements that nothing here reaches?"* — and answering it from them
would be answering it from our own imagination of the engine.

`tools/verb-coverage.py` is the same instrument for `EditSession`'s verbs and
cannot help here: **almost none of this surface is on `EditSession`.** Encryption
lives on `Document` and in `pdfcer_core::crypto`; signatures live in
`pdfcer_core::signature` as free functions and in two `EditSession` methods. A
scan of `edit.rs` sees two of the twenty-odd items below.

WHAT IT MEASURES, AND WHAT THAT MEASUREMENT IS WORTH
====================================================

For every public item in `pdfcer-core` whose name or module places it in the
encryption / signature surface, whether the identifier appears anywhere in
`crates/pdfcer-gui/src`.

★★ It is a **grep**, and the limits are the same as `verb-coverage.py`'s and
worth restating because a number from a tool reads as authoritative:

  - A **hit** means the NAME appears, not that a reachable operator route
    reaches it. A read behind a condition nothing sets is a hit here and dead in
    the running program. `tools/ui-verify` is the only instrument for that.
  - A **miss** is stronger: no occurrence of the identifier means nothing here
    names it, full stop. **The miss list is the useful output.**
  - A miss is not automatically a gap. Several of these are internals a front
    end has no business calling — `crypto::aes::decrypt_cbc_256` is how the
    engine decrypts, not something a GUI invokes — and the table below marks
    those, because a register that listed them as gaps would be crying wolf.

★★★ IT MEASURES THE LOCKED REVISION, NOT THE ENGINE'S WORKING TREE
==================================================================

Same discipline, same reason, and the reason was paid for again on the morning
this file was written: `D:/Dev/pdfcer` is a **dirty tree by design** — that
session runs in parallel and answers requests within the hour — so a capability
in the worktree and not in `Cargo.lock` is one this shell **cannot call**. On
2026-09-03 a paragraph was written from the worktree describing a field that did
not exist on our pin, and the compiler was the only thing that caught it.

So the scan reads `git show <locked-rev>:<file>` and prints where each answer
came from. A read that silently fell back to the worktree would be the failure
this paragraph describes, wearing a tool's authority.

★★★ THE HEADLINE FINDING, 2026-09-03, and it is the shape of the answer
=======================================================================

Run it and the summary says it: **this surface is entirely READ-SIDE.**
`pdfcer-core` has no `encrypt_document`, no `set_password`, no
`remove_encryption`, no `set_permissions`, no `sign_document`, no certificate
validation and no timestamping. What it has is the ability to *open* an
encrypted document, *report* its scheme and permission bits, and *count and
describe* the signatures a document already carries.

⇒ That is not a criticism of the engine and it is not a reason to withhold the
tab. It is the fact the tab has to be built around, and stating it is what keeps
the tab from being a row of controls that cannot work — which is R9's
placeholder rule at the scale of a whole surface.

USAGE
=====

    python tools/security-coverage.py               # the table
    python tools/security-coverage.py --misses      # just what nothing reaches

Exit code is 0 always: this is an instrument, not a gate.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys

# ★★★ DERIVED, NOT ASSUMED — see `tools/engine_path.py`.
#
# This was a hard-coded literal until 2026-09-03, when the project's rename
# pointed it at a directory the engine had not created yet and this instrument
# went silently blind. `engine_path.locate` reads the git URL out of the
# manifest Cargo actually builds from, so it follows the temporary
# `package = ...` shim and will follow the engine's rename without anybody
# remembering that this line exists.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import engine_path  # noqa: E402

ENGINE_REPO = engine_path.locate() or pathlib.Path("D:/Dev/pdfcer")
LOCK = pathlib.Path("Cargo.lock")
GUI = pathlib.Path("crates/pdfcer-gui/src")

#: The engine files this surface lives in.
#:
#: Listed rather than globbed, because "every file mentioning encryption" sweeps
#: in the parser's own decryption hooks and the answer stops being readable. If
#: the engine grows a new module for this, ADD IT HERE — and the fact that this
#: is a hand-written list is itself a known weakness, the same one recorded
#: against `panels::tool`'s region test: a new module is invisible to the check
#: built to find it, and the count still adds up. Re-derive it when the engine's
#: layout moves.
FILES = [
    f"crates/{engine_path.crate_name('core')}/src/signature.rs",
    f"crates/{engine_path.crate_name('core')}/src/crypto/mod.rs",
    f"crates/{engine_path.crate_name('core')}/src/crypto/standard.rs",
    f"crates/{engine_path.crate_name('core')}/src/crypto/apply.rs",
    f"crates/{engine_path.crate_name('core')}/src/crypto/r5.rs",
    f"crates/{engine_path.crate_name('core')}/src/document.rs",
]

#: Names in `document.rs` that belong to this surface. That file is the whole
#: reading entry point, so an unfiltered scan of it would report the entire
#: public API as "encryption".
DOCUMENT_ALLOW = re.compile(
    r"password|encrypt|decrypt|permission|Perms|auth", re.IGNORECASE
)

#: Items a front end has no business calling — the engine's own primitives.
#:
#: Marked rather than dropped, because a reader asking "is that everything?"
#: deserves to see them accounted for. A tool that silently filters is a tool
#: whose total cannot be checked.
INTERNAL = re.compile(
    r"^(decrypt_cbc|decrypt_ecb|decrypt_strings?|decrypt_stream|pad_password|"
    r"file_key_from_|authenticates_as_user|skip|object_key)$"
)

#: Names too generic for a grep to attribute.
#:
#: The first run of this tool reported `new`, `all`, `parse`, `position`,
#: `permissions`, `granted` and `as_bytes` as REACHED, and every one of those was
#: a match against unrelated GUI code -- `Vec::new`, a `parse` of a number, a
#: form field's `permissions`. A "26 reached" summary built on that is worse than
#: no summary: it is a completeness claim resting on coincidence, which is
#: precisely what this instrument was written to replace.
#:
#: They get a category of their own. NOT silently dropped -- a reader asking "is
#: that everything?" must see them accounted for -- and not counted as reached,
#: because the tool cannot tell.
AMBIGUOUS = re.compile(
    r"^(new|all|parse|position|permissions|granted|as_bytes|applies_at|"
    r"check_perms|annotate|assemble)$"
)

#: A line that is only a comment. Stripped before searching the GUI.
#:
#: Because a doc comment MENTIONING a verb is not a call to it, and the
#: difference is the whole point of the measurement. `load_with_password`
#: appears in `app::blank`'s module header, in a sentence listing the four
#: loading entry points, and appears nowhere else in the crate. Counting that as
#: "reached" would report the one capability the operator most needs -- opening
#: an encrypted document -- as already built.
COMMENT_ONLY = re.compile(r"^\s*(//|/\*|\*)")

PUB_ITEM = re.compile(
    r"^\s*pub (?:const |unsafe |async )*(?:fn|struct|enum|trait) ([A-Za-z0-9_]+)"
)


def locked_revision() -> str | None:
    """The `pdfcer-core` commit `Cargo.lock` pins. See `verb-coverage.py`."""
    if not LOCK.exists():
        return None
    for line in LOCK.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith('source = "git+file:') and "#" in line:
            return line.rsplit("#", 1)[1].rstrip('"')
    return None


def engine_source(rev: str | None, path: str) -> tuple[str, str]:
    """`path` as of `rev`, and where the answer came from.

    Falls back to the working tree and SAYS SO, because a silent fallback is the
    exact failure this function exists to prevent.
    """
    if rev:
        try:
            out = subprocess.run(
                ["git", "-C", str(ENGINE_REPO), "show", f"{rev}:{path}"],
                capture_output=True,
                check=True,
            )
            return out.stdout.decode("utf-8", errors="replace"), f"lock {rev[:7]}"
        except (OSError, subprocess.CalledProcessError):
            pass
    p = ENGINE_REPO / path
    if not p.exists():
        return "", "ABSENT"
    return p.read_text(encoding="utf-8", errors="replace"), "WORKTREE"


def items_in(text: str, path: str) -> list[str]:
    """Every public item name in `text`, filtered for `document.rs`."""
    out: list[str] = []
    filtered = path.endswith("document.rs")
    for line in text.splitlines():
        m = PUB_ITEM.match(line)
        if not m:
            continue
        name = m.group(1)
        if filtered and not DOCUMENT_ALLOW.search(name):
            continue
        if name not in out:
            out.append(name)
    return out


def gui_text() -> str:
    """Every `.rs` under the GUI crate, concatenated, with comment-only lines
    removed -- see `COMMENT_ONLY`."""
    parts: list[str] = []
    for p in sorted(GUI.rglob("*.rs")):
        text = p.read_text(encoding="utf-8", errors="replace")
        parts.append(
            "\n".join(l for l in text.splitlines() if not COMMENT_ONLY.match(l))
        )
    return "\n".join(parts)


def is_reached(name: str, gui: str) -> bool:
    """Does the GUI USE `name`, as opposed to merely containing the characters?

    A CamelCase item is a type and its bare name is distinctive. A snake_case
    item is a function, and a bare-word search for one matches a local variable
    of the same name -- so those require call or path syntax.
    """
    if name[:1].isupper():
        return re.search(rf"\b{re.escape(name)}\b", gui) is not None
    return re.search(rf"(?:\.|::)\s*{re.escape(name)}\s*\(", gui) is not None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--misses", action="store_true", help="only what nothing reaches")
    args = ap.parse_args()

    rev = locked_revision()
    gui = gui_text()

    rows: list[tuple[str, str, str]] = []  # file, name, kind
    sources: set[str] = set()
    for path in FILES:
        text, where = engine_source(rev, path)
        sources.add(where)
        for name in items_in(text, path):
            if INTERNAL.match(name):
                kind = "internal"
            elif AMBIGUOUS.match(name):
                kind = "ambiguous"
            elif is_reached(name, gui):
                kind = "reached"
            else:
                kind = "NOT REACHED"
            rows.append((path.rsplit("/", 1)[1], name, kind))

    if args.misses:
        for _f, name, kind in rows:
            if kind == "NOT REACHED":
                print(name)
        return 0

    print(f"pdfcer-core encryption + signature surface (lock {rev[:7] if rev else '?'})")
    if "WORKTREE" in sources:
        print("  ⚠ at least one file was read from the WORKTREE, not the lock — "
              "an item below may not exist in the revision this shell builds")
    print()
    width = max(len(n) for _f, n, _k in rows) if rows else 10
    for f, name, kind in rows:
        print(f"  {name:<{width}}  {kind:<12}  {f}")

    total = len(rows)
    counts = {k: sum(1 for *_x, kk in rows if kk == k) for k in
              ("reached", "NOT REACHED", "internal", "ambiguous")}
    print()
    print(f"  {total} public item(s): {counts['reached']} reached, "
          f"{counts['NOT REACHED']} NOT reached, "
          f"{counts['internal']} engine-internal, "
          f"{counts['ambiguous']} too generic to attribute")
    return 0


if __name__ == "__main__":
    sys.exit(main())
