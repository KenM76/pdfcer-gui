# `ui-verify/checks/navigate_commits_text`

`navigate_commits_text` — **arming another tool WRITES the draft**, so the
typing is on the page before the operator looks for it.

# The request

> *"when I add or edit text then click to another navigation tool my
> changes don't show. If I click elsewhere on the page it works and shows me
> my changes."*

`OPERATOR_REQUESTS.md` O222.

# ★★★ What the operator is actually reporting

Not a rendering bug. The text was never committed. A canvas draft lives in
`egui::Memory` and is drawn by the caret layer, not by the page; the only
thing that turned it into document content was the *next click on the
canvas*, because that is where the commit was written. An operator who
typed and then reached for the hand tool had a draft, a caret that was no
longer armed, and nothing anywhere that would write it — so the sheet showed
the text the document had, which is the correct rendering of a document that
does not have it yet.

The reading matters because it names what the fix had to be. "Redraw after a
tool change" would have been a second rendering path for provisional
content, which is the thing `R8b` forbids. The draft has to **land**.

# ★★ The oracle, and why the tool line is not enough on its own

Two lines, and each is worthless without the other.

`canvas-tool armed=Hand` says the rail click armed something — which is a
precondition, not the subject. An armed canvas and an un-armed one are the
same screenshot, so without this line a failure below cannot be told from a
click that missed the rail entirely, and the check would report the defect
it exists to find while measuring a mis-aimed pointer.

`add-text` is the engine's own line for characters reaching the page, and it
is the assertion. The behaviour being replaced produces the tool change and
**no commit at all** — that is precisely the operator's sentence — so a
check that watched only the tool would pass on the broken build.

Counted before the gesture rather than asserted absolutely, for the reason
[`crate::checks::escape_commits_text`] states: a phase inserted above would
silently convert an absolute test into one that passes on a build where
arming a tool commits nothing.

# The phases

| Phase | Does | Expected |
|---|---|---|
| A | Edit mode, Edit tab, click **Add text** | `text-edit-tool tool=TextEdit(Add)` |
| B | click blank paper | `text-edit-caret kind=Add` |
| C | type two real characters | `text-edit-typing … len>0` |
| D | **click the rail's Hand tool** | `canvas-tool armed=Hand` *and* `add-text` |
| E | `Ctrl+Z` | `undo-applied` — the typing is recoverable, not merely gone |

Phase E is not a bonus arm. Committing on a tool change is the eager
reading of the operator's gesture, and eager is only defensible while it is
undoable; a build that wrote the draft and could not take it back would have
satisfied phase D completely and still be a build that costs him work.

# Why the rail and not the ribbon

Because the rail is the surface his sentence describes — it is on screen in
every mode, needs no tab change, and is where a hand tool is reached from
while editing. Driving the ribbon instead would insert a tab switch between
the typing and the tool change, and a tab switch is a second event that
could plausibly be what committed the draft.

# What it cannot see

* **Whether the committed text is what was typed.** Phase D asserts the
  commit reached the engine, not its content — the same scope
  [`crate::checks::escape_commits_text`] and [`crate::checks::add_text`]
  take.
* **The other three navigation tools.** Select, Points and Text reach the
  canvas through the same single statement in the frame order, and the
  commit is not conditioned on which tool was armed; this drives Hand.
* **A draft orphaned by a document-tab switch.** A canvas draft is context
  global, so switching documents mid-draft remains unaddressed and
  unasserted.
* **The Edit-mode caret only.** A form field's editor carries text too and
  settles by its own path; `checks::form_field` drives that one.
