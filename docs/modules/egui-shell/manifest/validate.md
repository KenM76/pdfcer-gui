# `egui-shell/manifest/validate`

## Item notes

### `fn check_one_command_one_tab`

# Why this rule exists and is worth enforcing mechanically

The reason is navigational rather than aesthetic: if a command can sit
on two tabs, then "where is that control?" has two answers, and an
operator who found it once has learned nothing about where anything
else lives. One home per command is what makes the tab set a *map*
instead of a menu that repeats itself.

`SHELL_FRAMEWORK.md` §5 amends it in exactly one direction: **the
QAT and the status bar may mirror a command.** Those are not tabs
and are not navigated; they are shortcuts to something the operator
already knows the location of. So this check walks tabs only, and
[`Shell::validate`] checks the QAT separately for self-duplication
alone.

Contextual tabs count. A Format tab that re-hosted a command from
the Markup tab would produce precisely the two-answers problem, on
a surface that appears and disappears — which is worse, not better.

### `fn a_command_twice_on_the_same_tab_is_also_refused`

Worth its own case because the obvious implementation — compare
each tab's command set against the other tabs' — passes this and
is wrong. A duplicate within one tab is still two homes for one
command, on a surface where the operator can see both at once.

### `fn the_qat_and_keymap_may_mirror_a_tab_command`

`SHELL_FRAMEWORK.md` §5's amendment. Without this the rule would
forbid the quick-access toolbar from doing the one thing it exists
to do, and the test above would pass identically if the walk had
been written over every reference instead of over tabs.

### `fn an_unregistered_command_is_refused_with_its_id_and_site`

The site matters as much as the id. `view.fit_pge` appearing in the
keymap and `view.fit_pge` appearing in a group are two different
lines in the operator's file.
