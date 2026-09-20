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
# that believes such a warning is minutes from renaming every file in the
# folder to claw back bytes that are not the problem. **This gate is the live
# instrument. The prompt is not.** The memory is
# `an-injected-file-is-a-dated-snapshot`, cited by its `name:` slug because
# that is the half of a memory's identity a rename does not move.
#
# ★★ WHERE THE BYTES ACTUALLY ARE: **the gate prints the split itself** —
# `titles / FILENAMES / hooks / markup`, on every run that is inside
# NEAR_INDEX of the ceiling. No figure is quoted here, because the largest
# share is filenames and the remedy on offer is shortening them, so any number
# written into this header recommends its own obsolescence.
#
# ⚠ The hook is the obvious place to look for slack and it is the WRONG one,
# and the reason is compressibility rather than size. Rewriting the **36
# fattest rows** with intent — keeping every actionable clause, pushing counts
# and second examples down into the topic file where they already live — freed
# **506 bytes, 14 a row**. A hook is a relevance decision, not a summary, and at
# one clause each the next trim costs meaning rather than bytes. Do not read the
# census as a ranking of where to cut: it ranks where the bytes ARE, and the two
# orders are different.
#
# ⇒ THE LEVERS, IN THE ORDER THEY ARE NOW WORTH PULLING:
#
#   1. **Consolidate two entries of the same shape.** ~175 bytes each, loses
#      nothing but a pointer, and the folder has several families (absence
#      claims, stale citations, checks that cannot fail) whose members differ
#      by less than a row's worth of meaning.
#   2. **Write SHORTER FILENAMES for new memories.** The existing stems are
#      capped at 30 characters; a new memory arriving with a 60-character one
#      costs the index twice as much and buys nothing, because the filename is
#      an address and carries no meaning that is not carried better elsewhere —
#      the `name:` frontmatter holds the slug `[[wikilinks]]` resolve against,
#      and `MEMORY.md`'s title holds the prose.
#   3. **Renaming the existing files** is largely spent — the stems are already
#      at the cap, and the headroom line prints what that left. Before any
#      further sweep, know the invariant that makes one cost more than it looks:
#      ★ `[[wikilinks]]` resolve against a memory's `name:` frontmatter **OR**
#      its filename stem. A link written in the slug form survives any rename;
#      a link written in the stem form is broken by one, silently, because
#      nothing but a reading session ever follows these links. Every link in
#      the folder is currently the slug form, and new ones must be written that
#      way. CROSSREF below is what reddens when a rename outruns the rewrite.
#      Nothing outside this folder cites a memory filename — verify that with a
#      grep before believing it, rather than re-quoting this line.
#      *Tidying an input changes every instrument that reads it*, so still: do
#      it only when 1 and 2 are exhausted, and in a commit of its own.
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
#   6. CROSSREF   — a `[[link]]` inside a memory file that resolves to nothing
#                   while nearly naming a real memory. RED: that is a rename or
#                   a typo, and it breaks a reference nothing else can see. An
#                   unresolved link with no near match is the documented way to
#                   mark a memory worth writing later and is left alone — the
#                   self-test plants both directions, because a rule that
#                   reddened on every unresolved link would forbid the
#                   convention the folder is written in.
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
# check_folder <dir> — all six rules against one agent's memory folder.
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

    # ★★★ CROSSREF. A `[[link]]` inside a memory file resolves against another
    # memory's `name:` slug or its filename stem. An unresolved link is
    # DELIBERATELY allowed — the convention uses one to mark a memory worth
    # writing later — so a gate that reddened on every unresolved link would
    # forbid the convention and be turned off. What is red is an unresolved link
    # that NEARLY names a real target: that is a rename or a typo, and the
    # reference it broke is invisible, because nothing but a reading session ever
    # follows these links and a session that cannot follow one does not know it
    # was there. Eight of them had accumulated when this rule was written.
    #
    # "Nearly" is two deterministic tests, both cheap and both explainable:
    # equal after normalising (lowercase, `_`→`-`, drop a `feedback`/`project`/
    # `user` prefix), or a shared 20-character prefix. Twenty is chosen so that
    # `write-the-lesson-to-the-rag-not-the-chat` — a genuine forward marker that
    # shares `write-the-` with a real memory — stays green, while
    # `the-engine-session-runs-in-parallel`, which is a real memory's slug with
    # its tail lopped off, goes red.
    local targets links crossref
    targets=$( { sed -n 's/^name:[[:space:]]*//p' "$dir"/*.md 2>/dev/null
                 ls "$dir" | grep -E '\.md$' | grep -v '^MEMORY\.md$' | sed 's/\.md$//'
               } | sed 's/[[:space:]]*$//' | grep -v '^$' | sort -u )
    links=$(grep -ho '\[\[[^]]*\]\]' "$dir"/*.md 2>/dev/null \
                | sed 's/^\[\[//; s/\]\]$//' | sort -u)
    crossref=$(printf '%s\n' "$links" | awk -v known="$targets" '
        function norm(s) {
            s = tolower(s); gsub(/_/, "-", s)
            sub(/^(feedback|project|user)-/, "", s)
            return s
        }
        BEGIN {
            n = split(known, k, "\n")
            for (i = 1; i <= n; i++) if (k[i] != "") { raw[k[i]] = 1; nk[norm(k[i])] = k[i] }
        }
        $0 != "" {
            if ($0 in raw) next
            l = norm($0)
            if (l in nk) { printf "%s\t%s\n", $0, nk[l]; next }
            for (t in nk) {
                m = 0
                while (m < length(l) && m < length(t) && substr(l, m + 1, 1) == substr(t, m + 1, 1)) m++
                if (m >= 20) { printf "%s\t%s\n", $0, nk[t]; next }
            }
        }')
    if [[ -n "$crossref" ]]; then
        echo "  CROSSREF: $rel — [[link]](s) resolving to nothing but nearly naming a real memory:"
        printf '%s\n' "$crossref" | awk -F'\t' '{ printf "    [[%s]]\n      did you mean  [[%s]]\n", $1, $2 }'
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
# --self-test — eight plants: six the gate must catch, and two it must leave
# alone — an untouched folder, and the forward-marker wikilink CROSSREF is
# deliberately blind to. A rule with no green plant beside it drifts wider
# every time someone tightens it.
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

    # A folder whose one memory has a filename long enough for the 20-character
    # prefix rule to have something to bite on.
    long_index() {
        printf '%s\n' '# Memory index' '' \
            '- [A thing](a_thing_with_a_long_enough_name.md) — a short hook.' \
            > "$TMP/m/MEMORY.md"
        printf '%s\n' 'body' > "$TMP/m/a_thing_with_a_long_enough_name.md"
    }
    b_crossref() { long_index
                   printf '%s\n' 'see [[a-thing-with-a-long-enough-namex]]' \
                       >> "$TMP/m/a_thing_with_a_long_enough_name.md"; }
    # ★ The other direction, and it is the one that keeps the rule honest: an
    # unresolved link with no near match is the documented way to mark a memory
    # not yet written, and must stay GREEN. Without this plant the cheapest way
    # to make CROSSREF pass would be to redden on every unresolved link, which
    # would forbid the convention the folder is written in.
    b_forward()  { long_index
                   printf '%s\n' 'see [[something-entirely-different]]' \
                       >> "$TMP/m/a_thing_with_a_long_enough_name.md"; }

    plant "clean folder"        0 b_clean
    plant "orphan file"         1 b_orphan
    plant "dangling link"       1 b_dangling
    plant "unparsed row"        1 b_unparsed
    plant "over-long hook"      1 b_hook
    plant "oversize index"      1 b_oversize
    plant "broken crossref"     1 b_crossref
    plant "forward marker"      0 b_forward

    if [[ "$FAILURES" -gt 0 ]]; then
        echo "memory-index --self-test: FAIL — $FAILURES of 8 sabotages went undetected."
        exit 1
    fi
    echo "memory-index --self-test: clean — all 8 sabotages detected (clean, orphan,"
    echo "                          dangling, unparsed, long hook, oversize, broken"
    echo "                          crossref, and a forward marker left alone)."
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

A CROSSREF is a reference that used to work. Fix the link; do not delete it and
do not rename the target back. It is reported only when the dead name nearly
matches a live one, so it is a rename or a typo rather than a forward marker —
and it is invisible without this rule, because the only reader that follows
these links is a session, and a session that cannot follow one does not learn
it was there.
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
        # ★ MEASURED, not quoted. A census the gate computes cannot go stale,
        # and a frozen one recommends its own obsolescence: shortening filenames
        # is the remedy on offer, so the figure moves the first time anyone takes
        # the advice. The four shares are printed together because the obvious
        # target — the hook — is the least compressible of them.
        # "hooks" includes each row's " — " separator; "markup" is whatever the
        # other three do not account for.
        census=$(LC_ALL=C awk -v total="$size" '
            substr($0, 1, 3) == "- [" {
                p = index($0, "](");   if (p == 0) next
                rest = substr($0, p + 2)
                q = index(rest, ")");  if (q == 0) next
                t += p - 4; fn += q - 1; hk += length(rest) - q
            }
            END { printf "titles %d  FILENAMES %d  hooks %d  markup %d",
                         t, fn, hk, total - t - fn - hk }' "$ROOT/$rel/MEMORY.md")
        echo "                filenames — they are the largest share: $census."
    else
        echo "              $rel/MEMORY.md: $size bytes, $head of $MAX_INDEX to spare."
    fi
done
exit 0
