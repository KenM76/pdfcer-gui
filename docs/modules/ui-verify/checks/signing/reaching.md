# `ui-verify/checks/signing/reaching`

## Item notes

### `fn click_tab_tolerant`

[`click_tab`] asserts a new `ribbon-tab-activated` line, which is the right
test when a check is switching away from a tab it knows is active. It is the
wrong test for *"make sure this tab is on top"*: a tab that is already active
emits nothing when clicked, and the strict form then reports a perfectly good
click as a failure.

A missing tab is still an error. This tolerates *no change*, never *no tab*.
