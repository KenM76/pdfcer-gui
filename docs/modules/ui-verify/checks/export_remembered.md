# `ui-verify/checks/export_remembered`

`the_export_windows_open_on_the_settings_you_last_used` — **operator
request O196, driven.**

# The report


> *"the export windows forget everything. every time I export a dxf I have
> to set it up again."*

Twelve answers across three windows — Export image, Export text, Export to
DXF — were literals in three constructors until that day. An operator who
exports millimetre DXFs with the text omitted answered both questions again
on every export, on every drawing, for the whole life of the program.

The twelve are now written to `userdata/preferences.txt` when Export is
pressed, and read back the next time each window opens. **This check is the
evidence that the reading half works in a running process**, which is the
half the operator experiences as *"it remembered"*.

# What this check deliberately CANNOT establish, said first

**It never presses Export, and no future edit may make it.** Committing
opens a native save picker — a hard wall for synthetic input — and then
writes a file into a directory this harness has no business choosing. The
producing half (`habits()` on each of the three dialogs) is unreachable
from here, exactly as the Print window's committing half is unreachable
from `print_remembered`, and for a related reason: closing without
exporting is how a person says *"not this"*, so the write is inside the
commit block where it belongs.

What stands in for it:

| The claim | What holds it |
|---|---|
| every field of each `Export*Prefs` is written | each `habits()` is one struct literal with **no** `..Default::default()`, so a field added to the prefs type is a compile error there |
| every field is read back by `open` | the three dialogs' own unit tests, plus the trace this check reads, which is formatted from `dialog.*` and not from `remembered.*` |
| the file survives a round trip | `app::prefs`' round-trip and writer/parser-agreement tests |
| the file the operator gets is the file this check writes | **this check**, below — the seed is written in the writer's own token vocabulary |

# Why this one needs no mouse, and what that buys

`print_remembered` reaches its window by clicking a ribbon tab and then a
ribbon control. That costs it two skip reasons it cannot avoid —
`--no-input`, and the control having folded into the ribbon overflow — and
makes it unrunnable on a machine whose desktop is in use.

This check reaches all three windows through **`PDFCER_DIAG_INVOKE`**,
which takes a comma-separated list of command ids, **one rung per frame**,
each dispatched through the same `dispatch_command` choke point a keystroke
reaches. One launch therefore opens all three windows, and each emits its
own `-open` line as it is built. No pointer is moved, no key is pressed,
and the check runs identically under `--no-input`.

⚠ **This is why `frame_of` is absent from this file**, and the absence is
deliberate rather than an oversight. The standing rule — *inside a
dialogue, use `frame_of(&session, &trace, ui_rect, NAME)` and never
`session.frame()`* — exists because a dialogue is a second OS viewport, so
a rect measured against the main window's frame is measured from the wrong
origin. It does not bite here because **this check measures no rect at
all**: its entire oracle is three trace lines. An edit that starts
measuring a rect inherits that rule the moment it does.

# How it tells "read the file" from "agreed with the default"

The same trap `print_remembered`'s header names, and it is sharper here for
a reason worth stating: **`ExportDxfPrefs::default()` is field-for-field
identical to `DxfOptions::default()`.** A check that hard-coded its
expectations and seeded `units = inches` would pass against a build that
never opened the preferences file, never parsed a key, and ignored
`remembered` entirely — which is the exact regression O196 exists to
prevent.

Two things prevent it:

1. **A control launch, first.** The preferences file is reset to the bare
   sandbox seed — which carries no export keys at all, so every one of the
   twelve takes its compiled-in default — the three windows are opened, and
   the twelve shipped defaults are read off the trace. That is measurement,
   not assertion: the defaults belong to the application, and this file
   does not get to have an opinion about what they are.
2. **Every seeded value is then required to differ from the value the
   control run reported.** If any one agrees, the check reports a SKIP
   naming it rather than a pass, because that field would read back
   correctly whether the file was consulted or ignored.

The second rule is what keeps the seed maintainable. When a shipped
default moves, this check does not silently become decorative: it goes
yellow and says which value to change.

# The oracle: three lines, twelve tokens

```text
export-image-open page=0 pages=9 format=emf scope=all dpi=150 transparent=0 quality=72
export-text-open page=0 pages=9 scope=current separator=marker endings=windows bom=1
export-dxf-open page=0 groups=0 suggestion=uncalibrated scale=1 units=millimetres arcs=0 text=omit
```


# Why the seed table has five columns where `print_remembered`'s has three

Its table can make the file value and the trace token the same string by
construction, because both sides are spelled by a `*_key` function. Three
of these twelve are not:

| key | file | trace |
|---|---|---|
| `export_image_transparent` | `true` / `false` | `1` / `0` |
| `export_text_byte_order_mark` | `true` / `false` | `1` / `0` |
| `export_dxf_fit_arcs` | `true` / `false` | `1` / `0` |

The file spells a bool with `opening::bool_key`; the three traces spell it
with `u8::from`. Neither is wrong — a hand-editable file wants the word,
and a whitespace-split trace field is happier with a digit — but a table
that assumed they agreed would report three defects that do not exist.
Hence the fifth column, and
`only_the_boolean_rows_translate_between_the_file_and_the_trace`, which
pins the translation so a fourth spelling cannot arrive by hand.

⚠ And `scope` appears on **two different events** — the image window's and
the text window's — with different shipped defaults (`current` and `all`).
A row is therefore identified by `(event, field)` and never by `field`
alone; `each_seeded_row_reads_a_different_event_and_field` is the guard,
and it asserts the duplication rather than merely tolerating it.

# The DXF calibration trap, which is a feature and not a defect

`dialogs::export_dxf::seeded_options` does two things in order: ① it takes
the operator's habit from the file, then ② **a calibrated ce dimension
group on the page overrules the remembered units**, because a metric page
exported in inches is a DXF that opens cleanly, measures consistently, and
is wrong by 25.4× — discovered by whoever cuts from it.

So on a document whose page carries a calibrated group, `units=` reports
the page's units no matter what the file says, and a check that asserted
its seed would report a defect where the application is doing the single
most important thing that window does.

This check reads `suggestion=` off the control run's `export-dxf-open`
line. When it reads `calibrated`, the `export_dxf_units` row is dropped
from both the seed-differs test and the comparison, and the reason is
reported as a note naming the document. The other eleven are unaffected,
and the count in the pass note says eleven so that nobody reads the green
as covering twelve.

# Every way this reports SKIP

No binary; no `--pdf` (all three commands decline with nothing open, so
there would be no window and no line); the preferences file could not be
reset; the process never saw `PDFCER_DIAG`; one of the three `-open` lines
absent; a seeded value that turns out to equal the shipped default; or the
two runs disagreeing about `suggestion=`, which would mean the control and
the seeded measurement were taken against different pages. Each says which,
and none of them is reported as a pass.

## Item notes

### `const INVOKE_LIST`

Order is not significant here (unlike a mode-then-tool pair, where it is
everything), because the three windows are independent `Option` fields on
the dialog host and all three can be open at once.

### `const EVENTS`

Kept as a pair so a missing line can name the command that should have
produced it. *"No `export-text-open`"* sends a reader to the dialog;
*"no `export-text-open`, from `file.export_text`"* sends them to the
dispatcher first, which is where a rung that never fired actually shows.

### `const PREFS_FILE`

**Reset to the bare sandbox seed** before the control run and rewritten
before the second — never deleted. Those are not the same act: deletion
takes `ask_default_app = false` with it, and the symptom is the O173 offer
opening a real OS window in front of this check's own process. See
[`crate::sandbox::reset_prefs`], and `print_remembered`'s account of what
that cost when it was learned.

### `struct Seed`

Five columns rather than `print_remembered`'s three, for the two reasons in
the module header: three of the twelve are spelled differently in the file
and in the trace, and one field name (`scope`) appears on two events.

### `const SEED`

Every value is chosen to differ from this build's shipped default — and
that is *checked at run time* against the control launch rather than
trusted here, because a default that moves would otherwise turn this table
into decoration without anything going red.

⚠ `export_image_dpi` is written `150` and read back `150` because the
dialog's `dpi` is an `f32` printed with `Display`, which drops a trailing
`.0`. A seed of `150.5` would read back `150.5` and would also be fine; a
seed of `150.0` would read back `150` and would report a defect that does
not exist. That is the one row where the two vocabularies could drift
without either side being wrong, so it is spelled the way the trace spells
it.

### `fn launch_all_three`

Factored out because this check does it twice with different files on disk,
and the two runs must reach the windows by **identical** means — a control
that arrived by a different route would not be a control.

### `fn open_lines`

The FIRST of each, not the last. Each window emits its line once, as it
is built; a second would mean the window was opened twice, and the first is
the one produced by the preferences file this check just wrote.

### `fn every_seeded_value_is_a_single_file_token`

The seed is written straight into `preferences.txt`. A value the parser
does not recognise would be dropped with a `PrefNote::BadValue`, the
field would come back as its default, and this check would report a
defect in the application that is really a typo in this file.

This test cannot call `pdfcer-gui`'s parser — `ui-verify` does not
depend on it — so it pins the shape instead: non-empty, lower case, and
free of the whitespace that would split it into two trace fields.

### `fn the_seed_touches_only_exporting`

The file this check writes replaces the whole preferences file. Seeding
a key outside its subject would be this check quietly changing
something else — and, under `--shared-profile`, changing it for every
check that runs after it.

### `fn each_seeded_row_reads_a_different_event_and_field`

`scope=` appears on the image window's line and on the text window's,
with different shipped defaults. A uniqueness test over `field` alone
would fail on a correct table; one that then "fixed" it by dropping a
row would silently stop asserting one of the two windows' page scope.

So this asserts the pairs are distinct **and** that the duplication is
still there — if a future edit renamed one of them, this test says so
rather than going quietly green on eleven rows.

### `fn only_the_boolean_rows_translate_between_the_file_and_the_trace`

The file spells a bool `true`/`false` (`opening::bool_key`); the three
traces spell it `1`/`0` (`u8::from`). Every other row is the same
string on both sides. A row whose two columns differ for any *other*
reason is a typo that would report a defect in the application, and it
would look exactly like a deliberate translation.

### `fn the_invoke_list_names_the_command_behind_every_event_the_seed_reads`

The list is an environment variable and the events are constants, so
nothing but this ties them together. Adding a fourth export window
means both, and a seed row naming an event nobody rings would report
*"the window did not open"* for ever.
