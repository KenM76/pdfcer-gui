---
name: moving-command-tab-breaks-checks
description: Moving a command to another ribbon tab silently breaks ui-verify checks that click the old tab; the full chain stays green
metadata:
  type: feedback
---

When a command moves to a different ribbon tab, grep `tools/ui-verify/src/checks/` for `ribbon.tab.<old>` beside its `ribbon.item.<id>`, then fix and drive every check that turns up. Checks that dispatch through `INVOKE` (`mode.edit,edit.form_…`) are unaffected.

**Why:** in O324, Forms moved from Edit to its own tab. `radio_group` and `repair_form_fonts` were still looking for the item on `ribbon.tab.edit`. Tests, clippy and gates do not run ui-verify, so the chain would have been green while both checks were broken. I caught it only because the doc rename sent me to re-drive `repair_form_fonts`.

**How to apply:** run that grep in the same edit as any change to RIBBON_IA placement. If a check you find uses the OS `Driver`, convert it to `ScriptedPointer` and an off-screen viewport; otherwise it cannot be driven under `--no-input`. See [[feedback_drive_only_affected]].
