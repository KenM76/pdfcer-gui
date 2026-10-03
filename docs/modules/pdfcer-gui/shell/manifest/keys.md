# `pdfcer-gui/shell/manifest/keys`

**The live keymap: the loaded one, then the paste order, then the operator's
`shortcut.` lines.**

`apply` snapshots the keymap the first time it runs (`LOADED`) and rebuilds
from that snapshot every time after — at startup and on every Settings Save.
Editing the live keymap in place instead would make Reset impossible: once a
line moved `Ctrl+F` to another command, the program's own binding would exist
nowhere.

`catalog` builds what the Keyboard shortcuts page lists (see
`pdfcer-gui-base/settingspages/keys`). Its `kept` list names the keys bound
outside the keymap — `app::keyboard::OWNED` and the canvas keys in
`canvas::keys` and `canvas::moving::nudge`. A test asserts no kept key is also
a built-in binding, so the page never refuses the program's own key.
