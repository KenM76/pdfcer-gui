# `pdfcer-gui/redact/tests/reach`

## Item notes

### `fn secret_on_the_page_and_in_the_properties`

`/Info` is the commonest carrier there is and the one an operator never
sees: a CAD exporter writes the drawing title into it, and the title is
very often the words he is redacting. The page copy is what he marks; the
trailer copy is what the reach setting decides the fate of.

### `fn the_reach_setting_decides_whether_the_copy_in_the_properties_survives`

Threading a parameter through a call chain compiles whether or not the
value arrives anywhere that matters, and every test that asserts only
*"the argument was passed"* passes on a build where the engine ignores it.
So this one measures the file: the same marks, the same document, two
reaches, and the copy in the document properties survives exactly one of
them.

Three further properties ride along, and each is a defect if it inverts:

* **The page copy dies under both.** Without that control a build that
  redacted nothing at all would satisfy the headline assertion under the
  narrow reach.
* **`KEEPTHIS` survives both**, so neither reach is achieving its result by
  destroying the document.
* **The narrow reach still saves.** It saves behind the residual
  acknowledgement, and that is correct rather than a gate to remove: this
  shell's own byte sweep of the finished file genuinely finds the secret in
  it, and the tick is the operator confirming he is keeping a file that
  still contains the text he marked. What must never happen is a refusal he
  cannot satisfy, and this pins which of the two it is.

### `fn a_narrow_reach_names_the_copy_it_declines_to_touch`

The saved-bytes assertion above proves the setting reaches the engine; this
proves the *disclosure* does. A reach that silently leaves a copy behind is
the "sneaky" half of "fuzzy, never sneaky" — the operator chose a narrower
scope and is owed the census of what that choice costs, off-canvas, before
he commits.
