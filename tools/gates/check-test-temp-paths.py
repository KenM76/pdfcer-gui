#!/usr/bin/env python
"""check-test-temp-paths.py - a scratch path under %TEMP% must name its process.

THE PROPERTY ASSERTED
=====================

Every place in the workspace that asks for the system temporary directory
builds a path that is unique per PROCESS, or says in writing why it does not
need to be.

A test that writes to a FIXED path under the system temporary directory is
shared mutable state between every process on the machine that runs that test.
Inside one `cargo test` the sharing is invisible, because each fixed name has
exactly one user and the test harness never runs one test twice at once. The
moment two `cargo test` processes overlap - a backgrounded run plus a
foreground one, a watcher, an IDE, CI on a shared runner, or simply a session
that started a second sweep because the first looked stuck - they fight over
one file.

★★★ AND THE SYMPTOM IS NOT "TWO TESTS COLLIDED". The symptom is a single
unrelated assertion going red, several lines downstream of the real event, in
whichever process lost the race. It reads exactly like a regression in the
feature that test covers, and it passes when re-run alone, which reads exactly
like a flake. Neither reading points at shared state.

The concrete shape: two runs both call a helper that returns
`%TEMP%/<crate>-<tag>.pdf` with nothing process-unique in it. One is still
writing the file when the other opens it, so the reader gets a truncated
document and the assertion that fails is about whatever it checked next -
several lines past the corruption. Sites that call `std::fs::remove_dir_all`
on the way in are worse than a torn read: one process deletes the other's
fixture mid-test.

★ WHY A COMMENT WAS NOT ENOUGH, AND THIS IS THE REASON THE GATE EXISTS
-----------------------------------------------------------------------

Some sites with this defect already carried a confident note saying the hazard
was handled:

    // Tagged per caller: `cargo test` runs these in parallel, and two tests
    // writing one path is a flake that reproduces about a third of the time.

That reasoning is correct and the fix it describes is real - it separates
THREADS. It does nothing about PROCESSES, because two `cargo test` runs have
the same set of callers as each other and therefore ask for the same filenames.

**A note that names a hazard and fixes half of it is worse than no note at
all**: the next reader sees the hazard named, sees a mechanism beside it, and
stops looking. The correct pattern (`std::process::id()` in the name) already
existed elsewhere in this tree the whole time, so the convention was present
and simply not uniform - which is the textbook condition for a rule that lives
only in prose.

⇒ A lesson in a docstring is not an instrument. This file is the instrument.

THE RULE
========

Every `std::env::temp_dir()` call site in the workspace must either

  * name `std::process::id()` within the path-building extent (see below), or
  * carry the marker `temp-path-exempt: <reason>` inside that extent, or in the
    unbroken run of comment lines immediately above it.

There is deliberately no third option and no taxonomy. A nanosecond timestamp
is, in practice, just as unique across processes - four helpers in this tree
pair one with the pid - and the gate still requires the pid beside it. The
reason is that a rule reading *"a process-unique component, and here is how the
gate recognises one"* has a classification in it, and a classification is where
the next exception goes. `std::process::id()` or a written exemption: that
cannot drift, and it costs a compliant site one token.

WHAT THE EXEMPTION IS FOR, AND BOTH CASES ARE REAL
--------------------------------------------------

  1. **The path is never created.** An absence test wants a path with no file
     at it and does not care which one; two processes wanting the same
     non-existent path is not a collision. Four sites.

  2. **A stable name is the point.** `#[ignore]`d generators and dumps exist so
     a human can run them deliberately and then go and open the file they
     named. A pid in the name would mean hunting for it. Three sites.

The marker requires a reason on the line, so the exemption is a decision on the
record rather than an absence the gate happens not to notice. The two counts
above are also the cheapest staleness check there is: if the printed exemption
tally stops matching them, either a site was added without a reason or a
pattern stopped matching.

HOW A SITE'S EXTENT IS DECIDED, AND WHY IT IS NOT JUST "THE LINE"
=================================================================

The evidence cannot be searched for on the call's own line, because the
dominant idiom in this tree splits the path build across statements:

    let mut p = std::env::temp_dir();
    p.push(format!("pdfcer-protect-{tag}-{}.pdf", std::process::id()));

Nor can it be a fixed window of N lines, for the reason `check-gate-input-scope`
had to learn the hard way: **a condition that something other than the subject
can satisfy is not testing the subject.** A symmetric six-line window there let
a compliant call rescue three planted bad ones, and its own self-test reported
zero of three.

So the extent is derived from the code rather than counted:

  * It starts at the statement containing `env::temp_dir()`. A statement ends at
    the first `;`, `{` or `}` that is not inside a string or a comment - which
    is coarse for struct literals and exactly right for everything here.
  * If that statement BINDS a name (`let [mut] NAME = ... temp_dir() ...`), the
    extent continues over each following statement for as long as that statement
    is still building the same path: `NAME.push(`, `NAME.join(`,
    `NAME.set_extension(`, `NAME.set_file_name(`. The first statement that does
    anything else ends the extent.

That means a compliant site cannot vouch for its neighbour in either direction,
which is the property the self-test falsifies directly.

TWO MECHANISMS, BECAUSE THE FIRST HAS AN OBVIOUS ESCAPE
=======================================================

  1. `env::temp_dir()` call sites, as above.

  2. **A direct import of the function.** `use std::env::temp_dir;` followed by
     a bare `temp_dir()` would evade mechanism 1 entirely, and the evasion would
     be silent and accidental. None exists today; mechanism 2 reports any such
     import as a violation in its own right, with the instruction to qualify the
     call instead. This is the "a pattern that stops matching prints exactly
     what a clean run prints" defence, written down at the time the pattern was
     written rather than after it failed.

DETECTION AVOIDS PROSE
----------------------

These gates document themselves heavily, so `std::env::temp_dir()` appears in
comments and in this file's own self-test fixtures far more often than in code.
Sites are therefore detected on a STRIPPED copy of each line, with `//`
comments removed and the interiors of string literals blanked. The raw line is
what the evidence and the exemption marker are searched in, because an exemption
IS a comment.

WHAT IT PROVABLY CANNOT SEE
---------------------------

* **Any other route to a fixed scratch path.** A hard-coded `C:/Temp/...`, a
  `TMPDIR` read through `std::env::var`, a crate such as `tempfile` used with a
  fixed name - none of them mention `env::temp_dir`, and none is reported. The
  claim is about one function, not about temporary files in general.
* **Whether a pid in the name is actually USED.** The evidence test is textual:
  `process::id()` somewhere in the extent satisfies it. A site that computes the
  pid and then discards it passes.
* **Whether an exemption's stated reason is TRUE.** The marker records a
  decision; it does not verify one. A site claiming "never created" that creates
  the file is invisible here and only a reader can catch it.
* **A local helper *named* `temp_dir`** taking a tag, of which this tree has
  four. `SITE` requires the empty argument list precisely so those are not
  reported - which also means the gate says nothing about what such a helper
  builds, only about the `env::temp_dir()` call inside it.
* **Anything outside `.rs`**, and anything a Rust macro generates rather than
  spells.

INPUT SET
---------

`git ls-files --cached --others --exclude-standard -- '*.rs'` - the working
tree, not the index, per `check-gate-input-scope`. A scratch path written and
not yet staged is precisely the one nobody has run twice yet.

USAGE AND EXIT CODES - the project's three-state gate contract
==============================================================

  tools/gates/check-test-temp-paths.py              audit the workspace
  tools/gates/check-test-temp-paths.py --self-test  falsify the mechanism

  0  clean    - every site carries the pid or a written exemption
  1  FAIL     - one or more bare sites, or a bare import, each with `file:line`
  2  SKIPPED  - not a git working tree, or no `.rs` files found

HOW TO FALSIFY IT
-----------------

`--self-test` runs the detector over synthetic Rust covering both directions:
compliant one-line and split-across-statements sites, a bare site, an exempted
site with the marker inside the extent and another with it in the comment run
above, the bare import, and - the assertion that matters most - a compliant
site placed next to a bare one in both orders, which must still report exactly
one violation. A window-based extent passes every other case and fails that
one. Registered in `run-all.sh` ahead of the real run: a check that has never
been watched fail is not evidence.
"""

import os
import re
import subprocess
import sys

NL = chr(10)
CR = chr(13)
QUOTE = chr(34)
BACKSLASH = chr(92)

MARKER = "temp-path-exempt:"
EVIDENCE = "process::id()"

# `std::env::temp_dir()` / `env::temp_dir()`, with whitespace tolerated the way
# rustfmt might leave it. The `()` is required: this must not match a local
# helper *named* `temp_dir` taking a tag, of which this tree has four.
SITE = re.compile(r"(?:std\s*::\s*)?env\s*::\s*temp_dir\s*\(\s*\)")

# Mechanism 2: the import that would make mechanism 1 blind.
BARE_IMPORT = re.compile(r"\buse\s+std\s*::\s*env\s*::\s*(?:\{[^}]*\b)?temp_dir\b")

# A statement that is still building the path bound by the site's statement.
CONTINUES = "|".join(["push", "join", "set_extension", "set_file_name"])

BIND = re.compile(r"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=")


# --------------------------------------------------------------------------
def strip_code(line):
    """Blank out string-literal interiors and drop a trailing `//` comment.

    Returns a string the same length as `line` wherever that is possible, so a
    column index into the result still means something in the original. The
    quotes themselves are kept; only what is between them is replaced by
    spaces, because a literal such as `"pdfcer-protect-{tag}.pdf"` must not
    contribute its braces to statement splitting.

    Raw strings (`r"..."`) are handled as ordinary ones, which is adequate
    here: none of the call sites use one, and the failure mode if one appeared
    would be an extent ending early, not a missed site.
    """
    out = []
    i = 0
    n = len(line)
    in_str = False
    while i < n:
        ch = line[i]
        if in_str:
            if ch == BACKSLASH and i + 1 < n:
                out.append(" ")
                out.append(" ")
                i += 2
                continue
            if ch == QUOTE:
                in_str = False
                out.append(ch)
            else:
                out.append(" ")
            i += 1
            continue
        if ch == QUOTE:
            in_str = True
            out.append(ch)
            i += 1
            continue
        if ch == "/" and i + 1 < n and line[i + 1] == "/":
            out.append(" " * (n - i))
            break
        out.append(ch)
        i += 1
    return "".join(out)


def statement_bounds(stripped, start):
    """The line index at which the statement beginning on `start` ends.

    Coarse on purpose - see the header. A statement ends at the first `;`, `{`
    or `}` outside a string or comment. A tail expression with no semicolon
    (`fn f() -> PathBuf { env::temp_dir().join("x") }` split over lines) ends at
    the `}` that closes its function, which is one line of slop and harmless.
    """
    for k in range(start, len(stripped)):
        for ch in stripped[k]:
            if ch in ";{}":
                return k
    return len(stripped) - 1


def extent(lines, stripped, site_line):
    """[first, last] line indices of the path-building extent for a site.

    The forward walk over continuation statements is what makes the multi-
    statement idiom auditable without widening the window for everybody.
    """
    first = site_line
    last = statement_bounds(stripped, site_line)

    name = None
    joined = " ".join(stripped[first:last + 1])
    match = BIND.search(joined)
    if match:
        name = match.group(1)

    if name:
        cont = re.compile(
            r"\b" + re.escape(name) + r"\s*\.\s*(?:" + CONTINUES + r")\s*\("
        )
        probe = last + 1
        while probe < len(lines):
            # Skip blank lines and comment-only lines without consuming them as
            # the continuation itself; a comment between two halves of a path
            # build is ordinary here.
            if not stripped[probe].strip():
                probe += 1
                continue
            end = statement_bounds(stripped, probe)
            text = " ".join(stripped[probe:end + 1])
            if not cont.search(text):
                break
            last = end
            probe = end + 1

    return first, last


def comment_run_above(lines, first):
    """The unbroken run of comment lines immediately above `first`.

    Walked rather than counted, so an exemption may carry as much reasoning as
    it needs - which, given this project's documentation rule, it usually does.
    The walk stops at the first line that is not a comment, so a marker cannot
    leak across a statement.
    """
    out = []
    k = first - 1
    while k >= 0:
        body = lines[k].strip()
        if body.startswith("//") or body.startswith("///"):
            out.append(lines[k])
            k -= 1
            continue
        break
    return out


def scan(lines):
    """[(line_number, kind, excerpt)] for one file's violations.

    `kind` is 'site' for an unmarked fixed path and 'import' for the bare-import
    evasion of mechanism 2.
    """
    stripped = [strip_code(ln) for ln in lines]
    hits = []
    audited = 0

    for i, code in enumerate(stripped):
        if BARE_IMPORT.search(code):
            hits.append((i + 1, "import", lines[i].strip()[:100]))

    i = 0
    while i < len(stripped):
        if not SITE.search(stripped[i]):
            i += 1
            continue
        audited += 1
        first, last = extent(lines, stripped, i)
        body = NL.join(lines[first:last + 1])
        above = NL.join(comment_run_above(lines, first))
        if EVIDENCE in body:
            i = last + 1
            continue
        if MARKER in body or MARKER in above:
            i = last + 1
            continue
        hits.append((i + 1, "site", lines[i].strip()[:100]))
        i = last + 1

    return hits, audited


# --------------------------------------------------------------------------
def self_test():
    """Every mechanism, falsified in both directions.

    The fourth block is the one that matters most and is the reason this gate
    does not use a fixed line window: a COMPLIANT site sitting directly above a
    bare one must not rescue it. `check-gate-input-scope`'s first draft failed
    exactly there and reported zero of three planted violations while printing
    a confident PASS.
    """
    ok = True

    def check(label, src, want_lines):
        hits, _ = scan(src)
        got = sorted(h[0] for h in hits)
        if got != sorted(want_lines):
            print("SELF-TEST FAIL: " + label + " expected lines "
                  + str(sorted(want_lines)) + ", got " + str(got))
            return False
        return True

    # ---- 1. the bare fixed path is reported --------------------------------
    ok &= check(
        "a bare fixed path",
        ['    let dir = std::env::temp_dir().join("pdfcer-fixed-name");'],
        [1],
    )

    # ---- 2. the pid on the same line is silent -----------------------------
    ok &= check(
        "a pid on the call's own line",
        ['    let dir = std::env::temp_dir().join(format!("x-{}", std::process::id()));'],
        [],
    )

    # ---- 3. the pid on a CONTINUATION statement is silent -------------------
    # This is the idiom the gate was written around. A line-local search
    # reports it, and reporting a correct site is how a gate gets carved out
    # until it means nothing.
    ok &= check(
        "a pid on a following push()",
        [
            "    let mut p = std::env::temp_dir();",
            '    p.push(format!("pdfcer-protect-{tag}-{}.pdf", std::process::id()));',
            "    p",
        ],
        [],
    )

    # ---- 4. a compliant neighbour must NOT vouch for a bare site ------------
    ok &= check(
        "a compliant site directly above a bare one",
        [
            '    let good = std::env::temp_dir().join(format!("a-{}", std::process::id()));',
            '    let bad = std::env::temp_dir().join("b-fixed");',
        ],
        [2],
    )
    ok &= check(
        "a compliant site directly below a bare one",
        [
            '    let bad = std::env::temp_dir().join("b-fixed");',
            '    let good = std::env::temp_dir().join(format!("a-{}", std::process::id()));',
        ],
        [1],
    )

    # ---- 5. the continuation walk stops at a non-continuation ---------------
    # `p` is bound, then something unrelated happens, then a pid appears. The
    # pid is NOT part of this path build and must not count.
    ok &= check(
        "a pid beyond the end of the path build",
        [
            "    let mut p = std::env::temp_dir();",
            '    p.push("fixed.pdf");',
            "    std::fs::write(&p, b).expect(x);",
            '    let other = format!("{}", std::process::id());',
        ],
        [1],
    )

    # ---- 6. the exemption marker, in the extent and above it ---------------
    ok &= check(
        "an exemption on the line",
        ['    let p = std::env::temp_dir().join("nope"); // temp-path-exempt: never created'],
        [],
    )
    ok &= check(
        "an exemption in the comment run above",
        [
            "    // temp-path-exempt: nothing is created here; the test wants a",
            "    // path that is absent, and absence is not contended.",
            '    let p = std::env::temp_dir().join("nope");',
        ],
        [],
    )
    # ...but it must not leak across an intervening statement.
    ok &= check(
        "an exemption separated by a statement",
        [
            "    // temp-path-exempt: this reason belongs to the line below it",
            "    let a = 1;",
            '    let p = std::env::temp_dir().join("nope");',
        ],
        [3],
    )

    # ---- 7. prose and strings are not call sites ---------------------------
    ok &= check(
        "the phrase inside a comment",
        [
            "    // A scratch path from std::env::temp_dir() with a fixed name is",
            "    // shared state between processes.",
            "    let x = 1;",
        ],
        [],
    )
    ok &= check(
        "the phrase inside a string literal",
        ['    println!("call std::env::temp_dir() for this");'],
        [],
    )

    # ---- 8. a local helper NAMED temp_dir is not a site --------------------
    # Four of these exist in this tree and every one of them would be a false
    # positive that got the gate switched off.
    ok &= check(
        "a local fn named temp_dir",
        [
            "    fn temp_dir(tag: &str) -> PathBuf {",
            '        unreachable!("{tag}")',
            "    }",
            '        let d = temp_dir("modes");',
        ],
        [],
    )

    # ---- 9. mechanism 2: the import that blinds mechanism 1 ----------------
    ok &= check(
        "a direct import of temp_dir",
        [
            "    use std::env::temp_dir;",
            '    let d = temp_dir().join("fixed");',
        ],
        [1],
    )
    ok &= check(
        "a direct import inside a brace group",
        ["    use std::env::{self, temp_dir};"],
        [1],
    )
    # And the ordinary import must be silent, or every file fails.
    ok &= check(
        "the ordinary module import",
        ["    use std::env;"],
        [],
    )

    # ---- 10. the detector can see the real defect at all -------------------
    # Direct falsification against the shape that actually shipped, rather than
    # against a minimal toy: the doc comment that made it look solved is
    # included, because that comment contains the word `parallel` and an
    # earlier draft keyed on wording.
    real = [
        "/// A scratch path in the system temp directory, unique per caller.",
        "///",
        "/// Named per test rather than shared, because `cargo test` runs these",
        "/// in parallel and two tests writing one path is a flake.",
        "fn scratch(tag: &str) -> PathBuf {",
        "    let mut p = std::env::temp_dir();",
        '    p.push(format!("pdfcer-protect-{tag}.pdf"));',
        "    p",
        "}",
    ]
    ok &= check("the defect as it actually shipped", real, [6])

    print("self-test: " + ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def main():
    if "--self-test" in sys.argv[1:]:
        return self_test()

    try:
        listing = subprocess.check_output(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard",
             "--", "*.rs"],
            text=True)
    except Exception as exc:
        print("check-test-temp-paths: SKIPPED - not a git checkout ("
              + str(exc) + ")")
        return 2

    files = [f for f in listing.split(NL) if f.strip()]
    if not files:
        print("check-test-temp-paths: SKIPPED - no Rust source in the working tree")
        return 2

    violations = []
    scanned = 0
    audited = 0
    for path in files:
        if not os.path.isfile(path):
            continue
        try:
            with open(path, encoding="utf-8") as handle:
                raw = handle.read()
        except Exception:
            continue
        scanned += 1
        lines = [ln.rstrip(CR) for ln in raw.split(NL)]
        hits, n = scan(lines)
        audited += n
        for lineno, kind, excerpt in hits:
            violations.append((path, lineno, kind, excerpt))

    # ★ The tally is printed on the clean path as well as the dirty one, and it
    # is allowed to be zero. A gate that only ever prints "clean" cannot be
    # distinguished from a gate whose pattern stopped matching; this one says
    # how many sites it actually looked at, so a sudden drop is visible.
    if not violations:
        print("check-test-temp-paths: " + str(scanned) + " Rust files, "
              + str(audited) + " `env::temp_dir()` site(s), every one carrying"
              + " `std::process::id()` or a written exemption")
        if audited == 0:
            print("check-test-temp-paths: FAILED - it found NO sites at all,"
                  + " which means the pattern stopped matching, not that the"
                  + " tree stopped using the system temp directory.")
            return 1
        return 0

    for path, lineno, kind, excerpt in violations:
        if kind == "import":
            print(path + ":" + str(lineno) + ": `temp_dir` is imported directly,"
                  + " which makes every call to it invisible to this gate.")
            print("    " + excerpt)
            print("    Call it as `std::env::temp_dir()` so the site can be"
                  + " audited.")
        else:
            print(path + ":" + str(lineno) + ": this scratch path has no"
                  + " `std::process::id()` in it, so two `cargo test` processes"
                  + " share one file.")
            print("    " + excerpt)
            print("    Add the pid, or mark the site"
                  + " `// " + MARKER + " <reason>` if a stable name is the point.")

    print("")
    print("check-test-temp-paths: " + str(len(violations))
          + " violation(s) across " + str(scanned) + " files ("
          + str(audited) + " sites audited).")
    print("This does not fail inside one `cargo test` run - each fixed name has")
    print("exactly one user there. It fails when two runs overlap, and it")
    print("presents as an unrelated assertion going red in whichever process")
    print("lost the race, several lines past the real event.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
