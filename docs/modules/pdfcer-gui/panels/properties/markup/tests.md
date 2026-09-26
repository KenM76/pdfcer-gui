# `pdfcer-gui/panels/properties/markup/tests`

## Item notes

### `fn current`

The subtype is a **separate argument** rather than inferred from the
spec arm, and that is the shape of the whole change: the arm is where a
value comes from, the `/Subtype` is what the engine's capability
question is asked about, and a helper that derived the second from the
first would have put this module's deleted list straight back — in the
tests, where it could no longer be seen.

### `fn a_swatch_shows_grey_and_rgb_without_calling_them_converted`

Grey is not flagged because it is **lossless** in both directions —
`Gray(v)` and `Rgb(v, v, v)` are the same ink — where CMYK is not, which
is the distinction `swatch_of`'s own docs draw.

### `fn a_subtype_the_style_verb_refuses_offers_no_controls`

`spec_from_dict` answers `UnsupportedSubtype` for `/Text`, `/FreeText`
and `/Stamp` — verified against the engine source — and `None` here is
exactly what that refusal becomes on the way in. What this asserts is
that the refusal reaches the panel as a *reachability* verdict rather
than as "this mark has no colour", because the two produce different
screens: nothing plus a sentence, versus three live controls that cannot
commit.

### `fn a_subtype_the_style_verb_reads_offers_its_controls`

The companion assertion, and it is the one that stops the fix above from
being "return false always", which would pass the test above and remove
the whole section from the application.

### `fn only_a_shape_with_an_interior_gets_a_fill_row`

A square, a circle, a polygon and a cloud have an interior; a line, an
ink stroke, a polyline and a text markup do not. The cloud is the
interesting one — it is a `/Polygon` in the file and a
`MarkupSpec::Cloud` here — and it is the case this test exists for.

⚠ **Corrected 2026-09-06.** This doc used to justify the answer with
*"`apply_markup_style` reads `style.interior` on exactly the first
four"* and to call the alternative *"a subtype-string list"* that would
have got the cloud wrong. Both halves have moved: the mapping is now
`MarkupStyleSupport::for_subtype` — which **is** keyed on the subtype
string, and is right to be, because it is the engine's string keyed by
the engine — and this shell no longer restates what
`apply_markup_style` reads.

Falsified by pointing `offers_fill` at the interior swatch instead of at
`support.takes_interior`, which turned the line assertion red, and by
passing the cloud `b"Cloud"` — a name no file carries — which turned the
cloud assertion red and is the mistake the old comment feared.

### `fn the_fill_swatch_reads_back_the_interior_and_knows_when_it_is_absent`

`interior.rgb.is_some()` is precisely the condition `fill_row` puts the
Clear button behind, so this is that button's guard asserted where a test
can reach it.

### `fn only_a_line_gets_the_ending_choosers`

⚠ **Corrected 2026-09-06.** *"which is the set `apply_markup_style` acts
on"* was this shell restating a fact about the engine's source. The
engine now publishes it, so the test asks rather than remembers — and
the pair being asserted is the two questions kept apart: `offers_endings`
(the engine's) and `endings` (the value, from the one spec arm that has
one).

Falsified by returning `Some((LineEnding::None, LineEnding::None))` from
every arm, which turned the square's value assertion red, and by
dropping `support.takes_endings` from `offers_endings`, which turned its
visibility assertion red.

### `fn the_ending_list_covers_every_variant_the_engine_has`

`LineEnding` has no `ALL` of its own, so [`ALL_ENDINGS`] is written by
hand — and a hand-written list of an enum's variants is exactly the thing
that goes stale. This `match` has **no wildcard**: an ending the engine
gains fails to compile here, and the fix is to add it to both places at
once.

The count is asserted too, so a duplicated entry (three names, two
distinct variants) is caught as well as a missing one.

### `fn the_width_range_matches_the_pen_that_authors`

Two ranges for one quantity would let an operator author a 2 pt mark and
then be unable to set 2 pt on it — or, worse, set a width here the pen
could not have produced, so a document would carry marks the shell
cannot make.

### `fn the_engines_answer_is_what_hides_a_row_not_the_spec_arm`

The assertion that the workaround is gone. Until 2026-09-06 the Fill row
was decided by a four-arm `match` here, the width row by the
`TextMarkup` arm handing over no width, and the choosers by `endings`
being `None` off the `Line` arm — three restatements of a list
`pdfcer-core` owns, filed as: *"the first subtype that gains or loses a
border is the day our copy is wrong and nothing tells us."*

Every `Current` here is built with **every value present**, so a value
cannot be what differs. A row withheld below is withheld because
`MarkupStyleSupport::for_subtype` said so.

Both directions, per the engine's methodology note: the `Highlight`
row alone would pass with every predicate hard-coded to `false`. The
`Square` and `Line` rows are the positive control that makes it mean
something.

Falsified by restoring the old rules — `offers_width` reading only
`self.width.is_some()` turned the `Highlight` row red, and `offers_fill`
reading whether the interior swatch was set turned the `Line` row red.

### `fn the_removal_is_offered_only_when_the_file_carries_a_line_ending_entry`

The fifth state's guard, and the field it needs exists because
`spec_from_dict` cannot answer the question: Table 176's default makes
an absent `/LE` and a written `[/None /None]` read back identically, so
[`Current::endings`] is `Some((None, None))` either way. Only the
dictionary knows, which is why [`Current::read`] looks.

Both directions. *"Absent when the key is absent"* on its own passes
with the button deleted, which is precisely the vacuous shape the
engine's reply warned about.

Falsified by making `offers_endings_clear` return `self.offers_endings()`,
which turned the first assertion red, and by making it return `false`,
which turned the second red.

### `fn each_family_routes_to_its_own_verb`

The `Square` row is the **positive control** and it is not decoration.
Asserting only that a `/Text` reaches `TextAnnot` would pass on a
`from_spec` that had come to answer `TextAnnot` for everything, which would
send every rectangle in the document to a verb that refuses it — the exact
mirror of the defect this replaced. Both directions, per the engine's
methodology note.

Falsified by swapping the two arms of `from_spec`'s `reach` match; the
`Square` row went red first.

### `fn the_icon_chooser_belongs_to_a_sticky_note_alone`

`set_text_annot_style` refuses an icon on anything but a `/Text` **by name**
— `EditError::StylePropertyNotApplicable`, raised before anything is
written — so a chooser on a stamp would be live and would be refused. R9
says absent, and this is the predicate that makes it absent.

The `Sticky` half is the positive control the negative half needs. *"No
icon chooser for a stamp"* passes on a `takes_icon` hard-coded to `false`,
which would withhold the chooser from the one mark the whole request was
about.

### `fn a_text_box_is_withheld_and_an_unreadable_mark_is_not_the_same_case`

`set_text_annot_style` reaches a `/FreeText` and this shell declines to send
it one: `text_spec_from_dict` always reports `multiline: false`, the verb
re-bakes without measuring, and every text box this shell places is
`multiline: true` — so a colour change would re-lay a wrapped callout as one
line. `super::textannot`'s header carries the full account.

The two arms must stay **distinguishable**, which is what the last
assertion is for: they show different sentences, because one says a
capability is missing and the other says this shell declines one that
exists. Collapsing them would put a false premise under a true-looking
conclusion.

### `fn an_unmodelled_icon_name_is_carried_and_disclosed_and_a_modelled_one_is_not`

§12.5.6.4's seven names are *"a standard set, not a closed one"*, so
`/Sparkle` is **conforming**. This test used to assert the shape of a
workaround:

> *"`text_spec_from_dict` normalises it to `Note` on the way past, and
> `set_text_annot_style` re-bakes from that — so a change to the colour
> alone would write `/Name /Note` into the file with nothing on screen
> saying so."*

That was true, this shell filed it
(`request_set_text_annot_style_rewrites_a_foreign_icon_name.md`), and
`pdfcer-core` `Pass 253.5` answered it with `StickyIcon::Other(Vec<u8>)`
and `from_name_lossless`. **Nothing is lost now**, so the assertions turn
round: the name is a value the panel can show, and `foreign_icon` reports
*pdfcer draws its own picture for this* rather than *this is about to be
destroyed*.

Three cases, and the third is what makes the first mean something. A
`/Sparkle` is foreign; a `/Key` is not; and an **absent** `/Name` is not
either — Table 172's own default is `Note`, so a note carrying no `/Name`
arrives as `Note` and reporting it as such is reporting the standard rather
than inventing anything. Warning about the commonest case there is would
teach an operator to ignore the warning.

The engine's reader does the absent-vs-foreign discrimination now, which
is why this test builds its specs the way the reader would produce them
rather than passing raw bytes alongside.

### `fn every_label_size_source_is_named_distinctly`

`super::textannot::source_token` is the vocabulary a driven check matches on
when it asks *"where did the size in the properties box come from?"*, and
nothing else in the repository holds it to anything. Two variants that
collapsed to one token would make a check that distinguishes them pass on a
build that cannot; and any of them returning `unknown` would fire the
`#[non_exhaustive]` tripwire on a build where nothing had drifted, which is
worse — a tripwire that cries on every run is a tripwire nobody reads.

⚠ The `_` arm is deliberately NOT exercised here. It is reachable only from
a variant this build does not know about, so a test that reached it would
have to fabricate one, and what it would prove is that `match` works.
