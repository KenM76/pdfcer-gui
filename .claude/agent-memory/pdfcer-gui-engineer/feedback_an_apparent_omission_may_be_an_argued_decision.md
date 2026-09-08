---
name: an-apparent-omission-may-be-an-argued-decision
description: Before "fixing" a gap, open the module that owns it — correcting a file's stale claims does not make its remaining claims stale
metadata:
  type: feedback
---

**Before building a fix for something that looks missing, read the header of
the module that would own it.** A stated design position and an oversight look
identical from outside.

**Why:** 2026-09-08. A reply triage filed widget `/MK /BG` as HIGH with an
operator-visible consequence — *"the in-canvas field editor paints the theme's
own colours over a shaded field, so a green-tinted box turns grey while being
edited."* I opened the next tick to fix it, got as far as locating the theme's
contrast module, then read `canvas::forms` §3:

> *"A focused text field is replaced by an editor, and the editor is
> deliberately **not** a facsimile: it draws in the theme's own text-edit
> colours … not from the field's `/DA`."*

An argued position, with the reasoning (a substituted font cannot promise the
document font's glyph advances) and a precedent for what *is* admissible (`/Q`,
because quadding "says the same thing in any font").

★ **Three things the fix would have cost**, none visible from the note:
- it would have re-opened `DEFECTS.md` D2 (a background without a matched
  foreground) for a mismatch that reverses on commit;
- `editor_rect` **grows** past the field's bounds, so the elegant alternative —
  paint nothing, let the render show through — would have shown the page around
  the field and the baked text under the draft;
- the convention of the product class is that a focused field looks different.

**How to apply:**
- The tell is a note phrased as a *consequence* with no cited decision. If the
  triage row says "X paints Y" but names no module that chose to, open it.
- ⚠ **Correcting a file's false claims does not make its remaining claims
  false.** Two comments in that same file *were* genuinely stale and had been
  corrected the day before; the third sentence, in the same file, was still
  true — and was read as stale by association.
- When the verdict changes, **correct the row and say why**, rather than
  quietly re-scoping. The re-scope here was real (the honest consumer is the
  Properties panel, not the editor) and would have been invisible otherwise.

Related: [[a-backlog-row-is-a-record-not-evidence]],
[[triage-the-reply-channel-the-engine-fixes-faster-than-we-notice]],
[[a-citation-that-supports-my-theory-may-be-about-a-different-document]].
