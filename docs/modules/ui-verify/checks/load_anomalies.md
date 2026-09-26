# `ui-verify/checks/load_anomalies`

**A PDF that contradicts itself says so, and a PDF that does not stays
quiet** — the driven half of the load-anomaly disclosure.

# What this file is for

`crate::app::status::anomalies` (in the shell) turns the engine's
`Document::load_anomalies()` into two operator-facing surfaces:

* a one-line **census** in the status bar — *"1 duplicate dictionary key"* —
  published as the trace region `status-group:load-anomalies`;
* a **row per anomaly** in the Document properties panel, naming the object,
  the key, the value pdfcer used and the value it left — published as
  `properties.load-anomalies` with `properties.load-anomalies.N` per row.

Every part of that is unit-tested, including against the real engine loader
on a fixture authored for it. What no unit test can see is whether the two
surfaces are **connected to a running window**: whether the status bar draws
the line on the frame the document opens, and whether the panel draws its
rows where an operator can read them. That is this file's whole subject, and
it is the standing rule R1 — *verify by driving the binary, not by a passing
test*.

# ★★★ The second launch is the check

Each of the two checks below launches **twice**: once on
`fixtures/contradicts-itself.pdf`, asserting the disclosure is THERE, and
once on `fixtures/four-pages.pdf`, asserting it is NOT.

Without the control launch, "the region is declared" is satisfied by a build
that declares it on every file — a heading permanently pinned to the status
bar and a section permanently pinned to the panel, saying *"this file
contradicts itself"* about files that do not. That is a worse defect than
the one the check is for, it is the defect R8b's *fuzzy, never sneaky*
clause is most alert to (a disclosure that cries wolf trains an operator to
stop reading it), and it wears exactly the same green tick.

The control fixture's cleanliness is not assumed either: the shell's own
`the_control_fixtures_a_driven_run_uses_are_genuinely_clean` asserts through
the engine that both files this module names really do load without an
anomaly. If that ever changed, the absence assertion here would go red and
blame the program for something that is true of the fixture — so the
tripwire lives in the crate that runs on every `cargo test`, not here.

# ★★★ The third check is not a disclosure at all — it is the way out

`RereadingUnderTheOtherValueIsOffered` drives the **control** that arrived
at the foot of that same block on 2026-09-10. The operator's ruling is that
*"if the user can intervene in a decision that should always be an option"*,
and until that day this shell reported the decision and offered no way to
revisit it.

It is the check with the most between its ends. A press on that button
travels through an `Action`, both of `app::actions::document`'s guards, a
`PendingIntent` that carries the operator's chosen `LoadOptions` across a
dialog that may or may not appear, `reread_active_document`, and finally
`Document::load_with_options`. **Eight of those steps have unit tests and
not one of them can see the step in front of it** — the standing lesson of
this project, paid for by a feature that passed eight green tests while
doing one fourteenth of its job. So the two trace events at the far ends
are asserted together: the button was pressed, *and* a loader started.

# Why two checks and not one

Because they cost different things. The status-bar half needs **no pointer
and no keyboard**: the bar draws on the first frame after the document
opens, so the harness launches, waits, and reads the trace. The panel half
has to click a mode segment, a ribbon tab and a ribbon item.

Fusing them would make the cheap, always-runnable half inherit the
expensive half's `--no-input` SKIP — and a SKIP is not red, so the whole
disclosure would silently stop being evidence on every run where the
machine's pointer belonged to somebody else. That is a failure mode this
project has paid for repeatedly. Split, the status-bar half runs on every
sweep including the ones nobody can watch.

# What these do NOT prove

That the *wording* is right. The trace publishes a region name and a
rectangle, not a string, so a build that drew the census clause in the wrong
order, or spelled the discarded value where the kept one belongs, would pass
both. The wording is asserted in `crate::app::status::anomalies`' own tests,
against the same fixture, through the same engine call — deliberately,
because a string is exactly what a unit test CAN see. What it cannot see is
the window, and that is what is here.
