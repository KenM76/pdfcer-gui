---
name: engine-replace-all-verb-drops-keys
description: A write verb that replaces a list drops every key its Rust type doesn't carry; disclose and file.
metadata:
  type: feedback
---

An engine verb that replaces a whole list rewrites each entry through its Rust type, so keys that type does not carry are lost from the file.

**Why:** `EditSession::set_3d_views` replaces all saved views and `view_dict` drops /BG, /SA, /NA, /U3DPath and /IN, so keeping one measurement rewrote every view. Filed as G274, with an off-canvas disclosure.

**How to apply:** Before wiring a "set all" verb, compare its type's fields with what the file can hold. [[feedback_no_gui_workarounds]]
