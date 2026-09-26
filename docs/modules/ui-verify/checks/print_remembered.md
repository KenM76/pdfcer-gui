# `ui-verify/checks/print_remembered`

`the_print_window_opens_on_the_settings_you_last_used` — **operator request
O166, driven.**

# The report


> *"the printer dialogue box needs to remember our last settings."*

Before that day `PrintDialog::open` built every field from a literal, every
single time, and the dialog object was dropped when the window closed — so
nothing about a print survived even within one session. An operator who
prints landscape, two-sided, at 600 dpi re-answered all four questions on
every print.

Thirteen answers are now written to `userdata/preferences.txt` the moment
**Print** is pressed, and read back the next time the window opens. This
check is the evidence that the *reading* half works in a running process,
which is the half the operator experiences as *"it remembered."*

# ★★★ What this check deliberately CANNOT establish, said first

**It never presses Print, and no future edit may make it.** Committing is
how a print job reaches a real device, and this suite runs unattended on the
machine whose default printer is the operator's plotter. Four print checks
already state that rule in their own words rather than by reference, because
the day somebody adds a fifth by copying one of these files, the copied file
is what they will read.

`PrintDialog::remember` is called from inside the commit block — deliberately,
because closing without printing is how a person says *"not this"* — so the
**writing** half is unreachable from here. What stands in for it:

| The claim | What holds it |
|---|---|
| every field of `PrintPrefs` is written | `habits()` builds the struct with **no** `..Default::default()`, so a fourteenth field is a compile error |
| every field is read back by `open` | `every_remembered_field_is_read_back_by_the_print_dialog`, which parses the struct's fields out of its own source |
| the file survives a round trip | `every_preference_round_trips_through_the_file` and `the_writer_emits_no_key_the_parser_rejects` |
| the file the operator gets is the file this check writes | ★ **this check**, below — the seed is written in the writer's own token vocabulary |

So: *"press Print and the file is written"* is held by the compiler and
three unit tests. *"the file is read and the window opens on it"* is held
here, by a running process. Neither half is claimed by the other, and a
reader who took this green as covering both would be taking more than is
here.

# How it tells "read the file" from "agreed with the default"

This is the trap `page_display_pref` names in its own header, and it is
sharper here because there are twelve values rather than one. A check that
seeded `orientation = auto` and found `auto` would pass against a build that
never opened the file at all.

Two things prevent it:

1. **A control launch, first.** The preferences file is reset to the bare
   sandbox seed — which carries no print keys at all, so every one of the
   twelve takes its compiled-in default — the program is started, the Print
   window is opened, and the twelve shipped defaults
   are read off the trace. That is measurement, not assertion — the defaults
   are the *application's*, and this file does not get to have an opinion
   about what they are.
2. **Every seeded value is then required to differ from the value the
   control run reported.** If any one of them agrees, the check reports a
   SKIP naming it, rather than a pass — because that field's result would be
   the same whether the file was read or ignored.

★ The second rule is what makes the seed maintainable. When a shipped
default changes — and `MIN_PRINT_DPI` moved 50 → 36 the day this feature
landed — the check does not silently become decorative. It goes yellow and
says which value to change.

# The oracle

`print-open`, emitted once as the dialog is built:

```text
print-open printers=4 selected=1 remembered=matched unavailable=None page=0
           orientation=landscape duplex=long-edge paper=match-pages tray=true
           scale=custom percent=137 markup=document-and-markups dpi=600
           copies=3 collate=false subset=odd reverse=true
```


★★★ **And then it was wrong a second time, in a way that looked
finished.** As first written the twelve fields were formatted from
`remembered.*` — the parsed `PrintPrefs` handed *into* the constructor —
and the dialog's own fields were assigned forty lines further down in a
separate expression. **A trace emitted from the INPUT to a construction
proves parsing, not adoption.** Ten of the twelve assertions below were
therefore vacuous: they would have held on a build whose struct literal
ignored `remembered` entirely, which is precisely the regression O166
exists to prevent. Only `orientation` and `duplex` were covered, and only
by accident — via the `print-plan` cross-read described below, which comes
at the values from the far end.

The trace block now sits **after** the struct literal and reads `dialog.*`.
Nothing about this check changed; the thing it was pointed at did. That
matters more than it sounds, because a check is only ever as good as the
trace under it, and this one had no way to say it was reading the wrong
end. The defect was found by trying to make the check FAIL and being
unable to — see the section immediately below, which is why that section
exists.

Every token is produced by the **preferences file's own** `*_key` function,
not by `{:?}`. That is this project's standing rule about Debug-formatting a
field a machine reads, and it has a second benefit here: the seed below is
written in the same vocabulary, so the comparison is literal, token for
token, rather than against a second spelling free to drift.


*"A check that cannot fail is not evidence"* is a standing lesson in this
project, written after a long-green gate was found to be aiming at
nothing. So this one was made to fail on purpose before it was believed,
and the record of that is kept here rather than in a commit message,
because the next person to doubt this check will be reading this file.

**What was planted.** Not the obvious sabotage. Shadowing `remembered`
at the top of `open` would break the *parse* as well as the *adoption*,
and a check reading the wrong end of the constructor would go red on that
too — proving nothing about the correction described above. So the
narrow form was planted instead: the preferences file still parsed, the
trace's `remembered=` field and the printer lookup still reading it, and
**only the struct literal** switched to a `PrintPrefs::default()`.
Thirteen field reads, one line of sabotage. That is the exact regression
the previous oracle was blind to.

**What came back.** `RESULT: FAIL`, naming all twelve fields with the
seeded value beside the default that arrived instead — and the failure
text's own escalation fired correctly, reporting that twelve-of-twelve
*"points at the whole path rather than at one field"* and naming
`PrintDialog::open`'s seeding block as one of the two places to look.
It was the right answer. The file was then restored from a copy and the
check re-run green.

⚠ If a future edit to `PrintDialog::open` makes this check awkward,
**re-run that falsification rather than trusting the green.** It costs
two release builds and about six minutes, and it is the only thing that
has ever caught an oracle pointed at its own input.

# ★★ Why `print-plan` is read as well

Because `print-open` reports what the dialog's **fields** were set to, and a
build that stored the preferences into fields the job planner never consults
would pass every assertion above while printing portrait anyway. `print-plan`
carries `orientation=` and `duplex=` from `effective_device()` — the values
that actually reach the spooler — so the two lines are required to agree.

That is the same pairing argument `print_paper` makes about `paper=` beside
`sheet=`, one dimension along.

# Every way this reports SKIP

No binary; `--no-input`; no `--pdf` (the Print command is gated on a document
being open, so the ribbon control is greyed and there is no dialog to reach);
no ui-rect channel; the ribbon control not declared; the dialog not opening;
the spooler refusing on this machine; or a seeded value that turns out to
equal the shipped default. Each says which, and none of them is reported as
a pass.
