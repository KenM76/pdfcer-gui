#!/usr/bin/env python3
"""check-unreachable-refusals.py — A SENTENCE KEPT FOR A DEAD ENGINE SYMBOL
MUST BE WATCHED BY AN INSTRUMENT, NOT BY A READER.

===========================================================================
THE PROPERTY ASSERTED
===========================================================================

Every paragraph in this shell that says an engine error "cannot happen" is
still true of the engine revision this build is pinned to.

===========================================================================
★★★ WHY THIS GATE EXISTS
===========================================================================

This shell keeps refusal sentences alive after the engine stops producing the
error behind them. That is usually the *right* call and is not what this gate
argues against:

  * `ReflowDecline` is an exhaustive enum and the `match` over it is
    compiler-proved complete, so the arm is **mandatory** whether or not
    anything reaches it;
  * `unreachable!()` in its place would turn a future engine reinstating the
    guard into a **crash on a refusal path**, which is the wrong direction;
  * and the remedy sentence (*"save this file and open it again"*) took two
    corrections to get right. Deleting it because it is briefly unreachable is
    how it would be lost a third time.

⇒ So the sentences stay. **The defect is that nobody can tell they are dead.**

### The shapes this has taken, none of which a reader caught in time

* **A shell paragraph arguing from a shell forecast that had already been
  deleted.** It read as measured because it compiled. Nothing in a doc comment
  is checked against the code beside it.
* **A variant that became unreachable at an engine pass** and was found months
  later, by a reader looking for something else entirely.
* **The engine deleting the sole producer of a variant and SAYING SO in its own
  commit message.** Without an instrument, noticing that depends on somebody
  reading a commit message in a repository this one is forbidden to write to —
  which is to say, on luck.

Each time the stale sentence was quoted in reasoning about what the program
does. ★★ A retained sentence with no tripwire is not documentation — it is a
claim about an engine that has moved.

===========================================================================
WHAT IT ASSERTS
===========================================================================

For every marker of the form

    UNREACHABLE-FROM: <engine::path::To::Symbol> @ <pin>

found anywhere in `crates/pdfcer-gui/{src,tests}/**/*.rs`, the gate re-measures
the named symbol **in the engine source at the revision `Cargo.lock` pins** and
fails when the engine's use of it has changed at all since the baseline
recorded in `tools/gates/unreachable-refusals.txt`.

  * **Strong**: the day the engine gains a line mentioning that symbol — a
    constructor, a new producer, a move to another module, a deletion — this
    build goes red in the same commit that moves the pin, which is the only
    moment at which the claim can be re-measured cheaply and the only moment
    anybody is thinking about it.
  * **Weak, deliberately**: it does not try to decide whether a new line is a
    *constructor*. Classifying Rust expression-vs-pattern position with a
    regular expression is exactly the kind of clever heuristic that produces a
    gate which is confidently wrong — `matches!(e, X::Y)`, `Err(X::Y) =>`,
    `X::Y | X::Z if …` and a doctest that constructs the variant in prose all
    look alike from outside a parser. **The gate reports the diff and a human
    reads it.** That is a smaller claim and it is one this file can honour.

### Why a snapshot rather than a classifier

The same argument `check-engine-api-drift.py` makes: an instrument that judges
is an instrument that can be fooled quietly, and one that *diffs* cannot. The
failure message is therefore never "you have a bug" — it is **"the engine's use
of a symbol you have written a paragraph about has changed; go read the
paragraph."**

===========================================================================
★★★ WHAT IS AND IS NOT IN SCOPE
===========================================================================

**In scope:** a shell sentence whose truth depends on an ENGINE symbol having
no producer. That is the `UNREACHABLE-FROM:` marker, and the engine is the
side of the boundary this repository cannot see changing.

**NOT in scope:** a variant that is unreachable because of something in THIS
crate. `ReflowRefusal::PageSetChanged` is the live example — nothing maps to it
because no arm of `reflow_refusal` produces it, which is a fact about a file
twelve lines long that a unit test can assert directly and does
(`app::actions::textstyle::tests`). A gate is the wrong instrument for a
question a `cargo test` can answer, and claiming it covers that case would be
an unevidenced excuse, which is worse than silence.

===========================================================================
COMMENT STRIPPING, AND WHY IT IS THE WHOLE DESIGN
===========================================================================

The engine documents its dead variants heavily, and the documentation outweighs
the code by a wide margin. At the pinned revision, `reflow_apply.rs` mentions
`PageEditedThisSession` on five lines: **three are `///` doc comments — one of
them a doctest that CONSTRUCTS the variant — and only two are code**, the
variant declaration and one match arm. Its own doc comment runs past forty
lines.

If the gate counted raw grep hits it would:

  * go red every time the engine rewords a paragraph (noise, and noise is how
    a gate gets restamped without being read), and
  * count that doctest's `let e = ReflowApplyError::PageEditedThisSession;` as
    a producer, which it is not — a doc example is not a path an operator
    reaches.

⇒ So a line contributes to the baseline only if the symbol survives **removal
of `//`-comments**, string-literal-aware so that a `"file:///…"` in real code
is not mistaken for the start of a comment. The whole baseline is currently 5
code lines across 2 symbols, which is the number a clean run prints and the
cheapest possible check that the stripping has not started eating real code.

===========================================================================
WHAT IT PROVABLY CANNOT SEE
===========================================================================

  * **Whether a new line is a producer.** By design — see "Weak, deliberately"
    above. A drift is reported, never diagnosed.
  * **An engine change that does not touch a line mentioning the symbol.** A
    producer added through a `From` impl, a type alias, or a helper that
    returns the variant under another name moves nothing this gate watches.
  * **`/* … */` block comments**, which are not stripped. The engine uses none;
    a gate that pretends to parse more than it does is worse than one that says
    what it skips.
  * **A `//` inside a raw string** (`r"…"`, `r#"…"#`). Raw strings are treated
    as ordinary ones for the purpose of finding a comment start, so such a line
    would be mis-stripped. No engine line carrying a watched symbol contains
    one.
  * **A symbol reachable only through a macro that pastes its name**, as it is
    invisible to every other instrument in this directory.
  * **Whether the shell paragraph is well written.** The gate watches the
    engine; the sentence itself is a human's problem.

===========================================================================
EXIT CODES — the project's three-state gate contract
===========================================================================

    0   PASS      every marker re-measured and unchanged
    1   FAIL      a marker drifted, is unrecorded, or its symbol is gone
    2   SKIPPED   the engine, git, or the pinned revision is unavailable

A SKIP is never silent and never claims its own skip is expected: the runner
turns any skip into a non-zero overall result, because a check that stopped
running is indistinguishable from a check that passed unless somebody says so.

===========================================================================
USAGE
===========================================================================

    python tools/gates/check-unreachable-refusals.py
    python tools/gates/check-unreachable-refusals.py --self-test
    python tools/gates/check-unreachable-refusals.py --update   # restamp

`--update` rewrites the baseline from the engine as currently pinned. It is
the deliberate act a human performs **after** reading the diff and correcting
the paragraph — never a thing to run to make a red build green.

===========================================================================
HOW TO FALSIFY IT
===========================================================================

`--self-test` exercises the pure helpers on synthetic input and touches
neither the engine, git, nor the real baseline — a self-test that needed a
checkout would be skipped on the machine where it matters. It asserts, in both
directions:

  * `strip_comment` removes a trailing and a doc comment, and does NOT remove
    a `//` inside a string, a URL, an escaped quote, a lifetime or a char
    literal;
  * `normalise` erases indentation and wrapping but not code;
  * a doc comment does not become a site, while a planted constructor in a new
    file IS reported as arrived and nothing is falsely reported gone;
  * a vanished symbol reports every baseline line as gone;
  * ★ **a pure reword of the engine's prose is INVISIBLE** — the assertion
    that keeps this gate from becoming noise, and noise is how a baseline gets
    restamped unread;
  * the marker regex parses a qualified path and REJECTS a bare word;
  * the baseline round-trips through render and parse.
"""

from __future__ import annotations

import argparse
import io
import pathlib
import re
import subprocess
import sys
import tarfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import engine_path  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]
LOCK = ROOT / "Cargo.lock"
BASELINE = ROOT / "tools" / "gates" / "unreachable-refusals.txt"

#: Where markers may live. `tests/` is included because the probe files carry
#: the same kind of claim and are read by the same people.
MARKER_ROOTS = (
    ROOT / "crates" / "pdfcer-gui" / "src",
    ROOT / "crates" / "pdfcer-gui" / "tests",
    # The floor crate carries no refusal marker today. It is listed anyway,
    # because an unlisted root is not reported as unscanned: the gate finds no
    # marker there and passes.
    ROOT / "crates" / "pdfcer-gui-base" / "src",
)

#: `UNREACHABLE-FROM: a::b::C::D @ 025d703d`
#:
#: The pin is captured but is NOT compared against the lock. It records the
#: revision at which the claim was FIRST measured, which is history and stays
#: true; the gate re-measures at whatever the lock says today and reports which
#: revision it used. Failing on a pin mismatch alone would redden the build for
#: a claim that is still correct, and a gate that cries on every pin bump is a
#: gate that gets restamped without being read.
MARKER = re.compile(
    r"UNREACHABLE-FROM:\s+"
    r"(?P<symbol>[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+)"
    r"\s+@\s+(?P<pin>[0-9a-fA-F]{7,40})"
)


# ===========================================================================
# PURE HELPERS — everything the self-test can falsify without an engine
# ===========================================================================


def strip_comment(line: str) -> str:
    """`line` with any `//` comment removed, string-literal aware.

    The naive `line.split("//")[0]` is wrong on exactly the line this project
    has most of: `git = "file:///D:/Dev/pdfcer"`. It is also wrong on
    `let sep = "//";`. Neither carries a watched symbol today, and a helper
    that is right only for today's corpus is a helper that fails the week the
    corpus moves.

    Escapes are honoured so `"a\\"//b"` does not end the string early. Block
    comments are NOT handled — see the module header.
    """
    out: list[str] = []
    in_str = False
    quote = ""
    i = 0
    n = len(line)
    while i < n:
        ch = line[i]
        if in_str:
            if ch == "\\":
                out.append(line[i : i + 2])
                i += 2
                continue
            if ch == quote:
                in_str = False
            out.append(ch)
            i += 1
            continue
        if ch in ('"', "'"):
            # A lifetime (`'a`) is not a string. Treat `'` as a quote only
            # when it looks like a char literal: `'x'` or `'\n'`.
            if ch == "'" and not re.match(r"'(\\.|[^\\'])'", line[i:]):
                out.append(ch)
                i += 1
                continue
            in_str = True
            quote = ch
            out.append(ch)
            i += 1
            continue
        if ch == "/" and i + 1 < n and line[i + 1] == "/":
            break
        out.append(ch)
        i += 1
    return "".join(out)


def normalise(line: str) -> str:
    """One code line reduced to a form indentation and wrapping cannot change."""
    return " ".join(strip_comment(line).split())


def sites_for(files: dict[str, str], name: str) -> list[str]:
    """Every CODE line in `files` that names `name`, as `path | line`.

    `files` maps an archive-relative path to that file's text, so the same
    function serves the real engine and the self-test's synthetic tree. The
    result is sorted, because a set rendered in iteration order is a diff that
    changes for no reason.
    """
    word = re.compile(r"\b" + re.escape(name) + r"\b")
    out: list[str] = []
    for path in sorted(files):
        for raw in files[path].splitlines():
            code = normalise(raw)
            if code and word.search(code):
                out.append(f"{path} | {code}")
    return out


def parse_baseline(text: str) -> dict[str, dict]:
    """The baseline file as `{symbol: {"measured-at": str, "sites": [str]}}`."""
    out: dict[str, dict] = {}
    current: str | None = None
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("symbol "):
            current = line[len("symbol ") :].strip()
            out[current] = {"measured-at": "", "sites": []}
        elif line.startswith("measured-at ") and current:
            out[current]["measured-at"] = line[len("measured-at ") :].strip()
        elif line.startswith("site ") and current:
            out[current]["sites"].append(line[len("site ") :].strip())
    return out


BASELINE_HEADER = """\
# unreachable-refusals.txt — the engine's use of every symbol this shell has
# written a "this cannot happen" paragraph about.
#
# ★★★ THIS FILE IS NOT A LIST OF THINGS TO FIX. It is a photograph, taken at
# a named engine revision, of every CODE line in the engine that mentions a
# symbol some sentence in this shell depends on being DEAD. When the
# photograph and the engine disagree, `check-unreachable-refusals` fails and
# a human reads the paragraph the marker sits in.
#
# Written by `tools/gates/check-unreachable-refusals.py --update`, which is a
# deliberate act performed AFTER reading the diff and correcting the prose —
# never to make a red build green. Comments are stripped before a line is
# recorded, so rewording the engine's documentation cannot move this file;
# only its code can.
#
# Format:
#   symbol <fully::qualified::Path>
#     measured-at <engine revision>
#     site <archive-relative path> | <normalised code line>
"""


def render_baseline(entries: dict[str, dict]) -> str:
    """`entries` as the on-disk baseline text."""
    parts = [BASELINE_HEADER]
    for symbol in sorted(entries):
        e = entries[symbol]
        parts.append(f"\nsymbol {symbol}\n")
        parts.append(f"  measured-at {e['measured-at']}\n")
        for site in e["sites"]:
            parts.append(f"  site {site}\n")
    return "".join(parts)


def diff(expected: list[str], actual: list[str]) -> tuple[list[str], list[str]]:
    """`(gone, arrived)` — lines the baseline had and the engine no longer
    does, and lines the engine has that the baseline did not."""
    e, a = set(expected), set(actual)
    return sorted(e - a), sorted(a - e)


# ===========================================================================
# READING THE TWO SIDES
# ===========================================================================


def markers() -> dict[str, list[tuple[str, str]]]:
    """`{symbol: [(file, pin), …]}` for every marker in the shell's sources."""
    found: dict[str, list[tuple[str, str]]] = {}
    for root in MARKER_ROOTS:
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*.rs")):
            text = path.read_text(encoding="utf-8", errors="replace")
            for m in MARKER.finditer(text):
                rel = path.relative_to(ROOT).as_posix()
                found.setdefault(m.group("symbol"), []).append((rel, m.group("pin")))
    return found


def engine_crates() -> list[str]:
    """Every engine crate the shell takes, derived from the manifest.

    Deliberately the same derivation as `check-engine-api-drift.py`: matched on
    the `git = "file:///…"` marker rather than on a `pdfcer-` prefix, because
    the prefix is what a rename changes and the URL is what Cargo resolves.
    """
    manifest = ROOT / engine_path.MANIFEST
    if not manifest.is_file():
        return []
    names: list[str] = []
    for line in manifest.read_text(encoding="utf-8", errors="replace").splitlines():
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


def engine_sources(repo: pathlib.Path, rev: str, crates: list[str]) -> dict[str, str]:
    """`{archive-relative path: text}` for every `.rs` file at `rev`.

    One `git archive` for the whole set. `LookupError` when git will not
    produce the tree, which the caller turns into a SKIP: a revision genuinely
    absent from the clone is a real state on a fresh machine and is not
    evidence about anything.
    """
    paths = [f"crates/{c}/src" for c in crates]
    try:
        p = subprocess.run(
            ["git", "-C", str(repo), "archive", rev, "--"] + paths,
            capture_output=True,
        )
    except OSError as exc:
        raise LookupError(f"git could not be run: {exc}") from exc
    if p.returncode != 0:
        raise LookupError(p.stderr.decode("utf-8", errors="replace").strip())

    out: dict[str, str] = {}
    with tarfile.open(fileobj=io.BytesIO(p.stdout)) as tf:
        for member in tf:
            if not member.isfile() or not member.name.endswith(".rs"):
                continue
            fh = tf.extractfile(member)
            if fh is None:
                continue
            out[member.name] = fh.read().decode("utf-8", errors="replace")
    return out


# ===========================================================================
# THE GATE
# ===========================================================================


def skip(msg: str) -> int:
    print(f"SKIPPED: check-unreachable-refusals — {msg}")
    return 2


def run(update: bool) -> int:
    marks = markers()
    recorded = parse_baseline(
        BASELINE.read_text(encoding="utf-8", errors="replace")
        if BASELINE.is_file()
        else ""
    )

    if not marks and not recorded:
        # Not a pass dressed up as one: say plainly that nothing was measured.
        print(
            "SKIPPED: check-unreachable-refusals — no UNREACHABLE-FROM markers\n"
            "  and no baseline. The gate examined nothing, which is reported as a\n"
            "  SKIP rather than a PASS because a check that cannot fail is not\n"
            "  evidence."
        )
        return 2

    try:
        repo = engine_path.require("check-unreachable-refusals")
    except SystemExit as exc:
        return skip(str(exc).splitlines()[0])

    rev = locked_revision()
    if rev is None:
        return skip("Cargo.lock names no `git+file:…#<rev>` engine source")

    crates = engine_crates()
    if not crates:
        return skip("the manifest names no `git = \"file:///…\"` engine crate")

    try:
        files = engine_sources(repo, rev, crates)
    except LookupError as exc:
        return skip(f"`git archive {rev[:8]}` failed: {exc}")
    if not files:
        return skip(f"`git archive {rev[:8]}` produced no .rs files")

    live: dict[str, dict] = {}
    for symbol in marks:
        name = symbol.rsplit("::", 1)[-1]
        live[symbol] = {"measured-at": rev, "sites": sites_for(files, name)}

    if update:
        BASELINE.write_text(render_baseline(live), encoding="utf-8", newline="\n")
        print(
            f"check-unreachable-refusals: baseline rewritten at {rev[:8]} — "
            f"{len(live)} symbol(s)."
        )
        return 0

    failures: list[str] = []

    for symbol in sorted(set(recorded) - set(marks)):
        failures.append(
            f"STALE BASELINE ENTRY: {symbol}\n"
            f"  The baseline watches this symbol and no source file carries an\n"
            f"  `UNREACHABLE-FROM: {symbol} @ …` marker any more. Either the\n"
            f"  paragraph was deleted (then delete the entry, with --update) or\n"
            f"  a reword dropped the marker, which is how a gate keyed on a name\n"
            f"  gets discharged by prose."
        )

    for symbol in sorted(marks):
        where = ", ".join(f"{f}" for f, _ in marks[symbol])
        sites = live[symbol]["sites"]

        if not sites:
            failures.append(
                f"THE SYMBOL IS GONE: {symbol}\n"
                f"  named by: {where}\n"
                f"  Nothing in the engine at {rev[:8]} mentions "
                f"`{symbol.rsplit('::', 1)[-1]}` in code at all.\n"
                f"  A paragraph explaining why an absent symbol cannot happen is\n"
                f"  not a kept sentence, it is a fossil. Delete the marker and the\n"
                f"  prose around it, or correct the symbol if the engine moved it."
            )
            continue

        if symbol not in recorded:
            failures.append(
                f"UNRECORDED MARKER: {symbol}\n"
                f"  named by: {where}\n"
                f"  The engine's {len(sites)} code line(s) for it at {rev[:8]}:\n"
                + "\n".join(f"    {s}" for s in sites)
                + "\n  Read them, satisfy yourself the claim is true, then record\n"
                "  the baseline with --update."
            )
            continue

        gone, arrived = diff(recorded[symbol]["sites"], sites)
        if gone or arrived:
            body = [
                f"THE ENGINE'S USE OF {symbol} HAS CHANGED.",
                f"  claimed unreachable by: {where}",
                f"  baseline measured at:   {recorded[symbol]['measured-at'][:8] or '?'}",
                f"  re-measured at:         {rev[:8]}",
            ]
            if arrived:
                body.append("  ★★★ NEW engine lines naming it:")
                body += [f"      + {s}" for s in arrived]
            if gone:
                body.append("  lines the baseline had and the engine no longer does:")
                body += [f"      - {s}" for s in gone]
            body += [
                "",
                "  This gate does NOT claim one of these is a constructor — that",
                "  judgment needs a parser and a human, and a gate that guessed",
                "  would be confidently wrong on `matches!`, on `Err(X::Y) =>`",
                "  and on a doctest. What it claims is that a sentence in this",
                "  shell says this symbol has no producer, and the engine's code",
                "  around it has moved since anybody checked.",
                "",
                "  ⇒ Read the paragraph the marker sits in. Correct it, or",
                "    confirm it and restamp with --update.",
            ]
            failures.append("\n".join(body))

    if failures:
        print("FAIL: check-unreachable-refusals\n")
        for f in failures:
            print(f + "\n")
        return 1

    total = sum(len(live[s]["sites"]) for s in marks)
    print(
        f"check-unreachable-refusals: PASS — {len(marks)} symbol(s), "
        f"{total} engine code line(s), re-measured at {rev[:8]}."
    )
    for symbol in sorted(marks):
        print(f"  {symbol}: {len(live[symbol]['sites'])} line(s) — unchanged")
    return 0


# ===========================================================================
# SELF-TEST — the gate must detect its own planted violation
# ===========================================================================


def self_test() -> int:
    """Falsify every branch on synthetic input.

    None of this touches the engine, git or the real baseline: the parts that
    can be wrong are the pure helpers, and they are the parts exercised here.
    A self-test that needed a checkout would be skipped on the machine where
    it matters.
    """
    fails: list[str] = []

    def check(label: str, got, want):
        if got != want:
            fails.append(f"  {label}\n    got:  {got!r}\n    want: {want!r}")

    # --- strip_comment -----------------------------------------------------
    check("a plain comment goes", strip_comment("let a = 1; // X::Y"), "let a = 1; ")
    check("a doc comment goes", strip_comment("    /// X::Y here"), "    ")
    check(
        "a URL in a string survives",
        strip_comment('git = "file:///D:/Dev/pdfcer" // note'),
        'git = "file:///D:/Dev/pdfcer" ',
    )
    check(
        "an escaped quote does not end the string",
        strip_comment(r'let s = "a\"//b"; // gone'),
        r'let s = "a\"//b"; ',
    )
    check("a lifetime is not a char literal", strip_comment("fn f<'a>(x: &'a T) {}"),
          "fn f<'a>(x: &'a T) {}")
    check("a char literal is a string", strip_comment("let c = '/'; // gone"),
          "let c = '/'; ")

    # --- normalise ---------------------------------------------------------
    check(
        "indentation and wrapping do not matter",
        normalise("        Self::X   =>   Y,   // why"),
        "Self::X => Y,",
    )
    check("a comment-only line normalises to nothing", normalise("   // X"), "")

    # --- sites_for: the planted violation ---------------------------------
    CLEAN = {
        "crates/e/src/a.rs": (
            "/// A doctest that CONSTRUCTS it must not count:\n"
            "/// let e = Err::Dead;\n"
            "pub enum Err {\n"
            "    Dead,\n"
            "}\n"
            "impl Err {\n"
            "    fn d(&self) -> u8 {\n"
            "        match self {\n"
            "            Self::Dead => 1, // an arm, not a producer\n"
            "        }\n"
            "    }\n"
            "}\n"
        ),
    }
    clean = sites_for(CLEAN, "Dead")
    check(
        "a doc comment does not become a site",
        clean,
        ["crates/e/src/a.rs | Dead,", "crates/e/src/a.rs | Self::Dead => 1,"],
    )

    # ★ Now plant a constructor, which is the thing the gate exists to catch.
    PLANTED = dict(CLEAN)
    PLANTED["crates/e/src/b.rs"] = "fn g() -> Err {\n    return Err::Dead;\n}\n"
    planted = sites_for(PLANTED, "Dead")
    gone, arrived = diff(clean, planted)
    check("a planted constructor is detected", arrived,
          ["crates/e/src/b.rs | return Err::Dead;"])
    check("and nothing is falsely reported missing", gone, [])

    # ★ And a REMOVED symbol, the other failure the gate must make.
    gone2, arrived2 = diff(clean, [])
    check("a vanished symbol reports every baseline line as gone", gone2, clean)
    check("and reports nothing arriving", arrived2, [])

    # ★ A pure reword of the engine's documentation must be INVISIBLE, or the
    #   gate becomes noise and noise is how a baseline gets restamped unread.
    REWORDED = {
        "crates/e/src/a.rs": CLEAN["crates/e/src/a.rs"].replace(
            "an arm, not a producer", "completely different prose entirely"
        ).replace("A doctest that CONSTRUCTS it must not count:", "Reworded."),
    }
    check("a documentation reword is invisible", sites_for(REWORDED, "Dead"), clean)

    # --- markers -----------------------------------------------------------
    m = MARKER.search("/// UNREACHABLE-FROM: a::b::C::D @ 025d703d\n")
    check("the marker parses", (m.group("symbol"), m.group("pin")) if m else None,
          ("a::b::C::D", "025d703d"))
    check("a bare word is not a marker",
          MARKER.search("UNREACHABLE-FROM: Dead @ 025d703d"), None)

    # --- baseline round trip ----------------------------------------------
    entries = {"a::b::C::D": {"measured-at": "025d703d", "sites": clean}}
    check("the baseline round-trips", parse_baseline(render_baseline(entries)), entries)

    if fails:
        print("FAIL: check-unreachable-refusals --self-test")
        for f in fails:
            print(f)
        return 1
    print("check-unreachable-refusals --self-test: PASS")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--update", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    return run(args.update)


if __name__ == "__main__":
    sys.exit(main())
