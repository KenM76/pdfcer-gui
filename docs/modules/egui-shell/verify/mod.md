# `egui-shell/verify/mod`

## Item notes

### `fn a_disabled_line_builds_nothing`

The contract says a call site costs nothing when tracing is off,
and that is what allows trace calls to be left in permanently
rather than added and removed around each investigation. If a
disabled [`Line`] allocated, that claim would be false at every
call site in the frame loop.

Tests run without [`ENV_VAR`] set, so [`enabled`] is `false` here.
That is also why there is no test of the *enabled* path in this
file: [`enabled`] resolves once per process, so a test that armed
it would arm it for every other test in the binary — the very
coupling that makes a process-global cache correct in production
and untestable in place. The enabled path is exercised by
`tools/ui-verify`, which runs a real process with the variable
set, which is where a claim about a real process belongs.

### `fn an_enabled_line_has_the_key_equals_value_shape`

Constructed directly rather than through [`event`] so the shape can
be asserted without arming the process-global switch — see the
note on `a_disabled_line_builds_nothing`.

### `fn the_default_prefix_applies_until_an_application_sets_one`

Deliberately does **not** call [`set_prefix`]: it is a `OnceLock`,
so a test that set it would decide the prefix for every other test
in this binary and make their assertions order-dependent. What is
checkable in-process is the default, and that is what is checked.
