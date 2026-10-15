---
name: python-write-crlf
description: Python open(path,'w') on Windows rewrites an LF file as CRLF; git diff --stat hides it. Always pass newline='' or check bytes.
metadata:
  type: feedback
---

A Python edit script using `open(P, 'w', encoding='utf-8')` turns every LF line
ending of the file into CRLF on Windows (text-mode write translates `\n` to
`os.linesep`). `git diff --stat` still shows a one-line change, because git
normalises on compare, and the only visible sign is a "CRLF will be replaced by
LF" warning. Sixteen files were converted in one feature before it was noticed.

**Why:** the repo is LF; gates and anchors that match `\n`-terminated text, and
any later byte-level comparison, see a different file than git shows.

**How to apply:** read and write with `newline=''` in every edit script, or run a
bytes check (count `\r` in HEAD vs working copy) before committing. Related:
[[feedback_write_python_file]].

**Second cost, measured 2026-10-13:** a CRLF file committed (git normalised the
blob to LF) and then converted to LF in place stays `M` in `git status` with an
empty `git diff` — a stale index entry — and the packager stamped the release
`-dirty`. `git add <file>` re-stages identical bytes and clears it; then
re-package. Check `git status --short` is empty immediately before packaging.

**Third shape, 2026-10-15:** working-tree endings are mixed per file (prefs/file.rs
CRLF, promote.rs and extract.rs LF), so an anchor containing a newline matches in
one file and not its neighbour. Detect per file (`'\r\n' in s`), normalise to `\n`,
edit, write back in the file's own ending. A multi-line `\`-continued string
literal is easiest with the Edit tool; a `sed` on a CRLF file needs `\r$`.
