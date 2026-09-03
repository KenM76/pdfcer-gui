#!/usr/bin/env bash
# ===========================================================================
# check-third-party-licences.sh — THE SHIPPED ATTRIBUTION FILE MUST MATCH THE
# BUILD IT SHIPS BESIDE.
#
# ---------------------------------------------------------------------------
# ★★★ WHY THIS EXISTS, and the release it nearly went out on
# ---------------------------------------------------------------------------
#
# 2026-09-01. A `cargo update` of the engine crates pulled in a colour-management
# engine, and with it three crates that had never been in this build:
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
# ★★ **The same shape as `check-verb-coverage.sh`**, written the same morning
# for the same reason: an ADDITION on the other side of a boundary is silent by
# construction. A REMOVED dependency breaks the build; an added one does not,
# and every tool in the toolchain is oriented around the first.
#
# ---------------------------------------------------------------------------
# WHAT IT ASSERTS, and the wrong version it replaced
# ---------------------------------------------------------------------------
#
#     Regenerating the file produces exactly the file that is committed.
#
# ★★★ The first version of this gate compared `Cargo.lock`'s crate names against
# the licence file and reported **213 missing** on a correct tree. The premise
# was wrong: `Cargo.lock` holds every crate for every TARGET — Android, macOS,
# Wayland, WASM — and a Windows build links almost none of them. `cargo-about`
# resolves per-target and is right; a hand-rolled name comparison cannot be.
#
# That failure is worth keeping in the header rather than quietly deleting,
# because it is this project's own standing rule turned on itself: **a check
# that cries wolf is not a stricter check, it is a broken one**, and the way it
# was found was by running it before believing it.
#
# ⇒ So the gate delegates to the tool that owns the question. It answers exactly
# the thing that was actually got wrong — *is the shipped file older than the
# build?* — and nothing about licence text, compatibility or terms.
#
# ---------------------------------------------------------------------------
# THE FIX, WHEN IT FIRES
# ---------------------------------------------------------------------------
#
#     cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md
#
# That is the whole remedy, and it is exactly what this gate ran to compare.
# Do NOT edit the file by hand: its own header says it is generated, so a
# hand-edit makes it disagree with the tool that owns it and the next
# regeneration reverts it without a word.
#
# ---------------------------------------------------------------------------
# EXIT CODES
# ---------------------------------------------------------------------------
#   0  the committed file is what the generator produces
#   1  it is not — the build links something the file does not describe, or
#      describes something it no longer links
#
# ★ SKIPs, loudly, when `cargo-about` is absent or fails. A gate that passed
# quietly when it could not measure would be worse than no gate — the rule this
# project applies to its driven checks applies to its static ones too. The word
# SKIP is the signal, and `run-all.sh` counts them apart from passes.
#
# ★★ Byte-identical is the right comparison and was verified before being
# relied on: two consecutive generations of an unchanged tree produce identical
# files. If `cargo-about` ever gains a timestamp or a random ordering, this gate
# will fail on a clean tree — and the repair is to normalise the output here,
# never to weaken the comparison to "contains the names".
# ===========================================================================
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 1

LICENCES="THIRD_PARTY_LICENSES.md"
TEMPLATE="about.hbs"

if [ ! -f "$LICENCES" ]; then
  echo "SKIP: no $LICENCES. If this project has stopped shipping one, delete this"
  echo "      gate deliberately rather than leaving it to skip for ever."
  exit 0
fi
if [ ! -f "$TEMPLATE" ]; then
  echo "SKIP: no $TEMPLATE, so the file cannot be regenerated for comparison."
  exit 0
fi
if ! command -v cargo-about >/dev/null 2>&1; then
  echo "SKIP: cargo-about is not on PATH, so nothing was measured."
  echo "      cargo install cargo-about"
  exit 0
fi

FRESH="$(mktemp)"
if ! cargo about generate "$TEMPLATE" -o "$FRESH" >/dev/null 2>&1; then
  rm -f "$FRESH"
  echo "SKIP: cargo-about failed, so nothing was measured. Run it by hand to see why:"
  echo "      cargo about generate $TEMPLATE -o $LICENCES"
  exit 0
fi
if [ ! -s "$FRESH" ]; then
  rm -f "$FRESH"
  echo "SKIP: cargo-about produced an empty file, which is a tool failure rather"
  echo "      than a finding about this repository."
  exit 0
fi

if diff -q "$FRESH" "$LICENCES" >/dev/null 2>&1; then
  rm -f "$FRESH"
  echo "PASS: $LICENCES is what the generator produces from the current lock."
  exit 0
fi

echo "FAIL: $LICENCES does not match what cargo-about produces from the current"
echo "      Cargo.lock. The build links something the shipped attribution file"
echo "      does not describe, or describes something it no longer links."
echo
# ★★ Named by SET DIFFERENCE, not by reading the diff hunks. A line-diff of a
# 5,700-line generated file reports the NEIGHBOURS of a change as well as the
# change, so the first version of this message listed four `accesskit` crates
# when exactly one `iccce` line had been removed — a confident, precise and
# entirely wrong list, which is the failure mode this project keeps finding in
# its own diagnostics. Caught by falsifying the gate rather than by reading it.
names() { sed -n 's/^- \[\([^]]*\)\].*/\1/p' "$1" | sort -u; }
names "$FRESH"    > "$FRESH.want"
names "$LICENCES" > "$FRESH.have"
echo "  Linked by this build and NOT in the shipped file:"
echo
comm -23 "$FRESH.want" "$FRESH.have" | sed 's/^/    /' | head -40
echo
echo "  In the shipped file and no longer linked:"
echo
comm -13 "$FRESH.want" "$FRESH.have" | sed 's/^/    /' | head -40
rm -f "$FRESH" "$FRESH.want" "$FRESH.have"
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
exit 1
