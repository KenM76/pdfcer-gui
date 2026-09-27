# `ui-verify/checks/print_dialog`

`print_dialog_reaches_the_spooler` — the regression test for the defect
the operator reported as *"the print dialogue didn't work"*.

# The defect

`crates/pdfcer-gui-base/src/printspooler/` is the adapter between the
print dialog and `pdfcer-print`. It was written as four **holes** — named
functions whose bodies returned `Err(Unavailable::NotLinked)` — because at
the time it was written `pdfcer-print` genuinely was not a dependency of the
crate. Its module header set out, in full, the two edits that would make the
build print: add the manifest line, then fill the four holes.

**The manifest line landed and the holes were never filled.** `pdfcer-print`
sat in `Cargo.toml` and in `Cargo.lock`, was compiled and linked into every
shipped binary, and *no source file in the crate contained the identifier
`pdfcer_print` outside a doc comment*. So on a machine with twelve printers
installed, `File ▸ Print…` opened a window that said

> This build cannot reach a print device, so there is nothing to print to.

…and drew no printer selector, no preview, no page controls and no commit
button. Every one of those absences was **correct behaviour for the state
the adapter reported**, which is why nothing looked broken from the inside.

# Why the entire test suite was green

This is the part worth keeping, because it is a new shape of the failure
this project was founded on.

The adapter had a test. It was called `every_hole_refuses_rather_than_
guessing`, and it asserted that all four functions returned
`Err(Unavailable::NotLinked)`. That assertion was **correct** — provably,
obviously correct — for as long as `pdfcer-print` was not linked. It was
written to catch a specific and real hazard: somebody "helpfully" filling
`plan` with a local placement calculation so the preview would draw
something, producing a confidently wrong sheet indistinguishable from a
correct one until the paper came out.

The moment the manifest line landed, that test stopped protecting anything
and became **a lock holding the defect in place**. Filling the holes — the
correct next step, written down three inches above the test — would have
turned the suite red. A test that pins a refusal must name the condition
the refusal is conditional on, or it outlives its own premise and starts
defending the absence of the feature.

So the unit test that replaces it asserts the opposite property, and this
check asserts the same thing from outside the process, where no assumption
about the crate's internals can hold it up.

# What this check measures

One trace line, emitted by `PrintDialog::open`:

```text
print-open printers=12 selected=5 unavailable=None page=0
```

Three fields, and each carries a different half of the verdict:

| field | defect | fixed |
|---|---|---|
| `unavailable` | `Some(NotLinked)` | `None` |
| `printers` | `0` | however many the machine has |
| — | no `ribbon.item.file.print` click reaches an arm | the dialog opens |

## Why `printers` is asserted as well as `unavailable`

Because `unavailable=None` alone is satisfiable by a stub that returns
`Ok(vec![])`, and that is not a hypothetical: an empty list is exactly what
a lazy repair would produce, it renders as the *"This system reports no
printers"* sentence, and that sentence is **plausible**. A machine really
can have no printers.

So this check requires **at least one printer**, and reports SKIPPED rather
than PASSED when it finds none — the three-state discipline the whole
harness uses. On a machine with no printers the check has learned nothing,
and saying so is the only honest verdict. It is the same reasoning
`pdfcer-print` itself applies from the other side: *"reporting the same
value for 'this platform cannot enumerate printers at all' would collapse
two different facts into one and send a caller looking for hardware."*

# What this check deliberately does NOT do

**It does not press the commit button, and it never will.** That button is
the one control in the application that consumes paper, occupies a device
other people may share, and cannot be undone. `pdfcer-print`'s own header
states the contract — `spool` is the only function that reaches `StartDoc`,
and it is reached only from a control an operator deliberately clicked —
and a harness that can start a print job is a harness that will eventually
start one by accident, at three in the morning, on the office plotter.

The old shell's diagnostic harness reached the same conclusion in the same
words, and it is worth restating rather than rediscovering.

What that costs is real and should be named: this check proves the dialog
**reaches the spooler**, not that a sheet comes out correctly placed. The
placement arithmetic is `pdfcer-print`'s and is tested there; the conversion
from this crate's mirrored types into the engine's is pinned by
`spooler::tests::the_conversions_map_every_variant_to_its_own`; and the
last link — that the bytes reach paper — is verified by a human pressing
the button once, which is the correct amount of automation for an
irreversible act.

## Item notes

### `const TAB_ID`

**File, and that is load-bearing rather than incidental.** `file.print`
sits on the File tab, which is in *every* mode's tab list including Read's
(`["file", "view"]`) — so unlike the render-diagnostics check, this one
needs no mode change before it can find its control. If Print ever moved to
a tab Read does not carry, this check would begin skipping with *"the tab
strip is too narrow"*, which is a confident wrong diagnosis; the constant is
spelled out here so the failure names the real cause.
