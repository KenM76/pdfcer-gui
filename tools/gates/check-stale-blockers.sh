#!/usr/bin/env bash
#
# check-stale-blockers.sh — a row that says BLOCKED must not name a request the
# engine has already answered.
#
# ## Why this gate exists
#
# On 2026-09-02 `EditSession::reorder_annotations` shipped a few hours after the
# request that asked for it. Wiring it up took a morning. Finding everything
# that had *asserted the gap* took longer, and most of it was found by looking
# rather than by any instrument:
#
#   * the panel's on-screen explainer — "This view reports the order; it does
#     not change it";
#   * the module header, which was a flat prohibition on building the drag;
#   * a PASSING unit test that would have forbidden the feature;
#   * a ⛔ row in FEATURES.md that had stood for nineteen days.
#
# ★★ Every one of them was correct when written. That is exactly what makes the
# class survive — nothing about a true-when-written sentence looks wrong, and no
# other gate evaluates it. `check-ui-strings` proves a string is in the catalog,
# not that it is TRUE.
#
# ## What this gate can and cannot do
#
# It catches the one part of that class which is **mechanical**: a row that
# declares itself blocked *and* names the request file, where the channel shows
# that request has been consumed (renamed to `done_*`). Filing a request and
# retiring it are both explicit acts, so "the row still says blocked and the
# request is closed" is a contradiction a script can see.
#
# ⬜ It does NOT catch the other three. A stale sentence in a module header, a
# stale operator-facing string, or an absence test that outlived its absence are
# all semantic, and this gate makes no attempt at them. See HANDOFF.md §10 for
# the manual procedure, which this gate supplements and does not replace. A gate
# that silently implied full coverage of that class would be worse than none.
#
# ## Exit codes
#
# `--self-test` drives four hermetic cases and is what makes the widening
# below falsifiable rather than merely asserted; see the block near the top
# of the script body.
#
#   0  no contradiction found, having actually looked
#   1  at least one row claims to be blocked on a request that has been answered
#   2  SKIPPED — the channel is not on this machine, or it holds no
#      consumption notes at all, so there was nothing to check against
#
# ★ Exit 2 corrected 2026-09-14. The paragraph below has always said this
# gate SKIPs rather than passes when it cannot see its evidence — and the
# code exited **0**, which `run-all.sh` counts as a PASS. The prose was
# right, the statement was wrong, and nothing could see the difference
# because the two sat forty lines apart in the same file.
#
# ## Skipping honestly
#
# If the channel directory is absent — a clone on another machine, CI without
# the shared drive — this SKIPs with a stated reason rather than passing. A gate
# that cannot see its evidence has not checked anything, and reporting that as
# green is the failure this project has hit before.

set -uo pipefail

CHANNEL="${PDFCER_REQUEST_CHANNEL:-D:/Dev/FeatureRequests/pdfce_FeatureRequests/open}"
ARCHIVE="$(dirname "$CHANNEL")/archive"

# PDFCER_GATE_ROOT exists for --self-test and nothing else. The real run
# always resolves the repository from this script's own location, so an
# environment variable cannot quietly point the gate at an empty tree.
ROOT="${PDFCER_GATE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

# ---------------------------------------------------------------------------
# --self-test — four cases, each the only witness for one property.
# ---------------------------------------------------------------------------
#
# ★★★ The `archived` case is the regression guard for 2026-09-14. Before that
# date this gate globbed `open/*CONSUMED*.md`; the channel sweep moved every
# consumption note to `archive/`, the glob matched nothing, and the gate went
# permanently green over an evidence set of size zero. The `archived` case
# returns 1 only if `archive/` is actually read, so narrowing the evidence set
# back to `open/` turns this suite red instead of turning this gate blind.
#
# ⚠ Hermetic on purpose: a throwaway ROOT and a throwaway channel. A case
# keyed on a real ⛔ row in FEATURES.md would be testing that row's wording,
# and would evaporate the day somebody unblocks it.
if [ "${1:-}" = "--self-test" ]; then
  TMP="$(mktemp -d)"
  trap 'rm -rf "$TMP"' EXIT
  FAILURES=0

  # A root carrying exactly one blocked row that names a request file.
  mkdir -p "$TMP/root"
  printf '%s\n' \
    '| Deep zoom | ⛔ BLOCKED on `request_selftest_widget.md` | not yet |' \
    > "$TMP/root/OPERATOR_REQUESTS.md"

  note_body() {
    printf '%s\n' '# done' '' '**Status:** consumed.' '' \
      'Originally filed as: request_selftest_widget.md'
  }

  run_case() {  # <label> <expected-rc>  (channel already built)
    local label="$1" expect="$2" out rc
    out="$(PDFCER_GATE_ROOT="$TMP/root" PDFCER_REQUEST_CHANNEL="$TMP/ch/open" \
           bash "${BASH_SOURCE[0]}" 2>&1)"; rc=$?
    if [ "$rc" -ne "$expect" ]; then
      echo "stale-blockers --self-test: FAIL — '$label' returned $rc, expected $expect"
      printf '%s\n' "$out" | sed 's/^/    /'
      FAILURES=$((FAILURES + 1))
    fi
  }

  # empty  — a channel with no consumption notes has measured nothing.
  rm -rf "$TMP/ch"; mkdir -p "$TMP/ch/open" "$TMP/ch/archive"
  run_case "empty" 2

  # clean  — notes exist, but none of them names this request.
  rm -rf "$TMP/ch"; mkdir -p "$TMP/ch/open" "$TMP/ch/archive"
  printf '%s\n' '# done' '' '**Status:** consumed.' '' 'Originally filed as: request_something_else.md' \
    > "$TMP/ch/archive/2026-01-01-unrelated-done.md"
  run_case "clean" 0

  # archived — THE REGRESSION GUARD: the note is in archive/, not open/.
  rm -rf "$TMP/ch"; mkdir -p "$TMP/ch/open" "$TMP/ch/archive"
  note_body > "$TMP/ch/archive/2026-01-01-selftest-widget-done.md"
  run_case "archived" 1

  # open   — the original path still works.
  rm -rf "$TMP/ch"; mkdir -p "$TMP/ch/open" "$TMP/ch/archive"
  note_body > "$TMP/ch/open/done_selftest_widget_CONSUMED.md"
  run_case "open" 1

  if [ "$FAILURES" -ne 0 ]; then
    echo "stale-blockers --self-test: FAIL — $FAILURES of 4 cases misbehaved."
    exit 1
  fi
  echo "stale-blockers --self-test: clean — all 4 cases behaved (empty=SKIPPED,"
  echo "                            clean, archived, open)."
  exit 0
fi

if [ ! -d "$CHANNEL" ]; then
  echo "check-stale-blockers: SKIPPED — the request channel is not at '$CHANNEL'."
  echo "  Set PDFCER_REQUEST_CHANNEL to point at it. Reported as SKIPPED rather"
  echo "  than PASS: this gate has not looked at anything."
  exit 2
fi

# ---------------------------------------------------------------------------
# THE EVIDENCE SET — and the afternoon it silently became empty.
# ---------------------------------------------------------------------------
#
# ★★★ This gate used to glob `"$CHANNEL"/*CONSUMED*.md`, which is `open/` and
# nothing else. On 2026-09-14 this project swept the channel — `open/` went
# 48 → 4 — and all fifty-one consumption notes moved to `archive/`. **From
# that moment the glob matched nothing**, a `grep` over an empty file list
# never matches, and the gate could not go red however stale a row became. It
# printed "OK — no row declares a blocker that has been closed" over an
# evidence set of size **zero**, in the same suite run as the sweep that
# emptied it.
#
# Two lessons, both of which this file now ENACTS rather than merely records:
#
#   1. **Tidying an input is a change to the instrument.** Nothing about
#      archiving closed exchanges looks like touching a gate, and no check on
#      either side connects the two — the channel is in no git repository, so
#      its contents are invisible to all of them. The defence is that the
#      gate reads BOTH folders, below, and stops caring where a note lives.
#   2. **A check with an empty evidence set must not report PASS.** It has
#      measured nothing. Below, an empty set is SKIPPED (exit 2) with the
#      count stated, and the count is printed on GREEN runs too. A tally that
#      can be zero is the only thing that makes a green verdict legible.
#
# A consumption note is recognised under BOTH naming schemes the channel has
# used, because `archive/` holds both eras:
#
#   * `*CONSUMED*.md`            — the original `done_<topic>_CONSUMED.md`
#   * `*-done.md`, `*-done-N.md` — the dated archive stem, which drops the word
#   * any file whose opening lines carry `**Status:** … consumed`
#
# The third clause is the backstop for a scheme nobody has invented yet: the
# status line is the thing that actually means "taken", and a filename is only
# ever a shorthand for it.
NOTES=()
for f in "$CHANNEL"/*.md "$ARCHIVE"/*.md; do
  [ -f "$f" ] || continue
  b="$(basename "$f")"
  case "$b" in
    *CONSUMED*|*consumed*)            NOTES+=("$f"); continue ;;
    done_*|*-done.md|*-done-[0-9].md) NOTES+=("$f"); continue ;;
  esac
  if head -12 "$f" | grep -qiE '^\*\*Status:.*consumed'; then
    NOTES+=("$f")
  fi
done

if [ "${#NOTES[@]}" -eq 0 ]; then
  echo "check-stale-blockers: SKIPPED — the channel holds NO consumption notes."
  echo "  Looked in '$CHANNEL' and '$ARCHIVE'. The only evidence this gate has"
  echo "  that a blocker was closed is a note saying so, and there are none, so"
  echo "  it cannot tell a clean tree from a blind one. SKIPPED, not PASS — see"
  echo "  the note above about 2026-09-14."
  exit 2
fi

# Documents that carry status rows. Deliberately a short, named list rather than
# a sweep of every *.md: CONTINUE.md and HANDOFF.md are HISTORICAL records, and
# a past tick correctly says "blocked" about the day it was written. Rewriting
# history to keep a gate green would destroy the thing those files are for.
#
# ★★★ ENGINE_BACKLOG.md WAS MISSING FROM THIS LIST UNTIL 2026-09-09, AND IT IS
# THE FILE WHOSE ENTIRE PURPOSE IS BLOCKED ROWS.
#
# Three rows in it declared `BLOCKED` on requests the engine had answered on
# 2026-09-06 — dashed borders, a discarded markup width, and `endings` not being
# a `StyleEdit`. All three shipped, all three were wired here, and this gate ran
# green over them for three days because it never opened the file. The list is
# hand-written, so a document is invisible to the check built to find exactly
# this class until somebody remembers to add it; nothing about the omission
# looked wrong, and the count of documents scanned still added up.
#
# ⇒ When adding a document that carries status rows, add it HERE in the same
# commit. There is no discovery step that will do it for you, and that is
# deliberate: see the paragraph above about historical records, which is why
# this cannot simply become a sweep of every *.md.
#
# ★★ AND ADDING THE FILE WAS ONLY HALF THE FIX. This gate's evidence is a
# `*CONSUMED*.md` note that THIS side writes, which is correct — a reply that
# schedules, defers or refuses must not clear a blocker, and only this side
# knows when a capability has actually been taken. The consequence is that a
# consumption nobody records makes the gate blind by construction. All four of
# the 2026-09-06 markup asks were wired here and no note was ever written, so
# even with the file in this list the gate would still have passed. Writing the
# CONSUMED note is the act that arms this check; see
# `archive/2026-09-06-markup-style-four-CONSUMED.md` — archived 2026-09-14,
# which is why this gate now reads both folders rather than just `open/`.
DOCS=("OPERATOR_REQUESTS.md" "FEATURES.md" "GUI_ROADMAP.md" "ENGINE_BACKLOG.md")

status=0
found=0
excused=0

for doc in "${DOCS[@]}"; do
  path="$ROOT/$doc"
  [ -f "$path" ] || continue

  # A "blocked claim" is a line that says so in any of the shapes these
  # documents actually use, AND names a request file on the same line.
  # Same-line is deliberate: it keeps the rule unambiguous and keeps a
  # paragraph three screens below a heading from being attributed to it.
  while IFS= read -r hit; do
    lineno="${hit%%:*}"
    text="${hit#*:}"

    # ★★★ A ROW THAT HAS ALREADY BEEN CORRECTED STILL CONTAINS THE WORD IT WAS
    # CORRECTED ABOUT, AND FIRING ON IT IS WORSE THAN MISSING IT.
    #
    # Added 2026-09-09, immediately after this gate found three genuinely stale
    # rows. Re-run on the FIXED file, it reported two more — and both were
    # correct rows narrating their own history:
    #
    #   FEATURES.md      "⚠ THE ⛔ ON THIS ROW WAS STALE AND IS CORRECTED …"
    #   ENGINE_BACKLOG.md "**Reachable** … ★★★ This row was `blocked` for about
    #                      six hours and is the shortest-lived blocker …"
    #
    # Both are ✅/Reachable rows. Both name the request file, because a
    # correction that erased its own citation could not be audited — which this
    # project requires. ⇒ **The word "blocked" on those lines is a NARRATION OF
    # THE PAST, not a claim about today**, and the gate had no way to tell the
    # two apart because its predicate was "the token appears anywhere on the
    # line".
    #
    # ★★ Why this mattered enough to fix rather than tolerate. This gate's own
    # header records the first version firing on a TRUE warning and warns that
    # such a failure "would have had somebody delete a true warning to make a
    # build go green". A false positive on a correctly-updated row is the same
    # hazard pointing the other way: the cheapest way to clear it is to delete
    # the history sentence, and the history sentence is the most valuable part
    # of a corrected row.
    #
    # The rule: a Markdown table row's VERDICT lives at the start of a cell, so
    # a cell-initial closure token settles the row's status for today and any
    # blocked token elsewhere on the line is history. The tokens below are the
    # ones these four documents actually use — measured, not invented:
    #   FEATURES.md / OPERATOR_REQUESTS.md / GUI_ROADMAP.md → ✅
    #   ENGINE_BACKLOG.md → **Reachable, **Consumed, **WIRED, **UNBLOCKED
    #
    # ⬜ Known limit, stated rather than papered over: a row that is genuinely
    # blocked TODAY and whose author begins its verdict cell with one of these
    # tokens is excused. That shape does not exist in these documents and would
    # be self-contradictory prose if it did — but it is a hole, not a proof.
    #
    # ⚠ Excuses are COUNTED AND REPORTED on every run, including green ones. An
    # exclusion nobody can see is how a gate stops measuring without anybody
    # noticing; see the SKIP-set discipline in the same family of findings.
    if echo "$text" | grep -qE '\| *(✅|\*\*(Reachable|Consumed|WIRED|UNBLOCKED))'; then
      excused=$((excused + 1))
      continue
    fi

    # Every request file named on this line.
    for req in $(echo "$text" | grep -o 'request_[a-z0-9_]*\.md' | sort -u); do
      if [ -f "$CHANNEL/$req" ]; then
        continue  # still open — the claim is current
      fi
      # ★★★ THE PREDICATE IS "CONSUMED", NOT "ANSWERED", AND THE DIFFERENCE IS
      # THE WHOLE CORRECTNESS OF THIS GATE.
      #
      # The first version of this script asked only whether the request was
      # still in `open/` under its own name. It fired immediately, on
      # FEATURES.md's deep-zoom row, which is blocked on
      # `request_reusable_parsed_handle.md` — a request the engine answered on
      # 2026-08-13 with "scheduled as a Pass", and then archived. The reply
      # CLOSED THE THREAD. The Pass has not landed; there is still no reusable
      # handle anywhere in `pdfcer-render`. **The row was correct and the gate
      # was wrong**, and it was wrong in the direction that costs most: it would
      # have had somebody delete a true warning to make a build go green.
      #
      # So the signal is a `*CONSUMED*.md` note, which is written by THIS side
      # and only once the capability has actually been taken. A reply that
      # schedules, defers, refuses, or merely explains produces no CONSUMED
      # note — and none of those clear a blocker.
      #
      # ★ The note names the request by its ORIGINAL filename, because a
      # consumed pair gets renamed to a dated `done_*` stem and the name the
      # rows cite would otherwise be unrecoverable. That is a convention this
      # gate depends on; `done_2026-09-02-*-CONSUMED.md` carry it as an
      # "Originally filed as:" line.
      note="$(grep -lF "$req" "${NOTES[@]}" 2>/dev/null | head -1)"
      if [ -n "$note" ]; then
        echo "  $doc:$lineno claims to be blocked and names '$req',"
        echo "      which has been CONSUMED — the capability is wired on this side."
        echo "      Evidence: $(basename "$note")"
        found=$((found + 1))
        status=1
      fi
    done
  done < <(grep -n -i -E '(BLOCKED|⛔|no verb (can|that)|cannot be (changed|done) at all)' "$path" || true)
done

# ⚠ Report the exclusions BEFORE the verdict, on green runs and red ones alike.
# A number here that climbs without anybody noticing is this gate quietly
# narrowing its own scope; a number that is suddenly zero is a token these
# documents stopped using.
if [ "$excused" -ne 0 ]; then
  echo "check-stale-blockers: $excused line(s) excused as HISTORY — a cell-initial"
  echo "  ✅ / Reachable / Consumed / WIRED / UNBLOCKED settles the row for today,"
  echo "  so a 'blocked' elsewhere on the line is narration. Grep them if this"
  echo "  count moves unexpectedly."
fi

if [ "$status" -ne 0 ]; then
  echo
  echo "error: $found row(s) still declare a blocker the engine has closed."
  echo
  echo "A row that says BLOCKED is a claim about today, and it was true when it"
  echo "was written. Update the row — and then go and look for the other three"
  echo "places this gate CANNOT see: the module header that forbade the feature,"
  echo "the operator-facing string that describes the gap, and any test named"
  echo "no_* / never_* / not_* that asserts the absence. HANDOFF.md §10 has the"
  echo "procedure and the case that produced it."
  exit 1
fi

# ⚠ STATE THE EVIDENCE SET, NOT JUST THE VERDICT. On 2026-09-14 this gate
# reported the line below over an evidence set of size zero and nobody could
# tell, because "OK" is what it prints when it has looked and found nothing
# AND what it printed when it had nothing to look at. The two readings are
# now distinguishable from the output alone.
echo "check-stale-blockers: OK — no row declares a blocker that has been closed."
echo "  Evidence: ${#NOTES[@]} consumption note(s) across"
echo "    $CHANNEL"
echo "    $ARCHIVE"
echo "  A zero there would be this gate going blind, and is reported as SKIPPED."
exit 0
