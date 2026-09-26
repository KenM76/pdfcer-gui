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
