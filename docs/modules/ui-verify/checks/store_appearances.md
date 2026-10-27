# `store_appearances_writes_a_picture_into_the_file`

**Defect it catches.** Markup ▸ Comments ▸ *Store appearances* is missing, does
nothing, or saves the markup still without an `/AP` — so a file annotated in
Microsoft Edge (whose markups carry no stored picture) stays blank in other
viewers even after the operator asked pdfcer to fix it.

**Fixture.** `square-no-appearance.pdf`: one `/Square` with no `/AP`, the
shape Edge writes.

**Drive.** Review mode; press `ribbon.item.markup.store_appearances` on the
Markup tab; Save a copy.

**Oracles.**

1. The trace carries `appearances-stored baked=1` after the press: the command
   reached `EditSession::bake_missing_appearances` and stored the one markup.
2. Every `/Square` in the saved copy's page `/Annots` has an `/AP`: the
   picture reached the file, which is what another viewer reads.

**Falsified** on a build without the command (the press finds no item) and
proven to fail on the fixture unmodified (its `/Square` has no `/AP`).
