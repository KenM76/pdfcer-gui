# `ui-verify/checks/shortcut_change` — `a_changed_shortcut_takes_effect_and_persists`

**A key changed on Settings ▸ Keyboard shortcuts reaches the live keymap at
Save and is read back by the next launch** (O274). Driven off the desktop with
the scripted pointer; no OS input.

# Sequence

1. Clear the sandbox's `preferences.txt`, launch with Settings open.
2. Click the Keyboard shortcuts page, type `Find` into the filter, press
   Change on `edit.find`, press Ctrl+K in the dialog's viewport.
3. If a `shortcut-clash` follows, press Reassign. Require
   `shortcut-set command=edit.find chords=Ctrl+K`.
4. Save; require a `prefs-saved` line.
5. In the main window, Ctrl+K must dispatch `edit.find` (`chord-command
   id=edit.find`) and Ctrl+F must not.
6. The file must hold `shortcut.edit.find = Ctrl+K`.
7. Relaunch with no Settings: Ctrl+K dispatches Find, Ctrl+F does not.

# Why Ctrl+F is asserted too

A line gives a command **exactly** its chords. A keymap that added Ctrl+K and
left Ctrl+F would pass step 5's first half; the old key still firing is the
half that proves the replacement rather than an addition.

# Outcomes

A missing page, filter, Change button or Save is FAIL — each is a control the
operator cannot reach. A missing binary or viewport variable is SKIP.
