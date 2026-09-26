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

### `fn enabled`

Resolved once and cached: the check sits in a per-frame path, and
re-reading the environment there would put a lock and an allocation in
the frame loop to answer a question that cannot change after start-up.
After the first call this is one relaxed atomic load, which is what
makes leaving trace call sites in place permanently free.

### `fn set_prefix`

Every traced line becomes `<prefix>-diag <message>`, so a stream that
also carries the window manager's chatter and the graphics driver's
warnings can be reduced to this application's trace with one `grep`.

# Return value

`true` if this call set the prefix, `false` if one was already set (in
which case the existing prefix is kept). The boolean exists so an
application can `debug_assert!` that its start-up path ran once — a
second call is a symptom of two initialisation paths, which is worth
knowing about even though it is harmless here.

# Why the prefix is not simply an argument to [`trace`]

Because then every call site would carry it, and a call site that
carried the wrong one would produce lines that the harness's `grep`
silently drops. One process, one prefix, set where the process is
configured.

### `fn trace`

Takes a closure rather than a `String` so a disabled build path
performs no formatting — real call sites interpolate rects, pointer
positions and hit counts, and doing that work every frame to throw it
away would be a real cost in the one loop that must not get slower.

The message should be `key=value` fields separated by spaces, led by
an event name. [`event`] builds that shape without hand-formatting.

### `fn event`

The builder form of [`trace`], for the common case where a line is an
event name and some fields. Cheap when tracing is off: no buffer is
allocated and every [`Line::kv`] is a no-op.

It is *not* free when off — the field values are still evaluated by
the caller, since they are arguments rather than a closure. For a call
site whose values are expensive to compute (a hit-test count, a
formatted rect), prefer [`trace`] with a closure, which defers
everything.

### `struct Line`

`None` means tracing is off and this line will never be emitted; every
method is then a no-op. Holding the `Option` here rather than checking
[`enabled`] in each method keeps the disabled path to one branch per
field instead of one atomic load per field.

### `fn kv`

The value is rendered with [`Display`], so pointer positions,
counts and booleans all work without the caller formatting them.
Neither key nor value is escaped: this is a diagnostic read by a
`grep`, and a quoting scheme would make the output harder to read
in exchange for a case a `key=value` diagnostic does not need to
handle. Keep values free of spaces.

### `fn into_message`

Exists for this module's own tests, and for an application that
wants to route a trace somewhere other than stderr. It does not
emit; [`Self::emit`] does.
