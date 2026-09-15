#!/usr/bin/env bash
#
# check-stale-blockers.sh — a row that says BLOCKED must not name a request the
# engine has already answered.
#
# ## Why this gate exists
#
# When the engine answers a request, wiring the capability up is the short part.
# Finding everything on this side that had *asserted the gap* is the long part,
# and nothing but looking finds most of it:
#
#   * a panel's on-screen explainer — "This view reports the order; it does
#     not change it";
#   * a module header carrying a flat prohibition on building the verb;
#   * a PASSING unit test that would forbid the feature;
#   * a ⛔ row in FEATURES.md.
#
# ★★ Every one of those is correct when written. That is exactly what makes the
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
# all semantic, and this gate makes no attempt at them: they are swept by hand,
# and this gate supplements that sweep rather than replacing it. A gate that
# silently implied full coverage of that class would be worse than none.
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
# ★ Exit 2 is load-bearing: `run-all.sh` counts an exit of 0 as a PASS, so a
# SKIP that exits 0 reports a gate which looked at nothing as green. The
# paragraph below and the `exit` statement that enacts it sit forty lines
# apart in the same file, and nothing but this rule holds them together.
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
# ★★★ The `archived` case is the regression guard on the evidence set. A gate
# that globbed `open/*CONSUMED*.md` alone matches nothing the moment the channel
# sweeps its consumption notes into `archive/`, and then goes permanently green
# over an evidence set of size zero. The `archived` case returns 1 only if
# `archive/` is actually read, so narrowing the evidence set back to `open/`
# turns this suite red instead of turning this gate blind.
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
# THE EVIDENCE SET, AND HOW IT GOES SILENTLY EMPTY.
# ---------------------------------------------------------------------------
#
# ★★★ A glob of `"$CHANNEL"/*CONSUMED*.md` reads `open/` and nothing else.
# Sweep the channel — every consumption note moved to `archive/` — and **the
# glob matches nothing**; a `grep` over an empty file list never matches, and
# the gate cannot go red however stale a row becomes. It prints "OK — no row
# declares a blocker that has been closed" over an evidence set of size
# **zero**, in the same suite run as the sweep that emptied it.
#
# Two rules, both of which this file ENACTS rather than merely records:
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
# a sweep of every *.md: a dated record correctly says "blocked" about the day
# it was written, and rewriting history to keep a gate green would destroy the
# thing such a file is for.
#
# ★★★ THE LIST IS HAND-WRITTEN, AND THAT IS THE HOLE IN IT. A document carrying
# blocked rows is invisible to the check built to find exactly this class until
# somebody adds it here, and nothing about the omission looks wrong because the
# count of documents scanned still adds up. The list below is the entirety of
# this gate's scope: a blocked row in any other file is unchecked, and the run
# that missed it reports exactly the same green as the run that checked it.
#
# ⇒ When adding a document that carries status rows, add it HERE in the same
# commit. There is no discovery step that will do it for you, and that is
# deliberate: see the paragraph above about historical records, which is why
# this cannot simply become a sweep of every *.md.
#
# ★★ AND LISTING THE FILE IS ONLY HALF OF IT. This gate's evidence is a
# `*CONSUMED*.md` note that THIS side writes, which is correct — a reply that
# schedules, defers or refuses must not clear a blocker, and only this side
# knows when a capability has actually been taken. The consequence is that a
# consumption nobody records makes the gate blind by construction: every ask can
# be wired here and, with no note written, the gate still passes. **Writing the
# CONSUMED note is the act that arms this check.** A note may sit in either
# folder, because the channel sweeps consumed threads out of `open/`, which is
# why this gate reads both and not just `open/`.
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
    # A row that has been corrected keeps the sentence recording the correction,
    # so it still contains the word it was corrected about:
    #
    #   FEATURES.md      "⚠ THE ⛔ ON THIS ROW WAS STALE AND IS CORRECTED …"
    #   ENGINE_BACKLOG.md "**Reachable** … this row was `blocked` for about six
    #                      hours …"
    #
    # Both are ✅/Reachable rows. Both name the request file, because a
    # correction that erased its own citation could not be audited — which this
    # project requires. ⇒ **The word "blocked" on those lines is a NARRATION OF
    # THE PAST, not a claim about today**, and a predicate of "the token appears
    # anywhere on the line" cannot tell the two apart.
    #
    # ★★ Why this needs a rule rather than tolerance. Firing on a TRUE warning
    # — the hazard the CONSUMED-not-ANSWERED block below exists to prevent —
    # would have somebody delete a true warning to make a build go green. A
    # false positive on a correctly-updated row is the same hazard pointing the
    # other way: the cheapest way to clear it is to delete the history
    # sentence, and the history sentence is the most valuable part of a
    # corrected row.
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
      # A predicate of "the request is no longer in `open/` under its own name"
      # fires on every request the engine merely *replied* to. A reply that
      # schedules the work, and then archives the thread, leaves the capability
      # exactly as absent as it was — and the blocked row naming it is still
      # correct. A gate that goes red there is wrong in the direction that costs
      # most: it would have somebody delete a true warning to make a build go
      # green.
      #
      # So the signal is a `*CONSUMED*.md` note, which is written by THIS side
      # and only once the capability has actually been taken. A reply that
      # schedules, defers, refuses, or merely explains produces no CONSUMED
      # note — and none of those clear a blocker.
      #
      # ★ The note names the request by its ORIGINAL filename, because a
      # consumed pair gets renamed to a dated `done_*` stem and the name the
      # rows cite would otherwise be unrecoverable. That is a convention this
      # gate depends on: a `done_<date>-*-CONSUMED.md` note carries it as an
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
  echo "no_* / never_* / not_* that asserts the absence. Fixing the row alone"
  echo "leaves the shell still refusing what the engine now does."
  exit 1
fi

# ⚠ STATE THE EVIDENCE SET, NOT JUST THE VERDICT. "OK" on its own is what this
# gate prints when it has looked and found nothing AND what it would print with
# nothing to look at. The count below is what makes the two readings
# distinguishable from the output alone.
echo "check-stale-blockers: OK — no row declares a blocker that has been closed."
echo "  Evidence: ${#NOTES[@]} consumption note(s) across"
echo "    $CHANNEL"
echo "    $ARCHIVE"
echo "  A zero there would be this gate going blind, and is reported as SKIPPED."
exit 0
