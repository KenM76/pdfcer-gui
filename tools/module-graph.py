"""Measure a crate's internal module sizes and its cross-module reference graph.

The instrument behind `DESIGNS.md`'s crate-split section. Answers two questions
a split has to answer before it can be planned:

  - **How big is each top-level module?** Lines and code lines, so the payoff of
    moving one can be estimated against the measured lines-to-seconds curve.
  - **Which modules reference which?** Cargo forbids a dependency cycle between
    crates, so any pair that references each other BOTH ways cannot be split
    apart until one direction is cut. This prints the pairs.

Reference counts come from `crate::<module>` paths with whole-line comments
stripped first. Stripping matters: this crate is ~60% comments, and a module's
doc prose names its neighbours constantly. Counting prose would report a
dependency that does not exist, which is the failure mode that makes a source
grep look authoritative and be wrong.

`use super::…`, `use self::…` and re-exports are NOT counted, so a count is a
lower bound on coupling, never an upper one. A zero is therefore evidence; a
small number is not.

Usage:

    python tools/module-graph.py [crate-src-dir]     # default crates/pdfcer-gui/src
"""

import collections
import os
import re
import sys

SEP = chr(92)  # a literal backslash, spelled this way so no quoting layer eats it
CRATE_PATH = re.compile(r"crate::([a-z_][a-z0-9_]*)")
COMMENT_START = ("//", "/*", "*")


def top_level_modules(root: str) -> set[str]:
    """The crate's top-level module names, from the entries of its `src/`."""
    out = set()
    for entry in os.listdir(root):
        out.add(entry[:-3] if entry.endswith(".rs") else entry)
    return out - {"main", "lib"}


def strip_comments(text: str) -> str:
    """Drop whole-line comments. Not a parser — a trailing `// crate::foo` survives.

    That asymmetry is deliberate and safe in the direction that matters: it can
    only ever over-count, so a module reported as depending on nothing really
    depends on nothing.
    """
    return "\n".join(
        line for line in text.splitlines() if not line.strip().startswith(COMMENT_START)
    )


def main(argv: list[str]) -> int:
    root = argv[1] if len(argv) > 1 else os.path.join("crates", "pdfcer-gui", "src")
    if not os.path.isdir(root):
        print(f"module-graph: not a directory: {root}")
        return 1

    tops = top_level_modules(root)
    lines = collections.Counter()
    code = collections.Counter()
    files = collections.Counter()
    refs = collections.defaultdict(collections.Counter)

    for dirpath, _dirnames, filenames in os.walk(root):
        for name in filenames:
            if not name.endswith(".rs"):
                continue
            path = os.path.join(dirpath, name)
            rel = os.path.relpath(path, root).replace(SEP, "/")
            owner = rel.split("/")[0]
            if owner.endswith(".rs"):
                owner = owner[:-3]
            text = open(path, encoding="utf-8", errors="replace").read()
            body = text.splitlines()
            files[owner] += 1
            lines[owner] += len(body)
            code[owner] += sum(
                1
                for line in body
                if line.strip() and not line.strip().startswith(COMMENT_START)
            )
            for match in CRATE_PATH.finditer(strip_comments(text)):
                target = match.group(1)
                if target in tops and target != owner:
                    refs[owner][target] += 1

    print(f"{'module':16}{'files':>7}{'lines':>9}{'code':>8}   depends on")
    for mod, _n in code.most_common():
        top = ", ".join(f"{k}:{v}" for k, v in refs[mod].most_common(6))
        print(f"{mod:16}{files[mod]:>7}{lines[mod]:>9}{code[mod]:>8}   {top}")
    print(
        f"{'TOTAL':16}{sum(files.values()):>7}"
        f"{sum(lines.values()):>9}{sum(code.values()):>8}"
    )

    print("\nAcyclic floor — modules that reference no other module:")
    floor = sorted(m for m in code if not refs[m])
    print("  " + (", ".join(floor) if floor else "(none)"))

    print("\nMutually recursive pairs — these cannot become separate crates:")
    seen = set()
    cycles = 0
    for a in sorted(refs):
        for b in sorted(refs[a]):
            if a in refs.get(b, {}) and (b, a) not in seen:
                seen.add((a, b))
                cycles += 1
                print(f"  {a} <-> {b}   ({a}->{b}: {refs[a][b]}, {b}->{a}: {refs[b][a]})")
    if not cycles:
        print("  (none)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
