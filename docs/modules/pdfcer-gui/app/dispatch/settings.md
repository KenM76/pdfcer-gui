# `app::dispatch::settings` — the commands that open the Settings window

## The seam, and why two ids share one arm

`file.settings` and `tools.font_folders` open **one window, one draft, one
Save**. They differ in one thing: *where the window lands*.

| id | the question it asks | where it lands |
|---|---|---|
| `file.settings` | *"show me the settings"* | the top, correctly |
| `tools.font_folders` | *"where do font folders live"* | the Fonts group, opened and scrolled to |

These are **not** two routes in [`super::routes`], and the reason is the
rule that file states: *a second route to an existing command must not become
a second implementation of it*. That rule is intact here — there is still one
implementation — but the two ids are not the same **request**.

⇒ **A route stops being a pure route the moment the target needs to know WHY
it was reached.** `routes::target` returns a bare id and has nowhere to put an
operand, so the arm lives here rather than the routing mechanism growing a
parameter for one caller.

## What a pure route costs here, which is the whole reason

`OPERATOR_REQUESTS.md` **O50** opens with the operator asking for a
font-folder setting that had already shipped. A route lands at the top of the
window, and every group but `colour` is drawn collapsed, so the setting the
command is named after is behind a heading the operator has to find.

**Opening the right window is not the same as answering the question the
command's own name asks.**

## Item notes

### `fn focus`

One function, so [`handles`] and [`dispatch`] cannot answer differently
about the same id — `routes::target` states that rule and it applies here for
the same reason.

### `fn the_font_route_lands_on_a_group_the_dialog_draws`

The second half is the load-bearing one: the group key is a string
matched against `widgets::group_focused`'s `key` in the dialog, so a typo
produces a window that opens at the top with no error anywhere — the
exact failure the landing exists to prevent, restored silently.

### `fn handles`

`pub(crate)` for [`super::routes::handles`]' reason: `shell::commands::reach`'s
reachability checker must be able to evaluate every guard arm it finds, and a
guard it cannot evaluate is a place commands could hide from the check that
exists to find them.

### `fn dispatch`

**Application-scoped**, like About: these are choices about pdfcer, and an
operator who has just launched the program and wants a dark window should not
have to open a document first.

Two things a reader will ask, both answered on [`Draft`] rather than repeated
here. **The draft opens on the LIVE configuration** — the session's
`Settings`, not a re-read of the file — because a session honouring a choice
the disk does not have must show what pdfcer is *doing* rather than what it
wished it had written. **Re-opening does not reset a draft in progress**,
which is `DialogsState::open_print`'s guard and matters more here, because
some of these settings change saved bytes and the window's whole promise is
that nothing takes effect until Save.

The guard is also what makes the landing safe to re-fire: pressing Tools ▸
Font folders while the window is already open does **nothing**, rather than
scrolling a window the operator has since scrolled somewhere else.
