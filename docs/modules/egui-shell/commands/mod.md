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

### `struct HandlerToken`

The shell stores it, hands it back when the command is invoked, and
never looks inside. A `u64` because every plausible application-side
representation — an enum discriminant, a slot index, a hash of a
function name — fits in one, and because a type parameter here would
propagate to every signature in the shell that mentions a command.

The value has no meaning to the shell, so two commands may share a
token if the application wants two ids to run the same handler.

### `struct Command`

Built with [`Command::new`] and the `with_*` methods: a handful of
fields, most of them optional, is exactly the shape that makes a
positional constructor unreadable at the call site — and this
constructor is called once per command, which for a real application
fills a file.

### `struct CommandRegistry`

Ordered (`BTreeMap`) so [`Self::ids`] is stable: a failing validation
that lists the registered ids must list them the same way twice, or the
diff between two runs is noise.

### `fn register`

# Errors

[`RegistryError::DuplicateId`] if the id is already registered.

A duplicate is an error rather than a replacement because the two
registrations disagree about something — a label, a predicate, a
handler — and silently keeping the last one makes the application's
behaviour depend on the order of its own start-up code. That is a
defect that reproduces only after an unrelated reordering, which is
the worst kind to be handed.

### `fn register_all`

# Errors

As [`Self::register`].
