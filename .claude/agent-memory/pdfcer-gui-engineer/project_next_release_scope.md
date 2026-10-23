---
name: next-release-scope
description: The next release waits until O323 (overprint + Output Preview), O324 (Forms tab + Style) and O325 (size-reduction dialogs/menus/right-click) are all built
metadata:
  type: project
---

On 2026-10-23 Ken answered "Release after that work is done." to the question of whether the next release waits for the work he asked for: the Forms tab with next-field Style (O324), menu items, dialogs and right-click entries for every engine v0.92.0 size-reduction feature (O325), and overprint preview plus Output Preview (O323).

**Why:** he asked for all three "for the next release", so a release that ships only part of them doesn't deliver what he asked for.

**How to apply:** no release until all three rows are built and driven. Then release straight away without asking again (releases are pre-authorised; see [[feedback_always_publish_latest]]). Engine tags arriving in the meantime are folded in, but they don't trigger a release on their own until this scope is done. Delete this memory once the release ships.
