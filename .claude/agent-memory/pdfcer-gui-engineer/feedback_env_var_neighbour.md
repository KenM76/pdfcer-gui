---
name: a-remembered-unset-names-one-variable-not-its-neighbour
description: A memory saying "$TMPDIR is unset" is not a fact about $TMP — I expanded the wrong one into a filesystem-root path and reached for rm -f on it
metadata:
  type: feedback
---

**A remembered "`X` is unset" is a claim about `X` and about nothing that looks
like `X`. Before a derived path becomes the argument to a destructive command,
`echo` the variable.**

**Why:** on **2026-09-16** a standing note of mine read *"`$TMPDIR` is unset"*.
Writing a scratch script I reached for `"$TMP/reg.py"`, carried the note across
to the neighbouring name, and concluded the file had landed at **`/reg.py`** —
the MSYS filesystem root. The next command was `rm -f /reg.py`. Ken stopped it:
*"you were trying to modify or delete a file called reg.py in the root directory
that doesn't exist."*

**`$TMP` was set all along** — to `C:\Users\Ken\AppData\Local\Temp`, Windows-
style, backslashes and all. Bash expanded it happily, the script really did
write and really did run, and the registration edits landed correctly. Only my
account of *where* was invented. One `echo "[$TMP]"` would have closed it, and
that is the command that now goes first.

★ **The tell I walked straight past: the evidence contradicted the story.** The
same block that "wrote to `/reg.py`" then ran `python "$TMP/reg.py"` and printed
`ok`. A file at the root of a path I believed did not exist cannot execute. I
read the `ok` as success for the *edit* and never asked what it proved about the
*path* — so a working command became the cover for a wrong model of the
filesystem, and the correction arrived only because a human was watching.

⚠ **`rm -f` on a derived path is the one command with no second chance.** A
wrong `cat` prints nothing, a wrong `python` errors — a wrong `rm -f` is silent
by construction, and at the filesystem root it is silent over something that
might matter. Derived paths get `ls` first, always; the `ls` costs nothing and
is the only step that distinguishes "already gone" from "never was there".

**How to apply:** use the **session scratchpad the environment block names** —
`.../Temp/claude/<project>/<session>/scratchpad` — for every temporary script
and capture. It is named in the prompt, it is session-scoped, and it needs no
env var to be resolved correctly, which removes this whole class. Reserve `$TMP`
and `$TMPDIR` for cases where a tool demands them, and measure both when so.

Related: [[an-injected-file-is-a-dated-snapshot]],
[[a-quotation-i-wrote-myself-can-carry-a-line-number]],
[[verify-the-result-not-the-diff]],
[[never-git-checkout-to-undo-an-experiment]].
