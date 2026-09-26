# text — the operator-visible string catalog

**Every string a human can read in this application is defined here and
nowhere else.** That is a standing convention carried across from the
old crate (`ui_text.rs`, 7,912 lines and 1,193 entries), and it is
enforced mechanically rather than by review: a CI gate scans the module
tree for string literals outside the catalog and fails the build.

## Why a catalog rather than literals at the call site

Three reasons, in order of weight:

1. **Copy is a design surface with its own quality bar.** pdfcer's error
   prose distinguishes "your file is damaged" from "pdfcer is not
   finished yet" from "this page would not draw", and it does that
   consistently because all three sentences are visible in one file
   next to each other. Scattered literals drift into three different
   voices within a month.
2. **Translation, when it comes, is a mechanical job or an impossible
   one.** Which it is was decided the day the first literal was written.
3. **It makes "no placeholders" checkable.** A label that says `TODO`
   or `Panel` is visible in the catalog in a way it never is inline.

## Why this is a directory, not a file

The old catalog broke the project's 1,500-line ceiling by a factor of
five. It is split by AREA here from the first commit — `mod.rs` holds
shell-wide strings, and each future surface (ribbon, panels, dialogs,
tools) gets a sibling module — so the split never has to be done as a
migration. At S0 there is exactly one area, which is why `mod.rs` is
currently the whole catalog.

## Conventions

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.** A label is a name; a message is a statement.
- **Name the thing that went wrong and what the operator can do.**
  "Failed to open" is not a message; it is a shrug with a capital F.
- **Never apologise, never blame the file without evidence.** The three
  open-failure functions below exist precisely so the shell does not
  have to guess which of "your document is broken" and "pdfcer cannot do
  this yet" is true — `pdfcer-core` returns structured errors that say.

## Areas

| Module | Surface |
|---|---|
| `mod.rs` (this file) | shell-wide strings: window title, canvas states, the three open failures |
| [`about`] | the About dialog — the product line, pdfcer's own licence, and the **attribution catalog** for every third-party work the binary redistributes |
| [`ribbon`] | the ribbon's *structural* strings — tab labels, the one-line question each tab answers, group captions, mode labels |
| [`commands`] | the label and tooltip of every ribbon command |
| [`files`] | the open/close/recent surface: the file dialog's title and filters, and everything the Recent control draws |
| [`find`] | the Find bar — the field, the step buttons, the position readout, the four search options, and the status bar's Find toggle |
| [`markup`] | the Markup ▸ Style group's three tooltips and its unit — the only place a swatch and a number can say what they are |
| [`menus`] | the copy a **context menu** owns rather than borrows. Empty by construction — a menu row's words are its command's — and its header is the argument for why |
| [`ocr`] | the Recognise-text dialog and the Find bar's offer. The catalog with the hardest job in the crate: it has to disclose that **every word OCR produces is a guess and this engine scores none of them**, without ever implying a mark on the page |
| [`pages`] | the Pages panel — the page counts, the tile tooltip, the **four sentences an undrawn thumbnail can say**, and the preview control that stops the grid |
| [`panels`] | every string the dock's panel bodies show — Bookmarks, Layers, Signatures, Fonts, Objects, Properties |
| [`print`] | the print dialog — three tabs, the preview, the device refusals, and the commit button whose label carries the clip count |
| [`redact`] | marking, review, and the apply report. **The strictest wording rules in the catalog** — the one surface where a comfortable sentence is a security defect, and the only one entitled to the word *verified* |
| [`rotating`] | the ninth handle — four refusals and two disclosures, including the one the engine commissioned by name about a dimension's axis lock |
| [`status`] | the status bar — the render-notes disclosure, the fit/zoom mirrors, and the editable page box |
| [`window`] | **the way back out of read mode** — the exit statement the window title and the status bar both carry. The one catalog whose entries take a chord as a parameter, because a key it spelled itself would be worse than silence |

The split between `ribbon` and `commands` follows the seam in the data
itself: `crate::shell::manifest` consumes [`ribbon`] and
`crate::shell::commands` consumes [`commands`], so a change to one file
has one reviewer and one consumer. [`panels`] follows the same rule one
surface over — `crate::panels` is its sole consumer — and is itself a
directory, because six panel bodies' worth of copy is more than one file
should hold and the 1,500-line ceiling is not raised for catalogs.

## Item notes

### `fn the_three_open_failures_read_differently`

Not a tautology test: the whole value of the three-way distinction
is that an operator can tell from the words alone which of "my file
is broken", "pdfcer is not finished" and "I need to type a password"
is true. Three functions that produced near-identical prose would
satisfy the type system and defeat the design.

### `fn the_engine_keeps_its_panic_text_out_of_the_message`

This is a tripwire on `pdfcer-render`, not on this crate, and that is
the point. Until `Pass 296.5` (`4f6f5a5`), `RasterizerLimit`'s
`Display` carried `tiny-skia`'s panic text and the shell's named arm in
`crate::render::worker` was the only thing keeping *"range start index
442613758592 out of range for slice of length 1088737"* off a site
plan. The engine took it out, and the comment on that arm now states
plainly that **the wildcard arm below it would no longer leak
anything**.

That is a claim about somebody else's source, on a pin that moves
several times a day. A comment asserting it is worth nothing; this
constructs the variant and reads what the engine actually renders. If
the panic text ever returns to the format string, the sentence on that
arm becomes false and this goes red the same hour.

It deliberately does NOT assert the whole string. The wording is the
engine's to choose; what was promised is that the field does not appear
in it, and that the scale — the actionable half — still does.

### `fn the_zoom_refusal_tells_him_what_to_do_and_that_nothing_broke`

[`canvas_zoom_past_rasterizer`] outlived the leak it was written to
stop, and the doc on it explains why: the engine reports *what the
renderer could not do*, and an operator staring at a blank drawing
needs *what to do next* plus the assurance that nothing was damaged.

A future edit that "simplifies" this back to the engine's wording would
compile, pass every gate, and quietly delete the reason the function is
still here. So the three clauses are asserted, and the engine's phrasing
is asserted absent.

### `fn a_path_without_a_file_name_still_names_something`

`Path::file_name` returns `None` for a bare root or a path ending in
`..`, and an unwrap there would turn a nonsense command line into a
panic instead of a message.
