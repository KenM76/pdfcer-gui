---
name: scope-a-request-to-the-whole-expected-behaviour
description: Ken asks for a feature and expects everything surrounding it that a user would expect — enumerating deferrals is not delivering, it just moves the work onto him
metadata:
  type: feedback
---

**When Ken asks for a feature, deliver the behaviour a user would expect around
it — not the narrowest thing that satisfies the sentence.**

His words, 2026-08-18:

> *"I think that is how we are ending up with a gui that when I ask for
> something I get a very narrowly scoped part of what I wanted. Whereas when I
> ask for something, my expectation is usually that everything surrounding that
> request is also done to where it would match the behaviour a user would
> expect. Otherwise I am left typing out every little missing detail."*

And, on declining to ask the engine for two capabilities because nobody had
requested them:

> *"not adding such things just because they weren't explicitly asked for i
> think is how we end up with partially finished features."*

**Why:** the failure mode is mine and it is consistent. I ship the core of a
request, enumerate the deferrals honestly in a commit message and `RESUME.md`,
and treat the enumeration as sufficient. It is not — it relocates the work onto
him, who then has to notice the gap, remember it, and type it out. Several
features shipped this way in one day:

- **Insert from file** — inserted every page after the current one. No position
  choice, no page range, no page count shown before committing, and the source's
  bookmarks/labels/fields silently not carried.
- **New at a chosen page size** — sizes, but no remembered default.
- **Annotation selection** — select and delete, but not move or resize.

Each was defensible alone. Together they are a GUI that does the first 70 % of
everything.

**How to apply:**

- Before calling a feature done, ask *"what would a competent user reach for
  next, within this same gesture?"* — a position, a range, a count, a preview, a
  default that is remembered, an undo. Build those.
- **Deferring is still allowed — but it must be a decision with a reason, not
  a scope boundary drawn where the sentence ended.** "Blocked on an engine verb"
  is a reason. "He only literally asked for X" is not.
- **Ask the engine for the whole cluster**, not the members with a caller today.
  The "a verb with no caller is drift" rule (R151) applies to a *convenience
  query* duplicating something already reachable. It does **not** apply to the
  missing members of one feature — those make the feature permanently partial,
  and the partiality gets discovered by a user rather than by us.
- Enumerating what is missing is still required (see
  [[feedback-ui-verify-competes-for-the-machine]] on honest reporting) — it is
  just not a substitute for building it.
