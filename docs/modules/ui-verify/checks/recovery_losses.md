# `ui-verify/checks/recovery_losses`

**A document pdfcer had to rebuild says what the rebuild could not keep**,
and a document whose rebuild kept everything says nothing — the driven half
of the dropped-object disclosure.

# What this file is for

When a PDF's cross-reference table is unusable, `pdfcer-core` rebuilds one by
scanning the whole file for anything that looks like the start of an object.
Some of what that scan finds cannot be read back, and those candidates are
**dropped**. Until engine `fb6e004` there was no way for a caller to ask
which ones: a recovery that silently lost a page's content stream handed back
a shorter document and told nobody. `RecoveryReport::objects_dropped` now
carries an object number and a reason for each, and the engine's own filing
for it recorded that the disclosure *"is not yet visible to anyone"* — a
statement about **this shell**.

It is visible now, in Document properties, under the recovery note:

* the recovery note itself — heading and census — as `properties.recovery`;
* the dropped-object block under it, two sentences and a list of object
  numbers, as `properties.recovery-dropped`.

The strings are unit-tested from every corner, including through the real
engine on a fixture authored for it. What no unit test can see is whether
either block is **connected to a running window** — whether the panel draws
them where an operator can read them. That is this file's whole subject, and
it is standing rule R1.

# The control launch is the check, and it is a RECOVERED file

The obvious control would have been a document with a sound index, which is
what the neighbouring `load_anomalies` checks use. It would have been wrong
here. On a sound file `Document::recovery()` is `None`, so the entire
neighbourhood — note, census, dropped block — correctly draws nothing, and
"the dropped block is absent" is then satisfied by at least three states that
are not the one under test:

1. the properties panel never opened;
2. the document was never recovered, so nothing nearby drew at all;
3. the block is correctly driven by `objects_dropped`.

An assertion satisfied by all three is not a measurement of which one
shipped. So the control is `fixtures/recovered-no-losses.pdf`: the same
damage, the same recovery path, the same `RecoveryReason`, differing from its
sibling in **exactly one property** — the scan found nothing it could not
keep. Both launches require `properties.recovery`, which disposes of (1) and
(2) and leaves exactly one reading of the difference between them.

⚠ Both fixtures' properties are asserted **through the engine** by the
shell's own tests, in the suite that runs on every `cargo test`:
`the_recovery_fixture_drops_the_two_objects_it_was_built_to_drop` and
`the_control_fixture_for_the_dropped_disclosure_really_recovers_and_drops_nothing`.
That is deliberate placement. If an engine bump changed what the scan finds,
the absence assertion below would go red and blame the application for
something true of the fixture — so the tripwire lives where it costs a second
rather than a driven sweep.

# What this does NOT prove

That the **wording** is right. The trace publishes a region name and a
rectangle, not a string, so a build that printed the wrong object numbers, or
described a routine false positive in the language reserved for a real loss,
would pass. Those are asserted in `crate::text::panels::docprops`' own tests
against the same report — deliberately, because a string is exactly what a
unit test CAN see. What it cannot see is the window.
