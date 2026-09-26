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

### `enum Site`

Used both by [`ManifestError`] and by [`super::Skip`], because "the
place a command id was mentioned" is the same question whether the
answer is a rejection or a disclosure.

### `fn validate`

# What is checked

1. The schema is one this build understands.
2. Tab ids are unique — across ordinary **and** contextual tabs,
   because they share one namespace and a mode referring to `format`
   must resolve to one thing.
3. Every tab has a label and a `groups` key.
4. Group ids are unique within their tab, and every group has a
   caption.
5. Mode ids are unique, every mode has a label, and every tab a
   mode names exists.
6. The quick-access toolbar has no duplicate entry.
7. **A command appears on at most one tab.**

# Errors

The first failure found, in the order above. One error rather than
all of them: unlike the contrast gate, these are structural and the
second is very often a consequence of the first — a duplicated tab
id makes every command on it look duplicated too, and reporting
forty errors for one edit is how a reader learns to ignore the
list.

### `fn validate_against`

Walks every command reference in the manifest — see
[`Shell::command_references`] for the regions and the order — and
refuses any id the catalog does not know.

# Why this is a separate call rather than part of `validate`

So that a manifest is usable by a tool with no registry: a schema
linter, a diff viewer, or `tools/ui-verify` reading a `.ron` file
without linking the application. Structure and references are two
different questions and only one of them needs the application to
be present.

# Errors

Everything [`Shell::validate`] returns, plus
[`ManifestError::UnknownCommand`] naming the id **and** the site
that referenced it.

### `fn command_references`

Every region that can name a command is walked: tabs (ordinary then
contextual), the quick-access toolbar, the trailing controls, the rail,
and the keymap, in that order. The order is stable so a failing
validation names the same reference on every run.

A region added to [`Shell`] and not walked here is a region in which a
typo produces a silently absent control instead of a start-up failure,
so extending this function is part of adding one.
