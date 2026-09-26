# `refused_character_face` — **a refused character offers the face that can
type it**, driven end to end


> *"if the character isn't available in a pdf are we able to change to a
> different font?"*

**Yes**, and on the day he asked it every piece already existed — the engine
refuses by name and hands back `Refusal::character`, the face chooser has
offered the fourteen standard faces since `Pass 162.0`, and `set_font`
authors a resource the page does not carry. **Nothing connected the refusal
to the chooser**, so the answer to his question lived in the last clause of
an error message he never saw. This check drives the connection.

## HOW TO FALSIFY IT — read this before trusting a green run

[`crate::checks`]' founding rule: *a check that has only ever been seen to
pass is indistinguishable from one that cannot fail.* Seven separate
mutations must each turn it red, and each names the link it cuts:

| mutate | expected verdict |
|---|---|
| delete the `refusedchar::record(..)` call in `app::status::decline::textedit` | **FAIL** — *"THE REFUSAL LEADS NOWHERE"*: the `⊗` slot still draws and no `properties.refusedchar` region follows the refusal |
| make `missing_character` return `None` unconditionally | **FAIL** — same region, plus `said=UnsupportedFont` on the classification line where `FontLacksTheCharacter` is the only reading of a refusal that named a character |
| drop the character from `refusedchar::section`'s trace line | **FAIL** — *"THE OFFER DOES NOT NAME THE CHARACTER"*: the block drew and cannot say what it drew about, which is the whole distinction between this and a generic decline |
| delete the `face_addable_disclosure` label from `refusedchar::section` | **FAIL** — *"…AND THE DISCLOSURE IS NOT ON SCREEN"*, rule 4's half |
| stop `canvas::textedit::plan` writing `LAST_COMMIT`, or drop the `Action::CommitTextEdit` push from `refusedchar::section` | **FAIL** — *"THE OFFER DOES NOT FINISH THE JOB"*: the face swaps and no commit of any kind follows, although the harness pressed nothing |
| send the restyle to the current selection instead of `runs: vec![refused.run]` | **FAIL** — *"THE FACE SWAP DID NOT REACH THE RUN THE REFUSAL NAMED"*: the retype comes back on the **same** subset floor |
| make `RefusedCharUi::advance` return `true` unconditionally, or freeze `retried` at `false` | **FAIL** — the control point at step 0b, and the state walk at step 7 |

And two mutations that must **NOT** turn it red, because a check that
fires on them is asserting a limitation rather than a capability: swapping
which of the fourteen is first in `Std14::ALL` (the check clicks the first
addable row, whatever it is), and rewording any sentence in
`crate::text::panels::face` (the harness reads regions and one trace field,
never rendered prose — `check-ui-strings.sh` and the unit tests own the
words).

## THE DYNAMIC RANGE, and why it is two assertions rather than one

The oracle for most of this file is *"a region was published"*, and **a probe
whose baseline has no dynamic range cannot produce a verdict.** A build that
offered the chooser after every edit — because the block is drawn
unconditionally, because the state never retires, because a stale refusal
from ten minutes ago is still in the slot — would satisfy a one-sided check
*permanently*. This project filed three such false reports in a single day.

Two assertions hold the two sides:

* **step 0b** — on a freshly opened document that nothing has been done to,
  the block must be **absent**. That is what a build drawing it
  unconditionally fails.
* **step 7** — the block must be seen to **walk its own state machine**:
  `offer` → `swapped` on its own trace line, two states each caused by a
  different thing happening to the document. A block frozen on one state
  cannot produce that sequence.


The stronger classical control applies on top of it and is asserted in
step 6: the offer must be **gone** after a commit that succeeded, and its
absence is made non-vacuous by first proving frames were painted after that
commit.

## The states it walks, all measured with `pdfcer.exe` first

| # | gesture | what must happen |
|---|---|---|
| 0 | nothing yet | `properties.refusedchar` is **absent** — the control point |
| 1 | seed `q`, Ctrl+Enter | the engine refuses; the classification says `FontLacksTheCharacter`; the offer draws and **names `q`** |
| 2 | — | the disclosure is on screen and **does not overlap the page** (rule 4) |
| 3 | open the chooser, click the first addable row | `format-text` reaches the document |
| 4 | **nothing** | the block re-applies the operator's own edit by itself — a commit follows with no further input |
| 5 | — | that commit **lands**: the character goes in, in one gesture, and the block retires |


**This is the most important paragraph in the file**, because it is the one
that records a check being tightened rather than a program being fixed, and
the two are easy to confuse when reading a diff.

Until the engine bump to v0.41.0 the last step of this route was blocked by
a defect in `pdfcer-core`, and that was **measured rather than inferred**.
`pdfcer-gui`'s `canvas::textedit::facewall` held three experiments:

1. **one** `EditSession`, `format_text` then `edit_text`, located by find
   text alone so **no operand this shell computes is in the request** —
   refused;
2. the identical pair with a **save and reopen** between them — succeeded,
   and the character was in the extracted text;
3. a swap to a face the page **already carries** — succeeded at once.

So the trigger was the `/Font` object `format_text` creates: it lived in the
session's overlay, and `edit_text` planned with `plan_edit(&self.base, …)`,
so the name in the rewritten stream resolved to nothing. Filed as
`request_edit_text_resolves_font_names_against_the_base_revision.md`, and
while it stood this check accepted the refusal **provided the block said so**
— which asserted the shell's half of a route whose other half was owed.

**`Pass 257.0` paid it** (engine `5e95805`, released in v0.41.0). Every
text-edit planner and helper takes `&DocumentView<'_>`, every `EditSession`
verb passes `self.view()`, and with no `&Document → &DocumentView` coercion
the class is a **compile error** rather than a latent refusal. `facewall`'s
first test went red on the first run after the pin moved, from its own
`expect_err` message, and now asserts the SUCCESS in both request shapes.

⇒ **So the refusal branch is gone.** Keeping it would leave this check unable
to fail on the exact regression it is now the only instrument for: a build
where the engine's fix is present and the SHELL stops finishing the route
would take the refusal branch, print a confident note naming an engine defect
that no longer exists, and pass. **A check that accepts the outcome it exists
to forbid is not a check.**

What it will **not** accept, and each fails by name and separately, because
the three send a reader to different files:

* a second **embedded-subset** refusal — the swap never reached the run the
  refusal named, which is the shell defect this file was written to catch;
* **any other** refusal — the character did not go in; the failure message
  carries the one command that decides which repository to open
  (`cargo test -p pdfcer-gui --lib canvas::textedit::facewall`: red means the
  engine regressed, green means the shell is handing it something stale);
* a state sequence ending in **`blocked`** — the retype came back refused.

`state=blocked` and `text::panels::face::refused_char_blocked` were
**deliberately not deleted** when the engine shipped, and `facewall`'s header
records why the instruction to delete them was wrong: that arm is reached by
arithmetic (*the retype was raised and `doc.edit_epoch` did not move*), which
is agnostic about the cause, so deleting it would convert every other cause
into silence. It must not occur on **this** route on **this** fixture — which
is asserted — but it is still the shell's voice when something else refuses.

## Why `q` and not `€`

Because it is the operator's own example, and because it is the fact O141
says is worth seeing: *"it is narrower than 'no accents and no symbols' …
that font will not take a `€`, a `%`, an `@` — **or a plain lowercase `q`**,
because no word in those headings has one in it. Nothing about the font is
foreign or exotic. Whatever was not printed is simply not in the file."*

It is also the safer instrument: the character travels to the application in
an environment variable (`PDFCER_DIAG_TYPE`) and comes back through a trace
line, and an ASCII character cannot be lost to a codepage on either leg. The
non-ASCII half is covered where it belongs — by
`text::panels::face`'s `every_sentence_in_the_offer_names_the_character_itself`,
which asserts on `€` precisely so a build that formatted it as `\u{20ac}`
goes red.

## The fixture is PINNED and any `--pdf` is ignored

`fixtures/subset-font-floor.pdf`, aimed at **(115.2, 612.0) on page 1**.
`fixtures/subset-font-floor.PROVENANCE.md` carries the four measured engine
runs and the reason no other fixture in this repository can carry the case:
every one of them is either a non-embedded standard-14 face (whose
`WinAnsiEncoding` accepts the character and the edit simply works), a fully
embedded non-subset face (the `class.embedded && class.subset` floor never
fires), or a symbolic built-in-cmap face that refuses **every** edit for an
unrelated reason and has no remedy to offer. A check driven against any of
those would be measuring something else and would be **unable to fail**.

⚠ **And the operator's own file cannot be used**, which is worth knowing
before somebody re-points this check at it. `apartment work - signed.pdf`
raises the identical refusal — `this font has no glyph for '€'` on an
8,640-byte `AAAAAA+Arimo-Bold` subset — and its page 2 is written **one show
operator per glyph**, so the caret refuses before the font question is ever
asked (O140). The engine's command line reaches it because it searches the
whole page; a click cannot. So the fixture is the floor and his file is the
subject, and any claim made here should be re-measured against his document
with `pdfcer.exe` before it is repeated to him.

## What this deliberately does NOT assert

**Which** of the fourteen faces was applied. `text-style-applied` carries
`change=face` and not the selector, and adding the face name to a diagnostic
to satisfy a check would be the harness dictating a trace's contents —
`std14_face`'s own ruling, and it binds here for the same reason. The row
clicked is the *first* addable one on a fixture whose page fonts are known,
which is deterministic enough, and `format-text` landing is the claim that
matters because it is the one that says a `/Font` object was written into the
operator's document.

It also does not assert **what the block says**. The harness cannot read
rendered text — there is no accessibility reader and no OCR — so it asserts
that the block drew, and that the character it drew about is the one that was
refused. The wording is held by unit tests in `crate::text::panels::face` and
by `check-ui-strings.sh`.
