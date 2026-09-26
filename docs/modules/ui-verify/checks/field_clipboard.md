# `ui-verify/checks/field_clipboard`

`a_form_field_can_be_copied_and_pasted_both_ways` — the driven proof of
`OPERATOR_REQUESTS.md` **O58**.

# What was wrong

**Ken, 2026-08-29:** *"wire the request. ctrl v for paste as new. ctrl shift
v for paste as duplicate."*

Before this, `Ctrl+C` over a selected form field did **nothing** — and not
by refusal. There was no path at all: `canvas::clipboard::copy` reads
`doc.selection`, a selected form field lives on `doc.selected_field`, and
`/Widget` is deliberately excluded from annotation selection. So the chord
fell through to the *content* copy, found an empty content selection, and
refused with *"nothing is selected"* over an object with visible grips
around it.

# Why a unit test cannot discharge this, and it is R1's argument again

`canvas::fieldclip::tests` proves the offset rule and the loss list, and
`text::fieldclip::tests` proves the sentences. Neither can see the six
things standing between those functions and the operator's keyboard, and
**every one of them has been a real defect on this project**:


The last row is the one that matters most. This shell **asserts to the
operator** that a duplicate paste keeps the original's font, colour and
calculation script, and the whole basis of that claim is one branch inside
`pdfcer-core`. A green unit suite would restate the claim; only a driven run
against the real engine can test it.

# The oracle

Three trace lines, and the check reads all three because each answers a
question the others cannot:

| line | question |
|---|---|
| `fieldclip-copy` | did the chord reach the FIELD path, rather than the content path? |
| `fieldclip-paste` | was a paste raised, and in which of the two senses? |
| `form-target` | did a **second box** actually appear on the page? |

The third is the one that makes this check worth running. `fieldclip-paste`
proves an *intent* was raised; it does not prove the engine accepted it. A
build whose `FieldAction::Paste` arm was never written would emit the paste
line and add no field, and a check reading only the first two would pass
against it. Counting `form-target` boxes before and after is what makes the
difference between "the shell asked" and "the document changed".

# The sequence

1. launch in Edit with the text-field tool armed and the placement dialog
   auto-accepting, and place a field with one click;
2. Escape to disarm, click blank paper to clear the selection, click the
   field to select it;
3. `Ctrl+C` — assert a `fieldclip-copy` line;
4. `Ctrl+V` — assert `fieldclip-paste mode=NewField` **and one more box**;
5. `Ctrl+Shift+V` — assert `fieldclip-paste mode=Duplicate` **and one more
   box again**.

Step 5 runs against the clipboard written in step 3, not against anything
step 4 left behind, which is the property that makes the two chords
independent: an operator copies once and pastes several times.

# What this check does NOT prove, said out loud

That the duplicate **shares a value** with its source. That is the whole
point of the chord and it is invisible from the outside: proving it needs
typing into one box and reading the other, which needs a fill gesture this
harness does not yet have. The box count proves a second widget arrived; it
does not prove the two are one field.

⇒ Recorded here rather than left implied, because a check whose name says
"both ways" and whose body proves half of one of them is exactly the
green-result-reporting-nothing this harness exists to avoid. The engine
request in the channel asks for the verb that would make this assertable.

## Item notes

### `const PLACE_AT`

Well inside the sheet on both axes, because two pastes each displace the
copy ten points down and to the right and all three boxes must stay on
paper — a box pasted off the sheet would produce no `form-target` line and
the check would report a clipboard defect for a geometry problem.

### `struct Order`

Carried rather than assumed, because the whole subject of the second
check is that the SAME keystroke means the OTHER thing. A check that hard-
coded `Ctrl+V` -> new field could only ever test one of the two orders, and
would pass against a build whose setting did nothing at all.

### `fn boxes`

The same reader `field_menu` uses, kept in step with it deliberately: two
parsers of one trace line is how two checks come to disagree about what the
program said.

### `fn field_names`

Distinct **names**, not lines. The census is re-emitted every frame it
changes, so counting lines counts repaints. And it is names rather than
boxes because a paste-as-new must raise the count and this is the number
that says so unambiguously.

### `fn distinct_boxes`

**The first version of this function counted trace lines and it was
wrong, and it was wrong in the direction that still passes.** The census is
re-emitted on every frame it changes, so the cumulative line count went
1 → 3 → 6 across the two pastes: 1, then 1+2, then 3+3. Both assertions held
— the number did rise each time — and the check reported PASS while measuring
*repaints* rather than *widgets*.

That is this project's standing failure: **ask what the check SAMPLED before
believing what it says.** A build that pasted nothing but repainted twice
would have satisfied the old version exactly as well.

A set of `(field, centre)` is immune, because a re-emitted census re-states
the same pairs. Two widgets of one field differ by centre — the paste offset
guarantees it — so a duplicate raises this count without raising
[`field_names`], which is the distinction the whole feature is about.

The centre is rounded to whole canvas pixels before it enters the set: the
census prints one decimal, and a scroll of a fraction of a pixel between two
frames would otherwise make one box look like two.

### `struct TheAcrobatPasteOrderSwapsWhichChordDoesWhich`

Ken, 2026-08-29: *"let's make it an option to have it swap to match Acrobat
or work the way we have it now."*

# Why this is a second check and not an extra phase of the first

Because the setting is applied **once, at start-up**, when the shell's
keymap is assembled. Testing it needs a second process, not a second
gesture — and a check that relaunched mid-run would be asserting two
different programs under one name.

# What it would catch that the first check cannot

A setting that saves, reloads, reads back correctly in the pane, and
**changes nothing when the key is pressed**. That is the silently-inert
control this project has shipped before, and it is invisible to every unit
test: `PasteChords` round-trips through its file token, `apply_paste_chords`
rewrites the map, and the keystroke can still reach the old command if any
link between them is missed.

The assertion is deliberately the MIRROR of the first check's, not a copy
of it: under this order `Ctrl+V` must add a **box without a name** and
`Ctrl+Shift+V` must add a **name**. A build that ignored the setting would
pass the first check and fail this one on its first assertion.
