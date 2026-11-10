# `app::dispatch::region` — the region commands

| Command | Does |
|---|---|
| `edit.delete_in_region` | Deletes what the region takes, through the Delete key's own ladder (`canvas::deleting`) |
| `edit.redact_region` | Marks the region as one `/Redact` mark with the Redact panel's appearance |
| `edit.redact_outside_region` | Opens the Redact outside window; never acts directly, has no shortcut |
| `pages.crop_to_region` | Opens Crop pages with the region as the box |

All four are `enabled_when("doc.snapshot")`: greyed with the tooltip *"Greyed
until you draw a marquee first"* when there is no box. Esc or a tool change
clears the box (`canvas::snapshot::settle`), which greys them again.

Delete reports both counts on the status line, *"Deleted 7 objects inside
the region; 2 crossing its edge kept"*, so the Inside only choice is never
invisible. Redact has no Inside only choice: a redaction removes anything
crossing its quad, which is the engine's contract and the safe one.
