# `app::modes::capability` — what a mode lets the canvas do

**The one place the rule "Read does not edit the document" is written
down.** Everything else — the gesture machine, the key handler, the
context menus, the tool arming — asks this module and branches on the
answer; none of them knows what `"read"` is.

## 1. The operator's ask, and what it actually requires

> *"in read mode the document shouldn't allow editing and should allow
> only selecting of objects that acrobat reader would allow."*

`MODES_AND_PANELS.md` Part 1 had already specified this, one row of its
table:

| | **Read** | **Review** | **Edit** |
|---|---|---|---|
| **Canvas gestures** | pan, zoom, text selection for copy, follow links | + place and edit **your own** markup and dimensions | + full content selection and editing |

and its safety rule:

> **A mode changes what is *visible*. It never makes a visible control
> silently inert.**

The ribbon honours that rule — Read is shown File and View alone, so no
editing *command* is reachable. What the ribbon cannot reach is the
**canvas**, where a gesture is not a control and there is no tab to hide.
Without this module, clicking a line in Read selects it, dragging moves
it, and Delete deletes it: three edits in a mode whose entire purpose is
that it does not author anything.

## 2. Capability is derived from the mode's TABS, not from its id

The obvious implementation is `if mode == "read"`, and
[`crate::viewer::display::PageDisplay::default_for_mode`] is precedent
for it. This module deliberately does **not** do that, and the reason is
the safety rule above rather than taste.

A capability keyed on the id is a *second*, independent statement of what
a mode contains. It can disagree with the manifest, and the shape of the
disagreement is exactly the failure the rule forbids: a mode that shows
the Edit tab while the canvas refuses to select is a visible control that
is silently inert, and a mode that hides the Edit tab while the canvas
still moves objects is the defect this module was written to fix. Both
are unreachable if the canvas and the ribbon read the *same* sentence.

So the rule is:

| Capability | Granted when the mode contains |
|---|---|
| [`Capabilities::edit_content`] | the **`edit`** tab |
| [`Capabilities::author_markup`] | the **`markup`** tab |
| [`Capabilities::author_measure`] | the **`measure`** tab |

Against the built-in manifest (`crate::shell::manifest::built_in`) that
yields precisely the table in §1, and
[`tests::the_built_in_modes_match_the_specified_gesture_table`] pins it:

| mode | tabs | content | markup | measure |
|---|---|:-:|:-:|:-:|
| `read` | file, view | ✗ | ✗ | ✗ |
| `review` | file, view, pages, markup, measure | ✗ | ✓ | ✓ |
| `edit` | file, view, pages, edit, markup, measure, tools | ✓ | ✓ | ✓ |

It also means a customized manifest gets the behaviour it asked for
without this file learning its vocabulary: someone who adds the Markup
tab to Read gets markup gestures in Read, because they said so, and the
alternative — a canvas that ignores their manifest — is not a safer
product, it is a broken one.

## 3. An unknown mode gets EVERYTHING, and that is not an oversight

[`Capabilities::for_mode`] falls back to [`Capabilities::FULL`] when
there is no validated shell, no active mode, or an active mode the
manifest does not declare.

Falling back to *restricted* is the tempting choice and it is wrong
twice:

1. **It fails in the direction the safety rule forbids.** An unknown mode
   still renders whatever tabs it declares. Refusing its gestures gives
   the operator a full ribbon over a dead canvas — the `editing_enabled`
   master toggle, rebuilt by accident, which `RIBBON_IA.md` §5.4 removed
   at the operator's instruction.
2. **A mode is not a permissions system**, and `MODES_AND_PANELS.md`
   says so in those words: *"Read mode does not protect a document from
   anything; a determined operator moves the slider. It is an
   interface-complexity control."* Nothing here is load-bearing for
   safety, so there is no security argument for failing closed — and
   pretending there is would invite a future reader to rely on it.

This is the same shape of answer `default_for_mode` gives for the page
display (*"an unrecognised mode is not evidence that the operator wants a
different one"*), pointed at a different default: there, the default is
`Single`; here, the default is the canvas this shell had before modes
existed.

## 4. What is deliberately NOT gated

- **Filling a form field.** Operator decision: *Acrobat Reader fills
  forms in its default view, and replacing it is the stated goal.*
  `canvas::forms` reads no mode and must not learn to — it is the second
  surface that would have to learn about a mode gate, and it stays out.
  Filling is not authoring; it is the primary reason most form documents
  exist.
- **Pan, zoom, the hand tool, marquee *zoom*, Find, guides, rulers,
  grid.** Navigation and inspection, none of which touches the document.
  A marquee-zoom band shares its rubber band with marquee-*select* and is
  branched only at release, so the gate is on the intent rather than on
  the band — see [`content_gesture`].
- **Selecting a page in the Pages panel**, and every panel's own
  contents. A mode governs which panels *mount* (`app::modes::defaults`),
  which is a layout question this module has no part in.

## 5. Why selection and editing are one capability rather than two

[`Capabilities::edit_content`] gates the *selection* of page content as
well as the verbs that act on it, which conflates two things that are
separable in principle. It is one flag on purpose, for as long as both of
these hold:

- **Selection is the only route to the verbs.** Move, resize, delete and
  the Format tab all take the selection as their operand, so a mode that
  could select but not edit would need every verb gated *again*,
  separately — and the day one was missed, that mode would edit.
- **A selection with nothing to read it is not inspection.** Read and
  Review mount no Objects panel and no full Properties panel
  (`app::modes::defaults::spec`), so a selection there would be an
  outline on the page and nothing else.

The day canvas selection reaches **annotations** — which
`canvas::target`'s header already names as future work, and which Review
genuinely needs in order to *"edit your own markup"* — that is a
different operand space and it gets its own capability, gated on
`author_markup`. It is not this flag with a wider meaning.

## Item notes

### `const GATED_BY_THEIR_DISPATCHER`

See [`offers_command`]'s §"The one class that escapes its tab" for the whole
argument. In short: the tab gate is a *proxy* for *"may this mode do this?"*,
and where a dispatcher asks the real question — of the operand, per press —
the proxy is not merely redundant, it answers a **different** question and
gets it wrong.

A named constant rather than literals in the comparison so the class has a
name, and so a reader grepping for `edit.paste` finds the rule as well as
the registration.

**Membership is not a decoration.** Every id here must be one
`app::dispatch::clipboard` gates on `PdfcerApp::capabilities`, or this list
hands a mode a verb nothing stops.
[`tests::every_dispatcher_gated_command_is_one_the_clipboard_dispatcher_owns`]
binds the two ends mechanically rather than by this paragraph.

### `fn the_built_in_modes_match_the_specified_gesture_table`

This is the test that makes §2's claim true rather than merely
argued: the derivation is from tabs, so this asserts the *outcome*
for the three real modes. Change a mode's tab list and this fails,
which is correct — the canvas capability moved with it.

### `fn an_unknown_mode_gets_the_full_canvas`

Asserted as three separate routes to the same answer, because they
are three separate `return`s and a refactor could easily fix one and
break another.

### `fn a_command_on_no_tab_is_offered_by_every_mode`

This is the test that makes the exception list unnecessary. If any of
these ever moves onto a tab, this fails and names it — which is the
warning you want, because moving `edit.undo` onto the Edit tab would
silently take undo away from Read.

### `fn both_text_copy_commands_are_offered_by_every_mode`

> *Acrobat Reader copies text, and replacing Acrobat Reader is what Read
> is for. Copying is not authoring.*

That sentence is the whole reason the two verbs are `file.` ids rather
than `edit.` ones. It is asserted here **directly**, for every mode
including the two where it is never in doubt, because the placement is
otherwise invisible to the suite in the direction that matters: nothing
else fails if a later edit puts these two back on the Edit tab, or
invents a `clipboard` group on a tab Read does not show. The registry
count would not move, the group count would, and both are numbers a
reverting change edits on its way past.

`read_mode_refuses_exactly_these_bound_chords` covers the *chord* half
for the page-text command alone and only because a chord happens to be
bound to it; this covers **both commands**, chord or no chord, which is
the property the operator actually asked for. The document-text command
has no chord at all, so this test is the only thing standing under it.

Deliberately asserted through `offers_command` rather than by looking the
ids up on the File tab: the tab is *how* it is true today, and the
requirement is that the mode offers them however that comes about.

### `fn review_offers_every_clipboard_chord`

In the mode whose entire purpose is marking up somebody else's drawing,
an operator who can copy a comment and not paste it has nowhere to put
it. All four ids are asserted rather than paste alone, because the
failure is an **asymmetry**: a build that offered paste and left cut
behind would put the same trap one keystroke away.

# The four ids are LITERALS here, and that is the whole test

Written as `for id in GATED_BY_THEIR_DISPATCHER`, this test passes on a
build where that constant has shrunk to `["edit.copy"]`: it then asserts
"the one thing in the list is offered", which is true and worthless.

⇒ **A test that iterates the mechanism it is testing cannot fail by that
mechanism being narrowed**, which is the exact regression this test
exists to catch. The property is about *these four commands in this
mode*, so these four commands are written out; the constant is asserted
separately, below, so the two cannot drift without a named failure.

### `fn read_mode_still_refuses_the_clipboard_verbs_it_should`

The other half of the escape above, asserted at the layer that owns the
answer rather than at this one. `Capabilities::NONE` is what Read gets,
and that is what both of the dispatcher's gates read:
`edit_content` for content and a field, `author_markup` for markup and
for an empty clipboard. So every operand Read can present is refused.

It asserts the **capability**, not the gate's code, because the gate is
a match on `Clipped` that this module cannot construct without a
document. What it pins is the premise the gate rests on: if Read ever
gained either flag, this fails and names it — which is the warning worth
having, since the dispatcher would then quietly permit the paste.

### `fn every_dispatcher_gated_command_is_one_the_clipboard_dispatcher_owns`

The list in [`super::GATED_BY_THEIR_DISPATCHER`] is safe only because
each member's effect is gated somewhere else. This binds the two ends
mechanically: an id added to that list that no dispatcher claims would be
a command handed to every mode with nothing standing under it, and the
symptom would be silence rather than a failure.

### `fn the_escape_list_is_narrower_than_the_dispatchers_own`

`edit.copy_as_vector` is routed by the same dispatcher and is **not** on
the escape list, because it needs no escape: it takes no mode gate at all
(*copying is not authoring*) and it lives on the Edit tab, where its
button is absent outside Edit — visibility doing the work, which is the
rule `app::modes` states. A list that had simply been derived from
`handles` would have included it and would have been documenting nothing.

### `fn every_bound_chord_is_offered_by_the_fullest_mode`

It asserts the shape rather than a fixed list: every bound chord must
be offered by Edit, because Edit shows every tab. A binding that failed
that would be one pointing at a command on no tab of any mode, i.e. a
chord bound to something unreachable.

### `fn read_mode_refuses_exactly_these_bound_chords`

Every chord the shipped keymap binds, resolved against Read — the
mode that hides the most. Asserted as an exact set rather than a
spot-check, so that adding a binding, moving a command between tabs, or
changing a mode's tab list all fail here and print what changed.

**No text-copy id appears below**, and that absence is the whole
visible consequence of File ▸ Export owning them: `file.copy_page_text`
and `file.copy_document_text` sit on a tab every mode shows, so
`Ctrl+Shift+C` reaches Read with nothing added to `offers_command` to
achieve it. Asserting an exact set rather than a spot check is what
makes that visible here. See [`super::offers_command`]'s header, and
[`both_text_copy_commands_are_offered_by_every_mode`] for the property
that has a test of its own.

### `struct Capabilities`

Three independent facts rather than one ordered level, even though the
three built-in modes happen to form a ladder. The ladder is a property of
*that manifest*, not of the type: Review offers markup without content
editing, so the two are already independent in the shipped product, and a
customized manifest may offer any combination at all. An ordered
`enum { Read, Review, Edit }` would have to be re-derived — wrongly —
from any manifest that did.

Copied freely: three `bool`s, computed once per frame in
[`crate::app::PdfcerApp`] and passed down by value.

### `const NONE`

Not used as a fallback anywhere. It exists so a test can name the
expected value rather than spell three fields, and so the Read row of
the module header's table has a name in code.

### `fn for_mode`

The one place the derivation lives. `shell` is `Option` because
`PdfcerApp::shell` is: a build whose manifest failed to validate has
none, and that build gets [`Self::FULL`] along with every other
unknown case (module header §3).

### `fn authors_anything`

The predicate a surface asks when it wants to know "is this a reading
stance?" without caring which authoring verb it is about — the canvas
context menu uses it to decide whether it has anything to offer at
all.

### `fn content_gesture`

Free function rather than a method because it is the exact predicate the
gesture machine needs at three call sites and it reads as a sentence
there: `content_gesture(caps)`. The marquee is the interesting caller —
a *zoom* band is not a content gesture even though it is the same rubber
band, so the branch is on the release intent, not on the band.

### `fn publish_edit_content`

# Why a published value rather than a further parameter

`canvas::pressing::grabbable` decides which grips a selection offers, and
per `OPERATOR_REQUESTS.md` O71 that answer depends on the mode: a content
selection is reachable in **Read**, where every grip would commit an edit
the mode forbids. Most of its callers hold no `Capabilities`, so the
alternative is threading a boolean through call chains that have no other
interest in it.

This is the same shape `canvas::tool` uses for the armed tool and
`crate::pagedrag` for the active document, and it carries the same
obligation: **one writer**. `app::frame` publishes it once per frame before
any surface draws, so a reader cannot get last frame's answer.

### `fn edit_content_now`

`false` when nothing has been published — a unit test with a bare
`egui::Context`, or a frame before the publication. That is the safe
direction: the consequence of a wrong `false` is a selection that offers no
grips, and of a wrong `true` is eight controls whose drag is refused.
