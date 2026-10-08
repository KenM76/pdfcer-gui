# `ui-verify/checks/protect`

`checks::protect` — the Security band opens a window that shows the
document's own state, and **refuses a signed document instead of offering a
form**

`OPERATOR_REQUESTS.md` **O119**. Two ribbon controls —
`file.encrypt` and `file.permissions` — in a new Security ▸ Security group.

# The defect this exists to catch, and why a unit test cannot

Three things about this surface are true only of the **running program**,
and every one of them is a rule the crate's own tests assert about a value
rather than about a window:

1. **The controls are on the ribbon and reachable.** `crate::protect`'s
   suite proves the model; `shell::commands`' suite proves the registry
   holds 136 commands. Neither proves an operator can find either control
   at the harness's 1,100 pt window — where the File band is wide enough to
   collapse groups, and a collapsed group publishes no rect until it is
   opened. A build with a correct model and a Security group that never
   renders passes every test in the crate.
2. **A signed document draws NO FORM.** This is O119's second
   disclosure and it is **R9** in its sharpest form: *the control is absent
   or explained, never a button that fails on press.* The controls stay on
   the ribbon — whether THIS document is signed is not known when the
   registry is built — so the whole weight of R9 falls on the window
   refusing before it draws anything. A `Phase::Refused` that still drew the
   password boxes would be a form whose only possible outcome is an engine
   refusal, and no headless test can see that the boxes were drawn.
3. **The confirm control is not live on a blank form.** `ready_to_confirm`
   is asserted headlessly, but *whether the button it gates is actually
   published as a clickable rectangle* is a fact about drawing.

# The falsification, and where it is

**A test that checks a relation rather than a magnitude is satisfied by any
absurdity in the right direction.** That bites hardest on an **absence**
assertion, and two of the three findings above are absences. An absence
check passes on a window that never opened, on a build
where the click missed, and on a region name that was never spelled the way
the check spells it.

So every absence in this check is paired with a **presence measured in the
same trace**, and the presence is what makes the absence mean something:

| phase | document | click | must be PRESENT (the instrument works) | must be ABSENT (the verdict) |
|---|---|---|---|---|
| A | plain | `file.encrypt` | `protect-dialog`, `protect-standing`, `protect-advisory` | `protect-confirm` — the gates are shut on a blank form |
| B | plain | `file.permissions` | `protect-dialog` | `protect-standing` — permissions on an unprotected file is refused, not a form of eight ticked boxes |
| C | **signed** | `file.encrypt` | `protect-dialog`, `protect-signed-refusal` | `protect-standing`, `protect-advisory`, `protect-confirm` — **no form at all** |

Phase A's presences are exactly what phase C's absences deny, over the
same region names, in the same build, minutes apart. That is the pairing
that stops phase C passing vacuously: a build in which `protect-standing`
was never declared under ANY circumstances would fail phase A, so its
absence in phase C is evidence about the signed document rather than about
the spelling of a constant.

And `protect-advisory` is asserted **present** in phase A on its own
account, not merely as a control. It is the rectangle carrying
`EncryptionSettings::PERMISSIONS_DISCLOSURE` — O119's first disclosure, the
engine's own sentence, the one this surface may not ship without. A build
that drew the eight tick-boxes and dropped the sentence above them would be
the exact failure O119 warned about, and it would pass every test in
`crates/pdfcer-gui`.

# A second oracle: the application's own `protect-opened` line

Regions say what was drawn. The trace line says what was **read off the
document**, and the two are independent readings of the same act:

```text
protect-opened task=password encrypted=0 revision=0 signatures=0 on_disk=1 granted=8 refused=0
protect-opened task=password encrypted=0 revision=0 signatures=2 on_disk=1 granted=8 refused=1
```

`signatures=` and `refused=` are the pair the R9 finding turns on, and they
are emitted by `ProtectDialog::open` **before** anything is drawn — so a
build that read the document correctly and then drew the form anyway is
distinguishable here from one that never read it, which a region assertion
alone cannot do.

# NOT RUN

**Nothing in this file has been executed against a running binary.** It is
registered so that the next `ui-verify` run executes it, and until one does
every claim above is a claim about the code rather than about the program.

## Item notes

### `const MODE`

**Read**, deliberately, and it is itself an assertion. The File tab is in
every mode's tab list, and `app::dispatch`'s arm for these two commands says
in words why they are reachable from a reading stance: *protecting a drawing
before sending it out is not an act of authoring, and an operator reading a
document in Read mode is exactly the operator about to email it to
somebody.* Driving from Read is how that claim gets checked rather than
merely written.

### `fn press`

Through [`declared_or_in_overflow`] rather than a bare rect lookup, and
this is the whole reason phase 1 of the module header's finding list is a
finding at all. At the harness's 1,100 pt window the File band runs out of
width, and a Security group added at the END of an already-full band is
exactly the group that lands in a collapsed popup or past the overflow
button. Neither publishes a rect until it is opened, so a plain `declared`
would report *"the application declared no `ribbon.item.file.encrypt`
region"* — which would be true, and would be reported as a missing feature
when what is missing is a scroll.

The invoke count is read **before and after** rather than as a presence:
this check presses three different controls across two processes, and *"has
it ever been invoked?"* would be answered `true` by a press made a minute
earlier.

### `fn drawn`

A degenerate rect counts as **absent**, not present. A region declared at
zero area is not something an operator can see, so counting it as a presence
would let a build satisfy phase A's instrument assertions with three
invisible rectangles.

### `fn close_window`

That guard is deliberate and documented — a second press must not discard
a half-filled form — so this check has to close the window between phases
rather than pressing twice and wondering why nothing changed. Escape is the
host's own close, the same one the title-bar × reaches.

### `fn repo_fixture`

The `&CheckContext` parameter is gone, and its absence is the point. It
existed because this function once resolved the path from `ctx.source_root`
— the staleness root, which defaults to `crates` — and then kept the
parameter alive with a `let _ = ctx;` after that was corrected. A parameter
retained only to be discarded is an invitation to use it again.
