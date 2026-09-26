# `egui-shell/commands/mod`

## Item notes

### `impl crate`

The dependency direction matters: `manifest` must be usable — parsed,
merged, round-tripped — by a tool that has no registry at all, which
is how `tools/ui-verify` and a schema linter can work on a `.ron` file
without linking the application.

### `fn a_duplicate_id_is_refused_and_named`

Two registrations of one id disagree about something. Keeping the
last one makes behaviour depend on the order of start-up code,
which produces a defect that appears after an unrelated
reordering and is close to unattributable.

### `fn a_manifest_naming_an_unregistered_command_fails_validation_with_that_id`

This is the invariant the whole registry exists to make
enforceable, and it is what turns `SHELL_FRAMEWORK.md` §9's refusal
of "invent a command" from a policy into a check.

**Naming the id is the point, not a nicety.** The manifest is a
file an operator edits. "Your shell.ron is invalid" tells them to
go and bisect it; "`view.fit_pge` is not a command" tells them
where the typo is. The same message is what a merge turns into a
disclosed skip — see [`crate::manifest::Skip`] — so the id must be
carried structurally, not embedded in prose.
