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

# ★★★ What this check deliberately CANNOT establish, said first

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
| the file the operator gets is the file this check writes | ★ **this check**, below — the seed is written in the writer's own token vocabulary |

# ★★ Why this one needs no mouse, and what that buys

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

★ The second rule is what keeps the seed maintainable. When a shipped
default moves, this check does not silently become decorative: it goes
yellow and says which value to change.

# The oracle: three lines, twelve tokens

```text
export-image-open page=0 pages=9 format=emf scope=all dpi=150 transparent=0 quality=72
export-text-open page=0 pages=9 scope=current separator=marker endings=windows bom=1
export-dxf-open page=0 groups=0 suggestion=uncalibrated scale=1 units=millimetres arcs=0 text=omit
```


# ★★ Why the seed table has five columns where `print_remembered`'s has three

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

# ★★★ The DXF calibration trap, which is a feature and not a defect

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
