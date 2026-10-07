# `a_fields_alignment_can_go_back_to_inherited`

**Defect it guards.** A field's `/Q` is inherited: absent on the field, it is
the nearest `/Parent`'s, else the `/AcroForm`'s, else left. `Field::quadding`
is that resolved value, so a chooser offering only Left, Centred and Right
shows an inherited justification as if the field stated it, and once one is
picked nothing can remove it again. Left written on the field and nothing
written are different assertions: a later change to the form's `/Q` reaches
the second and not the first.

**Fixture.** `fixtures/inherited-alignment.pdf`, built by
`inherited-alignment.PROVENANCE.py` from `three-text-fields.PROVENANCE.py`
with `/Q 1` added to the `/AcroForm`. No field states its own `/Q`.

**Steps.** Launch with `FieldOne` selected and Properties open. The
`field-alignment-read` line must say `own=0 q=1`. Open the chooser
(`properties.field_edit.alignment.combo`), pick Left (`…option.0`): `own=1
q=0`. Open it again, pick Inherited (`…option.inherit`): `own=0 q=1`.

**What `own=` reads.** Whether the field's dictionary, through the session's
overlay, carries `/Q` (`panels::properties::formfield::states_own_quadding`).
The engine exposes only the resolved value; that lookup is reported to the
engine as a workaround.

**What it does not prove.** That the chooser's caption reads *Inherited
(centred)*: the caption is derived from the same two values the trace line
prints, and is checked by reading the code, not the pixels.
