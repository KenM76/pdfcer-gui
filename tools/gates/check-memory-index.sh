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
# ⇒ **This gate exists because that is not hypothetical.**
# `user_he_is_not_at_the_keyboard_unless_he_says_so.md` — the standing rule that
# decides whether a session drives the release binary or defers the work to the
# operator, one of the most operationally load-bearing memories in the folder —
# can exist on disk, be committed, and be named nowhere in `MEMORY.md`. Only a
# `comm` between the two lists can see it.
#
# ★★ It is the same shape this repository has recorded under several other
# names: a checker named in every document and registered in no runner; a
# `--self-test` written and never dispatched. **Writing the artifact and
# registering the artifact are two edits, and the second one is the one that
# gets skipped**, because the first is where the thinking was.
#
# ═══════════════════════════════════════════════════════════════════════════
# THE SECOND FAILURE: AN INDEX TOO LARGE TO LOAD
# ═══════════════════════════════════════════════════════════════════════════
#
# The index is injected into context wholesale and **TRUNCATED when it is too
# big** — the harness's own warning reads:
#
#     "WARNING: MEMORY.md is 25.4KB (limit: 24.4KB) — index entries are too
#      long. Only part of it was loaded."
#
# Truncation takes the END of the file, which is where the NEWEST entries are.
# An index that grows past the limit therefore hides the memories a cold session
# most needs — the ones written last — while continuing to display the oldest
# ones, so nothing about the loaded text looks wrong.
#
# ★★★ THAT CITATION IS ARITHMETIC RATHER THAN A BORROWED NUMBER, and the
# difference matters: a figure with an unknown shelf life gets re-guessed by
# every session, while a conversion can be checked in one line.
#
#   **25,411 bytes is exactly `git show dc5c0c3:…/MEMORY.md | wc -c`**, and the
#   harness called that file `25.4KB`.
#
# ⇒ The harness's KB is **bytes ÷ 1000**, and it measures **this file**, not the
# folder, not the topic files, not a rendering of them. So the ceiling is
# **24,400 bytes**, `MAX_INDEX = 24000` leaves 400 bytes of deliberate margin,
# and anyone doubting either number can re-derive both from one `wc -c` against
# one harness warning. **If this gate fails on SIZE, shorten the index — do not
# raise the constant**; and if the harness ever quotes a limit other than
# 24.4 KB, change the constant here, say so in the commit, and keep the check.
#
# ⚠ AND DO NOT TRUST THE WARNING'S FRESHNESS. The `MEMORY.md` block injected
# into a session, warning and all, is a SNAPSHOT that can be a day and a half
# behind the file — the quotation above describes 25,411 bytes against a file
# since cut to 23,987 (= 24.0 KB, under the limit, not truncated). A session
# that believes such a warning is minutes from renaming all 136 memory files to
# claw back bytes that are not the problem. **This gate is the live instrument.
# The prompt is not.** See
# `feedback_the_injected_memory_warning_is_a_snapshot.md`.
#
# ★★ WHERE THE BYTES ACTUALLY ARE, measured over 135 rows:
#
#     titles 6,914   FILENAMES 8,942   hooks 6,900   markup 1,080
#
# ⚠ The hook is the obvious place to look for slack and it is the WRONG one:
# it is the smallest third and the least compressible. Rewriting the **36
# fattest rows** with intent — keeping every actionable clause, pushing counts
# and second examples down into the topic file where they already live — freed
# **506 bytes, 14 a row**. A hook is a relevance decision, not a summary, and at
# one clause each the next trim costs meaning rather than bytes.
#
# ⇒ THE LEVERS, IN THE ORDER THEY ARE NOW WORTH PULLING:
#
#   1. **Consolidate two entries of the same shape.** ~175 bytes each, loses
#      nothing but a pointer, and the folder has several families (absence
#      claims, stale citations, checks that cannot fail) whose members differ
#      by less than a row's worth of meaning.
#   2. **Write SHORTER FILENAMES for new memories.** 66 bytes is the current
#      average and nothing needs it; the `name:` frontmatter carries the slug
#      that `[[wikilinks]]` resolve against, and `MEMORY.md`'s title carries
#      the prose. A 30-byte name costs the index half of a 60-byte one.
#   3. **Renaming the existing 136** is the big win (≈4,000 bytes) and the one
#      to resist: it rewrites 309 `[[wikilinks]]`, every citation of a memory
#      filename anywhere in the tree, and every filename a past session ever
#      grepped for. *Tidying an input changes every instrument that reads it.*
#      Do it only when 1 and 2 are exhausted, and in a commit of its own.
#   4. **Shortening hooks** — spent, see above. Listed last deliberately,
#      because it is the one a session reaches for first.
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
MAX_INDEX=24000  # bytes. The harness truncates at 24,400 — exactly, see the
                 # conversion above — so this is a 400-byte margin, chosen so
                 # the gate goes red BEFORE any entry is lost, and small enough
                 # that it does not silently ration the index either.
NEAR_INDEX=2400  # bytes of headroom below which a GREEN run says so loudly.

# ★★★ A PASS THAT DOES NOT SAY HOW CLOSE IT CAME IS A PASS THAT EXPIRES
# WITHOUT WARNING. Red at 24,177 bytes, fifteen hooks shortened, green again at
# **23,984** — sixteen bytes of headroom, and a green run that says only
# "clean". The next session to add one memory hits the same red and re-derives
# the same remedy from scratch, because nothing green ever told it the budget
# was spent.
#
# ⚠ AND NEAR THE CEILING THE REMEDY CHANGES. "Shorten
# hooks" works while the hooks are padded; **36 of the fattest rows, rewritten
# carefully, yielded 14 bytes each.** Below NEAR_INDEX the instruction is the
# lever list in the header: consolidate same-shape entries first, write shorter
# filenames for new memories second, and treat a mass rename as a commit of its
# own that must be argued for. Printing the headroom is what makes that
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
        echo "            Consolidate two entries of the same shape (~175 bytes),"
        echo "            or shorten hooks — but hooks yield ~14 bytes a row now."
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
exactly the ones a cold session needs. The remedy is NOT "shorten hooks"; that
yields about 14 bytes a row now, measured. Consolidate two entries of the same
shape, which frees a whole row and loses only a pointer — the header lists the
levers in the order they are worth pulling.
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
        echo "                left (the harness truncates at 24400). Hook-shortening is"
        echo "                SPENT: 36 rewritten rows freed 14 bytes each. CONSOLIDATE"
        echo "                two entries of the same shape, and give new memories short"
        echo "                filenames — filenames are 8,942 of these bytes."
    else
        echo "              $rel/MEMORY.md: $size bytes, $head of $MAX_INDEX to spare."
    fi
done
exit 0
