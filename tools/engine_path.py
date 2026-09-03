"""engine_path.py — where the engine actually is, derived rather than assumed.

★★★ WHY THIS EXISTS, AND IT IS A DEFECT REPORT
==============================================

Three instruments in this repository read `pdfcer-core`'s source directly —
`verb-coverage.py`, `security-coverage.py` and `gates/check-shipped-assets.py`.
Every one of them held the engine's location as a **hard-coded literal**.

On 2026-09-03 this project renamed to `pdfcer` before the engine did. The rename
rewrote those literals from `D:/Dev/pdfce` to `D:/Dev/pdfcer` — a directory that
does not exist yet — and the consequence was silent:

    $ bash tools/gates/check-verb-coverage.sh
    engine not found at D:\\Dev\\pdfcer\\crates\\pdfcer-core\\src\\edit.rs
    PASS: all 0 uncalled verb(s) are named in EDITABLE_SURFACES.md.

**A gate reported PASS having examined nothing.** It is the exact failure this
project has a standing rule about — *a check that cannot fail is not evidence* —
and it arrived through a rename rather than through a code change, which is why
no test caught it.

THE FIX IS TO STOP ASSUMING
===========================

The manifest already states where the engine is: `crates/pdfcer-gui/Cargo.toml`
carries the git URL Cargo itself resolves. Reading it there means:

* the answer is **the same one the compiler used**, not a second belief about it;
* it follows the temporary `package = ...` shim automatically, so the day the
  engine's rename lands and those lines change, every instrument follows without
  anybody remembering they exist;
* there is **one** claimant for "where is the engine", which is the rule this
  project has paid for twice (`text_edit_focused` cost the Delete key and then
  the space bar because two places each had their own idea of one question).

⇒ And [`require`] refuses rather than returning a default, because the whole
lesson above is that a missing engine must be loud.
"""

from __future__ import annotations

import pathlib
import re

#: The manifest that names the engine dependency, relative to the repo root.
MANIFEST = pathlib.Path("crates/pdfcer-gui/Cargo.toml")

#: A `file:///D:/Dev/…` git URL on a dependency line.
#:
#: Matched on the URL rather than on the dependency's name, because the name is
#: exactly what the rename changes and the URL is what Cargo resolves. Anchoring
#: on the volatile half is how an instrument comes to describe a world that has
#: moved.
_URL = re.compile(r'git\s*=\s*"file:///([^"]+)"')


def locate(root: pathlib.Path | None = None) -> pathlib.Path | None:
    """Where the engine is, according to the manifest Cargo builds from.

    `None` when the manifest cannot be read or names no `file:///` dependency —
    which is a real state on a checkout that has re-pointed at a published
    crate, and is reported rather than guessed at.
    """
    base = root or pathlib.Path(".")
    manifest = base / MANIFEST
    if not manifest.is_file():
        return None
    for line in manifest.read_text(encoding="utf-8", errors="replace").splitlines():
        stripped = line.lstrip()
        # Skip comments: the shim's own explanation quotes URLs, and an
        # instrument that read a comment would follow documentation instead of
        # the build.
        if stripped.startswith("#"):
            continue
        m = _URL.search(line)
        if m:
            return pathlib.Path(m.group(1))
    return None


def crate_name(kind: str = "core") -> str:
    """What the engine crate is CALLED in the revision we build against.

    Under the temporary shim the manifest says
    `pdfcer-core = { package = "pdfce-core", … }` — the local name is the new
    one and the real package is still the old one. An instrument reading the
    engine's *source directory* needs the real one, because that is what the
    folder on disk is called.

    Returns the new name when there is no shim, which is the destination state.
    """
    manifest = pathlib.Path(MANIFEST)
    if manifest.is_file():
        pattern = re.compile(
            rf'^\s*pdfcer-{kind}\s*=.*package\s*=\s*"([^"]+)"'
        )
        for line in manifest.read_text(encoding="utf-8", errors="replace").splitlines():
            m = pattern.match(line)
            if m:
                return m.group(1)
    return f"pdfcer-{kind}"


def require(what: str) -> pathlib.Path:
    """[`locate`], or a loud exit.

    ★ Never a default. `what` names the caller so the message says which
    instrument went blind, because the whole reason this module exists is a gate
    that reported PASS with nothing under it.
    """
    found = locate()
    if found is not None and found.is_dir():
        return found
    raise SystemExit(
        f"{what}: FAIL — cannot find the engine.\n"
        f"  {MANIFEST} was read for a `git = \"file:///…\"` dependency and the\n"
        f"  answer was {found or 'nothing'}, which is not a directory.\n"
        f"\n"
        f"  This is reported as a FAILURE rather than as an empty result. On\n"
        f"  2026-09-03 the same condition made check-verb-coverage print\n"
        f"  'PASS: all 0 uncalled verb(s)' having examined nothing at all, and\n"
        f"  a check that cannot fail is not evidence."
    )
