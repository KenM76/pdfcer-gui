# `the_security_tab_holds_security_and_protect`, `the_snapshot_tool_is_on_the_left_rail`

## `the_security_tab_holds_security_and_protect`

**Defect it guards.** The operator asked for one Security tab holding the
security and protect tools (`OPERATOR_REQUESTS.md` O289 item 10). Three ways
it goes wrong: the tools stay on File, a band is missing from the new tab, or
redaction — which edits page content — appears in Read.

**Fixture.** `fixtures/four-pages.pdf`, launched twice off the desktop.

**Steps.**

1. Edit mode (`PDFCER_DIAG_INVOKE=mode.edit`). The shell opens on File; no
   `ribbon.item.file.encrypt` may have been drawn yet. Press
   `ribbon.tab.security`; the frame on screen must hold a Security band and a
   Protect band — each proved by its first item or by its collapsed group
   button (`ribbon.group.security.<group>…`).
2. Read mode. Press `ribbon.tab.security`; the Security band must be on
   screen, and no `ribbon.item.edit.redact*` or `ribbon.group.security.protect*`
   region may appear anywhere in the run.

**Falsified** by deleting `.shown_when(EDIT_CONTENT)` from the Protect items in
`ribbontabs::security`: step 2 fails naming the redaction regions Read drew.

## `the_snapshot_tool_is_on_the_left_rail`

**Defect it guards.** O289 item 11: the snapshot tool reachable from the left
tool strip, not only from View ▸ Navigate.

**Steps.** Read mode. `rail.navigate.view.tool_snapshot` must be declared;
pressing it must trace `snapshot-tool armed=true`.

**Falsified** by removing `view.tool_snapshot` from the rail's Navigate group in
`railmanifest`: the check fails naming the Navigate rows the rail drew.

**What neither proves.** Placement within the band or the rail beyond
presence; the ribbon's own order is pinned by the manifest tests.
