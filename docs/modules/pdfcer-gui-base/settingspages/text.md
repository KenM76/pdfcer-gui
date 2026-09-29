# `pdfcer-gui-base/settingspages/text`

## Item notes

### `fn word_gap`

# A slider, not a text box

Free text invites a number that then has to be silently clamped, and a
silent clamp on a setting is an edit the operator did not make.

# The range MUST be the store's own accepted range

`MIN_WORD_GAP_RATIO` and `MAX_WORD_GAP_RATIO` are `pub` in `pdfcer-core`
**specifically so a front end can bound its control by the same numbers the
parser clamps to**, and using them rather than a "usable band" is
load-bearing rather than tidy.

The first attempt at this control used `0.05..=1.0`, on the reasoning that
nothing outside it is useful. That range cannot represent a legal value an
operator may already have hand-edited into their file — so **merely opening
this window would drag a hand-edited `2.0` down to `1.0`, and Save would
write the changed value back**. A silent, unrequested edit to a
configuration, invisible because the operator never touched the slider. The
file is explicitly meant to be hand-editable; a window that quietly narrows
what the file may say is a window that punishes using it.

# Logarithmic, unlike the tolerance slider in [`super::measuring`]

The useful resolution is all at the low end, where `0.15` against `0.25`
decides whether words run together. Everything above `1.0` behaves much the
same, so a linear slider would spend four fifths of its travel on
indistinguishable values and make the range that matters unhittable.

### `fn unmappable`

# The consequence the old note omitted

The source's warning for *Leave it out* was that extracted text reads as
complete when characters are missing. True, and the smaller half.

The larger half is documented in `pdfcer-core` and was shown nowhere: the
layout pass drops a run with no characters, so a run whose codes are **all**
unmappable **disappears entirely, glyph records included**. A page of
`Identity-H` text with no `/ToUnicode` yields *zero runs* rather than runs of
sentinels — so that text cannot be found, selected, or redacted by pattern
at all. That is the surprising failure and the one that breaks anything
needing per-glyph positions.

# What the setting cannot switch off

The rung-4 counter keeps counting whatever is chosen — it is the headline
honesty metric and this setting must not be able to silence it — and three
internal paths pin the sentinel to `ReplacementChar` regardless of the
operator's choice, each saying so at the call: the text-editing slot table
(a zero-length span is a glyph the operator can see and cannot address), the
redaction audit record (must not report a removal as nothing), and the
vector-object text preview (must not make an undecodable run look empty).

The setting's scope is **extraction output**, which is why the window does
not offer it as a global rendering choice.

### `fn actual_text`

# Three statements in the standard, and none dislodges the others

§14.9.4 says `/ActualText` *"shall be used as a replacement"* — the only
**shall**. §14.8.2.4.2's note 2 says a reader *"may choose to use"* it — a
**may**, inside an *informative* note. §9.10.1 says it *"may be used"*.

The only sentence addressing precedence is the `may`, and it sits in a note,
so under the standing normative-versus-informative rule it cannot be cited
alone as authority — and neither reading can be eliminated. That is why this
is a setting, and why the default is `Always`: it follows the one `shall`.

# The bound that is not a setting, disclosed under the group

**No length correspondence exists** between replacement text and the content
it replaces — the standard's own example maps two shown characters to one —
so character-level mapping back to glyph positions is *impossible* across
such a run. That bounds search highlighting, selection and redaction-by-text
to **sequence** granularity **whichever of the three options is chosen**.

`pdfcer-core` calls it *"a fact to disclose, not a direction to pick"*, and
the old window disclosed it nowhere. It is rendered as a
[`widgets::disclosure`] under the whole group rather than as a note on one
option, because an operator who read it as an argument for *Ignore it* has
been misled by the placement.

### `fn find_trim`

His report: *“trailing spaces/tabs/etc stops a search from finding text
on the page that doesn't have these symbols … copy pasting from excel
seems to give a trailing space that I have to remove to search … this
should be an option in the settings to include or exclude such items in
the search.”*

# In this group, and the group's title is the argument

*Copying and extracting text* is where it belongs because **search is
extraction** — the module header above already says so for the other
three settings here, each of which reaches search without saying so in
its own title. This one is the same subject approached from the other
end: not what the page yields, but what the operator asked for.

It is in Settings rather than in the find bar's own Options menu because
that is where he asked for it, and because the menu's two existing
entries are per-search choices an operator changes while hunting, where
this is a standing answer to how their clipboard behaves.

# The live half is applied in `PdfcerApp::save_settings`

This function edits `Draft::working_prefs`, which is the **file**. The
value the next search reads lives on [`crate::find::FindState`], and
`save_settings` copies it across when the operator presses Save, beside
the paste chords and the Acrobat path, for the reason recorded there:
adopting a preference without applying it gives the operator a control
that saves correctly, reloads correctly, reads back correctly and
changes nothing.
