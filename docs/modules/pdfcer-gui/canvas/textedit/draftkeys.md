# `pdfcer-gui/canvas/textedit/draftkeys`

**The commands an open text edit answers itself, on the operator's keys.**

While a draft is open, `app::keyboard::commands` yields so typed letters stay
text. The draft therefore routes select all, undo, redo, save, bold, italic
and underline (`TYPING`) on its own. It reads their chords from the live
keymap, which `publish` copies into egui memory each frame, so a key changed in
Settings ▸ Keyboard shortcuts works inside a text edit too.

Only a chord carrying Ctrl or Alt is honoured here: a bare key bound to a
command outside a draft is still typed as text inside one. Before anything is
published (unit tests, the first frame) `bound` falls back to the program's
own chords; a test pins that table to the built-in keymap.
