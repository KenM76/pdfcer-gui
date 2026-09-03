#!/usr/bin/env bash
# check-typing-guard.sh — "is the operator typing?" is asked in ONE place.
#
# ===========================================================================
# WHY THIS GATE EXISTS
# ===========================================================================
#
# This shell composes text in two places, and only one of them is visible to
# egui:
#
#   1. a real `egui::TextEdit` — a form field, the page-number box, the Find
#      bar. `ctx.text_edit_focused()` sees these.
#   2. the CANVAS CARET — deliberately not a widget, because it sits in PDF
#      space at the glyphs' own scale and a floating widget cannot do that.
#      `ctx.text_edit_focused()` is FALSE for an operator who is visibly
#      mid-word.
#
# So any guard that means "the operator is typing, leave this key alone" has to
# ask about both. `canvas::textedit::composing` is that question.
#
# ---------------------------------------------------------------------------
# The two defects that produced this file
# ---------------------------------------------------------------------------
#
# DEFECT D1, the founding defect of this project. The Delete key stopped
# working after any canvas click, because the guard asked
# `egui_wants_keyboard_input()` — ANY widget focused — where it meant
# `text_edit_focused()`. The commit said "analysis-confirmed, NOT empirically
# verified." The only test of it built a bare `egui::Context` with no widgets,
# so the condition that breaks the real application cannot occur in the
# harness.
#
# DEFECT D1 AGAIN, one rung along, 2026-08-20. `canvas::tool::arm::space_held`
# asked `!ctx.text_edit_focused()`. Space is the hand tool's modifier. So an
# operator typing on the canvas pressed space and PANNED THE PAGE — text
# editing could not type a space. Meanwhile `app::keyboard` asked both
# claimants and had a paragraph explaining why. One truth, two copies, one of
# them wrong.
#
#   "I can edit text now, but there is no live preview of that either, and it
#    doesn't accept spaces. Like how?"  — the operator, 2026-08-20
#
# ---------------------------------------------------------------------------
# Why a GATE and not a code review note
# ---------------------------------------------------------------------------
#
# Because both instances were written by somebody who had read the argument.
# The second was written in a codebase that already contained the correct
# predicate, documented at length, twenty files away. Care did not prevent it
# and will not prevent the third. A grep will.
#
# It is also invisible to unit tests by construction: a `Context` built in a
# test has no canvas draft and no focused field, so both spellings return the
# same answer for every input a test can produce. That is the same property
# that let D1 ship, and it is why this is a source gate rather than a test.
#
# ===========================================================================
# WHAT IS ALLOWED
# ===========================================================================
#
# * `canvas::textedit::composing` — the one implementation.
# * comments and doc comments, which discuss the predicate constantly.
# * `app::keyboard`'s tests, which assert on the raw egui state to prove the
#   harness reaches the condition at all.
#
# Anything else naming `text_edit_focused` is a second copy of a two-claimant
# question, which is the defect.
#
# Exit 0 clean, 1 on a violation.

set -uo pipefail
cd "$(dirname "$0")/../.." || exit 1

echo "check-typing-guard: one predicate for \"is the operator typing?\"…"

ALLOWED_FILE="crates/pdfcer-gui/src/canvas/textedit/mod.rs"

violations=0
while IFS= read -r hit; do
    file="${hit%%:*}"
    rest="${hit#*:}"
    line="${rest%%:*}"
    text="${rest#*:}"

    # The implementation itself.
    [ "$file" = "$ALLOWED_FILE" ] && continue

    # Prose. The argument is discussed in a dozen doc comments and every one of
    # them is welcome — what is forbidden is a second CALL.
    trimmed="$(printf '%s' "$text" | sed 's/^[[:space:]]*//')"
    case "$trimmed" in
        '//'*|'///'*|'//!'*|'*'*) continue ;;
    esac

    # An exemption is a marker on the line itself, or anywhere in the
    # contiguous comment block immediately above it.
    #
    # The block form is not laxity: every legitimate exemption here needs a
    # paragraph, not a clause — "this is self-referential", "Escape must still
    # reach the abandon rung", "this test asserts the harness reached the
    # state". A rule that only accepts a trailing clause would push those
    # reasons out of the file and leave a bare marker behind, which is the
    # exemption without the argument.
    if printf '%s' "$text" | grep -q 'typing-guard-exempt:'; then
        continue
    fi
    start=$((line > 14 ? line - 14 : 1))
    if sed -n "${start},$((line - 1))p" "$file" | grep -q 'typing-guard-exempt:'; then
        continue
    fi
    echo "  $file:$line: asks egui alone whether the operator is typing"
    echo "      $trimmed"
    violations=$((violations + 1))
done < <(grep -rn 'text_edit_focused' crates/ tools/ --include='*.rs' 2>/dev/null)

if [ "$violations" -gt 0 ]; then
    cat <<'MSG'

`ctx.text_edit_focused()` answers only half the question.

It is FALSE for an operator typing into the canvas caret, which is not an
`egui::TextEdit` — so a guard built on it silently steals keys from somebody who
is visibly mid-word. That is defect D1, and its second instance cost the
operator the SPACE BAR while editing text.

Call `crate::canvas::textedit::composing(ctx)` instead. It asks both claimants.

If you genuinely mean "an egui widget has focus" and not "the operator is
typing" — a test asserting the harness reached the state, for instance — say so
on the same line, or in the comment block just above it, with
`typing-guard-exempt:` and the reason.

The block form is deliberate. Every legitimate exemption here needs a paragraph
rather than a clause - "this is self-referential", "Escape must still reach the
abandon rung", "this test asserts the harness reached the state" - and a rule
that accepted only a trailing clause would push those reasons out of the file
and leave a bare marker behind, which is the exemption without the argument.
MSG
    echo
    echo "check-typing-guard: FAIL — $violations call site(s)."
    exit 1
fi

echo "check-typing-guard: PASS — the predicate exists once."
exit 0
