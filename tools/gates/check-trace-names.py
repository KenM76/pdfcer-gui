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

3. ★★ **A trace name that reads like a debugging leftover is a violation** —
   any uppercase letter in the token, or a `tmp`/`temp`/`dbg`/`debug`/`xxx`/
   `todo`/`fixme`/`hack` prefix.

===========================================================================
WHY MECHANISM 3 EXISTS — and why mechanism 1 could never have found it
===========================================================================

Found 2026-09-11, in a **release** binary, by an ordinary off-screen smoke
launch before a release:

    pdfcer-diag TMPASK title="Open PDFs with pdfcer" now=4 opened_at=0

`TMPASK` is the shape of a name somebody types while chasing a focus bug and
means to take out again. It had survived long enough to be swept through this
project's own rename, and it appears eight times per dialog in captured
`ui-verify` traces that several sessions have read.

⇒ **This gate was structurally incapable of seeing it.** `FIRST_TOKEN` is
anchored `[a-z]`, because every deliberate trace name in this crate is
lowercase-and-hyphens — so an all-caps scratch name did not even enter the
scan. The gate was not silent because the rule was weak; it was silent
because the name did not look like a trace name, which is exactly what makes a
leftover a leftover.

This project's recorded rule is that a temporary shim needs a tripwire naming
its own deletion. A `TMP` prefix IS that tripwire — it is the author telling
the future this is not meant to stay. Nothing was reading it. Now something is.

Exit 0 clean, 1 on a violation. `--self-test` falsifies both mechanisms
against planted inputs and exits 0 only if each one fires.
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
# The same shape as FIRST_TOKEN but case-blind and admitting `_`, so a token
# `FIRST_TOKEN` is anchored away from can be seen at all. Mechanism 3 needs
# exactly the names mechanism 1 is written to ignore.
ANY_TOKEN = re.compile(r'format!\(\s*(?://[^\n]*\n\s*)*"([A-Za-z][A-Za-z0-9_-]*)[ "]')
# ★★ A `crate::diag::trace…(` call, which is what makes mechanism 3 tractable.
#
# The first cut of mechanism 3 applied ANY_TOKEN to the whole file and reported
# **thirty-one** hits, of which one was real. `format!` in this crate also
# builds PDF content streams (`BT /F1 12 Tf …`), operator-facing strings
# (`Version 0.5.0`), and test fixture labels (`Stamp 3`) — all of which
# legitimately begin with a capital. ⇒ A rule about TRACE names has to find
# the traces first. This is the anchor that does it, and the window below is
# deliberately short so the `format!` it picks up is the one inside the call.
TRACE_CALL = re.compile(r"\btrace(?:_on_change|_changed)?\s*\(")
# How far past a trace call to look for its format literal. Measured rather
# than guessed: the longest gap in this crate is a `trace_on_change` whose slot
# name, closure header and two exemption comments precede the literal.
TRACE_WINDOW = 400
# Prefixes an author uses to tell the future the name is not meant to stay.
# `test` is deliberately NOT here: it is an ordinary English word and a trace
# about a self-test would be a legitimate use of it, which is this project's
# recorded failure mode *a gate keyed on a name is discharged by prose* in
# reverse — a rule so broad it has to be exempted becomes a rule nobody trusts.
SCRATCH_PREFIXES = ("tmp", "temp", "dbg", "debug", "xxx", "todo", "fixme", "hack")


def looks_like_scratch(token: str) -> bool:
    """Whether ``token`` reads as a debugging leftover rather than a trace name.

    Two independent tells, either sufficient:

    * **Any uppercase letter.** Every deliberate trace name in this crate is
      lowercase-and-hyphens, without exception — the convention is what lets
      `ui-verify` match on a first token at all. An uppercase letter is
      therefore not a style disagreement; it is a name written in a hurry.
      This is the tell that would have caught `TMPASK`.
    * **A scratch prefix.** `tmpask` in lowercase is just as much a leftover,
      and an author who types one usually types the prefix deliberately.

    ★ It is a pure function of the token so the self-test can falsify it
    without a filesystem, which is the half of a checker that most often goes
    untested and therefore most often cannot fail.
    """
    if any(ch.isupper() for ch in token):
        return True
    return token.lower().startswith(SCRATCH_PREFIXES)


def self_test() -> int:
    """Plant one violation per mechanism and assert each is seen.

    ★★ A gate is evidence only if it has been made to fail. Both of these
    mechanisms are regex-shaped, and a regex that stops matching is silent
    rather than loud: `FIRST_TOKEN` already has one recorded near-miss in its
    own history (a per-line scan that let a multi-line `format!(` through), and
    mechanism 3 exists because an anchor nobody questioned made a whole class
    of name invisible. So the four assertions below are, in order: each
    mechanism fires on its own planted input, and each declines a legitimate
    one.
    """
    multiline = 'format!(\n    "attach-file page={p}"\n)'
    assert FIRST_TOKEN.findall(multiline) == ["attach-file"], (
        "mechanism 1 stopped seeing a multi-line format! — the exact regression "
        "its own line_of() doc records"
    )
    assert LABEL.findall('vector_edit(doc, "attach-file", x)') == ["attach-file"], (
        "mechanism 1 stopped seeing a vector_edit label, so `labels` would come "
        "back empty and every collision would pass"
    )
    def scratch_hits(blob: str) -> list[str]:
        """Mechanism 3 exactly as `main` runs it, over one blob."""
        out = []
        for call in TRACE_CALL.finditer(blob):
            hit = ANY_TOKEN.search(blob, call.end(), call.end() + TRACE_WINDOW)
            if hit and looks_like_scratch(hit.group(1)):
                out.append(hit.group(1))
        return out

    leftover = (
        'crate::diag::trace(|| {\n'
        '    // ui-text-exempt: diagnostic trace, never displayed.\n'
        '    format!(\n        "TMPASK title={t} now={n}"\n    )\n});'
    )
    assert scratch_hits(leftover) == ["TMPASK"], (
        "mechanism 3 did not see the real leftover it was written for"
    )
    real = 'crate::diag::trace_on_change("dialog-focus", || format!("dialog-focus x={x}"));'
    assert scratch_hits(real) == [], (
        "mechanism 3 rejects an ordinary trace name — it would fail the build "
        "on correct code, which is how a gate gets switched off"
    )
    # ★ The one that justifies the TRACE_CALL anchor. Applied to whole files,
    # the first cut of this mechanism reported 31 hits of which one was real:
    # `format!` here also writes PDF content streams and operator copy, both of
    # which legitimately begin with a capital.
    content_stream = 'let content = format!("BT /F1 12 Tf 20 100 Td ({text}) Tj ET");'
    assert scratch_hits(content_stream) == [], (
        "mechanism 3 flags a PDF content stream — the false-positive flood that "
        "made the first cut of this rule unusable"
    )
    print("check-trace-names: SELF-TEST PASS - 5 planted inputs, all as expected.")
    return 0


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return self_test()
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
    scratch = 0
    for path, text in blobs.items():

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

        # Mechanism 3: only the format literal that belongs to a trace call,
        # and only its first token.
        for call in TRACE_CALL.finditer(text):
            match = ANY_TOKEN.search(text, call.end(), call.end() + TRACE_WINDOW)
            if match is None:
                continue
            token = match.group(1)
            if not looks_like_scratch(token):
                continue
            n = line_of(match.start())
            window = "\n".join(lines[max(0, n - 9):n + 1])
            if EXEMPT in window:
                continue
            rel = path.relative_to(ROOT).as_posix()
            print(f"  {rel}:{n}: traces `{token}`, which reads as a debugging leftover")
            print(f"      {lines[n - 1].strip()[:110]}")
            scratch += 1

    if scratch:
        print(f"""
{scratch} trace name(s) read as a debugging leftover rather than a diagnostic.

Every deliberate trace name in this crate is lowercase-and-hyphens, because
`ui-verify` matches a line by its first token and that convention is what makes
the match reliable. An uppercase letter or a `tmp`/`debug`/`xxx` prefix is the
shape of a name typed while chasing a bug and meant to come out again.

On 2026-09-11 `TMPASK` was found in a RELEASE binary by a smoke launch — it had
survived a whole-repository rename and appeared eight times per dialog in
captured traces several sessions had read. A `TMP` prefix is the author's own
tripwire; this is the thing that reads it.

Either give the line a real name and document what it measures, or delete it.
If it genuinely must keep the name, say so on the line or in the comment block
above it with `{EXEMPT}` and the reason.
""")

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

    if scratch:
        return 1

    print(
        f"check-trace-names: PASS - {len(labels)} funnel labels, no collisions, "
        f"no scratch names."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
