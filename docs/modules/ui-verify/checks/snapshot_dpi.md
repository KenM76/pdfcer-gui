# `ui-verify/checks/snapshot_dpi` — `the_snapshot_resolution_persists`

**The snapshot resolution typed in Settings ▸ Images is saved and read back by
the next launch** (O272, step 1). Driven off the desktop with the scripted
pointer; no OS input.

# Sequence

1. Clear the sandbox's `preferences.txt`, launch with Settings open.
2. Click the Images page, click `settings.snapshot.dpi`, select all, type
   `150`. Require the page's last `snapshot-dpi-setting` to say `dpi=150`.
3. Save; require a `prefs-saved` line. The file must hold
   `snapshot_dpi = 150`.
4. Relaunch with Settings open, click the Images page. Its
   `snapshot-dpi-setting` must say `dpi=150`.

# Why 150

Any value but the default (300) works; a check typing 300 would pass on a build
that never wrote or read the line.

# Outcomes

A missing page, box or Save is FAIL — each is a control the operator cannot
reach. A missing binary or viewport variable is SKIP.
