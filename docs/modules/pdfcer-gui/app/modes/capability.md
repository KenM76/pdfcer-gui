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
