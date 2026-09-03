#!/usr/bin/env bash
#
# check-old-name-absent.sh — no `pdfce` survives except where it must.
#
# ═══════════════════════════════════════════════════════════════════════════
# ★★★ WHY A GREP FOR THE OLD NAME CANNOT ANSWER THIS QUESTION
# ═══════════════════════════════════════════════════════════════════════════
#
# **`pdfcer` contains `pdfce`.** That one fact is what makes this rename unlike
# any other, and it defeats the obvious check: a grep for the old stem matches every
# correctly-renamed occurrence as well as every stale one, so it returns
# thousands of hits on a perfectly clean tree and tells you nothing.
#
# The only honest pattern is a negative lookahead — the stem NOT followed by `r`
# — and the same hazard applies to the two other case forms, `PDFCE` (the
# environment variables) and `Pdfce` (Rust type names).
#
# ⇒ This gate exists because the question is easy to ask WRONG and the wrong
# answer is reassuring. It is the third instrument in this project written for a
# question whose naive form returns a comfortable lie; `verb-coverage.py` and
# `security-coverage.py` are the other two.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT IS ALLOWED TO SURVIVE, AND WHY EACH ONE IS A REFERENCE RATHER THAN A MISS
# ═══════════════════════════════════════════════════════════════════════════
#
#   1. `D:\Dev\pdfce\crates\pdfce-gui` — the OLD GUI this project salvages
#      from. It still lives there: the engine deletes that crate from its own
#      clone, and `D:\Dev\pdfce` is kept frozen as the backup. The path is
#      correct exactly as written and BREAKS if it is renamed.
#
#   2. `pdfce_FeatureRequests` — the request-channel folder shared with the
#      engine. Its note of 2026-09-03 says it stays "unchanged until you say
#      otherwise", and both sides read it every session. Renaming one side of a
#      shared folder is how a channel goes silent.
#
#   3. RETIRED 2026-09-03. `package = "pdfce-core"` and its two siblings, plus
#      the one `is_pdfce_choice` call, were the temporary bridge to an engine
#      that had not renamed yet. The engine's `Pass 247.1` landed the same day
#      (`4db298d`, engine v0.28.0); the manifest now names `pdfcer-*` against
#      `file:///D:/Dev/pdfcer` directly and the call site is
#      `is_pdfcer_choice`. Both substrings are OUT of the allow-list below, so
#      either one coming back is now a failure rather than an exemption.
#
#   4. `Cargo.lock` — Cargo's own record of what RESOLVED. It is generated
#      rather than authored, and rewriting it would make it disagree with what
#      Cargo actually fetched. ★ Since the engine's rename the lock no longer
#      NEEDS this exemption for the engine crates — it names `pdfcer-*` — but it
#      is kept because the lock also records transitive crates.io packages this
#      project does not author and cannot rename.
#
#   5. Any line carrying `old-name-exempt:` with a reason, or any FILE that
#      carries `old-name-exempt-file:` with one.
#      Prose that has to SPELL the old name to explain the rename is the
#      obvious case — this gate's own header and `run-all.sh`'s registration
#      comment both do — and a blanket file exemption would take the whole
#      file out of scope for the sake of one sentence.
#
#      ★ The marker is the idiom this project already uses for deliberate
#      exceptions (`ui-text-exempt:`, `string-gap-exempt:`): the exception
#      lives at the point of use, carries its reason, and is visible to the
#      next reader of that line rather than buried in a list somewhere else.
#
# ★ Anything else is a miss, and the message names the file and the line.
#
# ═══════════════════════════════════════════════════════════════════════════
# WHAT IT DOES NOT SCAN
# ═══════════════════════════════════════════════════════════════════════════
#
# Git history. Commit messages say `pdfce` and should: that is what the project
# was called when they were written, and `pdfce` is now the product's
# pre-release code name rather than a mistake. Rewriting history to match a new
# name would be falsifying the record — and this gate reads the working tree,
# which is the only thing a rename can legitimately touch.

set -uo pipefail

# ★★★ `LC_ALL=C`, and its absence made this gate REPORT CLEAN WHILE BROKEN.
#
# On this machine GNU grep refuses `-P` outside a unibyte or UTF-8 locale:
#
#     grep: -P supports only unibyte and UTF-8 locales
#
# It writes that to stderr, exits non-zero, produces NO output — and the
# `|| true` below turned that into an empty `HITS`, which reads exactly like
# "nothing survived". The gate printed `clean` on its very first run while
# having examined nothing at all.
#
# ⇒ A check that cannot fail is not evidence, and this one was written FOR that
# class. The failure arriving inside its own author is the argument for the
# falsification below: every gate here is proved able to go red before it is
# believed, and this one now is.
export LC_ALL=C

# The three case forms, each with the lookahead that makes the question honest.
PATTERN='pdfce(?!r)|PDFCE(?!R)|Pdfce(?!r)'

# Lines that are allowed to carry a surviving occurrence. Anchored on the
# substrings above rather than on filenames, so moving a file cannot silently
# widen the exemption.
ALLOWED='Dev\\pdfce\\crates\\pdfce-gui|pdfce_FeatureRequests|^Cargo\.lock:|old-name-exempt:'

# ★★★ THE SCAN'S OWN EXIT STATUS IS CHECKED, and that is the whole lesson of
# this gate's first run.
#
# `git grep` exits 0 when it matched, 1 when it did not, and >1 on an ERROR.
# Collapsing the last two — which `|| true` does, and which the first version of
# this file did — makes a broken scan indistinguishable from a clean tree. It
# printed `clean` while having examined nothing.
#
# ★ The ALLOWED filter uses `grep -E`, not `-P`: it is a plain alternation with
# no lookahead, and plain `grep -P` is unavailable on this machine ("supports
# only unibyte and UTF-8 locales"). Only the PATTERN needs PCRE, and `git grep`
# carries its own PCRE, which works.
RAW=$(git grep -nIP "$PATTERN" -- . 2>/dev/null)
STATUS=$?
if [[ "$STATUS" -gt 1 ]]; then
    echo "old-name-absent: FAIL — the scan itself failed (git grep exited $STATUS)."
    echo "                 NOTHING was examined. This is reported as a failure rather"
    echo "                 than as a clean tree, because a check that cannot fail is"
    echo "                 not evidence — and this gate printed 'clean' on exactly"
    echo "                 that footing the first time it ran."
    exit 1
fi

# ★★ A FILE may exempt ITSELF, by carrying `old-name-exempt-file:` and a reason
# in its own text. Two memory files and this gate's siblings are *about* the
# rename, so their subject is the old name and marking 25 individual lines would
# bury the prose in machinery.
#
# The exemption is declared in the file rather than listed here on purpose: it
# is the same reasoning as the per-line marker and as `ui-text-exempt:` — the
# exception lives where the next reader will meet it, and a list somewhere else
# is a list nobody re-reads. A file that stops being about the rename loses its
# marker in the same edit that changes its subject.
EXEMPT_FILES=$(git grep -lF 'old-name-exempt-file:' -- . 2>/dev/null || true)

HITS=$(printf '%s
' "$RAW" | grep -vE "$ALLOWED" || true)
if [[ -n "$EXEMPT_FILES" ]]; then
    while IFS= read -r f; do
        [[ -z "$f" ]] && continue
        HITS=$(printf '%s
' "$HITS" | grep -v "^${f}:" || true)
    done <<< "$EXEMPT_FILES"
fi
HITS=$(printf '%s
' "$HITS" | sed '/^$/d')

if [[ -n "$HITS" ]]; then
    COUNT=$(printf '%s
' "$HITS" | wc -l)
    echo "old-name-absent: FAIL — $COUNT line(s) still name the old project:"
    printf '%s
' "$HITS" | head -40 | sed 's/^/  /'
    [[ "$COUNT" -gt 40 ]] && echo "  ... and $((COUNT - 40)) more"
    cat <<'MSG'

Each of these is either a miss from the rename or a reference that deserves an
entry in this gate's ALLOWED list with a reason written down. Do not widen the
list to make the gate pass; a reference that cannot be explained in one sentence
is a miss.

Note the substitution is NOT idempotent: `pdfce` -> `pdfcer` applied twice gives
`pdfcerr`. Fix these by hand, or with a lookahead.
MSG
    exit 1
fi

ALLOWED_COUNT=$(printf '%s
' "$RAW" | grep -cE "$ALLOWED" || true)
echo "old-name-absent: clean - nothing names the old project except the"
echo "                 $ALLOWED_COUNT documented reference(s): the salvaged GUI's path, the"
echo "                 shared request-channel folder, Cargo.lock, and lines that carry"
echo "                 an old-name-exempt: reason."
exit 0
