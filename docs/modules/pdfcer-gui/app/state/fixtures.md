# `pdfcer-gui/app/state/fixtures`

## Item notes

### `fn open_fixture`

At module level rather than inside `mod tests`, and `pub(crate)`, because
tests all over the crate need the identical starting point. A second fixture
opener would be a second way to assemble an `OpenDoc` — exactly what
[`OpenDoc::new`]'s own docs argue against — so the visibility widens rather
than the function being copied.

### `const ORPHAN_WIDGET`

Hand-authored from ISO 32000-1, generator checked in as
`fixtures/orphan-widget.PROVENANCE.py`, because **no pdfcer verb can
produce this file**: every field-creating verb registers its widget in
`/AcroForm /Fields` in the same commit. An unregistered widget is what a
damaged or third-party document looks like — a form flattened by a tool
that dropped `/AcroForm` and left the annotations, or a page extracted
from a form without its field tree. ⇒ *a fixture produced by the code
under test measures that code's agreement with itself.*

The widget carries its own `/T (Orphan)` and `/FT /Tx` deliberately.
That makes it the **merged field-widget** — a widget that IS its own
field and was simply never registered, which is the recoverable case. A
bare kid with no `/T` refuses with `WidgetHasNoFieldIdentity` before any
name is examined, which would make a name-refusal test pass for the wrong
reason. The generator's header lists `adopt_plan`'s preconditions and the
byte that clears each.

### `const THREE_TEXT_FIELDS`

Hand-authored as `fixtures/three-text-fields.PROVENANCE.py`, for a reason
the generator at the foot of `canvas::forms::boxes::tests` already
measured: not one of the engine corpus’ eleven form fixtures carries a
text field with an appearance, so none of them can be CLICKED on the page
-- an `/AP`-less field is routed to the properties panel instead. O204’s
driven check needs a field to click and then two more to tab between.

Three rather than two: a ring of two cannot distinguish "Tab moved on"
from "Tab wrapped", so forward and backward would land on the same field
and the direction could not be asserted.

### `const CONTRADICTS_ITSELF`

Both of the facts that make it worth its bytes are asserted rather than
assumed: it **loads**, and it produces **exactly one** anomaly carrying the
kept and discarded values the operator would see.
`fixtures/contradicts-itself.PROVENANCE.py` is the generator, and says why
its xref is deliberately sound.

### `const RECOVERED_WITH_LOSSES`

It is the only file in either corpus that reaches
`RecoveryReport::objects_dropped`. Every other fixture either has a sound
index (so `Document::recovery()` is `None` and there is no report to read)
or, like [`CONTRADICTS_ITSELF`], deliberately keeps its index sound in order
to test the anomaly path without lighting this one.

The two drops are the two different stories that share one reason code:
object 9 does not exist (the bytes `9 0 obj` appear inside the content
stream's own text, which the scan is obliged to try), and object 8 is real
and truncated. The disclosure has to be readable for both without making the
first sound like the second. `fixtures/recovered-with-losses.PROVENANCE.py`
carries the whole account, including why `DropReason::IdMismatch` cannot be
produced from a hand-written file.

### `const RECOVERED_NO_LOSSES`

It is a RECOVERED file rather than a sound one, and that is the whole
point of it. A driven check that opened a *sound* document to prove the
dropped-object block is absent would also pass if the panel never opened or
the document was never recovered, so it would measure nothing. This file
differs from its sibling in exactly one property, so the two launches
isolate exactly one variable.

Its property is asserted through the engine in `crate::panels::docprops`,
in the suite that runs on every `cargo test`, so a fixture that stopped
being a control surfaces there rather than as a red driven check blaming
the application. `fixtures/recovered-no-losses.PROVENANCE.py` carries the
rest.
