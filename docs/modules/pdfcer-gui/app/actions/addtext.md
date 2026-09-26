# `app::actions::addtext` — placing NEW page text, and the width question

One verb, `Action::CommitAddText`, and the one decision it has to make that
nothing upstream can: **how wide is the text the operator just typed?**

## Why it is its own file

The same seam [`super::funnel`] is cut along: [`super::apply`] is a
**router** — it answers *"which module handles this action?"* — and this
answers *"what does placing text mean?"*. The arm below decides something;
a router arm should not.

## A PDF HAS NO PARAGRAPH, and that is the whole subject

Every visible line of text in a PDF is its own show operator at its own
absolute position. There is no object that means *"this text, flowing"*. So
something has to decide where a second line starts — a **width to wrap
against** — and that decision has to be made by the time the engine is
called, because `AddTextRequest` offers exactly two shapes:

| request | what the engine does with `\n` |
|---|---|
| **point** — `new(page, origin, text)` | **refuses it, by name.** `\n` has no code in any standard encoding, so `encode_str` raises `Refusal { trigger: TargetAbsent, character: Some('\n') }` and the whole add fails |
| **boxed** — `…with_box(x, y, w, h)` | splits on it: each `\n` is a hard paragraph break, each paragraph is wrapped independently to the box's width, top-anchored from the box's top edge |

⇒ **A multi-line add MUST be boxed.** That is not a preference, and it is
why dragging a rectangle was the multi-line gesture in the first place: a
drag says how wide.

## Where the width of a CLICKED multi-line draft comes from

The operator: *"can the enter key create new lines when we are editing or
creating text?"* Enter inserts a line break at a **clicked** caret too — and
a click has no extent, so this arm meets a multi-line draft carrying no
width of its own.

The shell's own standing rule, from `canvas::textedit::place`'s header, is
that a width may not be **invented**: *"a click would have to invent a
width."* That rule is kept. The width is not invented — it is **read off the
operator's own page**:

```text
left   = where they clicked
right  = the crop box's right edge
top    = where they clicked          (so the first line starts at the caret)
bottom = the crop box's bottom edge
```

Every number is a fact about their document. Nothing here chooses a margin,
a column width or a default; the sheet does.

The promotion is **conditional**. A single-line point add is still a point
add, byte for byte, taking the path it would take if this rule did not
exist — which is what keeps the common case unchanged and makes the rule
impossible to regress into. See [`request`]'s three-way match.

## The promotion is DISCLOSED

A rectangle that is not drawn is a rectangle the operator cannot see, and
where a long line breaks depends on it. `crate::text::textedit::point_text_became_a_block`
is the sentence, shown once at the commit and not while typing, and it names
the gesture that puts the width back in their hands.

It rides the ordinary disclosure list — the same `Vec<String>` the engine's
own disclosures travel in — rather than getting a channel of its own,
because an edit *did* happen and this is the part of it they cannot see.
That is precisely what `⚑ About your last edit:` is for, and it is the
distinction O127's other two defects are both on the wrong side of.

## Item notes

### `fn select_what_was_authored`

The operator: *"the font selector and editing tools like bold
and italic ... that entire area is always greyed out in the menu, and the
properties area is uneditable too. **This is true even when I add a new line
of text.**"*

That last sentence is the one that rules out every "CAD text is hard"
explanation, and the cause is here rather than anywhere near a font. An arm
that authors an object and selects **nothing** leaves
`selection.formattable` unpublished, so the contextual Format tab does not
appear at all — the operator has just typed a line of text and the ribbon
has no Font group on it to be greyed.

# The same convention `InsertImage` follows, for the same reason

`super::apply`'s image arm carries the long-form argument. The operator:
*"if I add an image I expect to click on it to resize but dragging doesn't
resize"* — which was never about resizing: the image arrived unselected, so
the first press landed on unselected paper and `gesture::meaning` read it as
a marquee. Every one of the eight applications surveyed leaves a newly
placed object selected; text is not an exception to that.

# Why the LAST object, and why the model is rebuilt rather than counted

`add_text` appends one `BT` ... `ET` to the page's content, so the authored
object is last in paint order and therefore the decomposition's final index.
`AddTextReport` carries no object index, so there is nothing better to read;
see the hand-off in `D:/Dev/FeatureRequests/pdfce_FeatureRequests/` if that
ever changes.

The count is taken from the model rebuilt AFTER the edit, never from a
count kept before it: the edit invalidated the cache, `page_objects()`
rebuilds against the new epoch, and a remembered count would be a count of
the page as it was. The `Ref` is dropped in the same statement that reads
the length, because `select_placed` wants `&mut doc.selection` and holding
the borrow across it does not compile — the borrow checker enforcing the
short-borrow discipline `app::cache`'s docs ask for.

# The two ways this declines, and why each is silence rather than a guess

- **The page will not decompose.** `page_objects()` answers `None` and the
  selection is left alone. The text is on the page; what is missing is the
  shell's ability to NAME it, and inventing an index would put handles round
  whatever sits at that position.
- **The authored page is not the viewed page.** `page_objects()` builds for
  `doc.view.page_index` and for nothing else, so an index taken from it is
  only meaningful for that sheet. A caret is always on the viewed page
  today, so this is a guard against a future gesture rather than a live
  case — but it is the kind of mismatch that selects a plausible wrong
  object rather than failing, which is why it is checked rather than
  assumed.

### `fn a_single_line_click_is_not_promoted`

The commonest gesture in the program — click, type a label, click away —
must reach the engine as a point add. A build that boxed every add would
wrap a short label at a width nobody chose, and would do it silently.

### `fn a_multi_line_click_is_boxed_from_the_click_to_the_sheet_edges`

O127's defect 2 at the point where it meets the engine: `\n` in a point
add is a **named refusal** — `\n` has no code in any standard encoding —
so without this the operator's second line loses the whole add, with an
error about a character they cannot see.

Every number in the assertion is the operator's or the page's. That is
the property being pinned: no margin, no default, nothing chosen here.

### `fn the_hard_newline_reaches_the_engine_intact`

The fact the whole feature rests on: `with_box` splits on `\n` and wraps
each paragraph independently. A build that joined the lines with a space
on the way here would author plausible, wrong, single-line text — which
is the failure mode O127 named in advance (*"rather than silently
joining with spaces"*).

### `fn a_dragged_box_reaches_the_engine_as_origin_and_extent`

The transposition that compiles. The action carries corners, because
that is what a dragged rectangle is; `with_box` takes `(x, y, w, h)`.
Getting it wrong puts the text somewhere plausible and wrong, on a page
nobody will look at again until it is printed.

### `fn no_cropbox_means_no_invented_box`

The engine then refuses by page index, in its own words, which is a
sentence about the real problem. A guessed cropbox would turn it into a
sentence about a rectangle this shell made up.

### `fn a_click_outside_the_sheet_falls_back_to_a_point`

A zero- or negative-width rectangle is a request the engine would refuse
for a reason unrelated to what the operator did. Reaching for a minimum
width would be inventing the one number this module refuses to invent.

### `fn every_route_carries_the_pen`

Two branches each building a request would be two places for a font to
be forgotten. There are three exits, so the property is asserted across
all of them rather than argued for in a comment.
