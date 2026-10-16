# `the_inspector_is_one_tab_bar`

**Defect it guards.** O301: Edit's right dock opens as one tab bar, on the
selected object's Properties, with Objects the tab beside it. Two bars, a different opening tab, the document's metadata inside
the selection inspector, or object rows cut short at the default width all
fail.

**Fixture.** `fixtures/a1-titleblock.pdf`, launched off-screen with
`mode.edit,view.reset_layout`. Its object rows are long and carry
subset-tagged font names, so a narrow pane shortens them.

**Steps.**

1. Require `layout-reset ... changed=`, so nothing measured is a layout an
   earlier run saved.
2. Exactly one live `dock.right.*.tabbar`.
3. `dock.body.file.properties` drawn; `dock.tab.view.panel_objects` drawn on
   that bar (later tabs may sit behind its "more" button); `properties.info`
   not inside the Properties body.
4. Press the Objects tab: its body must draw, and the `objects-rows` line it
   writes must report rows with none shortened.
5. Screenshot (the window's own frame); the last 8 pt of the Objects pane must
   carry no text ink.

**Falsified** by restoring the two-stack default (Objects over a
Properties-led stack): a one-tab stack draws no bar of its own, so step 3
reports Objects missing from the one bar that is drawn.

**What it does not prove.** The one-time upgrade of a saved two-bar layout;
`app::modes::upgrade`'s unit tests cover it.
