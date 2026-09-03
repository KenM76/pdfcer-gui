#!/usr/bin/env python3
"""check-trace-names.py — a module's own trace line must not share its first
token with an edit-funnel label.

===========================================================================
WHY THIS GATE EXISTS
===========================================================================

`tools/ui-verify` reads a trace by its FIRST TOKEN:

    pub fn last(&self, name: &str) -> Option<&TraceLine> {
        self.lines.iter().rev().find(|l| l.event == name)
    }

`app::actions::apply::vector_edit(doc, LABEL, ..)` writes one line per edit:

    <LABEL> page=0 n=1 epoch=7 disclosures=1

So a module that also writes its own summary line beginning with the same
word has produced two lines with one name — and `last(name)` returns the
FUNNEL's, which carries `page`, `n`, `epoch` and `disclosures` and none of the
keys the module's line was written to publish.

The failure mode is the worst shape a diagnostic can have: a driven check asks
for `name=` or `chars=`, finds nothing, and reports **"the verb did nothing"**
about a verb that worked perfectly. A confident false negative.

---------------------------------------------------------------------------
This has happened three times
---------------------------------------------------------------------------

  * `text-style`, 2026-08-27. Written up the same day.
  * `import-form-data`, 2026-08-28 — **by the session that had written up the
    first one**. Reading the note did not prevent it, because the note was
    about *text-style* rather than about every edit through the funnel.
  * `attach-file`, 2026-08-28, in code written hours after the second. Caught
    only because somebody sat down to write a driven check against it.

The agreed fix after the second instance was *"a naming convention at the point
of use — a module's summary takes a verb suffix, the funnel keeps the bare
name"*. A convention held by memory has now failed once per day.

  > An incident does not generalise itself. A grep does.

===========================================================================
WHAT IT CHECKS
===========================================================================

1. Every string literal passed to `vector_edit(..)` is a funnel LABEL.
2. Every `format!("<token> ...")` in the crate whose first token equals a label
   is a violation, EXCEPT the `vector_edit` call itself (which does not
   `format!` its label) and any line carrying `trace-name-exempt:`.

Suffixed names are fine and are the point: `attach-file-read`,
`move-annotation-applied`, `detach-file-requested` all pass, because the token
compared is the whole first word.

Exit 0 clean, 1 on a violation.
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = ROOT / "crates" / "pdfcer-gui" / "src"
EXEMPT = "trace-name-exempt:"

# `vector_edit(doc, "label", …)` — the label is the first string literal on the
# call, and every call site in this crate writes it inline.
LABEL = re.compile(r'vector_edit\s*\(\s*[A-Za-z_][A-Za-z0-9_]*\s*,\s*"([a-z0-9-]+)"')
# The first token of a format string: `format!("some-name key=…")`. Anchored to
# the literal's start, so an interpolated or mid-sentence occurrence is ignored.
FIRST_TOKEN = re.compile(r'format!\(\s*(?://[^\n]*\n\s*)*"([a-z][a-z0-9-]*)[ "]')


def main() -> int:
    files = sorted(SRC.rglob("*.rs"))
    blobs = {p: p.read_text(encoding="utf-8", errors="replace") for p in files}

    labels: set[str] = set()
    for text in blobs.values():
        labels.update(LABEL.findall(text))
    if not labels:
        print("check-trace-names: found no vector_edit labels — the pattern has moved.",
              file=sys.stderr)
        return 1

    violations = 0
    for path, text in blobs.items():
        starts = [0]
        for ch in text:
            starts.append(starts[-1] + 1)

        def line_of(offset: int, blob: str = text) -> int:
            """1-based line number of a byte offset.

            ★ The scan is over the WHOLE FILE rather than line by line, and this
            function is the price. It is worth paying: the first cut of this
            gate matched per line, and `format!(` sits on its own line above the
            literal in every multi-line trace in this crate — so a **planted
            violation passed**, which is exactly the failure this gate exists to
            catch, in the gate itself. Falsify a new check before believing it.
            """
            return blob.count("\n", 0, offset) + 1

        lines = text.split("\n")
        for match in FIRST_TOKEN.finditer(text):
            token = match.group(1)
            if token not in labels:
                continue
            n = line_of(match.start())
            window = "\n".join(lines[max(0, n - 9):n + 1])
            if EXEMPT in window:
                continue
            # The funnel's own call does not `format!` its label, so a match
            # inside a `vector_edit(..)` call is the label itself being quoted.
            if "vector_edit" in window:
                continue
            rel = path.relative_to(ROOT).as_posix()
            print(f"  {rel}:{n}: traces `{token}`, which is also an edit-funnel label")
            print(f"      {lines[n - 1].strip()[:110]}")
            violations += 1

    if violations:
        print(f"""
{violations} trace line(s) share a first token with a `vector_edit` label.

`tools/ui-verify` reads a trace by its first token, and `vector_edit` writes
`<label> page=… n=… epoch=…` for the same edit — so `.last(<label>)` returns the
FUNNEL's line, not yours, and a check asking for your keys finds none and
reports "the verb did nothing" about a verb that worked.

Give the module's own line a verb suffix: `<label>-read`, `<label>-applied`,
`<label>-requested`. The funnel keeps the bare name.

If two lines genuinely must share a name, say so on the line or in the comment
block above it with `{EXEMPT}` and the reason.
""")
        return 1

    print(f"check-trace-names: PASS - {len(labels)} funnel labels, no collisions.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
