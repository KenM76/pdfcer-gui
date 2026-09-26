# `app::conditions` — what the shell may ask about the application

One function. It publishes the set of **named conditions** the ribbon
evaluates every frame to decide whether a control is enabled and whether
it renders pressed.

## Why the vocabulary is names rather than closures

`egui_shell` stores `Enable::When("doc.pages")` — a string — and asks this
set whether it is present. Data rather than a closure, because a name is
serializable (so an operator's customized manifest can reference it),
testable headlessly, and cannot capture state that would make a command's
availability depend on *when* it was registered.

The cost of that choice lands here: every name is a **promise the
application has to keep**, and the only thing keeping the two halves in
step is `shell::commands`' `KNOWN` list and the test that walks it.

## Why this is its own file


## Two sources, one convention

Conditions come from two different places and it matters that they arrive
the same way:

* **application state** — `doc.open`, `doc.pages`, `selection.any`,
  `selection.bounds`, and the page-display and view-chrome pressed states,
  all read from `PdfcerApp` and the open document; and
* **`egui::Memory`** — the armed canvas tool and the armed region zoom,
  which is why this function takes an `egui::Context`.

The second source is the reason this function grew a parameter. Three
separate pieces of work recorded that the hand tool and the region zoom had
no pressed state, and each declined to invent a mechanism for it — rightly.
The alternative was a shadow copy of the armed tool on `PdfcerApp` that the
canvas would have to remember to update, which puts the truth in two places
and fails as a ribbon that says Hand while the canvas selects: a
disagreement no test catches, because each half is self-consistent.

## Item notes

### `fn conditions`

Rebuilt every frame because that is what it describes — the state
*this* frame is drawn from. The set is **closed**, and the vocabulary
is written down once in `crate::shell::commands`' `KNOWN` list rather
than counted here — a count in prose drifts the moment a condition is
added, and this sentence has already been wrong once for saying
"five". That module has a test asserting no predicate names anything
outside the list, so a typo in a manifest cannot silently produce a
control that is disabled forever.

# `selection.any` is published from here, and only now

It was deliberately absent while the selection lived in
`egui::Memory`: this function has no `egui::Context`, so it could not
have read the selection even if it wanted to, and publishing a
condition it could not evaluate would have armed a **destructive**
control that could not work — the inverse of the no-placeholders rule
and the exact shape of defect D1.

The selection now lives on [`state::OpenDoc`], so the answer is one
field read. Two surfaces come alive with it, both of which the manifest
has been carrying unpowered: the contextual **Format** tab
(`visible_when: "selection.any"`, which is the appear-on-selection
affordance `RIBBON_IA.md` §5.8 calls the single largest usability
change) and the **Delete** inside it (`enabled_when` the same). One
spelling, one source — see `shell::manifest::format::VISIBLE_WHEN`.

**The Objects panel's focus is not a selection and must never satisfy
this**, which is what `panels::PanelsState::focus`'s own test asserts
through the enable machinery: a panel row being focused must not arm a
destructive command, because the operator would have no way to tell
which of two "selections" it was about to act on. This reads
`doc.selection` and nothing else.
`pub(super)` rather than private: this moved out of `app/mod.rs` and
its three callers stayed. Deliberately NOT `pub` — nothing outside
`app` may publish or read the condition set, because a second producer
is how a control comes to be enabled by one rule and drawn by another.
