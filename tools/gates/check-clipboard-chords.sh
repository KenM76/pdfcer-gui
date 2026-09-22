#!/usr/bin/env bash
# check-clipboard-chords.sh — Ctrl+C / Ctrl+X / Ctrl+V are EVENTS, never keys.
#
# ===========================================================================
# WHY THIS GATE EXISTS
# ===========================================================================
#
# `egui-winit` intercepts the three clipboard chords before they become key
# events. From `egui-winit-0.35.0/src/lib.rs`:
#
#     if is_cut_command(modifiers, active_key)   { events.push(Event::Cut);   return; }
#     if is_copy_command(modifiers, active_key)  { events.push(Event::Copy);  return; }
#     if is_paste_command(modifiers, active_key) { … events.push(Event::Paste(c)); return; }
#     events.push(Event::Key { … });
#
# THE `return` COMES BEFORE THE `Event::Key` PUSH. So in a real window:
#
#   * `Ctrl+C` produces `Event::Copy` and NO key event;
#   * `Ctrl+X` produces `Event::Cut` and NO key event;
#   * `Ctrl+V` produces `Event::Paste(contents)` and NO key event — and only
#     when the OS clipboard holds non-empty text. With an empty clipboard the
#     keystroke vanishes entirely.
#
# Consequently `InputState::key_pressed(Key::C)` — and the `X` and `V`
# equivalents, and any `Event::Key` pattern naming them — is **permanently
# false in the running application**. Code built on it compiles, reads
# correctly, passes a unit test that injects the key event, and never fires
# once for a real operator.
#
# ---------------------------------------------------------------------------
# The two defects that produced this file
# ---------------------------------------------------------------------------
#
# The operator, twice, three weeks apart: *"still no ctrl+c, ctrl+v, ctrl+x"*.
# On 2026-08-20 the chords were bound in the ribbon manifest, which was
# necessary and not sufficient, and `app::keyboard` was fixed to translate the
# three events through the keymap. It recorded the finding in capitals, in
# place, with the quotation above.
#
# DEFECT O18, 2026-08-21. Nobody asked who ELSE read the same broken signal.
# The answer was one grep away: `canvas::textsel::clipboard::pending_key` also
# asked `key_pressed(Key::C)`, so selecting text on the page and pressing
# Ctrl+C had never copied it — in any mode, since the day it was written. The
# object clipboard answered instead and wrote its marker, so the operator swept
# some text, pressed Ctrl+C, pasted into Notepad and read
# "1 object copied from pdfcer. Paste it back into pdfcer to place it."
#
# Its unit tests injected `Event::Key { key: C }` and passed throughout.
#
# ===========================================================================
# WHAT THIS GATE CHECKS, AND WHAT IT DELIBERATELY DOES NOT
# ===========================================================================
#
# It checks that no source file asks about `C`, `X` or `V` as a KEY. That is a
# textual property, and it is exactly the mistake that shipped twice.
#
# It cannot check that a handler exists, that it is reached, or that the
# clipboard ends up holding the right thing. Those need a driven run against a
# real window with a real clipboard, and R1 is unambiguous that the driven run
# is the thing that counts. This gate is the cheap half: it stops the specific
# wrong turn from being taken again, in a way a reviewer cannot forget to look
# for.
#
# ===========================================================================
# USAGE, THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ===========================================================================
#   tools/gates/check-clipboard-chords.sh [ROOT...]      (default: crates tools)
#   tools/gates/check-clipboard-chords.sh --self-test
#
#   0  no source file asks about C/X/V as a key, AND at least one was read
#   1  at least one does
#   2  NOTHING WAS SCANNED — a root holds no `.rs` file, or the search itself
#      errored. Not a pass: "no file asks" and "no file was read" produce the
#      same empty result set and must not produce the same line.
#
# `--self-test` builds synthetic roots and asserts nine arms, including the
# three that decide whether a silence is a finding: a comment quoting the
# broken form must pass, a marker fourteen lines above must silence, and the
# same marker one line further up must NOT.
# ===========================================================================

set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2

# How far above a hit the exemption marker may sit. Named because the self-test
# asserts both sides of it: a marker at the edge silences, one line beyond does
# not. A window nobody has measured either end of is a number, not a rule.
LOOKBACK=14

if [ "${1:-}" = "--self-test" ]; then
    SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
    TD="$(mktemp -d)"
    trap 'rm -rf "$TD"' EXIT
    fails=0

    arm() {   # arm <label> <expected-rc> <root>
        local got=0
        bash "$SELF" "$3" >/dev/null 2>&1 || got=$?
        if [ "$got" -eq "$2" ]; then
            printf '  ok    %-30s rc=%d\n' "$1" "$got"
        else
            printf '  FAIL  %-30s rc=%d, expected %d\n' "$1" "$got" "$2"
            fails=$((fails + 1))
        fi
    }

    mkdir -p "$TD/call" "$TD/pat" "$TD/comment" "$TD/online" "$TD/edge" \
             "$TD/beyond" "$TD/silent" "$TD/norust"

    printf 'fn f(i: &InputState) -> bool { i.key_pressed(egui::Key::C) }\n' \
        >"$TD/call/a.rs"
    arm "key_pressed(Key::C)" 1 "$TD/call"

    printf 'match e { Event::Key { key: Key::V, .. } => take(), _ => {} }\n' \
        >"$TD/pat/a.rs"
    arm "the Event::Key pattern" 1 "$TD/pat"

    # The header's own defence: files quote the broken form to explain it, and
    # a gate failing on the explanation deletes its own documentation.
    printf '%s\n' \
        '// Never write key_pressed(Key::C): winit returns before Event::Key.' \
        'fn f() {}' >"$TD/comment/a.rs"
    arm "a comment quoting it" 0 "$TD/comment"

    printf '%s\n' \
        'fn arm(i: &InputState) -> bool { i.key_pressed(Key::V) } // clipboard-chord-exempt: bare V arms the tool' \
        >"$TD/online/a.rs"
    arm "marker on the line" 0 "$TD/online"

    # Exactly at the window edge, and exactly one line beyond it. Two arms,
    # because a window is a claim about both sides and a single arm at the
    # edge passes just as well on a window of fifty.
    {
        echo '// clipboard-chord-exempt: bare V arms the tool'
        for i in $(seq 1 13); do echo "// filler $i"; done
        echo 'fn arm(i: &InputState) -> bool { i.key_pressed(Key::V) }'
    } >"$TD/edge/a.rs"
    arm "marker at the window edge" 0 "$TD/edge"

    {
        echo '// clipboard-chord-exempt: bare V arms the tool'
        for i in $(seq 1 14); do echo "// filler $i"; done
        echo 'fn arm(i: &InputState) -> bool { i.key_pressed(Key::V) }'
    } >"$TD/beyond/a.rs"
    arm "one line beyond the edge" 1 "$TD/beyond"

    printf 'fn f(i: &InputState) -> bool { i.key_pressed(Key::Delete) }\n' \
        >"$TD/silent/a.rs"
    arm "clean root passes" 0 "$TD/silent"

    # ★ The two arms that decide whether an empty result set is a finding.
    printf 'nothing here\n' >"$TD/norust/a.txt"
    arm "root with no .rs: not a pass" 2 "$TD/norust"
    arm "root that does not exist" 2 "$TD/absent"

    if [ "$fails" -ne 0 ]; then
        echo "check-clipboard-chords --self-test: FAIL — $fails arm(s) disagreed."
        echo "  Fix the gate. Both defects this file exists for were green in CI"
        echo "  and dead in the operator's hands; a gate that cannot fail is the"
        echo "  same condition wearing a tick."
        exit 1
    fi
    echo "check-clipboard-chords --self-test: ok — 9 arm(s): both broken forms"
    echo "  caught, a comment and an on-line marker exempt, the ${LOOKBACK}-line"
    echo "  lookback asserted at both edges, and an unread root exits 2"
    exit 0
fi

ROOTS=("$@")
[ "${#ROOTS[@]}" -eq 0 ] && ROOTS=(crates tools)

# ★ Count what was READ before counting what was found.
#
# An empty result set has two causes with opposite meanings — no file asks
# about the chords, or no file was looked at — and the search prints the same
# nothing for both. `grep -r` over a missing root, a mistyped `--include`, or a
# pattern the engine rejects all land here, and the gate's only output was its
# PASS line.
scanned=0
for r in "${ROOTS[@]}"; do
    [ -d "$r" ] || continue
    n=$(find "$r" -type f -name '*.rs' -not -path '*/target/*' 2>/dev/null | grep -c . || true)
    scanned=$((scanned + n))
done
if [ "$scanned" -eq 0 ]; then
    echo "check-clipboard-chords: SKIPPED — no .rs file under ${ROOTS[*]}." >&2
    echo "  Exiting 2, not 0: nothing was read, so nothing is clear." >&2
    exit 2
fi

violations=0
exempt_line=0
exempt_above=0
in_comment=0

# `key_pressed(...C)`, `key_down(...X)`, `key_released(...V)`, and the
# `Event::Key { key: ...C, ... }` pattern form. The `egui::` prefix is optional
# because a file may have imported `Key`.
pattern='key_(pressed|down|released)\(\s*(egui::)?Key::(C|X|V)\s*\)|key:\s*(egui::)?Key::(C|X|V)\b'

while IFS=: read -r file line text; do
    [ -z "${file:-}" ] && continue

    if [ "$file" = "SEARCH-FAILED" ]; then
        echo "check-clipboard-chords: SKIPPED — the search failed ($text)." >&2
        echo "  Exiting 2, not 0: a search that errored and a tree with no" >&2
        echo "  violations both produce no output lines." >&2
        exit 2
    fi

    trimmed="$(printf '%s' "$text" | sed 's/^[[:space:]]*//')"

    # Comments and doc comments are argument, not code. Several files quote the
    # broken form in order to explain why it is broken — including this one's
    # sibling in `app::keyboard` — and a gate that failed on the explanation
    # would delete its own documentation.
    case "$trimmed" in
        '//'*|'/*'*|'*'*) in_comment=$((in_comment + 1)); continue ;;
    esac

    # The exemption, same shape and same reasoning as check-typing-guard's:
    # on the line, or in the comment block directly above it, so the reason
    # stays in the file rather than becoming a bare marker.
    if printf '%s' "$text" | grep -q 'clipboard-chord-exempt:'; then
        exempt_line=$((exempt_line + 1))
        continue
    fi
    start=$((line > LOOKBACK ? line - LOOKBACK : 1))
    if sed -n "${start},$((line - 1))p" "$file" | grep -q 'clipboard-chord-exempt:'; then
        exempt_above=$((exempt_above + 1))
        continue
    fi

    echo "  $file:$line: asks about C/X/V as a KEY event"
    echo "      $trimmed"
    violations=$((violations + 1))
done < <(
    grep -rnE "$pattern" "${ROOTS[@]}" --include='*.rs'
    gs=$?
    # 0 matched, 1 matched nothing, >=2 the search itself failed. The third is
    # the one that reads as a clean tree, so it is turned into a line the loop
    # below cannot mistake for a hit and cannot ignore.
    [ "$gs" -ge 2 ] && printf 'SEARCH-FAILED:%d:grep exited %d\n' "$gs" "$gs"
    true
)

if [ "$violations" -gt 0 ]; then
    cat <<'MSG'

`Ctrl+C`, `Ctrl+X` and `Ctrl+V` never arrive as key events.

`egui-winit` intercepts all three and pushes `Event::Copy`, `Event::Cut` or
`Event::Paste(contents)`, returning BEFORE it would have pushed an
`Event::Key`. So `key_pressed(Key::C)` is permanently false in a real window,
and any handler built on it is dead code that a unit test will happily certify
by injecting the key event winit never sends.

Match the events instead:

    egui::Event::Copy           => …
    egui::Event::Cut            => …
    egui::Event::Paste(text)    => …

And write the test to inject THOSE. A test that injects
`Event::Key { key: Key::C }` is testing a keystroke the application cannot
receive — that is precisely how defect O18 stayed green for a day.

If you genuinely mean the letter key and not the chord — a `V` that arms a
tool, say, with no modifier — say so on the same line or in the comment block
directly above it with `clipboard-chord-exempt:` and the reason.
MSG
    echo
    echo "check-clipboard-chords: FAIL — $violations call site(s)."
    exit 1
fi

# The clean line carries what was read and what was let through, because those
# are the two numbers a reader needs in order to tell a real pass from a search
# that found nothing to look at. An exemption count that nobody prints is an
# allow-list that widens without a diff.
echo "check-clipboard-chords: PASS — $scanned .rs file(s) read; the three chords"
echo "                        are asked about as events, not keys."
echo "                        let through: $in_comment in a comment," \
     "$exempt_line marked on the line, $exempt_above marked within ${LOOKBACK} lines above"
exit 0
