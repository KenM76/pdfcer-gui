#!/usr/bin/env python3
"""check-trace-names.py — a module's own trace line must not share its first
token with an edit-funnel label, read as a debugging leftover, break in two, or
be named in a doc comment while nothing emits it.

===========================================================================
THE PROPERTY ASSERTED
===========================================================================

Every diagnostic line this crate writes is addressable: it has a name no other
line answers to, a name somebody chose on purpose, and it arrives at a reader
in one piece.

All four mechanisms below defend the same downstream consumer. `tools/ui-verify`
reads a trace by its FIRST TOKEN:

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
about a verb that worked perfectly. A confident false negative. Mechanisms 3, 4
and 5 produce the identical symptom by different routes, which is why they live
in one gate.

  > An incident does not generalise itself. A grep does.

The reason this is a gate and not a convention: the convention ("a module's
summary takes a verb suffix, the funnel keeps the bare name") was agreed, and
then broken on consecutive days by the same sessions that had just written it
down — because a note about one collision reads as being about that collision
rather than about every edit through the funnel.

===========================================================================
MECHANISM 1 — A NAME THAT COLLIDES WITH A FUNNEL LABEL
===========================================================================

1. Every string literal passed to `vector_edit(..)` is a funnel LABEL. The set
   is collected across the whole crate first; an empty set is treated as the
   pattern having moved, not as a clean tree (see the exit contract).
2. Every `format!("<token> ...")` in the crate whose first token equals a label
   is a violation, EXCEPT the `vector_edit` call itself (which does not
   `format!` its label) and any line carrying `trace-name-exempt:`.

Suffixed names are fine and are the point: `attach-file-read`,
`move-annotation-applied`, `detach-file-requested` all pass, because the token
compared is the whole first word.

The scan is over whole file text rather than line by line, because `format!(`
sits on its own line above the literal in every multi-line trace here. See
`line_of` in `main` for why that cost is worth paying.

===========================================================================
MECHANISM 3 — A NAME THAT READS LIKE A DEBUGGING LEFTOVER
===========================================================================

Any uppercase letter in the token, or a `tmp`/`temp`/`dbg`/`debug`/`xxx`/
`todo`/`fixme`/`hack` prefix, is a violation in its own right.

★★ WHY MECHANISM 1 COULD NEVER HAVE FOUND IT. The shape this catches is a
release binary emitting

    pdfcer-diag TMPASK title="Open PDFs with pdfcer" now=4 opened_at=0

`FIRST_TOKEN` is anchored `[a-z]`, because every deliberate trace name in this
crate is lowercase-and-hyphens — so an all-caps scratch name never entered the
scan at all. The gate was not silent because the rule was weak; it was silent
because the name did not look like a trace name, which is exactly what makes a
leftover a leftover. Such a name survives a whole-repository rename and appears
in captured `ui-verify` traces that several sessions read without noticing.

This project's recorded rule is that a temporary shim needs a tripwire naming
its own deletion. A `TMP` prefix IS that tripwire — it is the author telling
the future this is not meant to stay. This is the thing that reads it.

Mechanism 3 is anchored at a `trace…(` call rather than applied to whole files,
and that anchor is load-bearing: applied file-wide it reported thirty-one hits
of which one was real, because `format!` in this crate also builds PDF content
streams (`BT /F1 12 Tf …`), operator-facing strings and test fixture labels,
all of which legitimately begin with a capital. A rule about TRACE names has to
find the traces first.

===========================================================================
MECHANISM 4 — A TRACE LITERAL THAT BREAKS INTO TWO LINES
===========================================================================

In Rust a SINGLE backslash at the end of a line inside a string literal is a
CONTINUATION: the newline and the following indentation are dropped and the
literal stays one line. That is how every long trace line in this crate is
written. TWO backslashes are a literal backslash, and the newline after it is
emitted verbatim.

A trace line is a RECORD, and `Trace::parse` keys on the `pdfcer-diag` prefix.
A record that breaks in the middle does not continue: the remainder is not a
trace line at all, every field after the break is invisible to every reader,
and the field the break lands in comes back with a stray backslash glued to it,
so a numeric field stops parsing. A driven check then reports that a thing did
not happen, while quoting a line that proves it did.

The doubling is a patch-script artefact — a backslash in a raw-string payload
that nothing decoded — so the tell is a pair of backslashes in the RAW literal
body, not in the emitted string, which only a running program has.

It also fires on a deliberate newline escape or a Windows path in a trace, and
that is wanted: a reader splits the trace on lines, so an escaped newline has
the same consequence as a real one, and a path in a trace should be
forward-slashed anyway.

★ ANCHORED, NOT SEARCHED, and this is the difference between a rule that can
fail and one that cannot. `TRACE_LITERAL` matches from the trace call itself
across a prefix that admits only whitespace, closure syntax, a slot-name string
and whole `//` comment lines — none of which can contain a `format!`. So an
anchored match either finds the literal belonging to THIS call or finds
nothing; it can never walk forward into an unrelated one, however long the
comment block above the literal runs. The first cut searched forward within a
fixed window instead, could not reach past a fifteen-line comment block, and
printed PASS over the live defect it had just been written for.

===========================================================================
MECHANISM 5 — A NAME DOCUMENTED IN A MODULE HEADER THAT NOTHING EMITS
===========================================================================

Every module here that writes traces opens with a sample block naming its
lines, and that block is where a harness author reads a name from:

    //! form-escape field=Name

If the emitter is renamed and the sample is not, the name in the header is a
name nothing answers to. `Trace::last("form-abandon")` returns `None`, and the
check reports *"the field editor did nothing"* about an editor that committed
the draft correctly — the same confident false negative as mechanisms 1, 3 and
4, arriving by a fourth route. It is the most tempting of the four to copy,
because a module header is written to be authoritative.

Measured when this was added: 26 documented sample names, one of them dead.

★★ THE OBVIOUS DESIGN IS UNSOUND AND WAS REJECTED. The first cut compared the
documented names against a CENSUS OF EMITTED NAMES — every first token of a
`format!` inside a trace call — and immediately accused `ui-rect`, which is
live. Names reach the channel by at least four routes: `format!`, a bare
`"...".to_owned()` literal, `eprintln!` in `diag`, and helpers like
`diag::ui_rect(name)` whose argument is a **runtime string that no static
census can enumerate at all**. An incomplete census does not merely miss
cases — it BLAMES LIVE NAMES, and a gate that fails on correct code is a gate
somebody switches off.

So the emitted side here is not a census. It is a plain substring search of all
non-comment source text in both crates: a documented name must appear
SOMEWHERE outside a comment. That is deliberately loose, and loose in the safe
direction — the mechanism fires only when a name appears nowhere at all, so its
errors are missed dead names rather than accusations against live ones.

Comments of every kind are excluded from the searched text, not just doc
comments. A name that survives only in an ordinary `//` note is not emitted
either, and excluding only `//!` and `///` would let a header's own duplicate
elsewhere in the file discharge it.

===========================================================================
WHAT IT PROVABLY CANNOT SEE
===========================================================================

* **Anything outside the two GUI crates' `src/**.rs`.** The trees are walked
  with `rglob`, so a trace written in `egui-shell`, in `tools/ui-verify`
  itself, or in any other crate is not examined. A name documented in one of
  these crates but emitted from `egui-shell` would be reported dead by
  mechanism 5; none exists today, and the fix would be to widen `SRCS` rather
  than to exempt the line.
* **A documented name with no `key=` field.** `DOC_SAMPLE` requires a
  `name key=` shape, because that is what makes a line in a doc comment
  unmistakably a trace sample rather than prose. A header that names a line
  bare is not checked.
* **A documented name that is a substring of an unrelated live one.** Per
  mechanism 5's note above, the emitted side is a substring search, so
  `form-escape` in a header is discharged by `form-escape-late` in code. The
  looseness is chosen, and it is the safe direction.
* **Whether a documented name's FIELDS are still emitted.** Mechanism 5 reads
  the first token only. A sample promising `field=` beside a line that no
  longer writes `field=` passes.
* **A trace built without `format!`.** A bare `"...".to_owned()` literal is not
  matched by any mechanism. Deliberate for mechanism 4 — such a literal is
  written in one piece and has no continuation to double — but it also means
  mechanisms 1 and 3 do not see those names.
* **A name assembled at runtime.** An interpolated or concatenated first token
  is invisible; `FIRST_TOKEN` and `ANY_TOKEN` are anchored to the literal's
  start.
* **A funnel label that is not an inline string literal** at the `vector_edit`
  call site. A label passed through a constant or a variable never enters
  `labels`, so a genuine collision with it goes unreported.
* **Whether an exemption is DESERVED.** `trace-name-exempt:` records a
  decision; nothing verifies the reason beside it.
* **Whether the name is a GOOD one.** Mechanism 3 tests the shape of the token,
  not its meaning: `zzz-thing` is lowercase-and-hyphens and passes.
* **A collision between two module lines** that share a first token with each
  other but not with any funnel label. The claim is about the funnel.

===========================================================================
USAGE AND EXIT CODES
===========================================================================

  tools/gates/check-trace-names.py              scan the crate
  tools/gates/check-trace-names.py --self-test  falsify all five mechanisms

  0  clean  — every trace name is unique against the funnel, deliberate,
              emitted on one line, and emitted at all
  1  FAIL   — one or more violations of any mechanism, each printed with
              `file:line` and the offending source line

There is deliberately **no SKIPPED state**. This gate reads a fixed path in
this repository rather than a git query, so "no input" is not a legitimate
condition here: an empty label set means the `vector_edit` pattern moved and
mechanism 1 has gone blind, and an empty documented-name set means the header
sample convention moved and mechanism 5 has. Both are reported on stderr and
exit 1. A gate whose pattern has stopped matching must not print what a clean
run prints.

===========================================================================
HOW TO FALSIFY IT
===========================================================================

`--self-test` plants inputs for all five mechanisms and asserts each fires on
its own defect AND declines the legitimate construct it would otherwise flood:
a multi-line `format!` (the per-line regression mechanism 1 already suffered
once), a `vector_edit` label, the all-caps leftover, an ordinary trace name, a
PDF content stream, a doubled continuation, a correct single continuation, a
literal carrying Debug-quoted fields, a literal under a 400-character comment
block, a `trace_on_change` slot name, a documented name emitted nowhere, a
documented name whose emitter is present, and a prose line in a doc comment.
It exits 0 only if every one behaves.

Both halves matter equally here: four of these mechanisms are regex-shaped,
and a regex that stops matching is silent rather than loud.

★ A self-test is not by itself evidence for mechanism 5, because a self-test
only proves the mechanism fires on the input the same author planted. It was
also falsified against the real tree: run over the sources at the commit
BEFORE the dead name was fixed, it reported `form-abandon` at the right file
and line and cleared the other 25; run over the tree after, it reported none.
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
# BOTH GUI CRATES. `diag` — the trace facility the names in question are
# emitted through — lives in `pdfcer-gui-base`, and a root list naming only
# `pdfcer-gui` would stop seeing its call sites without failing: the gate
# would find nothing and report clean.
SRCS = [
    ROOT / "crates" / "pdfcer-gui" / "src",
    ROOT / "crates" / "pdfcer-gui-base" / "src",
]
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
# ★ A trace sample inside a doc comment: `//! form-escape field=Name`.
#
# The trailing `key=` is load-bearing. Without it the pattern is "a hyphenated
# word in a doc comment", which matches English prose and would make mechanism
# 5 a rule about writing rather than about traces. At least one hyphen is
# required for the same reason.
DOC_SAMPLE = re.compile(r"^[ " + chr(9) + r"]*//[!/][ " + chr(9)
                        + r"]*([a-z][a-z0-9]*(?:-[a-z0-9]+)+)[ " + chr(9)
                        + r"]+[a-z_]+=")
# A whole line that is nothing but a comment, of any kind. Mechanism 5 strips
# these before searching for a name's emitter; see `emitting_text`.
ANY_COMMENT = re.compile(r"^[ " + chr(9) + r"]*//")
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


def doc_sample_name(line: str) -> str | None:
    """The trace name a doc-comment line publishes as a sample, if it is one.

    A module header's sample block is where a harness author reads a name from:

        //! form-escape field=Name

    The `key=` is what makes the shape unmistakable. Prose in a doc comment is
    full of hyphenated words, and a rule keyed on "a hyphenated token in a doc
    comment" would be a rule about English rather than about traces — this
    project's recorded failure mode *a gate keyed on a name is discharged by
    prose*, in reverse.

    ★ A pure function of one line, so the self-test can falsify it without a
    filesystem — the half of a checker that most often goes untested and
    therefore most often cannot fail.
    """
    match = DOC_SAMPLE.match(line)
    return match.group(1) if match else None


def emitting_text(text: str) -> str:
    """``text`` with every comment line removed: what may discharge a name.

    Doc comments are excluded because a header discharging its own sample is
    the whole defect. Ordinary `//` comments are excluded too — a name that
    survives only in a note beside the code it used to describe is not emitted
    either, and keeping them would let a duplicate mention elsewhere in the
    same file stand in for an emitter.

    Only WHOLE comment lines go. A trailing `// …` after code is rare here and
    dropping the code with it would manufacture a false positive, which is the
    one direction mechanism 5 must not err in.
    """
    return NL.join(
        line for line in text.split(NL) if not ANY_COMMENT.match(line)
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
    """Plant a violation per mechanism, and the legitimate construct it must not flag.

    ★★ A gate is evidence only if it has been made to fail. Three of these
    mechanisms are regex-shaped, and a regex that stops matching is silent
    rather than loud: `FIRST_TOKEN` already has one recorded near-miss in its
    own history (a per-line scan that let a multi-line `format!(` through), and
    mechanism 3 exists because an anchor nobody questioned made a whole class
    of name invisible.

    So the planted inputs below come in pairs, in mechanism order: each
    mechanism must fire on its own defect, and must decline the correct
    construct that looks most like it — an ordinary trace name and a PDF
    content stream for mechanism 3, a single-backslash continuation, a
    Debug-quoted literal, a literal under a long comment block and a
    `trace_on_change` slot name for mechanism 4. A one-directional self-test
    would pass for a checker that answers "yes" unconditionally.
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
    # --- mechanism 5 -----------------------------------------------------
    #
    # The planted input is the real defect, reduced: a module header naming
    # `form-abandon` beside an emitter that says `form-escape`.
    stale_header = (
        "//! form-abandon field=Name" + NL
        + 'crate::diag::trace(|| format!("form-escape field={f}"));'
    )
    names = [n for n in map(doc_sample_name, stale_header.split(NL)) if n]
    assert names == ["form-abandon"], (
        "mechanism 5 did not read the sample name out of a module header, so "
        "it would have nothing to check and would report clean on every tree"
    )
    assert names[0] not in emitting_text(stale_header), (
        "mechanism 5 did not see a documented name that nothing emits -- the "
        "exact defect it was written for, which makes a driven check report "
        "the verb did nothing about a verb that worked"
    )
    live_header = (
        "//! form-escape field=Name" + NL
        + 'crate::diag::trace(|| format!("form-escape field={f}"));'
    )
    assert "form-escape" in emitting_text(live_header), (
        "mechanism 5 calls a LIVE documented name dead -- it would fail the "
        "build on correct code, which is how a gate gets switched off"
    )
    # ★ The half that makes the emitted side sound. The header line itself must
    # NOT discharge the name, or the mechanism answers "emitted" unconditionally
    # and is a checker that cannot fail.
    assert "form-escape" not in emitting_text("//! form-escape field=Name"), (
        "mechanism 5 lets a doc comment discharge its own sample, so every "
        "documented name would look emitted and nothing could ever be found"
    )
    # ★ Lowercase, hyphenated, and in a doc comment, so it clears every part of
    # the pattern EXCEPT the trailing `key=`. Prose beginning with a capital
    # would be declined by a pattern that had no `key=` requirement at all, and
    # would therefore have tested nothing.
    assert doc_sample_name("//! read-only until fold-in day, never edited.") is None, (
        "mechanism 5 reads ordinary prose as a trace sample -- a rule about "
        "traces that is really a rule about English, which this project has "
        "already been bitten by in the other direction"
    )
    print("check-trace-names: SELF-TEST PASS - 16 planted inputs, all as expected.")
    return 0


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return self_test()
    files = sorted(p for src in SRCS if src.is_dir() for p in src.rglob("*.rs"))
    blobs = {p: p.read_text(encoding="utf-8", errors="replace") for p in files}

    labels: set[str] = set()
    for text in blobs.values():
        labels.update(LABEL.findall(text))
    if not labels:
        print("check-trace-names: found no vector_edit labels — the pattern has moved.",
              file=sys.stderr)
        return 1

    # Mechanism 5's two sides, both collected across the WHOLE tree before
    # anything is judged. It is the one mechanism here that is not per-file: a
    # header in one module routinely documents a line emitted from another, so
    # a per-file comparison would accuse every such name.
    documented: dict[str, list[str]] = {}
    emitters: list[str] = []
    # Counted BEFORE the exemption filter, and that is what the pattern-moved
    # test below reads. An exempted sample still proves `DOC_SAMPLE` matches,
    # so a tree that exempted every one of them must not be reported as a tree
    # whose convention moved — a refusal sentence that names the wrong cause
    # sends the next reader to the wrong place.
    samples_seen = 0
    for path, text in blobs.items():
        rel = path.relative_to(ROOT).as_posix()
        doc_lines = text.split("\n")
        for n, line in enumerate(doc_lines, 1):
            name = doc_sample_name(line)
            if name is None:
                continue
            samples_seen += 1
            # Same exemption window as every other mechanism: the line itself
            # or the comment block above it.
            if EXEMPT in "\n".join(doc_lines[max(0, n - 9):n]):
                continue
            documented.setdefault(name, []).append(f"{rel}:{n}")
        emitters.append(emitting_text(text))
    if not samples_seen:
        print("check-trace-names: found no documented trace samples — the module "
              "header convention has moved.", file=sys.stderr)
        return 1
    corpus = "\n".join(emitters)

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

    # Mechanism 5: a documented name must appear somewhere outside a comment.
    # Not per-file — see the collection pass.
    dead = 0
    for name in sorted(documented):
        if name in corpus:
            continue
        for where in documented[name]:
            print(f"  {where}: documents `{name}`, which nothing in either crate emits")
        dead += 1

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

    if dead:
        print(f"""
{dead} trace name(s) are documented in a module header and emitted by nothing.

A module header's sample block is where a harness author reads a name from, and
it is written to be authoritative. When the emitter is renamed and the sample
is not, `Trace::last(<name>)` returns None and the driven check reports *"the
verb did nothing"* about a verb that worked — the same confident false negative
mechanisms 1, 3 and 4 exist for, by a fourth route.

Measured on the tree this was added to: `form-abandon` outlived the line it
named. The emitter had said `form-escape` since Escape was changed to COMMIT a
form draft rather than discard it, and the header still taught the old name.

Fix the HEADER to match what the code emits — the code is the thing that runs.
If the name is genuinely emitted somewhere this gate cannot see (another crate;
built from a runtime string), say so on the line or in the comment block above
it with `{EXEMPT}` and where it is emitted from.
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

    if scratch or split or dead:
        return 1

    print(
        f"check-trace-names: PASS - {len(labels)} funnel labels, "
        f"{len(documented)} documented names, no collisions, no scratch names, "
        f"no split lines, none documented-but-unemitted."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
