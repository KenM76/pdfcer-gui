---
name: unset-vs-default-reads-as-difference
description: An Option field where None means "the default applies" counts as a different value from Some(default); a minimal writer flips between them with no visible change
metadata:
  type: feedback
---

Before counting distinct values of an engine field, read its doc for what `None` means. `GlyphProvenance::fill_color` is `None` when no colour was set and black `DeviceGray 0` applies. `edit_block_text` writes a colour only where the inherited one differs, so a block that kept its 2 looks re-opened as `looks=3` in the shell's count: `None` and `Some(Gray(0))` are the same black.

**Why:** I nearly filed a look drift against the engine. Its CLI said `looks: 2`, and a variant with `0 0 1 rg` before `BT` showed it emitting `0 g` exactly when needed.

**How to apply:** when a before/after count moves after a rewrite, dump the rewritten stream and test the inherited-state case before blaming the writer. Normalise `None` to the documented default in the comparison key.
