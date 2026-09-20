---
name: a-gate-that-crashes-after-its-headline-reads-as-a-broken-tool
description: A checker that prints "1 row failed:" and then dies on an encoding error sends you to debug the checker instead of fixing the finding — the failure message IS the product
metadata:
  type: feedback
---

**A checker's failure message is its product. Make the naming of the finding
un-killable — never let a decoration take the process down.**

**Why:** 2026-09-11. `tools/walk-engine-backlog.py --check` printed

    1 row(s) exceed the 1200-character cap:

and then died with `UnicodeEncodeError: 'charmap' codec can't encode characters`
on the very next line — the one that would have said *which* row. Python opens
stdout as **cp1252** on this machine and every interesting row in that register
opens with `★`.

The damage is not the missing line. It is that **a traceback looks like a broken
tool**, so the obvious next act is to go and debug the tool, while the real
defect — a 3,856-character row — sits unfixed and unnamed. It is worse than
silence in that one specific way: silence makes you suspicious; a crash makes
you confident you know what is wrong, and wrong.

The fix is one line at the top of `main`, and `errors="replace"` is the load-
bearing half:

```python
for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except (AttributeError, ValueError):
        pass
```

A character a console cannot draw must degrade to `?` and the measurement must
survive.

**How to apply:**
- Any tool whose output quotes project text (rows, identifiers, operator
  strings) needs this at the top. Assume the content contains `★`, `⚠`, `—`.
- When a checker traces back, ask *what did it print before it died* before
  assuming the tool is at fault — the headline may already be the answer.
- Related: [[an-unevidenced-excuse-is-worse-than-silence]],
  [[a-trace-grepping-check-passes-on-a-build-that-crashed]].
