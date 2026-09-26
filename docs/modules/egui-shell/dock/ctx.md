# `egui-shell/dock/ctx`

## Item notes

### `enum Intent`

Collected during a frame and applied after it — see the module
header. A struct-like enum rather than a callback so the whole set of
mutations the dock can perform is **one readable list**, which is what
makes "nothing else writes to the layout" a claim a reviewer can
check.

### `struct Ctx`

Not `Clone`, not `Copy`, and never stored: it borrows the
application's registry and sink for the duration of one
[`super::Dock::show`] call and is dropped at the end of it.

### `fn id`

Every interactive element in the dock derives its id from the
**structural address** — side, column, stack, role — rather than
from a counter or from the panel's id. That is deliberate and it
is the property `egui` needs: an id that changed when a tab was
activated would end the in-flight interaction that caused the
activation, and an id that changed when a panel moved would reset
a splitter drag mid-gesture. An id that changes between frames
silently resets every in-flight interaction keyed on it, and the
symptom — a drag that stops responding partway — never points back
at the id.

### `fn describe`

**Falling back rather than skipping is load-bearing.** A panel
mounted in the layout but absent from the registry is normally
dropped at load time with a disclosed reason
([`crate::layout`]), so reaching this fallback means the
application registered panels *after* loading, or mutated the
layout by hand. Drawing the raw id is ugly and truthful; drawing
nothing would give a tab with no name, which is
indistinguishable from a rendering fault and impossible to report
usefully.
