#!/usr/bin/env python
"""check-gate-input-scope.py — a verification whose input set is the index.

THE PROPERTY ASSERTED
=====================

Every `git grep` / `git ls-files` invocation under `tools/` either reads the
WORKING TREE or carries a written reason for not doing so.

A check that enumerates the files it will examine with `git grep` or
`git ls-files` is asking git what is **in the index**. The working tree is a
different set, and the two disagree on exactly one thing: **the work you have
just done and not yet staged.**

That is the worst possible blind spot, because it is also the only moment a
verification is run by hand. A session writes three files, runs the gate suite,
sees green, commits, and the same tree goes red on the next run with nothing
edited in between. All that changed was `git add`.

THE TELL IS THE TIMING, NOT THE CONTENT. Green before the commit, red
after it, nothing edited. If you ever see that, stop looking at the content and
look at how the check chose its files.

WHY THIS IS A GATE RATHER THAN A PARAGRAPH
------------------------------------------

Because the paragraph does not work. The generalisation — *"a gate whose input
set is 'what is already committed' cannot see the commit you are about to
make"* — is written into a docstring in this tree, and the same mechanism
recurs in the same directory regardless, because prose does not sweep.
**A lesson in a docstring is not an instrument.** This file is the sweep, so
the next sibling cannot be written the same way.

The three shapes it takes, all of which occur here:

1. **A bare query.** `git ls-files` or `git grep` with no working-tree flag
   cannot see the commit the gate is being run to clear, so the gate goes green
   by hand and red in CI on newly-written files — including, on occasion, the
   scrub script of the gate itself.

2. **A repair aimed at the instance.** An untracked file under one directory
   goes unscanned, and the fix **excludes that one directory**. The mechanism
   survives and fires again on the next directory. The repair has to be to the
   query, never to the path list.

3. **An index listing with a glob.** An index listing of `*.md` skips a
   Markdown file written and not yet added — and a brand-new document is
   precisely where an unescaped pipe or an inert `**` lives, because nobody has
   ever rendered it. Such a gate can run for its whole life without ever firing.

WHAT IT CHECKS
==============

Every `*.sh` and `*.py` under `tools/` is read **from the working tree** (a walk,
not a git query — an auditor with the defect it audits is worthless), and every
real invocation of `git grep` or `git ls-files` must either

* carry a working-tree flag in the same call — `--untracked` for `git grep`,
  `--others` for `git ls-files`; or
* carry an exemption marker `gate-input-scope-exempt: <reason>` on the call's
  own line, or on one of the two lines above it.

**Both of those are deliberately tight, and the tightness is the gate.** The
flag is looked for in the invocation's own extent — from the command forward
while its argument list is open — and the backward walk for an exemption stops
at any line holding another git call.

The rejected design worth naming, because it is the obvious simplification: a
single symmetric window of N lines for both. It lets a compliant call sitting
above a bare one satisfy the condition on its behalf, and a self-test over that
design reports **zero of three** planted bare calls while printing a confident
PASS. **A condition that something other than the subject can satisfy is not
testing the subject.**

WHAT THE EXEMPTION IS FOR, AND IT IS A REAL CASE
------------------------------------------------

Some checks are **supposed** to read the index, or a revision. The question to
ask at every hit is: *which side of `git add` does this check's subject live
on?*

* A check about what a reader sees, what ships, or what is on disk wants the
  **working tree**. This is almost every gate here.
* A check about what is **recorded** wants the index or a revision.
  `check-engine-api-drift` reads the engine's `.rs` bytes at a git revision on
  purpose, because its subject is the pinned commit that compiles, not whatever
  the engine's working tree happens to hold at the moment — the engine repo is
  READ-ONLY to this project and may be mid-edit by another session.

So the exemption is not a carve-out for convenience. It is the place the
distinction gets written down, which is why it requires a reason on the line.

HOW THE DETECTION AVOIDS PROSE
------------------------------

These gates document themselves heavily, so the phrase `git ls-files` appears in
far more comments than code — including in this file. Matching text would make
the gate unusable, and a gate that reports correct files gets carved out until it
means nothing.

* **Either language:** a line whose first non-blank character is `#` is a
  comment and is skipped.
* **Shell:** a match inside an unclosed quote is skipped as well. The shape this
  closes is a gate's own FAILURE MESSAGE naming the command it runs — an `echo`
  saying the scan itself failed and quoting the exit status. **The documentation
  a gate prints when it fails is the documentation most likely to name the
  command it runs.**
* **Python:** only the **argv form** counts — a quoted `git` followed by a
  quoted subcommand, as a subprocess argument list. Prose cannot produce that
  shape, and Python code cannot avoid it, because `subprocess` takes a list.

WHAT IT PROVABLY CANNOT SEE
---------------------------

* **A `shell=True` string invocation in Python**, which never takes the argv
  shape. None exists here and the repository's style forbids it; if one is ever
  written, extend `PY_CALL`.
* **An indirection** — a command name held in a variable, built by
  concatenation, or reached through a wrapper function or shell alias.
* **A non-git input set with the same defect**, such as a tool reading a
  committed file list from disk. The subject here is specifically the two git
  queries.
* **Whether an exemption's stated reason is TRUE.** The gate enforces that a
  reason was written, not that it is correct; that judgment needs a reader.

USAGE AND EXIT CODES — the project's three-state gate contract
==============================================================

  tools/gates/check-gate-input-scope.py              audit tools/
  tools/gates/check-gate-input-scope.py --self-test  falsify the mechanism

  0  clean    — every invocation reads the working tree or says why it does not
  1  FAIL     — one or more index-scoped invocations, each with `file:line`
  2  SKIPPED  — `tools/` not found

HOW TO FALSIFY IT
-----------------

`--self-test` falsifies in BOTH directions, per this repository's rule that a
check which cannot fail is not evidence. It plants a bare call, a flagged call, a
call whose flag is on its second line, an exempted call, two calls under one
exemption, three bare calls below a compliant one, a shell comment, a Python
comment quoting the argv form, a docstring mention, a command name inside an
`echo`, and a real call whose argument is quoted — and asserts exactly which of
them is reported. Each of the two prose filters is falsified in both directions,
because each is a carve-out added in response to a false positive, and an
unfalsified carve-out is how a gate stops seeing its subject.

Note that the fixture lines embedding bare calls carry an exemption marker on
their own SOURCE line, outside the fixture string: the scanner under test sees a
bare call, while this gate's live scan of its own file sees a declared one.
Deleting those trailing markers makes this gate report its own test data.
"""

import os
import re
import sys

NL = chr(10)
HASH = chr(35)
BACKSLASH = chr(92)
Q = chr(34)
SQ = chr(39)

# How far the scan will follow one invocation. A subprocess argument list is
# routinely wrapped across three or four lines, by hand and by formatters, so the
# flag is often not on the line that names the command. The cap is a runaway
# guard, not a policy: the extent normally ends first, at the closing bracket.
CALL_MAX_LINES = 6

# How far ABOVE a call an exemption comment may sit. Deliberately tiny, and the
# walk stops at any line containing another git call.
#
# The asymmetry with `CALL_MAX_LINES` is load-bearing. A symmetric window lets
# a compliant call, or an exemption comment meant for something else, sit above
# a bare call and launder it: the self-test over such a window reports ZERO of
# three planted bare calls. A condition that something other than the subject
# can satisfy is not testing the subject.
EXEMPT_ABOVE = 2

EXEMPT = "gate-input-scope-exempt:"

# The flag that turns each command's index query into a working-tree query.
NEEDS = {
    "grep": "--untracked",
    "ls-files": "--others",
}

# Python: the argv form only. `["git", "grep", ...]` in any quoting style, with
# arbitrary whitespace, possibly wrapped after the comma.
PY_CALL = re.compile(
    r"""['"]git['"]\s*,\s*['"](grep|ls-files)['"]"""
)

# Shell: the command as written. Word boundaries on both sides so `git ls-files`
# is not matched inside a longer token.
SH_CALL = re.compile(r"""\bgit\s+(grep|ls-files)\b""")


def in_string(line, col):
    """True when `line[col]` sits inside a quoted string, counted not parsed.

    An odd number of quote characters before the position means the position is
    inside one. That is a crude model of shell quoting and it is the right one
    for the only question asked here: *is this occurrence of a command name
    actually prose?*

    It exists because skipping whole comment lines is not enough. A gate's own
    FAILURE MESSAGE names the command it runs —
    `echo "... the scan itself failed (git grep exited $STATUS)."` is a real
    example from a sibling — and **the documentation a gate prints when it
    fails is the documentation most likely to name the command it runs**.
    """
    head = line[:col]
    return (head.count(Q) % 2 == 1) or (head.count(SQ) % 2 == 1)


def call_extent(lines, i, col, is_python):
    """The text of ONE invocation, starting at `lines[i][col:]`.

    Python: follow the argument list while brackets opened at or after the
    match are still unbalanced. Shell: follow a trailing line continuation.
    Either way, stop at `CALL_MAX_LINES` so a malformed file cannot make the
    extent swallow the rest of the script and launder every call below it.

    Returns the joined text. The flag is looked for in HERE and nowhere else:
    a flag belongs to the call that carries it.
    """
    parts = [lines[i][col:]]
    if is_python:
        # Count the depth over the WHOLE line, not from `col`. The brackets that
        # opened the call -- `subprocess.check_output([` -- sit BEFORE the matched
        # quoted `git`, so starting the count at the match sees depth 0 and stops
        # the extent at the first line. That is what made the first self-test
        # report a correctly-wrapped `--others` call as a violation: the flag was
        # on the call's second line and the extent never reached it.
        depth = 0
        for ch in lines[i]:
            if ch in '([{':
                depth += 1
            elif ch in ')]}':
                depth -= 1
        n = i + 1
        while depth > 0 and n < len(lines) and n - i < CALL_MAX_LINES:
            parts.append(lines[n])
            for ch in lines[n]:
                if ch in '([{':
                    depth += 1
                elif ch in ')]}':
                    depth -= 1
            n += 1
    else:
        n = i
        while (lines[n].rstrip().endswith(BACKSLASH)
               and n + 1 < len(lines) and n + 1 - i < CALL_MAX_LINES):
            n += 1
            parts.append(lines[n])
    return NL.join(parts)


def exempted(lines, i):
    """True when an exemption marker covers the call on `lines[i]`.

    On the line itself, or within `EXEMPT_ABOVE` lines above it -- and the walk
    STOPS at any line that contains another git call, so one comment can never
    cover two. A marker on a call's own source line is the normal form when the
    call is inside a string, which is how this gate's own self-test fixtures
    declare themselves without becoming violations.
    """
    if EXEMPT in lines[i]:
        return True
    for n in range(i - 1, max(-1, i - 1 - EXEMPT_ABOVE), -1):
        if EXEMPT in lines[n]:
            return True
        if PY_CALL.search(lines[n]) or SH_CALL.search(lines[n]):
            break
    return False


def scan_lines(lines, is_python):
    """Violating invocations in one file's lines.

    Returns a list of `(lineno, command, missing_flag)`, 1-based. An invocation
    is accepted when its OWN extent carries the working-tree flag for that
    command, or when an exemption marker is adjacent to it.
    """
    out = []
    for i, raw in enumerate(lines):
        # A comment is prose in either language, and these files document
        # themselves by QUOTING the shape they forbid -- this gate's own comment
        # spells out the argv form to explain its regex, and was reported by its
        # own first live run.
        if raw.strip().startswith(HASH):
            continue

        if is_python:
            hit = PY_CALL.search(raw)
        else:
            hit = SH_CALL.search(raw)

        if not hit:
            continue
        if not is_python and in_string(raw, hit.start()):
            # A command name inside an echo is documentation, not an invocation.
            continue

        cmd = hit.group(1)
        if exempted(lines, i):
            continue
        if NEEDS[cmd] in call_extent(lines, i, hit.start(), is_python):
            continue

        out.append((i + 1, cmd, NEEDS[cmd]))
    return out

def self_test():
    """Falsify both mechanisms in both directions.

    Every fixture line below that embeds a bare call carries an exemption
    marker on its own SOURCE line, outside the fixture string. So the scanner
    under test sees a bare call (which is the point of the fixture) while the
    live scan of this very file sees a declared one. Without that the gate
    reports its own test data, and a gate that reports correct files is a gate
    that gets carved out until it means nothing.
    """
    ok = True

    # ---- Python: prose, compliant calls, an exemption, and a bare call ----
    py = [
        '"""A docstring that mentions git ls-files and git grep in prose.',
        '',
        'Neither of those is a call and neither may be reported.',
        '"""',
        '    subprocess.check_output(["git", "ls-files", "--cached",',
        '                            "--others", "--exclude-standard"])',
        '    subprocess.run(["git", "grep", "--untracked", "-n", "x"])',
        '    # ' + EXEMPT + ' this one reads a revision on purpose',
        '    subprocess.run(["git", "grep", "-n", "y"])',  # gate-input-scope-exempt: self-test fixture, not an invocation
        '    subprocess.run(["git", "ls-files", "*.md"])',  # gate-input-scope-exempt: self-test fixture, not an invocation
    ]
    hits = scan_lines(py, True)
    if [(h[0], h[1]) for h in hits] != [(10, 'ls-files')]:
        print('SELF-TEST FAIL: python scan expected only line 10 (the bare'
              ' ls-files), got ' + str(hits))
        ok = False

    # ---- the laundering a symmetric window allows, by name ---------------
    # A compliant call above a bare one must NOT cover it. A symmetric window
    # breaks this property silently: the gate simply goes green.
    launder = [
        '    subprocess.run(["git", "grep", "--untracked", "-n", "x"])',
        '    subprocess.run(["git", "grep", "-n", "a"])',  # gate-input-scope-exempt: self-test fixture, not an invocation
        '    subprocess.run(["git", "grep", "-n", "b"])',  # gate-input-scope-exempt: self-test fixture, not an invocation
        '    subprocess.run(["git", "ls-files"])',  # gate-input-scope-exempt: self-test fixture, not an invocation
    ]
    if [h[0] for h in scan_lines(launder, True)] != [2, 3, 4]:
        print('SELF-TEST FAIL: a compliant call is laundering the bare calls'
              ' below it, got ' + str(scan_lines(launder, True)))
        ok = False

    # ---- one exemption comment may not cover two calls --------------------
    pair = [
        '    # ' + EXEMPT + ' only the call immediately below',
        '    subprocess.run(["git", "grep", "-n", "a"])',  # gate-input-scope-exempt: self-test fixture, not an invocation
        '    subprocess.run(["git", "grep", "-n", "b"])',  # gate-input-scope-exempt: self-test fixture, not an invocation
    ]
    if [h[0] for h in scan_lines(pair, True)] != [3]:
        print('SELF-TEST FAIL: one exemption covered two calls, got '
              + str(scan_lines(pair, True)))
        ok = False

    # ---- a wrapped argument list: the flag is not on the command's line ---
    wrapped = [
        '    subprocess.check_output(["git", "ls-files",',
        '                            "--cached", "--others",',
        '                            "--exclude-standard"])',
    ]
    if scan_lines(wrapped, True) != []:
        print('SELF-TEST FAIL: a flag on the call' + chr(39) + 's second line was'
              ' not seen')
        ok = False

    # ---- Shell: comments are prose, code is not ---------------------------
    sh = [
        '# The obvious implementation is a bare git grep, and it is wrong.',
        '#   git ls-files ' + chr(39) + '*.md' + chr(39),
        'RAW=$(git grep --untracked -nIP "$PATTERN" -- .)',
        'FILES=$(git ls-files --others --cached --exclude-standard)',
        'BAD=$(git grep -n needle)',  # gate-input-scope-exempt: self-test fixture, not an invocation
    ]
    hits = scan_lines(sh, False)
    if [(h[0], h[1]) for h in hits] != [(5, 'grep')]:
        print('SELF-TEST FAIL: shell scan expected only line 5, got ' + str(hits))
        ok = False

    # ---- the two prose filters, each falsified in BOTH directions ---------
    # Each was added in response to a real false positive, so each is a
    # carve-out, and an unfalsified carve-out is how a gate stops seeing its
    # subject. The quote-parity rule is the dangerous one: unchecked, it would
    # silence every call on a line holding an odd number of quotes.

    # A comment that QUOTES the argv form -- this gate's own header does.
    commented = ['    # the argv form is ["git", "ls-files", ...] in any quoting']  # gate-input-scope-exempt: self-test fixture, not an invocation
    if scan_lines(commented, True) != []:
        print('SELF-TEST FAIL: a python comment quoting the argv form was reported')
        ok = False

    # ...and the same shape as code on the next line must still be reported.
    both = [
        '    # the argv form is ["git", "ls-files", ...] in any quoting',  # gate-input-scope-exempt: self-test fixture, not an invocation
        '    subprocess.run(["git", "ls-files"])',  # gate-input-scope-exempt: self-test fixture, not an invocation
    ]
    if [h[0] for h in scan_lines(both, True)] != [2]:
        print('SELF-TEST FAIL: the comment filter is swallowing the call'
              ' under it, got ' + str(scan_lines(both, True)))
        ok = False

    # A command name inside an echo is documentation. This is verbatim the
    # shape of check-old-name-absent.sh's own failure message.
    echoed = ['    echo "FAIL: the scan itself failed (git grep exited $S)."']
    if scan_lines(echoed, False) != []:
        print('SELF-TEST FAIL: a git grep inside an echo string was reported')
        ok = False

    # ...and a real call on a line that also carries quotes must still be seen,
    # which is the direction the quote-parity rule could silently break.
    quoted_call = ['BAD=$(git grep -n "needle" -- .)']  # gate-input-scope-exempt: self-test fixture, not an invocation
    if [h[0] for h in scan_lines(quoted_call, False)] != [1]:
        print('SELF-TEST FAIL: quote parity silenced a real call whose'
              ' ARGUMENT is quoted, got ' + str(scan_lines(quoted_call, False)))
        ok = False

    print('self-test: ' + ('PASS' if ok else 'FAIL'))
    return 0 if ok else 1

def main():
    if "--self-test" in sys.argv[1:]:
        return self_test()

    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.dirname(here)          # tools/
    if not os.path.isdir(root):
        print("check-gate-input-scope: SKIPPED — tools/ not found at " + root)
        return 2

    violations = []
    scanned = 0
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames
                       if d not in ("__pycache__", "out", "target", "evidence")]
        for name in sorted(filenames):
            if not (name.endswith(".py") or name.endswith(".sh")):
                continue
            path = os.path.join(dirpath, name)
            try:
                with open(path, encoding="utf-8") as handle:
                    lines = handle.read().split(NL)
            except Exception:
                continue
            scanned += 1
            rel = os.path.relpath(path, os.path.dirname(root)).replace(os.sep, "/")
            for lineno, cmd, flag in scan_lines(lines, name.endswith(".py")):
                violations.append((rel, lineno, cmd, flag))

    if not violations:
        print("check-gate-input-scope: " + str(scanned)
              + " scripts, every `git grep` / `git ls-files` reads the working"
              + " tree or says why it does not")
        return 0

    for rel, lineno, cmd, flag in violations:
        print(rel + ":" + str(lineno) + ": `git " + cmd + "` reads the INDEX."
              " Pass " + flag + " to read the working tree.")
        print("    A file written and not yet `git add`-ed is invisible to this"
              " call, which is")
        print("    every file the current session wrote. If the check's subject"
              " really is what")
        print("    has been recorded, say so: `" + EXEMPT + " <reason>`"
              " on the call's own line or the line above.")

    print("")
    print("check-gate-input-scope: " + str(len(violations))
          + " invocation(s) whose input set is the index, in "
          + str(scanned) + " scripts.")
    print("The tell for this defect in the wild is the TIMING, not the content:")
    print("green before the commit, red after it, with nothing edited.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
