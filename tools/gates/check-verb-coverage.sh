#!/usr/bin/env bash
# ===========================================================================
# check-verb-coverage.sh — EVERY ENGINE VERB THE SHELL DOES NOT CALL OWES A
# WRITTEN REASON.
#
# ---------------------------------------------------------------------------
# THE PROPERTY ASSERTED
# ---------------------------------------------------------------------------
#
# Every `pub fn` declared on `impl EditSession` in the engine revision
# `Cargo.lock` pins, whose name appears nowhere under `crates/pdfcer-gui/src`,
# is named in backticks inside a table row of `EDITABLE_SURFACES.md`.
#
# `tools/verb-coverage.py` supplies the list of uncalled verbs; this gate is
# the wrapper that turns "uncalled" into "unaccounted for". The rule is
# deliberately weak in one direction and strong in the other:
#
#   * Weak: it does not judge the reason. A row saying "not built" passes.
#     This gate cannot read English and must not pretend to.
#   * Strong: a verb the engine exports and nothing here names fails the build
#     on the first `cargo update` that brings it. Somebody has to look at it
#     and write a sentence — which is the entire mechanism.
#
# So the failure it reports is not "you have a gap". It is "a capability
# landed and nobody has said anything about it", which is a different and much
# more actionable statement.
#
# ---------------------------------------------------------------------------
# WHY A HUMAN CANNOT HOLD IT
# ---------------------------------------------------------------------------
#
# The engine is a separate repository. A capability starts existing in a
# commit this tree never reads; it arrives here as a changed hash in
# `Cargo.lock` and nothing else. There is no diff, no review, no moment at
# which a person is looking at the new verb.
#
# The case that gets missed longest is the one that feels most finished: a
# verb the engine shipped BECAUSE this shell asked for it. The request was
# answered, the reply was read, the answer was satisfying — and the wiring is
# a separate act that nothing prompts. Meanwhile the surface goes on telling
# the operator the capability does not exist, which is the direction of
# falsehood that costs something. A reply arriving is not a capability
# landing.
#
# No document in this repository can close that gap, because none of them is
# KEYED ON THE ENGINE'S VERB LIST. `OPERATOR_REQUESTS.md` is keyed on what was
# asked for, `FEATURES.md` on what the shell does, `GUI_ROADMAP.md` on what is
# planned. A completeness question about the other side's API needs an
# instrument whose key is that API; a document structurally cannot answer it.
# And an instrument that must be REMEMBERED is an instrument that will be
# forgotten, which is why this one is a gate rather than a tool.
#
# ---------------------------------------------------------------------------
# WHAT IT PROVABLY CANNOT SEE
# ---------------------------------------------------------------------------
#
#   * Whether a reason is true, or good, or still current. It checks that the
#     verb is named in a row, nothing more.
#   * Whether a verb the shell NAMES is actually reachable. The underlying
#     measurement is a grep: an identifier inside a dead branch, a test, or a
#     comment counts as called. A hit is weak evidence; only a miss is strong,
#     and this gate fails only on misses.
#   * Any verb outside `impl EditSession` — free functions, other types, other
#     engine crates. The parse has one shape and does not grow by itself.
#   * The engine's WORKING TREE. Its oracle is the locked revision, because
#     that is the API this shell could actually be compiled against. Verbs the
#     worktree has and the lock does not are reported by the instrument as
#     COMING and are not demanded here.
#   * A verb named in a register row that discharges some OTHER verb. The
#     match is row-level, not first-cell, so a reason cell mentioning a verb
#     in passing accounts for it.
#
# ---------------------------------------------------------------------------
# THE EXIT CONTRACT, AND HOW TO FALSIFY IT
# ---------------------------------------------------------------------------
#
#   0  every uncalled verb is named in a register row
#   1  at least one uncalled verb is named nowhere
#   2  SKIPPED — NOTHING WAS MEASURED: no register, no instrument, an
#      instrument that died, or an instrument that printed no summary and so
#      never reached the engine checkout.
#
# ★★ `run-all.sh` classifies purely by exit code — 0 pass, 2 skip, anything
# else fail — so the exit code is the only thing that carries the distinction.
# The word SKIP printed above an `exit 0` lands in the PASSED column and
# survives only in scrollback nobody diffs, which is the standing rule turned
# on this gate: a check which cannot fail is not evidence, and an unmeasured
# run counted as a pass is exactly that. Falsify by pointing
# `PDFCER_EDITABLE_SURFACES` at a path that does not exist; the exit must be 2.
#
# To falsify: copy `EDITABLE_SURFACES.md`, delete one verb's table row, point
# `PDFCER_EDITABLE_SURFACES` at the copy — it must name that verb and exit 1.
# Point the variable at an empty file and it must name every uncalled verb.
# The override exists so that falsifying the gate never needs a `git checkout`
# in a tree where other work is uncommitted.
# ===========================================================================
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 1

# Overridable so this gate can be PROVEN FALLIBLE without editing the committed
# register — the same affordance `check-engine-backlog.sh` gives via
# `PDFCER_ENGINE_BACKLOG`, and for the same reason: a falsification that needs a
# `git checkout` to undo is one that will discard another track's uncommitted
# work the first time it is run during parallel sessions.
REGISTER="${PDFCER_EDITABLE_SURFACES:-EDITABLE_SURFACES.md}"
INSTRUMENT="tools/verb-coverage.py"

if [ ! -f "$REGISTER" ]; then
  echo "SKIP: $REGISTER is missing, so there is nothing to check reasons against."
  exit 2
fi
if [ ! -f "$INSTRUMENT" ]; then
  echo "SKIP: $INSTRUMENT is missing. This gate is a wrapper around it and has"
  echo "      no independent way to enumerate the engine's verbs."
  exit 2
fi

# The instrument prints one uncalled verb per line on stdout and its summary on
# stderr. Both are wanted: the summary is what a reader needs in order to judge
# whether the measurement was against the LOCKED revision or a working tree.
#
# Piped through `tr -d` because the instrument is python on Windows and prints
# CRLF. Without it every pattern below becomes `verb<CR>`, which matches
# nothing, and the gate reports EVERY verb as unexplained — a total failure
# that is indistinguishable, on screen, from a total finding.
#
# `STATUS` is read after that pipeline with no `pipefail` in force, so it holds
# `tr`'s status and not python's. `tr` succeeds on anything it is handed, so
# the branch below that tests `STATUS` cannot fire; a dead instrument is caught
# one branch later, by the empty summary.
SUMMARY_FILE="$(mktemp)"
MISSING="$(python "$INSTRUMENT" 2>"$SUMMARY_FILE" | tr -d '\r')"
STATUS=$?
SUMMARY="$(cat "$SUMMARY_FILE" 2>/dev/null)"
rm -f "$SUMMARY_FILE"

if [ "$STATUS" -ne 0 ]; then
  echo "SKIP: $INSTRUMENT exited $STATUS, so nothing was measured."
  echo "$SUMMARY"
  exit 2
fi
if [ -z "$SUMMARY" ]; then
  echo "SKIP: $INSTRUMENT printed no summary, which means it did not reach the"
  echo "      engine checkout. A gate that passes without measuring is not a gate."
  exit 2
fi

echo "$SUMMARY" | tail -1

# Written to a file rather than accumulated in a variable. A shell string is a
# perfectly good list until the line endings are not what they look like, and
# an emptiness test cannot tell a list of one blank entry from no list at all.
UNEXPLAINED_FILE="$(mktemp)"
COUNT=0
while IFS= read -r verb; do
  [ -z "$verb" ] && continue
  COUNT=$((COUNT + 1))
  # Fixed-string, backticked, AND ONLY INSIDE A TABLE ROW.
  #
  # Prose ABOUT a verb is indistinguishable from prose ACCOUNTING FOR a verb,
  # to an instrument that cannot read English — and that blindness runs in both
  # directions. A paragraph naming, in backticks, the verbs it is declaring OUT
  # of scope silences a whole-file search exactly as well as a row discharging
  # them does: the sentence that admits the gap is the sentence that hides it.
  #
  # A row is the unit the register uses to discharge a verb: `| verb | pass |
  # status |`. Restricting the match to lines beginning with `|` means an
  # explanation must be ENTERED IN THE TABLE to count, and a sentence in an
  # introduction — however emphatic — cannot silence anything.
  #
  # Still fixed-string and still row-level, not first-cell. The register has a
  # legitimate table of ALTERNATE SPELLINGS whose reason cell names the verb the
  # shell calls instead, and a first-cell rule would reject those.
  if ! grep '^|' "$REGISTER" | grep -qF -- "\`${verb}\`"; then
    printf '%s\n' "$verb" >> "$UNEXPLAINED_FILE"
  fi
done <<EOF
$MISSING
EOF

if [ ! -s "$UNEXPLAINED_FILE" ]; then
  rm -f "$UNEXPLAINED_FILE"
  echo "PASS: all $COUNT uncalled verb(s) are named in $REGISTER."
  exit 0
fi

echo
echo "FAIL: the engine has verb(s) this shell never names, and $REGISTER does not"
echo "      mention them either:"
echo
sed 's/^/        /' "$UNEXPLAINED_FILE"
rm -f "$UNEXPLAINED_FILE"
cat <<'EOF'

  A verb in this list is one of two things, and BOTH need an act from you:

    1. A capability that landed and nobody noticed. That is what this gate is
       for. `set_button_action` sat here for two days in August 2026 while the
       Button tool stayed greyed and the placement dialog told the operator a
       falsehood. Go and read the reply in
       `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`, then wire it.

    2. A verb this shell genuinely should not call. Fine — say so, in
       `EDITABLE_SURFACES.md`, in backticks, with the reason. A session query
       the shell has no use for, an alternate spelling of a verb already
       called, a capability declined on an operator ruling: all legitimate,
       all one sentence.

  What is NOT allowed is silence, because silence is indistinguishable from
  (1) and reads as (2).
EOF
exit 1
