---
name: engine-feature-unconsumed
description: An engine capability can ship for days while the GUI's own help text keeps denying it; Ken found vector printing that way, not a gate
metadata:
  type: feedback
---

The engine's vector print route (`pdfcer_render::emf::print_metafile`, `Sheet::metafile`) landed and the GUI kept spooling pixels, with its own resolution note still saying every page goes as an image. Ken caught it by asking "shouldn't there be a setting for this now". The same pass found MANUAL.md still saying n-up, booklet and poster "are not in the GUI" though all three ship.

**Why:** a pin move compiles clean when an additive API is ignored, and the API-drift gate is satisfied once a symbol is named anywhere in this repository, consumed or not. A sentence denying a capability is a claim that rots silently, the way [[feedback_limitation_sentence]] does.

**How to apply:** on every pin move, read the engine log from the old tag to the new one and list each new `pub` verb/field. For each, either wire it or file why not. Then grep our help text and MANUAL.md for sentences that deny the capability ("not sent as", "not in the GUI", "only as an image"). Related: [[feedback_api_drift_hit]], [[feedback_grep_sibling_crate]].
