# `checks::redaction_reach` — the reach setting decides what survives the
save, measured on the bytes, twice

## The defect this exists to catch

The preference exists, the settings window offers it, the enum maps onto
the engine's `ResidualScope` — and the value never arrives. Every unit test
under `crates/pdfcer-gui/src/redact/` calls the apply verb with a reach
handed to it directly; none can see the chain in front of that verb:
`preferences.txt` → the loaded preference → the action → the dialog →
`EditSession::set_residual_scope`. A build where any link dropped the value
passes all of them, ships a control that does nothing, and removes text the
operator explicitly asked it to leave.

That is the sharp direction of the failure, and it is why this is driven:
the operator's complaint was *"content I did not select was removed"*, and a
setting that silently does not apply is that complaint again with a control
painted on top of it.

## The fixture puts one string in two places, and that is the whole design

* page 1 **draws** [`SECRET`] — the copy that gets marked, which every reach
  must remove;
* the trailer's `/Info` dictionary holds a **second copy** in `/Title` — the
  copy no viewer shows, whose fate the reach setting decides;
* page 2 draws [`SURVIVOR`] and is never marked.

`/Info` is not an exotic carrier. A CAD exporter writes the drawing title
into it, and on this operator's files that title is very often the words he
is redacting.

## The three byte assertions, and which one is the verdict

| needle | narrow run | wide run | what a wrong answer means |
|---|---|---|---|
| `/Title (SECRET)` | present | absent | **the verdict.** Present in both: the setting is inert in the widening direction. Absent in both: inert in the narrowing direction — which is the original complaint |
| `(SECRET) Tj` | absent | absent | the marked copy on the page survived a redaction that reported success. No reach makes that acceptable |
| `SURVIVOR` | present | present | the control. It is drawn on a page nobody marked, so its absence means either the removal took a page it was never given, or the output's streams are compressed — in which case the rows above are absences from a scan that could not have seen anything |

## The second instrument: the dialog either asks for a tick or it does not

Under the narrow reach the saved file really does still contain the text, so
the dialog raises its residual acknowledgement — a region declared **only
while it is being asked for**. Under the default reach there is nothing left
to acknowledge and it must not appear.

Two independent instruments on one question: what is in the file, and what
the program said about it before writing. A build that got one right and the
other wrong is disclosing something it did not do.

## What this covers that [`super::redaction`] deliberately does not

**The search-and-mark route.** Its neighbour marks whole pages, by choice,
so that it needs no keyboard, and its header names the typed route as its
own gap. This check has to type, because a reach setting is about *a string
found elsewhere* and a whole-page mark carries no string to look for.

## Item notes

### `const REACH_KEY`

A copy of a name owned by another crate, and it carries that arrangement's
tripwire: a rename there makes every launch here run at the shipped default,
both runs then agree, and this check reports the setting as inert when it was
merely asked for under the wrong name. The truth in that event is the
preference writer's own emission of this key.

### `fn fixture_bytes`

Uncompressed for its neighbour's reason: the verdict is a byte scan, and a
`/FlateDecode` stream would hide the page copy from it — a false pass in the
direction that matters most.

### `fn reveal`

The panel is taller than the slot a side dock gives it, so at the harness's
window size its search field and its *Find & mark* control start below the
fold. They are declared with the shell's visible-only helper — a rect appears
only while the control is actually drawable — so *absent* here means *not on
screen*, and the answer to that is a wheel notch rather than a failure.

Rewinds to the top first, then walks down one notch at a time, because the
controls this check needs are not in one direction from each other: the apply
control sits above the search row, so reaching the second scrolls the first
away.

An exhausted walk is an `Err`, i.e. a SKIP: the check could not reach the
control, which is a different claim from the control not working.

### `fn write_reach`

Through `sandbox::write_prefs` rather than `fs::write`, which carries the
startup-offer suppression as a header — see its own contract for what a
direct write costs. Only this one key is written: every other preference is
then absent, which the loader reads as *use the default*, isolating the
variable under test from whatever a previous check left behind.

### `fn run_once`

`Err` is a SKIP — the sequence could not be completed, so nothing was
measured. A finished run always yields an [`Outcome`]; the verdicts are taken
in [`drive`], where both runs can be compared.

### `fn the_generated_fixture_carries_all_three_needles`

The same falsification the run performs, asserted at build time so a
fixture that stopped carrying one of the three needles fails in the
suite rather than on the operator's desk.

### `fn the_page_copy_and_the_properties_copy_are_different_strings`

The whole check rests on telling the page copy from the properties copy.
If one were a substring of the other, a single removal would satisfy both
assertions and the verdict would mean nothing.

### `fn the_two_reaches_are_not_the_same_token`

A copy-paste that made both runs write the same token would produce two
identical outputs, and the verdict would then report the setting as inert
when it had never been varied.
