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
#   0  no contradiction found (or the request channel is not on this machine)
#   1  at least one row claims to be blocked on a request that has been answered
#
# ## Skipping honestly
#
# If the channel directory is absent — a clone on another machine, CI without
# the shared drive — this SKIPs with a stated reason rather than passing. A gate
# that cannot see its evidence has not checked anything, and reporting that as
# green is the failure this project has hit before.

set -uo pipefail

CHANNEL="${PDFCER_REQUEST_CHANNEL:-D:/Dev/FeatureRequests/pdfce_FeatureRequests/open}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [ ! -d "$CHANNEL" ]; then
  echo "check-stale-blockers: SKIP — the request channel is not at '$CHANNEL'."
  echo "  Set PDFCER_REQUEST_CHANNEL to point at it. Reported as SKIP rather than"
  echo "  PASS: this gate has not looked at anything."
  exit 0
fi

# Documents that carry status rows. Deliberately a short, named list rather than
# a sweep of every *.md: CONTINUE.md and HANDOFF.md are HISTORICAL records, and
# a past tick correctly says "blocked" about the day it was written. Rewriting
# history to keep a gate green would destroy the thing those files are for.
DOCS=("OPERATOR_REQUESTS.md" "FEATURES.md" "GUI_ROADMAP.md")

status=0
found=0

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
      if grep -qlF "$req" "$CHANNEL"/*CONSUMED*.md 2>/dev/null; then
        echo "  $doc:$lineno claims to be blocked and names '$req',"
        echo "      which has been CONSUMED — the capability is wired on this side."
        found=$((found + 1))
        status=1
      fi
    done
  done < <(grep -n -i -E '(BLOCKED|⛔|no verb (can|that)|cannot be (changed|done) at all)' "$path" || true)
done

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

echo "check-stale-blockers: OK — no row declares a blocker that has been closed."
exit 0
