#!/usr/bin/env bash
#
# check-string-gaps.sh — a wrapped string literal must not bake a gap into
# the sentence the operator reads.
#
# WHAT THIS GATE IS FOR
# =====================
#
# Rust continues a string literal across source lines with a trailing
# backslash, which eats the newline AND the next line's leading whitespace:
#
#     "the scale is given as a ratio. To set it by \
#      pointing at a dimension"                        -> "…by pointing at…"
#
# Drop the backslash and the literal is still valid, still compiles, still
# passes every test that does not compare it to a hand-written expectation —
# and now contains a run of spaces where the indentation was:
#
#     "the scale is given as a ratio. To set it by
#      pointing at a dimension"                        -> "…by / six spaces / pointing…"
#
# `rustfmt` then joins the two source lines, so what is left in the file is one
# long line with the gap sitting in the middle of it, looking deliberate.
#
# WHY NOTHING ELSE CATCHES IT
# ===========================
#
# This gate exists because on 2026-08-18 `pdfcer-core` reported finding SIX of
# these in its own shipped error messages, two of them live since `95c3416`,
# and named the reason nothing had caught them:
#
#   - `cargo fmt` does not reflow the CONTENTS of a string literal, so the gap
#     is invisible to the formatter that reformatted the line around it;
#   - clippy has no lint for it;
#   - a mirror test that asserts two copies of the string agree compares one
#     broken copy against another and passes.
#
# The same grep over this workspace found **36 across 22 files**, eight of them
# in `crates/pdfcer-gui/src/text/` — copy an operator reads on screen, including
# every sentence of the Set-scale dialog written the same afternoon. So this is
# not a defect one author makes once; it is what the language's line-
# continuation syntax does when a hand-edit loses one character, and the whole
# failure mode is that the result looks fine in the diff and wrong in the app.
#
# ★ AND IT IS INVISIBLE FROM INSIDE THE EDITOR.
#
# That is the property that makes it worth a gate rather than a note. Reviewing
# the source you see a wrapped sentence; the six spaces are indentation, which
# is exactly what your eye is trained to skip. It only becomes visible in the
# rendered window — which is R1's whole point, and R1 does not scale to every
# string in the tree. A grep does.
#
# WHAT COUNTS AS A VIOLATION
# ==========================
#
# Three or more consecutive spaces between two word-ish characters, on a line
# containing a double quote, outside a comment. Three rather than two because
# two spaces after a full stop is a typographic convention somebody may hold
# deliberately, and this gate should not adjudicate that.
#
# WHAT THIS GATE CANNOT SEE, AND THE ONE ESCAPE HATCH
# ===================================================
#
# It reads text, not a syntax tree, so a line with a quote anywhere on it is
# treated as carrying a literal. Comments are stripped first, which removes the
# aligned tables in `egui-shell`'s manifest docs and every `#[expect(reason =
# …)]` justification — none of which an operator ever reads. The direction of
# the error is the same trade `check-strong-text.sh` records: a false positive
# costs one reflow, and there is no false NEGATIVE that ships anything inert.
#
# A literal that genuinely needs the run of spaces — a test fixture holding
# escaped Rust source, an aligned report column — says so with a comment
# containing `string-gap-exempt:` and a reason. There is exactly one in the
# tree today, in `icons/glyphs.rs`, and it holds the input to the glyph
# scanner's own test.
#
# ★ THE MARKER MAY SIT IN THE COMMENT BLOCK ABOVE, NOT ONLY ON THE LINE.
#
# The obvious spelling is a trailing same-line comment, and that works. But a
# line long enough to trip this gate is already long, and R5 says the reason a
# rule is being set aside is exactly the kind of thing this project writes at
# length. A one-line-only marker would mean the better-documented exemption
# fails a gate the terse one passes — the same backwards incentive
# `check-strong-text.sh` records about measuring source lines instead of code
# lines, and it would push the next person to shorten the explanation to
# appease the tool.
#
# So the marker arms the NEXT code line, and blank and comment-only lines in
# between hold the arming. It covers one line, deliberately: an exemption that
# leaked down a file would silence violations nobody had looked at.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 2

SELF_TEST=0
[ "${1:-}" = "--self-test" ] && SELF_TEST=1

EXEMPT='string-gap-exempt'

# ---------------------------------------------------------------------------
# The scan — ONE `awk` PASS, NOT A SHELL LOOP.
#
# The first cut of this gate ran `sed` and `grep` as subprocesses per LINE. On
# Windows that is tens of thousands of process spawns and it did not finish in
# two minutes. **A gate slow enough to skip is a gate that gets skipped**,
# which is the failure mode `run-all.sh` exists to prevent — so the shape of
# the implementation is part of the gate working, not an optimisation. `awk`
# reads every file in one process and the whole scan is well under a second.
#
# Comments are stripped BEFORE the match rather than the line being skipped
# when it has a comment on it, because a shipped literal routinely carries a
# trailing `// ui-text-exempt:` note. Stripping first means a gap in the code
# half is still found on a line whose comment half is clean.
# ---------------------------------------------------------------------------
scan() {
    local root="$1"
    local out
    out="$(find "$root" -name '*.rs' -not -path '*/target/*' -print0 2>/dev/null |
        xargs -0 -r awk -v exempt="$EXEMPT" '
            FNR == 1 { armed = 0 }
            index($0, exempt) { armed = 1; next }
            {
                code = $0
                sub(/\/\/.*$/, "", code)
                if (code !~ /[^[:space:]]/) next    # blank or comment-only: hold the arming
                if (armed) { armed = 0; next }      # the marker above covers THIS line
                if (code !~ /"/) next
                # ★ ANY non-space on the left, not an enumerated class.
                #
                # This read `[A-Za-z,.:;)]` and had a hole the width of the format syntax of Rust
                # itself: `}`. A mangled continuation inside
                #
                #     "… transformed={transformed}      m=[{:.4} …]"
                #
                # has a BRACE before the gap, so the gate passed it, and the
                # defect shipped into a trace line on 2026-08-20 — where a
                # driven check then read it back with twenty-six spaces in the
                # middle. Interpolation is the commonest thing at the end of a
                # clause in this codebase, which made the omitted character the
                # likeliest one.
                #
                # Enumerating a class is a guess about what precedes a gap.
                # "Something, then three spaces, then a letter" is the fact.
                #
                # ★ Two characters ARE excluded, and both for the same reason:
                # a gap that begins a literal is INDENTATION, not a lost
                # continuation. `println!("      default exe: {}")` is a report
                # laying out a column, and four of them in this harness are the
                # false positives that widening the class first produced.
                if (code ~ /[^[:space:]("]   +[A-Za-z]/) {
                    body = code
                    sub(/^[[:space:]]+/, "", body)
                    print "  " FILENAME ":" FNR ": a run of spaces baked into a string literal"
                    print "      " substr(body, 1, 100)
                }
            }
        ')"
    [ -z "$out" ] && return 0
    printf '%s\n' "$out"
    return 1
}

# ---------------------------------------------------------------------------
# Self-test — the discipline every gate here holds itself to.
#
# A gate that has only ever been seen to pass is indistinguishable from a gate
# that cannot fail, and this project has a recorded instance of exactly that:
# a deliberately planted violation `check-ui-strings.sh` did not detect, which
# briefly made a broken gate look like a working one.
#
# The clean fixture carries the four shapes most likely to produce a false
# positive: a doc comment aligning a table, a line comment mentioning the
# defect, an exempt literal, and a properly continued one — the correct
# spelling this gate exists to preserve.
# ---------------------------------------------------------------------------
if [ "$SELF_TEST" = "1" ]; then
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/dirty" "$tmp/clean" "$tmp/leak"

    cat > "$tmp/dirty/bad.rs" <<'EOF'
pub const fn note() -> &'static str {
    "With no line drawn yet, the scale is given as a ratio. To set it by      pointing at a stated length, measure it first."
}
EOF
    # ★ The shape that got PAST this gate on 2026-08-20: the character before
    # the gap is a closing brace, because the clause ended in an interpolation.
    # The old pattern enumerated `[A-Za-z,.:;)]` and `}` was not in it.
    cat > "$tmp/dirty/brace.rs" <<'EOF'
pub fn line(page: usize, n: u64) -> String {
    format!("transform-objects page={page} transformed={n}          m=[1 0 0 1 0 0]")
}
EOF
    cat > "$tmp/clean/good.rs" <<'EOF'
//! A doc comment may align a table:
//!     Mode(id: "read",   label: "Read")
pub const fn note() -> &'static str {
    // A comment describing the       baked-gap defect must not trip the gate.
    "With no line drawn yet, the scale is given as a ratio. To set it by \
     pointing at a stated length, measure it first."
}
pub const fn fixture() -> &'static str {
    "one two     three"  // string-gap-exempt: holds escaped Rust source
}
pub const fn block_marked() -> &'static str {
    // string-gap-exempt: the marker may sit in the comment block above, so
    // that the reason can be written at length.
    "one two     three"
}
pub const fn short() -> &'static str {
    "Two spaces after a stop.  That is a convention, not a defect."
}
EOF
    # An exemption must cover ONE line, not leak down the file.
    cat > "$tmp/leak/leak.rs" <<'EOF'
pub const fn marked() -> &'static str {
    // string-gap-exempt: this one is deliberate.
    "one two     three"
}
pub const fn unmarked() -> &'static str {
    "four five     six"
}
EOF

    fail=0
    if scan "$tmp/dirty" > /dev/null; then
        echo "SELF-TEST FAILED: a baked gap was not detected"
        fail=1
    fi
    if ! scan "$tmp/clean"; then
        echo "SELF-TEST FAILED: a clean file was reported as a violation"
        fail=1
    fi
    # The arming must expire after one code line. If it leaked, the second
    # literal here would be silently exempt and the gate would go quiet
    # exactly where somebody had already used the escape hatch once.
    if scan "$tmp/leak" > /dev/null; then
        echo "SELF-TEST FAILED: an exemption leaked past the line it marks"
        fail=1
    fi
    [ "$fail" = "1" ] && exit 1
    echo "check-string-gaps self-test: PASS"
    exit 0
fi

echo "check-string-gaps: scanning crates/ and tools/ for baked-in gaps…"
found=0
for root in crates tools; do
    [ -d "$root" ] || continue
    scan "$root" || found=1
done

if [ "$found" = "1" ]; then
    cat <<'MSG'

A string literal contains a run of three or more spaces mid-sentence.

Almost always this is a line continuation that lost its trailing backslash:
the literal still compiles and the gap ships into whatever the operator reads.
Rejoin the sentence, or continue it properly with a trailing backslash — Rust
eats the newline and the next line's indentation.

If the spaces are wanted, say so on the same line with a trailing comment
containing `string-gap-exempt:` and the reason.
MSG
    exit 1
fi

echo "check-string-gaps: PASS — no baked-in gaps."
exit 0
