# `ui-verify/checks/colour_clicked_text`

`clicking_text_offers_its_colour` — **O89's object route, driven.**


This header carried a three-warning banner saying **"THIS CHECK HAS NOT BEEN
RUN"** from 2026-09-05 to 2026-09-14: it was written in a session whose
instructions forbade launching the GUI, so no line below had been observed
against a running binary. That is no longer true and the banner is deleted
rather than softened, because a warning that has stopped applying is read as
a warning that still does.

What the first run produced, and it is worth keeping because two of the
three findings were about this check rather than about the program:

1. **A real defect.** The Properties panel opened on the three *When you
   resize something* switches, which draw whenever the Select tool is armed
   and therefore nearly always. The colour swatch was at y 783-807 in a
   viewport ending at 766. `panels::properties::tool::Slot` is the fix, and
   `OPERATOR_REQUESTS.md` O198 is the operator's report of it.
2. **A wrong sentence in this file**, corrected below: the failure message
   said the colour row had *"grown a fourth arm that draws nothing"*. It had
   not. It had drawn, off the bottom of its clip.
3. **A mechanism that existed and was not called.** `driving::clipped_away`
   was added on 2026-09-12 after the identical confusion in `restyle_text`.
   This branch did not use it, so the fix did not carry.

# The defect

`OPERATOR_REQUESTS.md` **O89**, in his words:

> *"I don't see where I am able to edit the color of text, vectors, etc."*

Text colour had shipped in two places and **both were gated on a swept
range**. Clicking a piece of text with the Select tool selects the *object*,
so both controls stayed greyed and the way to un-grey them — press `T`, arm
the Text tool, sweep across the words — is not guessable from anything on
screen. The capability was there; the route was not.

`panels::properties::textobject` is the route: a **working colour control on
the clicked object**, whose operand is derived from the object's own
`BT`…`ET` byte span rather than from any guess at geometry.

# Why this check is not a subset of `font_group`

`font_group`'s phase 1 asserts that the panel drew the **face row** for a
clicked text object. This check asserts, in the same state, that the
**colour** row drew as well, that its control OPENS, and that a pick reaches
the engine. A build that drew the face row and lost the colour control
passes `font_group` completely and is the exact program the operator
complained about.


# The oracle, in the order it is read

| # | assertion | what its absence means |
|---|---|---|
| 0 | the click left **one text object** selected | the harness aimed wrong — **SKIP**, never fail |
| 1 | `properties.textobject` drew | the section is gone, or returned before drawing |
| 2 | `properties.textobject.swatch` **or** `properties.textobject.ink` drew | the colour row drew neither a control nor its refusal |
| 3 | the four `properties.text.*` style rows drew | the clicked object did not resolve to an operand, so the other four controls are missing |
| 4 | clicking the swatch opened `properties.textobject.swatch.picker` | the control is drawn and inert — this project's founding defect |
| 5 | a pick then a close traced `text-style-applied` **and** `format-text` | the gesture decided to act and the action never reached the engine |

**4 and 5 are two assertions and not one**, for the reason
`restyle_text`'s own header gives about its pair: a control that opens a
picker and never commits, and a control that never opens, are different
defects with different fixes, and one message covering both would name
neither.

**Step 2 is a disjunction on purpose, and it is not a weakened
assertion.** Which of the two draws is a fact about the *fixture*, not about
the program: text painted in CMYK or a spot colour must get the sentence and
no swatch, and text in RGB or Gray must get the swatch. Requiring the swatch
unconditionally would make this check FAIL on a correctly-behaving build over
a spot-inked drawing — which is precisely the class of false red
`RESUME.md` records four separate instances of. Which one appeared is
recorded as a note, and step 4 runs **only** when it was the swatch: there is
nothing to click when the program has correctly refused to draw a control.

# The aim

Needs a `--doc-point` on a real run of text. `RESUME.md`'s aim table gives
`D:/Dev/pdfTests/SW41177/SW41177.pdf` at `0,1140,62` for the text family,
and that is this check's calibration point: a 5 pt title-block run at PDF
(1135.7, 58.4)–(1190.5, 63.4). Anything else and step 0 skips.

# THE FALSIFICATION TABLE — what to break, and what must go red

## Item notes

### `const STYLE_ROWS`

THIS REPLACED A CONSTANT CALLED `ROUTE`, AND THE REPLACEMENT IS O198.

`ROUTE` was spelled `properties.text.route`: a sentence saying *"press T for
the Text tool and sweep across them"*, which was the only surface in the
program that told an operator how to reach the face, size, bold and italic
controls for text they had clicked. O198 (2026-09-14) removed the reason for
it -- `app::textoperand` resolves a clicked text object's byte span into the
run indices the five Font verbs take -- so the sentence was deleted and the
controls themselves are what this step now asserts.

Asserted as a LIST rather than as the section region, for the reason
`font_group`'s `FONT_ITEMS` gives: a section that draws its heading and
returns before any control publishes the section region and nothing else,
which is exactly the regression this step exists to catch.

### `const PROPERTIES_TAB`

Not optional. The dock draws only the ACTIVE tab's body, so a pane behind
another tab publishes **nothing** — indistinguishable, from here, from a
panel with nothing to say. `font_group`'s own note records the false bug
report that cost.

### `const APPLIED`

The second half of the two-line oracle. `text-style-applied` alone says a
module decided to act; this says the act landed in the document. A build
where the two disagree is exactly the shape `RESUME.md` records for
`import-form-data`, twice.

### `fn aim_verdict`

Order is *what* before *how many*. A click that lands on a path inside a
marquee of eleven is an aim problem twice over, and the kind is the half
that names the fixture coordinate to change.

### `fn aimed_at_one_text_object`

SKIPPED, never failed. A `--doc-point` that is not on text is the
harness's aim, and a harness that reports its own aim as the program's
behaviour is worse than one that reports nothing — `RESUME.md` records that
costing a day on `font_group`, about a program that was working.

### `fn wait_for_verdict`

A bounded poll rather than a fixed sleep, for `restyle_text`'s reason: a
restyle re-resolves its pin from a fresh provenance extraction **per run**,
and this route's operand is a whole text object, which can be many runs. A
fixed sleep long enough for the worst case makes every run slow, and a short
one reads the trace mid-gesture and reports *"nothing happened"* about a
gesture that is still running.

### `const ON_ONE_TEXT_OBJECT`

Written with `concat!` rather than as a multi-line literal, and it is
not style: an indented continuation line inside a `"…"` keeps its
leading spaces, `Trace::parse` strips the prefix from the **start** of
the line, and the fixture silently becomes a trace with one line in it.
That cost a red on the first run of these tests.

### `fn the_aim_read_tells_its_four_answers_apart`

The guard that stops this check reporting its own aim as the program's
behaviour, tested without a running program — which is the only way it
can be tested at all in a session forbidden to drive the GUI. If these
four collapsed to one, the SKIP messages would be interchangeable and a
reader would be sent to the wrong place.

### `fn a_missing_selection_line_is_not_a_selection_of_one`

`canvas-selection` is written through `trace_changed`, so a run with no
line is a run where the selection never changed. A guard that defaulted
to one would pass on a click that selected nothing, and every oracle
below it would then be asserting sentences about a selection that does
not exist.

# What else it covers

The Properties text section draws only when `pin::object_text` gathers the
clicked object's runs, and that gathering admits a glyph only where the
engine's `vector::text_locate::locate_text_run` places it in that object. The
check therefore also drives glyph-to-run resolution. Falsified: with
`engine_places_in` answering `false`, it fails at `no-section`.
