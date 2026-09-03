#!/usr/bin/env python3
"""check-suite-name-absent -- the licensed suite's name must not appear in this repository.

Operator ruling, 2026-08-25: the name of the licensed print-conformance suite
pdfcer measures itself against is kept out of this public repository entirely --
file contents AND file names. `docs/ROADMAP.md` open question `(bt)`.

WHY THIS SCRIPT EXISTS AT ALL, AND WHY IT LOOKS ODD
===================================================
A check for "this word must never appear" has a bootstrapping problem: the
obvious implementation is `git grep -i <word>`, and the moment that command is
written into a tracked file, the tracked file contains the word. The gate
becomes its own first violation, and a reader cannot tell a real occurrence
from the gate that hunts them.

So the needles are stored **base64-encoded** and decoded at run time. This is
NOT obfuscation for its own sake and NOT security by obscurity -- there is
nothing secret here, and `manifest.json` in the private map directory names the
suite in full. It is the only way for the rule and its enforcement to live in
the same repository without contradicting each other.

WHAT IT CHECKS
==============
1. File CONTENTS, case-insensitively, excluding binaries -- tracked files AND
   untracked, non-ignored ones. A binary OCR model matches a naive grep
   because its weights happen to contain the byte sequence; `git grep -I`
   excludes it, and that exclusion is deliberate rather than convenient -- a
   model's weights are not a mention.
2. File NAMES, over the same set. A scrubbed file called `<name>-check.py`
   still fails.

TWO DEFECTS FOUND BY THE SIBLING PROJECT, 2026-08-25, AND FIXED HERE
====================================================================
`iccce` built the same gate, hit both of these on its own copy, then read
this file and found them here too
(`D:\\Dev\\FeatureRequests\\iccce_FeatureRequests\\open\\`,
`note_your_name_gate_has_the_two_defects_mine_had.md`). Both are recorded
rather than quietly repaired, because each is a mistake a careful reader
would make again.

**1. The gate published the term it exists to suppress.** The failure path
printed a violating file NAME verbatim -- and the path IS the violation, so
every time the gate fired it wrote the forbidden term into a public CI log,
on a public repository, in an artifact that outlives the fix. The contents
branch was already safe, which is exactly why this was easy to miss: a
`path:line` locator carries no text, so the care that produced the
base64-encoded needles was never applied one function further down. Both
output paths are masked now.

**2. ★★ The gate could not see the commit it was gating.** `git ls-files`
lists what is already in the index, and a bare `git grep` searches only
tracked files. Run locally BEFORE staging -- which is when anyone naturally
runs a verification -- both silently excluded precisely the files the session
had just written, which are the only files that could have introduced a new
violation. `iccce` paid for this with a red CI run on three files, one of
them its own scrub script, whose docstring's worked example spelled the name
out. **The generalisation is worth more than the fix: a gate whose input set
is "what is already committed" cannot see the commit you are about to make.**
Locally it answers a question about the past; on CI, where everything is
checked out, it answers one about the present. The two disagree exactly on
the new work, so a green local run carried no information about the push that
followed it. Both queries now include untracked, non-ignored files.

EXIT CODES
==========
0  clean -- no occurrence in any tracked or newly-written file's content or name
1  at least one occurrence, printed with `path:line` so it can be opened
2  the check could not run (not a git work tree, `git` missing)

The output deliberately prints the offending LINE NUMBER but NOT the line's
text, because printing it would reproduce the term in CI logs -- which are
themselves public on a public repository.
"""

import base64
import subprocess
import sys

# The two forbidden needles, base64 of the lowercase forms. Decoded at run time
# so that this file -- which is itself tracked -- does not contain them.
NEEDLES_B64 = ("Z2hlbnQ=", "Z3dn")


def needles():
    """Decode the forbidden terms.

    Kept as a function rather than a module constant so the decoded strings are
    short-lived and never appear in a traceback's frame locals dump.
    """
    return [base64.b64decode(n).decode("ascii") for n in NEEDLES_B64]


def mask(path):
    """Replace every occurrence of a needle in `path` with `***`.

    The point of the gate is that the term must not be published, and a
    failure message is published in a CI log on a public repository. Masking
    keeps the path locatable -- `docs/_probe_***_name.md` names exactly one
    file -- while printing the one thing that must not be printed.

    Applied to BOTH output paths, not just the file-name one: a violating
    line inside a file whose name also carries the term would otherwise leak
    it through the content branch instead.
    """
    out = path
    for needle in needles():
        # Case-insensitive replacement without a regex, so nothing in the
        # path is treated as a metacharacter.
        low = out.lower()
        i = low.find(needle)
        while i >= 0:
            out = out[:i] + "***" + out[i + len(needle) :]
            low = out.lower()
            i = low.find(needle)
    return out


def tracked_hits(term):
    """`path:line` for every tracked TEXT file containing `term`, case-insensitively.

    `-I` excludes binary files (see the module docstring on why that is correct
    rather than expedient). `-n` gives line numbers, `-i` case-insensitivity, and
    `--name-only` is deliberately NOT used -- a reviewer needs the line number to
    open the right place without the text being echoed.
    """
    # `--untracked` is defect 2's half of the fix: without it this searches
    # only files git already knows about, so a file written five minutes ago
    # and not yet staged is invisible -- and that is the only file that could
    # have introduced a new violation. `--exclude-standard` is implied for
    # `--untracked`, so `.gitignore`d build output is still skipped.
    proc = subprocess.run(
        ["git", "grep", "-I", "-i", "-n", "-o", "--untracked", "-e", term],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode not in (0, 1):
        print("check-suite-name-absent: git grep failed:", proc.stderr.strip())
        sys.exit(2)
    hits = []
    for line in proc.stdout.splitlines():
        # `-o` prints `path:line:match`; drop the match so the term is not echoed.
        parts = line.rsplit(":", 1)
        if parts:
            hits.append(parts[0])
    return hits


def tracked_names(term):
    """Tracked file PATHS containing `term`, case-insensitively.

    A file whose contents are clean but whose NAME still carries the term has
    not been scrubbed -- the name is published in every directory listing, every
    commit diff and the repository's web view.
    """
    # `--cached --others --exclude-standard` is defect 2's other half: the
    # index PLUS files written but not yet staged, minus anything ignored.
    proc = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        print("check-suite-name-absent: not a git work tree")
        sys.exit(2)
    return [p for p in proc.stdout.splitlines() if term in p.lower()]


def main():
    bad_content, bad_names = [], []
    for term in needles():
        bad_content.extend(tracked_hits(term))
        bad_names.extend(tracked_names(term))

    if not bad_content and not bad_names:
        print(
            "suite-name-absent: clean -- nothing in the work tree, staged or not, "
            "names it or mentions it"
        )
        return 0

    for path in sorted(set(bad_names)):
        print("FILENAME  %s" % mask(path))
    for where in sorted(set(bad_content)):
        print("CONTENT   %s" % mask(where))
    print(
        "suite-name-absent: %d file name(s) and %d line(s) still carry it "
        "(operator ruling 2026-08-25; see the private map directory)"
        % (len(set(bad_names)), len(set(bad_content)))
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
