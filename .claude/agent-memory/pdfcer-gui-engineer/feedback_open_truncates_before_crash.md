---
name: open-truncates-before-crash
description: A Python edit script that opens the target 'w' and then fails mid-write leaves the source file EMPTY; write to a scratch path and copy, or build the whole string first
metadata:
  type: feedback
---

A script did `open(target,'w').write(head + tail)` where building `tail` raised
(a cp1252 encode of '★'). `open` had already truncated the file: a 233-line
check became 0 bytes, recoverable only because a draft sat in D:\scratch.

**Why:** `open(...,'w')` truncates the file the moment it opens; the encode happens later, inside `write`. The default Windows encoding is cp1252, which cannot encode most symbols this repo uses in failure messages.

**How to apply:** always pass `encoding='utf-8', newline=''`; compute the full
text into a variable BEFORE opening the target; keep a copy of any file a
script rewrites in D:\scratch first. See [[python-write-crlf]], [[feedback_write_python_file]].
