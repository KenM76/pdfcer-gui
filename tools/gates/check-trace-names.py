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

# ★ Assembled from `chr()` rather than written inline, and deliberately: the
# defect mechanism 4 catches is a backslash that did not survive a patch
# script's quoting, and a gate written the way the bug was written is a gate
# that can acquire the same bug.
BS = chr(92)
NL = chr(10)
WS = "[ " + chr(9) + chr(13) + NL + "]*"

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
# What may sit between a trace call and its format literal, and nothing else.
#
# This is what lets mechanism 4 be ANCHORED at the call rather than searching
# forward from it, which is the difference between a rule that cannot fail and
# one that can:
#
#   * whitespace;
#   * closure syntax -- `|`, `{`, and the `,` after a slot name;
#   * a simple double-quoted string, which is `trace_on_change`'s slot name;
#   * whole `//` comment lines, of which the literal this was written for has
#     fifteen above it.
#
# None of those can contain a `format!`, so an anchored match either finds the
# literal belonging to THIS call or finds nothing -- it can never walk forward
# into an unrelated one. A larger `TRACE_WINDOW` would have traded this
# mechanism's false negative for a false positive; the anchor trades it for
# nothing.
#
# A trace whose closure does not use `format!` (a bare `"...".to_owned()`) is
# simply not matched, and so is not checked. That is deliberate: such a literal
# is written in one piece and has no continuation to double.
TRACE_PREFIX = (
    "(?:[ " + chr(9) + chr(13) + NL + "|{,]"
    + '|"[^"' + NL + ']*"'
    + "|//[^" + NL + "]*" + NL
    + ")*"
)
# Prefixes an author uses to tell the future the name is not meant to stay.
# `test` is deliberately NOT here: it is an ordinary English word and a trace
# about a self-test would be a legitimate use of it, which is this project's
# recorded failure mode *a gate keyed on a name is discharged by prose* in
# reverse — a rule so broad it has to be exempted becomes a rule nobody trusts.
SCRATCH_PREFIXES = ("tmp", "temp", "dbg", "debug", "xxx", "todo", "fixme", "hack")
# ★★ The whole BODY of a trace call's format literal -- escapes, continuations
# and Debug-quoted fields included.
#
# Mechanisms 1 and 3 want the first token; mechanism 4 wants everything, because
# what it looks for can sit anywhere in the line. The alternation is ordered so
# that an escape pair is consumed before a bare character, which is what stops a
# `"` inside the literal (`name={n:?}` prints one) from ending the match early.
#
# Built by concatenation for the same reason `BS` is: a regex about backslashes,
# written with backslashes, is a thing a patch script can silently corrupt.
#
# WARNING: ANCHORED, not searched. See `TRACE_PREFIX` and the note on it -- the
# first cut of this searched forward within `TRACE_WINDOW` and could not fail.
TRACE_LITERAL = re.compile(
    TRACE_PREFIX
    + "format!"
    + re.escape("(")
    + WS
    + "(?://[^" + NL + "]*" + NL + WS + ")*"
    + '"((?:[^"' + BS + BS + "]|" + BS + BS + "(?:.|" + NL + "))*)" + '"'
)


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


def split_across_lines(body: str) -> bool:
    """Whether a trace literal would emit a literal backslash, breaking the line.

    In Rust source a SINGLE backslash at the end of a line is a
    **continuation**: the newline and the following indentation are dropped and
    the literal stays one line. That is how every long trace line in this crate
    is written. TWO backslashes are a literal backslash, and whatever follows --
    a real newline, in the case this was written for -- is emitted verbatim.

    So the tell is a pair of backslashes, and the check is on the RAW literal
    body rather than on the emitted string, because the emitted string is
    something only a running program has.

    It also fires on a deliberate newline escape or a Windows path in a trace,
    and that is wanted: a reader splits the trace on lines, so an escaped
    newline has exactly the same consequence as a real one, and a path in a
    trace should be forward-slashed anyway.

    ★ A pure function of the body, so the self-test can falsify it without a
    filesystem -- the half of a checker that most often goes untested and
    therefore most often cannot fail.
    """
    return BS + BS in body


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
    # --- mechanism 4 -----------------------------------------------------
    #
    # The planted input is the real defect, reduced: a trace literal whose
    # continuation backslash was doubled by a patch script.
    broken = (
        'crate::diag::trace(|| format!("scale-seeded a={x} ratio=100 '
        + BS + BS + NL + '     basis={b}"));'
    )
    hit = TRACE_LITERAL.match(broken, broken.index("(") + 1)
    assert hit is not None, "mechanism 4 could not find the format literal at all"
    bodies = [hit.group(1)]
    assert split_across_lines(bodies[0]), (
        "mechanism 4 did not see a doubled continuation -- the exact defect it "
        "was written for, which made a driven check report the opposite of the "
        "truth while quoting the truth in its own message"
    )
    good = (
        'crate::diag::trace(|| format!("scale-seeded a={x} ratio=100 '
        + BS + NL + '     basis={b}"));'
    )
    hit = TRACE_LITERAL.match(good, good.index("(") + 1)
    assert hit is not None, "mechanism 4 lost the literal on a CORRECT continuation"
    bodies = [hit.group(1)]
    assert not split_across_lines(bodies[0]), (
        "mechanism 4 flags an ordinary Rust line continuation, which is how "
        "every long trace line in this crate is written -- it would fail the "
        "build on correct code, which is how a gate gets switched off"
    )
    quoted = 'crate::diag::trace(|| format!("scale-seeded name={n:?} unit={u:?}"));'
    hit = TRACE_LITERAL.match(quoted, quoted.index("(") + 1)
    assert hit is not None and not split_across_lines(hit.group(1)), (
        "mechanism 4 mis-parses a literal carrying Debug-quoted fields, which "
        "most trace lines in this crate do"
    )
    # The regression that made the first cut of mechanism 4 unable to fail: a
    # long explanatory comment block between the call and its literal. The real
    # one is fifteen lines; 400 characters is enough to break a windowed
    # search, and the anchored form does not care how long it is.
    commented = (
        "crate::diag::trace(|| {" + NL
        + "    // ui-text-exempt: diagnostic trace." + NL
        + "    //" + NL
        + "    // " + ("x" * 400) + NL
        + '    format!("scale-seeded a={x} ' + BS + BS + NL + '        b={y}")' + NL
        + "});"
    )
    hit = TRACE_LITERAL.match(commented, commented.index("(") + 1)
    assert hit is not None, (
        "mechanism 4 cannot reach a literal under a long comment block -- the "
        "exact shape it failed on when first written, where it printed PASS "
        "over the live defect it had just been written for"
    )
    assert split_across_lines(hit.group(1)), (
        "mechanism 4 reached the literal under a comment block and did not see "
        "the doubled continuation in it"
    )
    slot = 'crate::diag::trace_on_change("dialog-focus", || format!("dialog-focus x={x}"));'
    hit = TRACE_LITERAL.match(slot, slot.index("(") + 1)
    assert hit is not None and not split_across_lines(hit.group(1)), (
        "mechanism 4 cannot reach past a `trace_on_change` slot name, so every "
        "line written through that helper would go unchecked in silence"
    )
    print("check-trace-names: SELF-TEST PASS - 11 planted inputs, all as expected.")
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
    split = 0
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

        # Mechanism 4: the same trace calls, but reading the whole literal.
        # A record that breaks across two physical lines has lost every field
        # after the break, and no reader can tell that from a field that was
        # never written.
        for call in TRACE_CALL.finditer(text):
            # `.match`, not `.search`: anchored at the call, so the literal
            # found is the one belonging to it however long the comment block
            # above it runs. See TRACE_PREFIX for why no window is needed here
            # and why one would be wrong.
            match = TRACE_LITERAL.match(text, call.end())
            if match is None or not split_across_lines(match.group(1)):
                continue
            # `start(1)` -- the LITERAL's offset, not the call's. The match is
            # anchored at the call, which on a multi-line trace can be fifteen
            # comment lines above the thing that is wrong; a reader sent to the
            # closure header has to find the defect themselves.
            n = line_of(match.start(1))
            window = "\n".join(lines[max(0, n - 9):n + 2])
            if EXEMPT in window:
                continue
            rel = path.relative_to(ROOT).as_posix()
            print(f"  {rel}:{n}: this trace literal emits a literal backslash")
            print(f"      {lines[n - 1].strip()[:110]}")
            split += 1

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

    if split:
        print(f"""
{split} trace literal(s) would emit a literal backslash, breaking the line.

A trace line is a RECORD. `Trace::parse` keys on the `pdfcer-diag` prefix, so a
line that breaks in the middle does not continue -- the remainder is not a trace
line at all, every field after the break is invisible to every reader, and the
field the break lands in comes back with a stray backslash glued to it, so a
numeric field stops parsing.

Measured on 2026-09-13: `scale-seeded` lost `basis` and `unit` this way and
returned `ratio_real` as an unparseable string. The driven check reading it
reported *"the window was not seeded from the document at all"* about a window
that had been seeded correctly -- a confident false negative, which is the same
failure shape mechanism 1 exists for.

In Rust a SINGLE backslash before a newline is a continuation and is how every
long trace line in this crate is written. Two is a literal backslash. If the
doubling came from a patch script, its payload was a raw string -- this
project's standing note on that is in its agent memory.

If a backslash genuinely belongs in the line, say so with `{EXEMPT}`.
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

    if scratch or split:
        return 1

    print(
        f"check-trace-names: PASS - {len(labels)} funnel labels, no collisions, "
        f"no scratch names, no split lines."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
