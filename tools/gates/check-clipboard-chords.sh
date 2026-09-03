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

set -uo pipefail
cd "$(dirname "$0")/../.." || exit 2

violations=0

# `key_pressed(...C)`, `key_down(...X)`, `key_released(...V)`, and the
# `Event::Key { key: ...C, ... }` pattern form. The `egui::` prefix is optional
# because a file may have imported `Key`.
pattern='key_(pressed|down|released)\(\s*(egui::)?Key::(C|X|V)\s*\)|key:\s*(egui::)?Key::(C|X|V)\b'

while IFS=: read -r file line text; do
    [ -z "${file:-}" ] && continue

    trimmed="$(printf '%s' "$text" | sed 's/^[[:space:]]*//')"

    # Comments and doc comments are argument, not code. Several files quote the
    # broken form in order to explain why it is broken — including this one's
    # sibling in `app::keyboard` — and a gate that failed on the explanation
    # would delete its own documentation.
    case "$trimmed" in
        '//'*|'/*'*|'*'*) continue ;;
    esac

    # The exemption, same shape and same reasoning as check-typing-guard's:
    # on the line, or in the comment block directly above it, so the reason
    # stays in the file rather than becoming a bare marker.
    if printf '%s' "$text" | grep -q 'clipboard-chord-exempt:'; then
        continue
    fi
    start=$((line > 14 ? line - 14 : 1))
    if sed -n "${start},$((line - 1))p" "$file" | grep -q 'clipboard-chord-exempt:'; then
        continue
    fi

    echo "  $file:$line: asks about C/X/V as a KEY event"
    echo "      $trimmed"
    violations=$((violations + 1))
done < <(grep -rnE "$pattern" crates/ tools/ --include='*.rs' 2>/dev/null)

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

echo "check-clipboard-chords: PASS — the three chords are read as events."
exit 0
