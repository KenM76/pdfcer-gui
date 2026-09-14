#!/usr/bin/env bash
#
# check-memory-index.sh — every agent memory is reachable, and the index fits.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★★★ WHY A FILE THAT EXISTS CAN STILL BE A FILE THAT WAS NEVER WRITTEN
# ═══════════════════════════════════════════════════════════════════════════
#
# `.claude/agent-memory/<agent>/` holds one markdown file per remembered
# finding, plus a `MEMORY.md` index. **Only the index is loaded into a session.**
# The topic files are read on demand, by a session that already knows they
# exist — which it learns from the index and from nowhere else.
#
# So a topic file that the index does not name is not "a memory with a missing
# pointer". It is an unreachable file: written, committed, reviewed, and never
# read again by anything. The failure is silent in both directions — the folder
# listing looks complete, and the index looks complete, because neither is
# measured against the other.
#
# ⇒ **This gate was written on 2026-09-13, after finding exactly that.**
# `user_he_is_not_at_the_keyboard_unless_he_says_so.md` — the standing rule that
# decides whether a session drives the release binary or defers the work to the
# operator, one of the most operationally load-bearing memories in the folder —
# existed on disk, in git, and was named nowhere in `MEMORY.md`. It was found by
# a `comm` run for an unrelated reason.
#
# That is the same shape this repository has now recorded several times under
# different names: a checker named in every document and registered in no
# runner; a `--self-test` written and never dispatched. **Writing the artifact
# and registering the artifact are two edits, and the second one is the one that
# gets skipped**, because the first is where the thinking was.
#
# ═══════════════════════════════════════════════════════════════════════════
# THE SECOND FAILURE: AN INDEX TOO LARGE TO LOAD
# ═══════════════════════════════════════════════════════════════════════════
#
# The index is injected into context wholesale and **TRUNCATED when it is too
# big** — the harness's own warning, printed on 2026-09-13, reads:
#
#     "WARNING: MEMORY.md is 25.4KB (limit: 24.4KB) — index entries are too
#      long. Only part of it was loaded."
#
# Truncation takes the END of the file, which is where the NEWEST entries are.
# An index that grows past the limit therefore hides the memories a cold session
# most needs — the ones written last — while continuing to display the oldest
# ones, so nothing about the loaded text looks wrong.
#
# ★ The number 24.4 KB belongs to the harness, not to this repository, and it is
# a cross-system citation with an unknown shelf life. It is quoted here with its
# date and its source so that the next reader can re-measure it rather than
# inherit it. **If this gate fails on SIZE, re-read the harness warning before
# raising the constant** — and if the harness has changed its mind, change the
# constant here and say so in the commit, do not delete the check.
#
# The lever that actually controls the size is the HOOK — the prose after the
# link. Filenames are long and are not freely renameable (`[[wikilinks]]` in the
# topic files point at them by slug), so the hook is the only slack there is.
# A hook is a relevance decision, not a summary: the detail belongs in the file
# the line points at, which is the whole reason that file exists.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT IT CHECKS
# ═══════════════════════════════════════════════════════════════════════════
#
#   1. ORPHAN     — a `*.md` in the folder that `MEMORY.md` never names. RED.
#   2. DANGLING   — a link in `MEMORY.md` with no file behind it. RED.
#   3. UNPARSED   — a `- [` line the index format cannot read. RED, because a
#                   row this gate cannot parse is a row it cannot check, and a
#                   silently-skipped row is the failure mode being guarded.
#   4. HOOK LONG  — a hook over MAX_HOOK bytes. RED.
#   5. OVERSIZE   — the index over MAX_INDEX bytes. RED. See the note above.
#
# Exit: 0 clean, 1 a violation, 2 precondition absent (no agent-memory tree).
#
# ═══════════════════════════════════════════════════════════════════════════
# A NOTE ON BYTES VERSUS CHARACTERS
# ═══════════════════════════════════════════════════════════════════════════
#
# Hook length is measured in BYTES, deliberately. The separator is an em dash
# and the hooks contain arrows, stars and curly quotes; `awk` in this
# environment counts bytes, not code points, and a gate that quietly disagreed
# with its own error message about what it measured would be worse than one
# whose units are simply stated. The budget is set with that slack allowed for.
#
# ═══════════════════════════════════════════════════════════════════════════

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

MAX_HOOK=170     # bytes of prose after the link. Longest real hook: 115 chars.
MAX_INDEX=24000  # bytes. Harness limit quoted above is 24.4 KB; this leaves a
                 # small margin so the gate goes red BEFORE truncation starts.
NEAR_INDEX=2400  # bytes of headroom below which a GREEN run says so loudly.

# ★★★ A PASS THAT DOES NOT SAY HOW CLOSE IT CAME IS A PASS THAT EXPIRES
# WITHOUT WARNING. On 2026-09-14 this gate went red at 24,177 bytes, fifteen
# hooks were shortened, and it went green again at **23,984** — sixteen bytes
# of headroom. The next session to add one memory would have hit the same red
# and re-derived the same remedy from scratch, because nothing green had ever
# told it the budget was spent.
#
# ⚠ AND NEAR THE CEILING THE REMEDY CHANGES. "Shorten hooks" works while the
# hooks are padded; at 135 memories they are already one clause each and the
# next trim costs five bytes and a unit of meaning. Below NEAR_INDEX the
# instruction is to **consolidate two entries whose lessons are the same
# shape**, which buys ~200 bytes and loses nothing, or to accept that the
# index needs a structural answer. Printing the headroom is what makes that
# decision available to the session that has time for it rather than to the
# one that is mid-commit.

# ---------------------------------------------------------------------------
# check_folder <dir> — all five rules against one agent's memory folder.
# Echoes findings; returns 1 if any rule failed.
# ---------------------------------------------------------------------------
check_folder() {
    local dir="$1" rc=0 idx="$1/MEMORY.md"
    local rel="${dir#"$ROOT"/}"

    local size
    size=$(wc -c < "$idx" | tr -d ' ')
    SIZES+=("$rel|$size")
    if [[ "$size" -gt "$MAX_INDEX" ]]; then
        echo "  OVERSIZE: $rel/MEMORY.md is $size bytes (max $MAX_INDEX)."
        echo "            Shorten hooks; the detail lives in the topic files."
        rc=1
    fi

    # Links named by the index, and files present on disk.
    local linked present orphans dangling
    linked=$(grep -oE '\(([a-z0-9_.-]+\.md)\)' "$idx" | tr -d '()' | sort -u)
    present=$(ls "$dir" | grep -E '\.md$' | grep -v '^MEMORY\.md$' | sort -u)

    orphans=$(comm -23 <(printf '%s\n' "$present") <(printf '%s\n' "$linked"))
    if [[ -n "$orphans" ]]; then
        echo "  ORPHAN: $rel — file(s) on disk that MEMORY.md never names:"
        printf '%s\n' "$orphans" | sed 's/^/    /'
        rc=1
    fi

    dangling=$(comm -13 <(printf '%s\n' "$present") <(printf '%s\n' "$linked"))
    if [[ -n "$dangling" ]]; then
        echo "  DANGLING: $rel — MEMORY.md names file(s) that do not exist:"
        printf '%s\n' "$dangling" | sed 's/^/    /'
        rc=1
    fi

    # Row shape and hook length, in one awk pass.
    local rows
    rows=$(awk -v maxhook="$MAX_HOOK" '
        /^- \[/ {
            # "- [title](target) <sep> hook"  — sep is an em dash or a hyphen.
            if (match($0, /^- \[[^]]*\]\([^)]*\)/) == 0) {
                printf "UNPARSED\t%d\t%s\n", NR, substr($0, 1, 70); next
            }
            rest = substr($0, RLENGTH + 1)
            sub(/^[ \t]*(\342\200\224|-)[ \t]*/, "", rest)
            if (rest == substr($0, RLENGTH + 1)) {
                printf "UNPARSED\t%d\t%s\n", NR, substr($0, 1, 70); next
            }
            if (length(rest) > maxhook)
                printf "HOOK\t%d\t%d\t%s\n", NR, length(rest), substr($0, 1, 70)
        }' "$idx")

    if [[ -n "$rows" ]]; then
        echo "  ROWS: $rel — $(printf '%s\n' "$rows" | wc -l | tr -d ' ') bad row(s):"
        printf '%s\n' "$rows" | sed 's/^/    /'
        rc=1
    fi

    return "$rc"
}

# ---------------------------------------------------------------------------
# --self-test — five planted violations, each of which the gate must catch.
#
# A grep over files is the category that fails SILENTLY: a pattern that stops
# matching and a directory that stops resolving both print exactly what a clean
# run prints. So the gate is sabotaged before it is believed.
# ---------------------------------------------------------------------------
if [[ "${1:-}" == "--self-test" ]]; then
    TMP=$(mktemp -d)
    trap 'rm -rf "$TMP"' EXIT
    FAILURES=0

    plant() {  # plant <label> <expect 0|1> <builder-fn>
        local label="$1" expect="$2" build="$3"
        rm -rf "$TMP/m"; mkdir -p "$TMP/m"
        "$build"
        local out rc
        out=$(ROOT="$TMP" check_folder "$TMP/m" 2>&1); rc=$?
        if [[ "$rc" -ne "$expect" ]]; then
            echo "memory-index --self-test: FAIL — '$label' returned $rc, expected $expect"
            printf '%s\n' "$out" | sed 's/^/    /'
            FAILURES=$((FAILURES + 1))
        fi
    }

    good_index() {
        printf '%s\n' '# Memory index' '' \
            '- [A thing](a_thing.md) — a short hook.' > "$TMP/m/MEMORY.md"
        echo 'body' > "$TMP/m/a_thing.md"
    }
    b_clean()    { good_index; }
    b_orphan()   { good_index; echo 'body' > "$TMP/m/b_other.md"; }
    b_dangling() { good_index; printf '%s\n' '- [Gone](c_gone.md) — hook.' >> "$TMP/m/MEMORY.md"; }
    b_unparsed() { good_index; printf '%s\n' '- [No separator](a_thing.md) hook with no dash' >> "$TMP/m/MEMORY.md"; }
    b_hook()     { good_index
                   printf -- '- [Long](a_thing.md) %s %s\n' "$(printf '\342\200\224')" \
                       "$(head -c 200 < /dev/zero | tr '\0' 'x')" >> "$TMP/m/MEMORY.md"; }
    b_oversize() { good_index
                   head -c 30000 < /dev/zero | tr '\0' 'x' >> "$TMP/m/MEMORY.md"; }

    plant "clean folder"        0 b_clean
    plant "orphan file"         1 b_orphan
    plant "dangling link"       1 b_dangling
    plant "unparsed row"        1 b_unparsed
    plant "over-long hook"      1 b_hook
    plant "oversize index"      1 b_oversize

    if [[ "$FAILURES" -gt 0 ]]; then
        echo "memory-index --self-test: FAIL — $FAILURES of 6 sabotages went undetected."
        exit 1
    fi
    echo "memory-index --self-test: clean — all 6 sabotages detected (clean, orphan,"
    echo "                          dangling, unparsed, long hook, oversize)."
    exit 0
fi

# ---------------------------------------------------------------------------
# Real run.
# ---------------------------------------------------------------------------
BASE="$ROOT/.claude/agent-memory"
if [[ ! -d "$BASE" ]]; then
    echo "memory-index: SKIPPED — no .claude/agent-memory tree in $ROOT."
    exit 2
fi

FOLDERS=()
while IFS= read -r f; do FOLDERS+=("$(dirname "$f")"); done \
    < <(find "$BASE" -name MEMORY.md -type f | sort)

if [[ "${#FOLDERS[@]}" -eq 0 ]]; then
    echo "memory-index: SKIPPED — .claude/agent-memory exists but holds no MEMORY.md."
    exit 2
fi

RC=0
TOTAL=0
SIZES=()
for d in "${FOLDERS[@]}"; do
    n=$(ls "$d" | grep -cE '\.md$' || true)
    TOTAL=$((TOTAL + n - 1))
    check_folder "$d" || RC=1
done

if [[ "$RC" -ne 0 ]]; then
    cat <<'MSG'

memory-index: FAIL.

An ORPHAN is the serious one: only MEMORY.md is loaded into a session, so a
topic file it does not name is unreachable rather than merely unlinked. Add the
one-line pointer; do not delete the file to make this pass.

An OVERSIZE index is truncated from the END, which hides the NEWEST entries —
exactly the ones a cold session needs. Shorten hooks. The hook decides
relevance; the topic file carries the detail.
MSG
    exit 1
fi

echo "memory-index: clean — ${#FOLDERS[@]} folder(s), $TOTAL memories, every one"
echo "              indexed, every link resolving, every hook inside $MAX_HOOK bytes."

# ⚠ The headroom line prints on GREEN runs. See the NEAR_INDEX note above: a
# budget nobody is told about is spent in silence, and the session that finds
# out is always the one that has no time to fix it properly.
for entry in "${SIZES[@]}"; do
    rel="${entry%%|*}"; size="${entry##*|}"
    head=$((MAX_INDEX - size))
    if [[ "$head" -lt "$NEAR_INDEX" ]]; then
        echo "              ⚠ $rel/MEMORY.md is $size bytes — only $head of $MAX_INDEX"
        echo "                left. Shortening hooks is nearly exhausted at this size;"
        echo "                CONSOLIDATE two entries of the same shape instead."
    else
        echo "              $rel/MEMORY.md: $size bytes, $head of $MAX_INDEX to spare."
    fi
done
exit 0
