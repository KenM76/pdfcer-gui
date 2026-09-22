#!/usr/bin/env bash
# check-escape-disposition.sh — every text field says what Escape does to it.
#
# ===========================================================================
# WHY THIS GATE EXISTS
# ===========================================================================
#
#   "I think for adding and editing text when using any tool that has text
#    escape should also save changes to the text. The user can always undo if
#    they want, but it is easy to accidentally press escape and lose a lot of
#    text that has been entered."   — the operator, `OPERATOR_REQUESTS.md` O223
#
# The reasoning is the stronger half of the ask: a discard is unrecoverable
# and a commit is one undo away, so the asymmetry decides it. Escape commits.
#
# "Any tool that has text" is the scope, and this project's contract clause 7
# says a scope like that is COUNTED rather than discovered one complaint at a
# time. Counting it once, by hand, is a paragraph in a document that is true
# on the day it is written and silently false the next time somebody adds a
# field. This gate is the counting, standing.
#
# ---------------------------------------------------------------------------
# Why a source gate rather than a test
# ---------------------------------------------------------------------------
#
# Because the defect is an ABSENCE. There is nothing to call: the field is
# built, the operator types, the key is pressed, and the draft is dropped by a
# code path that does not exist. A unit test can only assert about a surface
# somebody remembered to write a test for, which is precisely the surface that
# does not have the problem.
#
# It is also invisible to the driven harness for a cheaper reason: nobody
# writes a driven check for the field they just forgot to think about.
#
# ---------------------------------------------------------------------------
# What it asks, and what it deliberately does not
# ---------------------------------------------------------------------------
#
# It asks ONE question: does a human being, at this call site, state what
# Escape does to what has been typed? It does not, and cannot, check that the
# answer is true. A site labelled `commits` whose commit path is broken passes
# here and fails in `ui-verify`.
#
# That narrowness is the point. The failure this guards against is nobody
# having asked the question at all, and that is answerable by grep.
#
# ===========================================================================
# THE VOCABULARY, AND WHY IT IS CLOSED
# ===========================================================================
#
# A marker with free-form text after it degenerates into a marker, because the
# next author copies the nearest one. So the word after the colon must be one
# of five, and a sixth is a FAILURE rather than something waved through:
#
#   escape-disposition: commits
#       The draft reaches the document. Either an explicit `Key::Escape` arm
#       that writes, or a commit keyed on `lost_focus()` — egui surrenders a
#       `TextEdit`'s focus on Escape, so `lost_focus()` is true on that frame
#       and a `lost_focus` commit IS an Escape commit.
#
#   escape-disposition: keeps-draft
#       The draft lives in panel or UI state and is written back every frame
#       regardless of what happened, so the key takes the caret out of the box
#       and leaves the words in it. Nothing is lost, and the next Enter or
#       button press still commits. This is not a weaker `commits`; it is a
#       different shape, and conflating them would hide a field that changed
#       from one to the other.
#
#   escape-disposition: dialog-cancels
#       Inside a dialog, where Cancel is what a dialog is FOR. Protected by
#       `dialogs::host`'s two-press rung — the first Escape leaves the field,
#       the second closes the window — which is what Acrobat, Word and every
#       options window in Windows do, and therefore the spec under this
#       project's use-the-conventional-interaction rule.
#
#   escape-disposition: not-content
#       The box holds nothing the operator typed as their work, so Escape may
#       discard it freely. Three shapes qualify and no others: a search or
#       filter query, a navigation box such as the page-number field, and a
#       DISABLED box mirroring a value that cannot be typed into at all.
#
#   escape-disposition: not-a-surface
#       A field a test builds to put egui into a known state. It is never
#       drawn for an operator and there is nothing to lose.
#
# ===========================================================================
# WHERE THE MARKER MAY SIT
# ===========================================================================
#
# On the call site's own line, or anywhere in the fourteen lines above it.
#
# The block form is not laxity. Several of these need a sentence rather than a
# clause — WHY a rename box keeps its draft, WHICH commit path a `lost_focus`
# site rides — and a rule that accepted only a trailing clause would push those
# reasons out of the file and leave a bare marker behind, which is the marker
# without the argument. `check-typing-guard.sh` made the same choice for the
# same reason and the shape is deliberately identical.
#
# Exit 0 clean, 1 on a violation or a self-test that did not catch its plant,
# 2 SKIPPED when the scan root holds no text fields at all — which is a moved
# root, not a clean tree, and says so.

set -uo pipefail
cd "$(dirname "$0")/../.." || exit 1

ROOT="crates"

# The five legal answers. Anything else after the colon is a typo or an
# invention, and both are failures: a marker the gate does not understand is a
# marker nobody will notice has stopped meaning anything.
LEGAL="commits keeps-draft dialog-cancels not-content not-a-surface"

# Every construction of a text field. `singleline` and `multiline` are the two
# constructors; `TextEdit::` alone would also match `TextEditState`, `TextEdit`
# in a type position and the several dozen doc comments that name the type.
PATTERN='TextEdit::\(singleline\|multiline\)('

# --------------------------------------------------------------------------
# sites <root> — every call site, as `file:line:text`.
#
# Lines whose trimmed form opens a comment are dropped BEFORE anything else.
# This codebase documents heavily and several comments quote the constructor
# to explain a rule about it; a gate that counted those would demand a marker
# on a sentence and, worse, would be satisfiable by prose describing a field
# rather than by a field.
# --------------------------------------------------------------------------
sites() {
    local root="$1"
    grep -rn "$PATTERN" "$root" --include='*.rs' 2>/dev/null | while IFS= read -r hit; do
        local rest text trimmed
        rest="${hit#*:}"
        text="${rest#*:}"
        trimmed="$(printf '%s' "$text" | sed 's/^[[:space:]]*//')"
        case "$trimmed" in
            '//'* | '*'* | '/*'*) continue ;;
        esac
        printf '%s\n' "$hit"
    done
}

# --------------------------------------------------------------------------
# marker_near <file> <line> — the word after `escape-disposition:` on that
# line or in the fourteen above it, or the empty string.
# --------------------------------------------------------------------------
marker_near() {
    local file="$1" line="$2" start
    start=$((line > 14 ? line - 14 : 1))
    sed -n "${start},${line}p" "$file" |
        grep -o 'escape-disposition:[[:space:]]*[A-Za-z-]*' |
        tail -1 |
        sed -E 's/escape-disposition:[[:space:]]*//'
}

# --------------------------------------------------------------------------
# scan <root> — report every unmarked or wrongly marked site.
#
# Echoes one line per violation and returns the count through `$found`.
# --------------------------------------------------------------------------
scan() {
    local root="$1"
    found=0
    total=0
    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        local file rest line text trimmed marker legal
        file="${hit%%:*}"
        rest="${hit#*:}"
        line="${rest%%:*}"
        text="${rest#*:}"
        trimmed="$(printf '%s' "$text" | sed 's/^[[:space:]]*//')"
        total=$((total + 1))

        marker="$(marker_near "$file" "$line")"

        if [ -z "$marker" ]; then
            printf '  %s:%s: a text field that does not say what Escape does to it\n' \
                "$file" "$line"
            printf '      %s\n' "$trimmed"
            found=$((found + 1))
            continue
        fi

        legal=no
        for word in $LEGAL; do
            [ "$marker" = "$word" ] && legal=yes && break
        done
        if [ "$legal" = no ]; then
            printf '  %s:%s: `escape-disposition: %s` is not one of the five answers\n' \
                "$file" "$line" "$marker"
            printf '      %s\n' "$trimmed"
            found=$((found + 1))
        fi
    done < <(sites "$root")
}

# ===========================================================================
# --self-test
# ===========================================================================
#
# Four shapes, because the three ways this check can be wrong are all silent:
# it can miss a bare field, it can accept a marker it does not understand, and
# it can demand a marker on a comment that merely names the constructor.
if [ "${1:-}" = "--self-test" ]; then
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/src"

    cat >"$tmp/src/good.rs" <<'RS'
fn draw(ui: &mut Ui, draft: &mut String) {
    // escape-disposition: commits — committed on `lost_focus`, and egui
    // surrenders focus on Escape, so the key rides that path.
    let r = ui.add(egui::TextEdit::singleline(draft));
    if r.lost_focus() { commit(draft); }
}
RS

    cat >"$tmp/src/bare.rs" <<'RS'
fn draw(ui: &mut Ui, draft: &mut String) {
    let _ = ui.add(egui::TextEdit::multiline(draft));
}
RS

    cat >"$tmp/src/invented.rs" <<'RS'
fn draw(ui: &mut Ui, draft: &mut String) {
    // escape-disposition: probably-fine
    let _ = ui.add(egui::TextEdit::singleline(draft));
}
RS

    cat >"$tmp/src/prose.rs" <<'RS'
//! A field is built with `egui::TextEdit::singleline(` and a draft, which is
//! the shape this module explains rather than uses.
fn draw() {}
RS

    scan "$tmp/src" >"$tmp/out.txt"
    rc_found=$found

    fails=0
    grep -q 'bare.rs:2' "$tmp/out.txt" || {
        echo "self-test: MISSED the unmarked field in bare.rs" >&2
        fails=1
    }
    grep -q 'invented.rs:3' "$tmp/out.txt" || {
        echo "self-test: ACCEPTED an answer outside the closed vocabulary" >&2
        fails=1
    }
    grep -q 'good.rs' "$tmp/out.txt" && {
        echo "self-test: flagged a correctly marked field" >&2
        fails=1
    }
    grep -q 'prose.rs' "$tmp/out.txt" && {
        echo "self-test: demanded a marker on a doc comment naming the type" >&2
        fails=1
    }
    [ "$rc_found" -eq 2 ] || {
        echo "self-test: expected 2 violations, counted $rc_found" >&2
        fails=1
    }

    # And the empty case, which must SKIP rather than pass: a root holding no
    # text fields is a root that has moved.
    mkdir -p "$tmp/empty"
    scan "$tmp/empty" >/dev/null
    [ "$total" -eq 0 ] || {
        echo "self-test: counted fields in an empty tree" >&2
        fails=1
    }

    if [ "$fails" -ne 0 ]; then
        echo "check-escape-disposition --self-test: FAIL"
        exit 1
    fi
    echo "check-escape-disposition --self-test: PASS — 4 planted shapes, 2 caught, 2 passed."
    exit 0
fi

if [ "$#" -gt 0 ]; then
    echo "check-escape-disposition: takes no arguments but --self-test (got '$1')." >&2
    exit 1
fi

echo "check-escape-disposition: every text field says what Escape does to it…"

scan "$ROOT"

if [ "$total" -eq 0 ]; then
    echo "check-escape-disposition: SKIPPED — no text fields found at all."
    echo "  That is a moved scan root, not a clean tree: $ROOT"
    exit 2
fi

if [ "$found" -gt 0 ]; then
    cat <<'MSG'

A text field with no stated Escape disposition is a field nobody has asked the
operator's question about:

  "it is easy to accidentally press escape and lose a lot of text that has
   been entered."

Say what happens, on the line or in the fourteen above it, with
`escape-disposition:` and ONE of:

  commits          the draft reaches the document — an explicit Escape arm, or
                   a commit keyed on `lost_focus()`, which Escape triggers
  keeps-draft      the draft is written back to panel state every frame, so
                   the key leaves the box and leaves the words in it
  dialog-cancels   inside a dialog, where Cancel is the point, protected by
                   `dialogs::host`'s two-press rung
  not-content      a search or filter query, a navigation box, or a disabled
                   box mirroring a value that cannot be typed into
  not-a-surface    a field a test builds to put egui into a known state

The word is checked against that list. It is not a free-text note: a marker the
gate cannot read is a marker that has quietly stopped meaning anything.

If the honest answer is "it discards typed content", the answer is not a marker.
It is O223, and the fix is a commit.
MSG
    echo
    echo "check-escape-disposition: FAIL — $found of $total text field(s) unanswered."
    exit 1
fi

echo "check-escape-disposition: clean — all $total text field(s) under $ROOT/ state what Escape does."
exit 0
