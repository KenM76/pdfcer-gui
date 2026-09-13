#!/usr/bin/env python3
r"""check-completeness-tests.py — a completeness test may not carry its own copy
of the set it is checking.

===========================================================================
WHY THIS EXISTS, AND WHY IT TOOK FIVE RECURRENCES TO BUILD
===========================================================================

A test called `every_unit_is_named_distinctly` makes a promise in its name: that
if a unit ever arrives without a distinct label, this will go red. On 2026-09-13
the engine shipped three new units — kilometres, yards, miles — and that test
stayed green, silent, and useless, because it iterated a HAND-WRITTEN array of
the six variants that existed when it was written.

What actually caught the three unlabelled units was an exhaustive `match` in the
function above it. The compiler, not the test. The test whose entire job was to
notice noticed nothing, and would have gone on not noticing for ever, because a
private copy of a set does not grow when the set does.

⇒ **A completeness test that carries its own copy of the set is testing the
copy.** It is invisible for exactly as long as the set is stable, which is
exactly as long as nobody needs it.

That lesson had been written into agent memory FIVE times, under five different
incidents, and the instrument was never built — the sixth recurrence is what
finally paid for this file. A lesson in a docstring is not an instrument.

===========================================================================
THE PREDICATE, AND WHY IT IS NARROW ON PURPOSE
===========================================================================

The first draft flagged any literal array inside a completeness-named test and
returned 53 hits on a clean tree. Most were test INPUTS — a list of widths, a
pair of drag corners — and a gate that fires on those teaches people to write
exemptions, which is how a gate becomes scenery.

So the predicate is: a literal array, inside a function whose name begins
`every_` / `all_` / `each_` or contains `_complete` / `_exhaustive`, in which
**three or more elements are `Type::Variant` paths sharing one `Type`**. That is
not a list of inputs. That is a second, private copy of an enumeration.

It returns 29 on the tree it was written against — a number small enough to work
down and large enough to prove the gate is aimed at something.

★ FOREIGN vs LOCAL is the severity axis. A hand-copied LOCAL enum is bad: the
copy and the enum sit in the same repository, so at least one commit touches
both neighbourhoods. A hand-copied FOREIGN enum — `pdfcer_core::EditError`,
`Object`, `RecompressReason` — is worse by a category, because the set grows in
a repository this one does not build, on a branch pin that moves without a
`cargo update`, and NOTHING on this side changes on the day it happens. **Nine
of the twenty-nine are foreign** — and that nine is measured, not counted by
eye: three of the sites enumerate `E`, `D` and `R`, which are import aliases
rather than type names, and a first pass that grepped the repository for
`enum X` called all three foreign when one of them is ours. The gate resolves
the `use` statement now, because a severity count wrong by one in the direction
of alarm devalues the other nine.

===========================================================================
THE SNAPSHOT, AND THE HONEST NAME FOR IT
===========================================================================

`completeness-snapshot.txt` lists every site that existed when the gate was
adopted. It is **a debt register, not an exemption list**, and the distinction
is written here because it is the one that erodes: an exemption says "this is
fine"; a debt entry says "this is wrong and has not been fixed yet". Every run
prints the outstanding count so the number is in front of whoever reads it.

Two failures, both red:

  1. A site NOT in the snapshot — a new hand-written set. This is the whole
     point: the debt may shrink, never grow.
  2. A snapshot line matching NOTHING — the site was fixed, renamed or deleted
     and the register was not updated. A stale entry is how a register stops
     describing the tree; `check-strong-text.sh` carried a carve-out whose
     premise had quietly stopped being true, and that is the lesson taken
     literally.

A site is identified by `path::fn`, never by line number, because line numbers
churn on every edit above them and a register that goes stale on unrelated work
is a register people delete.

===========================================================================
EXIT CODES
===========================================================================

  0  clean   — the set of sites equals the register.
  1  FAIL    — a new site, or a stale register line.
  2  SKIPPED — no `crates/` tree to scan (partial checkout).

`--self-test` plants six cases, and THREE of them must come back clean: a
registered site, a list that really is inputs, and a hand-copied enum inside a
test whose name makes no completeness promise. A gate that answered 1
unconditionally would pass three of six; the quiet cases are what make the
loud ones mean anything.
"""

import io
import os
import re
import sys
import tempfile

NL = chr(10)
TAB = chr(9)
OB, CB = chr(123), chr(125)
LB, RB = chr(91), chr(93)
BS = chr(92)

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
SNAPSHOT = os.path.join(HERE, "completeness-snapshot.txt")

# A function whose NAME promises completeness. The promise is the thing being
# enforced: nothing here objects to a hand-written array in a test called
# `two_widths_wrap_the_same_way`.
FN = re.compile(
    r"^[ \t]*(?:pub )?fn ("
    r"(?:every|all|each)_[a-z0-9_]*"
    r"|[a-z0-9_]*_(?:complete|completeness|exhaustive)[a-z0-9_]*"
    r")\s*\("
)

OPEN = re.compile(r"=\s*&?" + re.escape(LB))
PATH = re.compile(r"\b([A-Z][A-Za-z0-9_]*)::([A-Z][A-Za-z0-9_]*)\b")
ENUM = re.compile(r"enum ([A-Z][A-Za-z0-9_]*)\b")

# `use a::b::Thing;` and `use a::b::Thing as T;` - the import is a better oracle
# for where a type comes from than a repo-wide `enum` grep, because it works for
# an ALIAS (`E`, `D`, `R` are not type names) and for a type this repository
# declares but this file imports from somewhere else.
USE = re.compile(r"^\s*(?:pub\s+)?use\s+([A-Za-z0-9_:]+?)(?:\s+as\s+([A-Z][A-Za-z0-9_]*))?\s*;",
                 re.M)


def imports(src):
    """{local-name: full path} for every single-item `use` in one file."""
    out = {}
    for path, alias in USE.findall(src):
        segs = path.split("::")
        if not segs or not segs[-1][:1].isupper():
            continue
        out[alias or segs[-1]] = path
    return out


def origin_of(ty, src, local):
    """LOCAL or FOREIGN, and the real type name behind any alias.

    Precedence is deliberate: the file's own import wins over the repo-wide
    declaration set, because the import is a statement about THIS use of the
    name and the declaration set is a statement about the repository.
    """
    imp = imports(src)
    if ty in imp:
        path = imp[ty]
        real = path.split("::")[-1]
        root = path.split("::")[0]
        return ("LOCAL" if root in ("crate", "super", "self") else "FOREIGN"), real
    return ("LOCAL" if ty in local else "FOREIGN"), ty

MIN_VARIANTS = 3   # below this it is a pair of cases, not a copy of a set.
MAX_FN_LINES = 400
MAX_LIT_LINES = 60


def scan(root):
    """Every (relpath, fn_name, type_name, variant_count) in the tree.

    Walks `crates/` and `tools/`. For each completeness-named function, brace-
    matches its body, then looks for the first array literal in it that
    enumerates a type. One finding per function: a test that copies two sets by
    hand has one defect, not two, and reporting it twice would make the debt
    register disagree with itself the first time half of it was fixed.
    """
    found = []
    for top in ("crates", "tools"):
        base = os.path.join(root, top)
        if not os.path.isdir(base):
            continue
        for dirpath, _dirs, files in os.walk(base):
            for f in sorted(files):
                if not f.endswith(".rs"):
                    continue
                p = os.path.join(dirpath, f)
                rel = os.path.relpath(p, root).replace(BS, "/")
                src = io.open(p, encoding="utf-8", errors="replace").read()
                lines = src.split(NL)
                for i, ln in enumerate(lines):
                    m = FN.match(ln)
                    if not m:
                        continue
                    hit = _body_hit(lines, i, m.group(1))
                    if hit:
                        found.append((rel, hit[0], hit[1], hit[2], src))
    return sorted(found)


def _body_hit(lines, i, fn_name):
    """The first type-enumerating literal inside the function starting at `i`."""
    depth, started, end = 0, False, i
    for j in range(i, min(i + MAX_FN_LINES, len(lines))):
        depth += lines[j].count(OB) - lines[j].count(CB)
        if lines[j].count(OB):
            started = True
        end = j
        if started and depth <= 0:
            break

    j = i + 1
    while j <= end:
        code = lines[j].split("//")[0]
        if OPEN.search(code):
            d, buf = 0, []
            for k in range(j, min(end + 1, j + MAX_LIT_LINES)):
                c = lines[k].split("//")[0]
                buf.append(c)
                d += c.count(LB) - c.count(RB)
                if (k > j or LB in c) and d <= 0:
                    break
            counts = {}
            for pref, _var in PATH.findall(" ".join(buf)):
                counts[pref] = counts.get(pref, 0) + 1
            if counts:
                ty, n = max(sorted(counts.items()), key=lambda kv: kv[1])
                if n >= MIN_VARIANTS:
                    return (fn_name, ty, n)
        j += 1
    return None


def origins(root):
    """Every enum name DECLARED in this repository.

    Anything enumerated by hand that is not in this set comes from another
    crate — which is the severe case, because that set grows on a branch pin
    that moves without a `cargo update` and nothing on this side is touched.
    """
    names = set()
    for top in ("crates", "tools"):
        base = os.path.join(root, top)
        if not os.path.isdir(base):
            continue
        for dirpath, _dirs, files in os.walk(base):
            for f in files:
                if not f.endswith(".rs"):
                    continue
                src = io.open(os.path.join(dirpath, f), encoding="utf-8",
                              errors="replace").read()
                names.update(ENUM.findall(src))
    return names


def key(rel, fn):
    return rel + "::" + fn


def read_snapshot(path):
    """`path::fn <TAB> Type <TAB> ORIGIN <TAB> note` — comments and blanks skipped."""
    entries = {}
    if not os.path.isfile(path):
        return entries
    for ln in io.open(path, encoding="utf-8").read().split(NL):
        s = ln.strip()
        if not s or s.startswith("#"):
            continue
        parts = ln.split(TAB)
        entries[parts[0].strip()] = [p.strip() for p in parts[1:]]
    return entries


def report(root, snap_path, out):
    if not os.path.isdir(os.path.join(root, "crates")):
        out.append("completeness-tests: SKIPPED - no crates/ tree in " + root)
        return 2

    sites = scan(root)
    local = origins(root)
    snap = read_snapshot(snap_path)

    live = {}
    for rel, fn, ty, n, src in sites:
        org, real = origin_of(ty, src, local)
        live[key(rel, fn)] = (real, n, org)

    new = sorted(k for k in live if k not in snap)
    stale = sorted(k for k in snap if k not in live)

    rc = 0
    if new:
        out.append("completeness-tests: FAIL - " + str(len(new))
                   + " completeness test(s) carry a hand-written copy of a set:")
        for k in new:
            ty, n, org = live[k]
            out.append("    " + k)
            out.append("        enumerates " + ty + " by hand (" + str(n)
                       + " variants, " + org + ")")
        out.append("")
        out.append("The name promises the test goes red when the set grows. A private")
        out.append("copy of the set cannot. Iterate the authoritative enumerator -")
        out.append("`Type::all()`, a `strum` iterator, or an exhaustive `match` whose")
        out.append("arms the compiler counts for you - or, if the list really is a set")
        out.append("of INPUTS rather than a copy of a type, rename the test so its name")
        out.append("stops making a promise it does not keep.")
        rc = 1

    if stale:
        out.append("completeness-tests: FAIL - " + str(len(stale))
                   + " register line(s) match nothing in the tree:")
        for k in stale:
            out.append("    " + k)
        out.append("")
        out.append("Either the site was fixed - delete the line, the debt went down -")
        out.append("or it was renamed and the register stopped describing the tree. A")
        out.append("register that has quietly stopped matching is how an exemption")
        out.append("list outlives its premise.")
        rc = 1

    if rc:
        return rc

    foreign = sorted(k for k in live if live[k][2] == "FOREIGN")
    out.append("completeness-tests: clean - " + str(len(live))
               + " known site(s), none new, none stale.")
    out.append("                    OUTSTANDING DEBT: " + str(len(live))
               + " completeness tests still carry their own copy of a set,")
    out.append("                    " + str(len(foreign))
               + " of them copying a type this repository does not declare"
               + " (engine enums,")
    out.append("                    which grow on a branch pin that moves without"
               + " a cargo update).")
    return 0


# ---------------------------------------------------------------------------
# --self-test
# ---------------------------------------------------------------------------
def _plant(tmp, rs, snap_lines):
    crates = os.path.join(tmp, "crates", "x", "src")
    os.makedirs(crates, exist_ok=True)
    io.open(os.path.join(crates, "lib.rs"), "w", encoding="utf-8",
            newline="").write(rs)
    sp = os.path.join(tmp, "snapshot.txt")
    io.open(sp, "w", encoding="utf-8", newline="").write(NL.join(snap_lines) + NL)
    return sp


COPY_FN = (
    "enum Colour " + OB + " A, B, C " + CB + NL
    + "fn every_colour_is_named() " + OB + NL
    + "    let all = " + LB + "Colour::A, Colour::B, Colour::C" + RB + ";" + NL
    + CB + NL
)
INPUTS_FN = (
    "fn every_width_wraps() " + OB + NL
    + "    let widths = " + LB + "90.0, 70.0, 130.0" + RB + ";" + NL
    + CB + NL
)
PLAIN_FN = (
    "enum Colour " + OB + " A, B, C " + CB + NL
    + "fn two_colours_differ() " + OB + NL
    + "    let some = " + LB + "Colour::A, Colour::B, Colour::C" + RB + ";" + NL
    + CB + NL
)
FOREIGN_FN = (
    "fn all_errors_speak() " + OB + NL
    + "    let all = " + LB + "EditError::A, EditError::B, EditError::C" + RB + ";" + NL
    + CB + NL
)

SITE = "crates/x/src/lib.rs::every_colour_is_named"
FSITE = "crates/x/src/lib.rs::all_errors_speak"


def self_test():
    cases = [
        ("registered site is quiet", 0, COPY_FN, [SITE + TAB + "Colour" + TAB + "LOCAL"]),
        ("new hand-written set",     1, COPY_FN, []),
        ("stale register line",      1, INPUTS_FN, [SITE + TAB + "Colour" + TAB + "LOCAL"]),
        ("inputs are not a set",     0, INPUTS_FN, []),
        ("name makes no promise",    0, PLAIN_FN, []),
        ("foreign type is caught",   1, FOREIGN_FN, []),
    ]
    failures = 0
    for label, expect, rs, snap in cases:
        tmp = tempfile.mkdtemp(prefix="completeness-")
        sp = _plant(tmp, rs, snap)
        out = []
        rc = report(tmp, sp, out)
        if rc != expect:
            failures += 1
            print("completeness-tests --self-test: FAIL - '" + label
                  + "' returned " + str(rc) + ", expected " + str(expect))
            for ln in out:
                print("    " + ln)
    if failures:
        print("completeness-tests --self-test: FAIL - " + str(failures)
              + " of " + str(len(cases)) + " sabotages went undetected.")
        return 1
    print("completeness-tests --self-test: clean - all " + str(len(cases))
          + " cases behaved (registered, new, stale, inputs, plain name, foreign).")
    return 0


def main(argv):
    if "--self-test" in argv:
        return self_test()
    out = []
    rc = report(ROOT, SNAPSHOT, out)
    for ln in out:
        print(ln)
    if "--write-snapshot" in argv and rc == 1:
        sites = scan(ROOT)
        local = origins(ROOT)
        body = ["# completeness-snapshot.txt - DEBT REGISTER, not an exemption list.",
                "# Regenerate with: python tools/gates/check-completeness-tests.py --write-snapshot",
                "# Columns: path::fn <TAB> Type <TAB> LOCAL|FOREIGN <TAB> variants copied",
                ""]
        for rel, fn, ty, n, src in sites:
            org, real = origin_of(ty, src, local)
            body.append(key(rel, fn) + TAB + real + TAB + org + TAB + str(n))
        io.open(SNAPSHOT, "w", encoding="utf-8", newline="").write(NL.join(body) + NL)
        print("completeness-tests: snapshot rewritten with " + str(len(sites)) + " site(s).")
        return 0
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
