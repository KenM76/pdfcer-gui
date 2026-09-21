#!/usr/bin/env bash
# ===========================================================================
# check-third-party-licences.sh — THE SHIPPED ATTRIBUTION FILE MUST MATCH THE
# BUILD IT SHIPS BESIDE.
#
# ---------------------------------------------------------------------------
# ★★★ WHY THIS EXISTS, and the release it nearly went out on
# ---------------------------------------------------------------------------
#
# A `cargo update` of the engine crates pulled in a colour-management engine,
# and with it three crates that had never been in this build:
#
#     Adding iccce-cmm     0.3.0  (MIT)
#     Adding iccce-color   0.3.0  (MIT)
#     Adding iccce-profile 0.3.0  (MIT)
#
# `THIRD_PARTY_LICENSES.md` named none of them. That file is **shipped beside
# the exe** by `tools/package-portable.py`, and `pdfcer-gui.exe` statically links
# all three — so the package would have distributed MIT-licensed code while its
# own attribution file said it did not.
#
# ⇒ It was caught because those three "Adding" lines happened to be printed by a
# command whose output was being read for an unrelated reason. Nothing checked
# it. Every test passed, every other gate passed, and the release was built.
#
# ★★ **The same shape as `check-verb-coverage.sh`**: an ADDITION on the other
# side of a boundary is silent by construction. A REMOVED dependency breaks the
# build; an added one does not, and every tool in the toolchain is oriented
# around the first.
#
# ---------------------------------------------------------------------------
# WHAT IT ASSERTS
# ---------------------------------------------------------------------------
#
#     Regenerating the file produces the file that is committed, compared with
#     carriage returns normalised away on both sides.
#
# ★★★ An earlier version compared `Cargo.lock`'s crate names against the licence
# file and reported **213 missing** on a correct tree. The premise was wrong:
# `Cargo.lock` holds every crate for every TARGET — Android, macOS, Wayland,
# WASM — and a Windows build links almost none of them. `cargo-about` resolves
# per-target and is right; a hand-rolled name comparison cannot be. That is this
# project's own rule turned on itself: **a check that cries wolf is not a
# stricter check, it is a broken one.**
#
# ⇒ So the gate delegates to the tool that owns the question. It answers exactly
# the thing that was actually got wrong — *is the shipped file older than the
# build?* — and nothing about licence text, compatibility or terms.
#
# ---------------------------------------------------------------------------
# ★★★ WHY THE COMPARISON NORMALISES LINE ENDINGS, and the loop it breaks
# ---------------------------------------------------------------------------
#
# `cargo-about` embeds each crate's LICENSE file verbatim, and some of them are
# stored with CRLF — glow's Apache-2.0 text contributes 200 such lines here.
# `.gitattributes` pins `*.md text eol=lf`, so `git add` strips exactly those
# carriage returns and the committed blob can never hold them.
#
# ⇒ A byte-identical comparison therefore passes **only on a machine where the
# generator was the last thing to write the file**, and fails on every fresh
# clone and after any checkout. It is not measuring staleness at all in that
# state; it is measuring who wrote the working copy last.
#
# ★★ That is not a hypothesis. The gate failed, the file was regenerated, the
# gate went green — and it failed again identically the next day the working
# copy was rewritten from the committed bytes, with the file unchanged in git
# throughout. `git diff` reports nothing either way, because the same `text`
# attribute normalises the worktree side before comparing, so the one
# instrument that can see this condition is this gate.
#
# ⇒ Normalising here is the repair the comparison needs, and the alternative —
# exempting the file from `text` so the CRs survive a commit — would put mixed
# line endings in a shipped Markdown file to satisfy a diff. Stripping CR
# cannot hide a licence change: no attribution difference that matters is
# spelled in carriage returns.
#
# ---------------------------------------------------------------------------
# THE FIX, WHEN IT FIRES
# ---------------------------------------------------------------------------
#
#     cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md
#
# Then `git add` it; git will normalise the line endings and that is correct.
# Do NOT edit the file by hand: its own header says it is generated, so a
# hand-edit makes it disagree with the tool that owns it and the next
# regeneration reverts it without a word.
#
# ---------------------------------------------------------------------------
# EXIT CODES
# ---------------------------------------------------------------------------
#   0  the committed file is what the generator produces
#   1  it is not, and the message says which of the two shapes it is: either the
#      crate SET differs — the build links something the file does not describe,
#      or describes something it no longer links — or the set matches and only
#      the generated body differs, which is a stale file rather than an
#      attribution gap. The two have different remedies and the wrong headline
#      sends a reader hunting a dependency change that never happened.
#   2  SKIPPED — a precondition was absent and NOTHING WAS MEASURED.
#
# ★★ The skip branches used to `exit 0`, which `run-all.sh` classifies as a
# PASS. They printed the word SKIP into a log nobody diffs and were counted
# among the passes, so on any machine without `cargo-about` installed this gate
# reported success while measuring nothing. A SKIP is only a SKIP if the runner
# is told; the word in the output is not the signal, the exit code is.
#
# ---------------------------------------------------------------------------
# --self-test
# ---------------------------------------------------------------------------
#
# Plants all three outcomes against synthetic files and requires the verdict to
# distinguish them: a CRLF-only difference must PASS, a body-only difference
# must fail naming NO crate, and a missing crate must fail naming THAT crate.
# It needs neither `cargo-about` nor the real file, so it runs everywhere.
# ===========================================================================
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 1

LICENCES="THIRD_PARTY_LICENSES.md"
TEMPLATE="about.hbs"

# ★★ Named by SET DIFFERENCE, not by reading the diff hunks. A line-diff of a
# 7,000-line generated file reports the NEIGHBOURS of a change as well as the
# change, so the first version of this message listed four `accesskit` crates
# when exactly one `iccce` line had been removed — a confident, precise and
# entirely wrong list. Caught by falsifying the gate rather than by reading it.
names() { sed -n 's/^- \[\([^]]*\)\].*/\1/p' "$1" | sort -u; }

# verdict <fresh> <have> — prints the whole finding, returns 0 (match) or 1.
# The self-test drives this same function, so what it falsifies is what runs.
verdict() {
    local fresh="$1" have="$2"
    local nf nh added gone
    nf="$(mktemp)"; nh="$(mktemp)"
    sed 's/\r$//' "$fresh" > "$nf"
    sed 's/\r$//' "$have"  > "$nh"

    if diff -q "$nf" "$nh" >/dev/null 2>&1; then
        rm -f "$nf" "$nh"
        echo "PASS: $LICENCES is what the generator produces from the current lock."
        return 0
    fi

    added=$(comm -23 <(names "$nf") <(names "$nh"))
    gone=$(comm -13 <(names "$nf") <(names "$nh"))
    rm -f "$nf" "$nh"

    # ★★★ The two outcomes need two headlines, because they are different
    # problems with different remedies. The files can differ while describing
    # exactly the same crate set — a licence body that regenerated differently
    # does it — and the dependency-drift headline then sends the reader hunting
    # a dependency change that never happened, with an empty list underneath
    # contradicting the sentence above it. A message both outcomes satisfy is
    # not a diagnosis of which one occurred.
    if [ -z "$added" ] && [ -z "$gone" ]; then
        echo "FAIL: $LICENCES differs from what cargo-about produces, but describes"
        echo "      exactly the same crate set. Nothing was added or dropped: the"
        echo "      difference is in the generated BODY — licence text or ordering —"
        echo "      so this is a stale file rather than an attribution gap."
        echo "      Regenerate it; do not go looking for a dependency change."
        cat <<'EOF'

  The remedy is one command:

      cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md
EOF
        return 1
    fi

    echo "FAIL: $LICENCES does not match what cargo-about produces from the current"
    echo "      Cargo.lock. The build links something the shipped attribution file"
    echo "      does not describe, or describes something it no longer links."
    echo
    # `printf '%s\n' "$EMPTY"` prints a BLANK LINE, not nothing, so an empty
    # side has to say so in words. A silent gap under a heading reads as output
    # that got cut off, which is the same ambiguity this branch was split to
    # remove.
    show() { if [ -z "$1" ]; then echo "    (none)"; else printf '%s\n' "$1" | sed 's/^/    /' | head -40; fi; }
    echo "  Linked by this build and NOT in the shipped file:"
    echo
    show "$added"
    echo
    echo "  In the shipped file and no longer linked:"
    echo
    show "$gone"
    cat <<'EOF'

  That file ships beside the exe. Distributing a crate's code while the
  attribution file says you do not is a real exposure rather than an
  untidiness — and it is invisible to every test, because an ADDED dependency
  breaks nothing.

  The remedy is one command:

      cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md

  Do NOT add the name by hand. The file is generated and says so in its own
  header; a hand-edit makes it disagree with the tool that owns it, and the
  next regeneration reverts it without a word.
EOF
    return 1
}

# ---------------------------------------------------------------------------
# --self-test
# ---------------------------------------------------------------------------
if [ "${1:-}" = "--self-test" ]; then
    D="$(mktemp -d)"
    trap 'rm -rf "$D"' EXIT
    fail=0

    body() {
        printf -- '- [alpha 1.0.0](https://example.invalid/alpha)\n'
        printf '\n```\nPermission is hereby granted, free of charge.\n```\n\n'
        printf -- '- [beta 2.0.0](https://example.invalid/beta)\n'
        printf '\n```\n%s\n```\n' "$1"
    }

    body 'TERMS AND CONDITIONS' > "$D/fresh"
    sed 's/$/\r/' "$D/fresh"    > "$D/crlf"
    body 'TERMS AND CONDITIONS, amended' > "$D/body"
    grep -v '^- \[beta' "$D/fresh"      > "$D/missing"

    out="$(verdict "$D/fresh" "$D/crlf")"; rc=$?
    if [ "$rc" -ne 0 ]; then
        echo "SELF-TEST FAIL: a CRLF-only difference was reported as a finding."
        printf '%s\n' "$out" | sed 's/^/    /'
        fail=1
    fi

    out="$(verdict "$D/fresh" "$D/body")"; rc=$?
    if [ "$rc" -ne 1 ]; then
        echo "SELF-TEST FAIL: a changed licence body was not reported at all."
        fail=1
    elif ! printf '%s' "$out" | grep -q 'exactly the same crate set'; then
        echo "SELF-TEST FAIL: a body-only difference did not get the stale-file"
        echo "                headline, so it will send a reader hunting a"
        echo "                dependency change that did not happen."
        printf '%s\n' "$out" | sed 's/^/    /'
        fail=1
    elif printf '%s' "$out" | grep -q 'beta 2.0.0'; then
        echo "SELF-TEST FAIL: a body-only difference named a crate. The set is"
        echo "                unchanged; naming one is the wrong-list defect."
        fail=1
    fi

    out="$(verdict "$D/fresh" "$D/missing")"; rc=$?
    if [ "$rc" -ne 1 ]; then
        echo "SELF-TEST FAIL: an unattributed crate was not reported."
        fail=1
    elif ! printf '%s' "$out" | grep -q 'beta 2.0.0'; then
        echo "SELF-TEST FAIL: the unattributed crate was not NAMED, so the"
        echo "                message does not say which one to look at."
        printf '%s\n' "$out" | sed 's/^/    /'
        fail=1
    elif printf '%s' "$out" | grep -q 'exactly the same crate set'; then
        echo "SELF-TEST FAIL: a missing crate got the stale-file headline, which"
        echo "                denies the attribution gap it just found."
        fail=1
    fi

    if [ "$fail" -ne 0 ]; then exit 1; fi
    echo "PASS: self-test — CRLF-only passes, a changed body names no crate, a"
    echo "      missing crate is named."
    exit 0
fi

# ---------------------------------------------------------------------------
# the measurement
# ---------------------------------------------------------------------------
if [ ! -f "$LICENCES" ]; then
  echo "SKIP: no $LICENCES. If this project has stopped shipping one, delete this"
  echo "      gate deliberately rather than leaving it to skip for ever."
  exit 2
fi
if [ ! -f "$TEMPLATE" ]; then
  echo "SKIP: no $TEMPLATE, so the file cannot be regenerated for comparison."
  exit 2
fi
if ! command -v cargo-about >/dev/null 2>&1; then
  echo "SKIP: cargo-about is not on PATH, so nothing was measured."
  echo "      cargo install cargo-about"
  exit 2
fi

FRESH="$(mktemp)"
if ! cargo about generate "$TEMPLATE" -o "$FRESH" >/dev/null 2>&1; then
  rm -f "$FRESH"
  echo "SKIP: cargo-about failed, so nothing was measured. Run it by hand to see why:"
  echo "      cargo about generate $TEMPLATE -o $LICENCES"
  exit 2
fi
if [ ! -s "$FRESH" ]; then
  rm -f "$FRESH"
  echo "SKIP: cargo-about produced an empty file, which is a tool failure rather"
  echo "      than a finding about this repository."
  exit 2
fi

verdict "$FRESH" "$LICENCES"
rc=$?
rm -f "$FRESH"
exit "$rc"
