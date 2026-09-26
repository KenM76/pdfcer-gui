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
