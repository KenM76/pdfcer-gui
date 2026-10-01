# `pdfcer-gui-base/panelid`

Which dockable panels exist, and the ribbon command that opens each. The
application's `panels` module re-exports [`Panel`] and owns the drawing
(`panels::show`); this half needs neither egui nor a document, which is what
lets `shell` resolve a command to a panel without depending on `panels`.

The reachability sweep that holds every `command_id` to a registered,
manifest-referenced command needs the shell manifest, so it stays in
`pdfcer-gui`'s `panels::tests` (`every_panel_is_reachable_from_the_ribbon`).

## Placements that are not `view.panel_*`

Every placement is `RIBBON_IA.md`'s or argued from it. The ids that open
authoring surfaces sit on tabs Read is not shown (Read sees `file` and `view`
only), so the mode taxonomy gates them with no capability flag of its own.

- **Forms is `view.panel_forms`**, not on Edit: the question is which modes may
  fill, and the answer is all three — Acrobat Reader fills forms in its default
  view.
- **Pages is `view.panel_pages`** and is not registered in this build, so
  `SHELL_FRAMEWORK.md` §5b filters it out of every arrangement; the `pages`
  panel's header carries what is needed to register it.
- **Comments is `markup.comments`.** §5.2 lists Comments under View ▸ Panels
  and §5.5 gives Markup a Comments group; P1 gives a command one tab, and §7's
  migration map names the control (`Review ▸ Comments ▸ Comments` →
  `Markup ▸ Comments`), which is the more specific ruling. Review and Edit both
  mount it and are both shown `markup`.
- **Redact is `edit.redact`** (§5.4, Edit ▸ Protect). There is deliberately no
  `view.panel_redact`: a second id would put a marking surface on a tab Read is
  shown.
- **DimensionGroups is `measure.manage_groups`**, on Measure ▸ Scale, for
  Redact's reason: `read` is not shown `measure`.
- **Attachments is `edit.attachments`.** `RIBBON_IA.md` names it nowhere; a
  surface that embeds and removes whole files must not be reachable from a
  reading stance, which is Redact's argument applied to the same question.
- **AlignDistribute is `edit.align`**: aligning moves content.

## Item notes

### `enum Panel`

An enum rather than a trait object, for one reason that matters and one
that follows from it. The reason that matters: [`Panel::ALL`] makes the
set **enumerable**, which is what lets a test sweep every panel and
assert something about each one — the app's reachability sweep is exactly
that. A registry of
boxed closures would be extensible and unsweepable.

The reason that follows: a dock hosting these needs to persist which
panels are open, and a `Copy`, `Eq`, `Debug` enum serialises to a token
that survives a restart. A closure does not.

### `const ALL`

Hand-written, because Rust cannot enumerate an enum. That makes it
the classic array that silently stops being exhaustive when a variant
is added — so [`tests::the_panel_catalog_is_complete`] pins its
length against a match that the compiler *does* check, which is the
only way to make a hand-written catalog self-defending.

### `fn command_id`

**This is the reachability contract.** Every panel names a command;
`pdfcer-gui`'s `panels::tests::every_panel_is_reachable_from_the_ribbon`
asserts every one is both registered in `shell::commands` and referenced by
`shell::manifest::built_in`. A panel with no route from the ribbon cannot get
past that.

Besides those listed above:

- **Fonts is `file.fonts`.** §7's migration map moves it from View ▸
  Panels to File ▸ Document, because the Fonts panel answers "what is
  inside this file", not "what is on my screen".
- **Properties is `file.properties`.** The document's own title,
  author, subject and keywords are a second panel and a second command,
  so this tooltip says only what its own panel does. See
  [`Panel::DocumentProperties`].
- **Document properties is `file.document_properties`**, beside it in
  File ▸ Document for the reason Fonts is there: it answers *"what is
  inside this file"*.

### `fn from_command_id`

The dock stores opaque ids, so something has to turn one back into a
panel, and this is deliberately the only thing that does. Written as
a search over [`Panel::ALL`] rather than a second `match`: a second
`match` is a second list to keep in step, and the failure when it
drifts is a panel that opens from the ribbon and draws nothing in
the dock — which looks like a rendering bug and is not.

Returns `None` for an id this build does not have, which is a
reachable state: a saved layout can name a panel whose capability
was compiled out.

## Tests

### `fn no_two_panels_share_a_command`

A shared id would make the reachability sweep pass for both while only one
could ever be opened: [`Panel::from_command_id`] returns the first match, and
the other panel sits in the arrangement drawing nothing. `file.properties` and
`file.document_properties` are the pair this guards — two subjects one tooltip
once covered, now two ids.

### `fn the_panel_catalog_is_complete`

[`Panel::ALL`] is an array, and an array cannot notice a new variant.
The `match` below can: it has no catch-all arm, so adding a variant
to [`Panel`] fails to compile until it is listed here, and the length
assertion then fails until it is added to `ALL`. That chain is what
makes a hand-written enumeration self-defending, and it matters
because every sweep over panels — reachability included — is only
as complete as `ALL`.

### `fn every_command_id_resolves_to_its_own_panel`

The round trip `from_command_id(command_id(p)) == Some(p)` for every panel,
and `None` for an id no panel claims — the reachable case of a saved layout
naming a panel this build compiled out.
