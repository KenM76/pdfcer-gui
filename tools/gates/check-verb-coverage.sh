#!/usr/bin/env bash
# ===========================================================================
# check-verb-coverage.sh — EVERY ENGINE VERB THE SHELL DOES NOT CALL OWES A
# WRITTEN REASON.
#
# ---------------------------------------------------------------------------
# ★★★ WHY THIS GATE EXISTS, and the two days that bought it
# ---------------------------------------------------------------------------
#
# `pdfcer-core` shipped `EditSession::set_button_action` on 2026-08-30, in
# answer to this shell's own request. The reply was read the same night and
# answered point by point. It even said, in as many words:
#
#     "Please check your own copy. If your surface tells the operator that
#      pdfcer never authors an action, it is now saying something untrue in the
#      direction that matters."
#
# The verb was consumed on 2026-09-01 — two days later — and only because
# `tools/verb-coverage.py` happened to be run for an unrelated reason. In the
# meantime the Button tool stayed greyed and the placement dialog kept telling
# the operator that pdfcer "cannot give a button something to do yet", which was
# false, on a capability two open operator rows were waiting for.
#
# ⇒ The instrument existed. Nobody ran it. **A tool that must be remembered is
#   a tool that will be forgotten**, and the fix for that is never a note — it
#   is a gate that fails.
#
# This is the third time the same shape has been recorded here:
#
#   * `EDITABLE_SURFACES.md` §"The sweep found..." — three of the first four
#     gaps were capabilities the engine shipped BECAUSE this shell asked, and
#     then never consumed. "A reply arriving is not a capability landing."
#   * `check-string-gaps.sh` — a catalogued string that reaches no rectangle.
#   * This.
#
# ---------------------------------------------------------------------------
# WHAT IT ASSERTS
# ---------------------------------------------------------------------------
#
# `tools/verb-coverage.py` parses `impl EditSession` out of the LOCKED engine
# revision, takes every `pub fn`, and greps `crates/pdfcer-gui/src` for each
# name. The verbs it reports as named nowhere are the input to this gate.
#
#     For every such verb, `EDITABLE_SURFACES.md` must mention it by name,
#     in backticks.
#
# That is the whole rule, and it is deliberately weak in one direction and
# strong in the other:
#
#   * **Weak**: it does not judge the reason. A register entry saying "not
#     built" passes. This gate cannot read English and must not pretend to.
#   * **Strong**: a verb that appears in the engine and is mentioned NOWHERE
#     fails the build, on the first `cargo update` that brings it. Somebody has
#     to look at it and write a sentence — which is the entire mechanism, and
#     is exactly what did not happen on 2026-08-30.
#
# ★★ The failure is therefore not "you have a gap". It is **"a capability
# landed and nobody has said anything about it"**, which is a different and
# much more actionable statement.
#
# ---------------------------------------------------------------------------
# WHY IT IS KEYED ON THE ENGINE AND NOT ON OUR OWN DOCUMENTS
# ---------------------------------------------------------------------------
#
# `OPERATOR_REQUESTS.md` says what the operator asked for. `FEATURES.md` says
# what the shell does. `GUI_ROADMAP.md` says what is planned. **None of the
# three is keyed on the engine's verb list**, so none of them can answer "is
# there something `pdfcer-core` implements that nothing here calls?" — the
# question this gate exists for. A completeness question needs an instrument
# whose key is the OTHER side's API; a document structurally cannot answer it.
#
# ---------------------------------------------------------------------------
# EXIT CODES
# ---------------------------------------------------------------------------
#   0  every uncalled verb is named in the register (or the instrument could
#      not run, which SKIPs — see below)
#   1  at least one uncalled verb is named nowhere
#
# ★ SKIPs rather than fails when the engine checkout is unreadable or python is
# absent, and says so loudly. The standing rule is that a check which cannot
# fail is not evidence — so a gate that silently passed when it could not
# measure would be worse than no gate. The word SKIP in the output is the
# signal; `run-all.sh` counts them separately from passes.
# ===========================================================================
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 1

REGISTER="EDITABLE_SURFACES.md"
INSTRUMENT="tools/verb-coverage.py"

if [ ! -f "$REGISTER" ]; then
  echo "SKIP: $REGISTER is missing, so there is nothing to check reasons against."
  exit 0
fi
if [ ! -f "$INSTRUMENT" ]; then
  echo "SKIP: $INSTRUMENT is missing. This gate is a wrapper around it and has"
  echo "      no independent way to enumerate the engine's verbs."
  exit 0
fi

# The instrument prints one uncalled verb per line on stdout and its summary on
# stderr. Both are wanted: the summary is what a reader needs in order to judge
# whether the measurement was against the LOCKED revision or a working tree.
#
# Piped through `tr -d` because the instrument is python on Windows and prints
# CRLF. Without it every pattern below becomes `verb<CR>`, matches nothing, and
# the gate reports EVERY verb as unexplained — which it did on its first run,
# convincingly enough to be believed for a minute.
SUMMARY_FILE="$(mktemp)"
MISSING="$(python "$INSTRUMENT" 2>"$SUMMARY_FILE" | tr -d '\r')"
STATUS=$?
SUMMARY="$(cat "$SUMMARY_FILE" 2>/dev/null)"
rm -f "$SUMMARY_FILE"

if [ "$STATUS" -ne 0 ]; then
  echo "SKIP: $INSTRUMENT exited $STATUS, so nothing was measured."
  echo "$SUMMARY"
  exit 0
fi
if [ -z "$SUMMARY" ]; then
  echo "SKIP: $INSTRUMENT printed no summary, which means it did not reach the"
  echo "      engine checkout. A gate that passes without measuring is not a gate."
  exit 0
fi

echo "$SUMMARY" | tail -1

# Written to a file rather than accumulated in a variable. A shell string is a
# perfectly good list until somebody's line endings are not what they look like,
# and this gate has already spent one debugging session on exactly that.
UNEXPLAINED_FILE="$(mktemp)"
COUNT=0
while IFS= read -r verb; do
  [ -z "$verb" ] && continue
  COUNT=$((COUNT + 1))
  # Fixed-string, backticked. A bare name would match prose about a different
  # topic; the backticks are how this register names a verb everywhere.
  if ! grep -qF -- "\`${verb}\`" "$REGISTER"; then
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
