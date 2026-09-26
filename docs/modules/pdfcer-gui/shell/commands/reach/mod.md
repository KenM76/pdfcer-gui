# `shell::commands::reach` — the sixth obligation: a registered command
must be **reachable**

Registering a command creates five obligations, and every one of them fails
loudly: a count assertion, a group assertion, a `PLANNED` disjointness
test, a RON round-trip, a `KNOWN` lookup. **None of the five asks whether
the command does anything**, and that is the gap this module closes.

## What went wrong, and how many surfaces agreed with it

`file.save_copy` was registered, drawn on the **quick-access toolbar**,
bound to `Ctrl+S`, listed in the shortcuts reference, and printed
"(Ctrl+S)" in its own tooltip — with **no dispatch arm**. Nothing this
shell built could be written to disk, for the whole life of the project,
and it was within an hour of being released that way. An audit the same
day found the identical shape in `edit.undo`/`edit.redo` (QAT, three
chords) and in **every** page operation, six of which the Pages panel's
context menu offered while `panels/pages/select.rs` maintained a
multi-select model to feed them.

Five surfaces promise a command works — the registry, the ribbon, the
QAT, the keymap, the tooltip — and **not one of them is a dispatch arm.**
The only honest signal is a `command-unimplemented` line in the trace, and
nothing read it.

## The property asserted here

> Every command in the registry is either named by a **literal arm** of
> `PdfcerApp::dispatch_command`'s `match`, or claimed by one of its
> **guard arms**, or listed in [`SCAFFOLDED`] with a written reason.

It is a statement about **routing**, not about behaviour. An arm that
declines — `measure.finish` refusing because the mode cannot author
dimensions — is reachable, and correctly so: the operator's press produced
a decision instead of falling through to `command-unimplemented`. What this
catches is the *absence of a decision*.

# Why the arms are READ from the source rather than run

Three mechanisms were available. The two that lost are worth recording,
because each lost for a reason a future session would otherwise re-derive.

## Rejected — dispatch every command in a test and assert it was handled

The truest signal, and unavailable. `crate::app::files`' header states the
rule in its own words as **Rule 3: no test may dispatch `file.open`** —
"on the machine this is built on, dispatching `file.open` opens a **real
modal dialog** and blocks until a human dismisses it. A `cargo test` that
did that would hang the suite with an invisible window behind the
terminal."

The escape hatch does not open either. `PDFCER_DIAG_OPEN_PATH` answers the
dialog without a human, but setting it needs `std::env::set_var`, which is
`unsafe` in edition 2024 while this crate is `#![forbid(unsafe_code)]` —
the same wall that leaves `files::from_env`'s environment read as its one
untested millimetre.

So a dispatching test would have to **exempt `file.open`**, and that is
decisive rather than inconvenient: `file.open` is the command this defect
struck with the largest blast radius — registered, on the File tab, on the
QAT, bound to `Ctrl+O`, and armless, so the only way to open a document was
`argv`. A check that must skip the worst historical instance of the defect
it exists to prevent is not a check.

(The other hazards are real but were *not* the deciding ones, and it is
worth saying which, so nobody re-opens this on the wrong grounds. Almost
every arm is safe in a test, because of this shell's first invariant:
**actions, not mutations.** `pages.delete` pushes
`Action::DeletePages` into a `Vec` the caller owns and deletes nothing;
`file.ocr` sets a dialog-open flag and starts no recogniser. The genuinely
effectful arms are the clipboard writes and the native picker, and of those
only the picker cannot be reached in a state where it does nothing.)

## Rejected — a `bash` gate that greps `dispatch.rs`

Honest for a gate, and wrong for this file, for two independent reasons.

**A `match` is not a regular language.** The failure that matters is a
*false pass*, and a grep for `"some.id" =>` has at least three ways to
produce one: a string inside a comment, the left-hand side of a **nested**
`match` inside an arm's body (`dispatch_command` contains four), and a doc
comment quoting an id whose arm has since been deleted — which is the D5
shape exactly, a list that agrees with itself about something that stopped
being true. `check-ui-strings.sh`'s header records the same class of error
from the other side: its first regex read `"svg" | "?xml"` as one literal
containing `" | "`, and "three of the four remaining hits… were exactly
that artefact — i.e. most of what was left after the real exclusions was
the detector misreading Rust, not the code violating the rule."

**The guard arms are expressions, and a shell cannot evaluate them.** Six
arms have the shape `id if …_for_command(id).is_some()`, and the functions
behind them search enum tables in three other modules. A gate that decided
which ids they claim would have to re-derive `markup_command`,
`measure_command`, `chrome_command`, `page_display_command`,
`text_mark_command` and `Panel::command_id` from source — six more `match`
blocks to parse, and, worse, a **second table** of exactly the kind
[`super::mapping`]'s header exists to forbid: *"two hand-written tables can
disagree, and one table plus a derived search cannot."*

## Rejected — restructure so one `fn arm_for(id) -> Option<Arm>` is the
source of truth

The best signal and the most invasive, and it contradicts the file it would
restructure. `app::dispatch`'s header states the property being protected:
**"the arms route; they do not compute"**, each arm one line that pushes an
`Action` or calls the one function that owns the rule. Interposing a table
turns every arm into two lookups — a variant, then a body — and the thing
a reader currently gets for free, that `"file.close" => actions.push(
Action::Close)` is the whole story, is precisely what would be lost. The
brief for this work says so in the same words: it must not turn one
readable `match` into a table nobody can follow.

# What is done instead, and why it is not the rejected grep

**The literal arms are read from the abstract syntax tree**, with `syn` —
a real Rust parser, already in `D:\Dev\pdfcer`'s lockfile, taken as a
dev-dependency on the standard `flate2` and `rfd` are held to (see this
crate's `Cargo.toml`). Every objection above is an objection to treating
Rust as text, and none of them survives parsing it as Rust: an arm pattern
is an `Arm`'s `pat`, a comment is not in the tree at all, and a nested
`match` inside an arm's **body** is never visited, because only the arms of
the one `match` on `id` are read. `the_reader_does_not_see_a_nested_match`
pins that last one against a fixture, since it is the case a grep gets
wrong and the case nobody would notice.

**The guard arms are not parsed. They are CALLED.** The tree says *which*
functions guard arms consult ([`Arms::guards`], the last path segment of
whatever is invoked with `id`); Rust then answers, for every registered id,
whether any of them claims it — by running the real
[`super::markup_for_command`] and its five siblings against the real
registry. There is no second table, because there is no table: the one
`match` in [`super::mapping`] is the only statement of each mapping, and
this reads it by executing it.

The two halves are then held together by
[`tests::the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has`],
which asserts that the guard names found **in the source** and the guard
names this module **evaluates** are the same set. That closes the hole a
hand-kept list would otherwise open in both directions: a *seventh* guard
arm added to `dispatch.rs` fails by name rather than silently reporting its
whole family unreachable, and a guard arm *deleted* from `dispatch.rs`
stops making its family reachable rather than being vouched for by a
function that still exists.

# Why this is a test and not a `tools/gates/` script

Because both halves of the answer are Rust. The registry is built by
[`super::register`], the guards are functions with enum tables behind them,
and a shell script could reach neither without re-deriving both — which is
the failure the paragraphs above are about. What a gate script contributes
that a test does not is a *precondition* guarantee, and this has a stronger
one than any script can offer: [`DISPATCH_SRC`] is an `include_str!`, so a
dispatcher that has been moved, renamed or deleted **fails to compile**
rather than being quietly scanned as an empty tree. `run-all.sh`'s
three-state model exists because "found nothing" and "looked at nothing"
print the same thing; here the second state cannot be reached.

# What the register holds, and what empties it

[`SCAFFOLDED`] is the allow-list of registered commands with no dispatch
arm, each carrying the reason it is inert. **It is empty**, and
[`tests::the_p3_tension_is_counted`] pins both its length and the `P3`
subset at zero. A `P3` mark is this module's judgement that the honest
answer is *the control should not be drawn yet* — `RIBBON_IA.md` P3: "An
unavailable capability renders nothing, not a disabled stub".

An entry leaves this list two ways, and only two: the command is **wired**,
or the command is **unregistered**, which is R9's answer — a capability that
is not built renders nothing — and is legitimate because of R8: *registering
a command is the only way the GUI may learn that a capability exists.* An
entry is never **reworded** when its reason expires;
`no_scaffolded_entry_is_stale`'s middle assertion exists to force that,
because a reason rewritten after a blocker clears gets none of the scrutiny
the original had.

## The kinds of reason, because only one kind can be worked on

* a **blocker** — something is missing and somebody can build it. It expires
  when they do, and the entry is then deleted.
* a **decision** — a deferral. It expires only when the person who made it
  changes it. Whether a `P3` entry loses its control is a taxonomy
  decision and the operator's; nothing here removes one.
* **no reason at all**, which is not a deferral. An entry that admits it has
  no recorded reason is the FIRST one to re-derive, not the last: somebody
  has already established that nothing is defending it, and the confession
  reads like the output of a search that already happened.

⚠ A blocker can also be **correct for the wrong reason** — the id genuinely
has no arm, and the recorded cause is not the real one. Nothing about such
an entry looks wrong; the only thing that finds it is asking what the verb's
own REQUEST STRUCT requires rather than whether the verb exists.

The reliable half is identifiable in advance: **an entry whose truth
condition is inside THIS repository is the strong kind** — nothing makes a
missing window appear except somebody building it, so it cannot go stale by
accident. An entry that cites another document or another repository can,
and does. A reason that is a citation of a citation, with nothing re-reading
either, is the commonest way this list goes false.

⇒ **Re-derive the list on a schedule, not on a collision**, and when you
touch it for any purpose re-derive the reason of the entry beside the one
you came for. This assertion cannot help: it asks whether an id has an arm,
and an entry whose id has no arm and whose reason is nonsense is
indistinguishable from a correct one. A reason is prose; a reader is the
only instrument.

## The honest verdict on this list

It forces an explanation for every dead control. It has never forced a fix.
An entry can sit for weeks with a reason that is true about one thing and
false about the requirement, nine lines above the note naming that exact
failure mode, and survive an audit that re-derives its neighbours — and then
be found by the operator pressing the button.

⇒ The replacement is not a better list. It is a **driven check that presses
every registered id and fails on `command-unimplemented`** — a claim about
the running program, which no paragraph can satisfy. See `tools/ui-verify`.

The empty list is **kept rather than deleted**, exactly as
[`UNREACHED_ARMS`] is kept at zero and for the same reason: an empty
allow-list is still a gate. A new entry cannot be added quietly — it has to
be written here with a reason, and the count assertion is what makes adding
one a visible act.

**And the mirror defect, which the same reader finds.** A literal arm can
name a command that is **not registered at all**, so no token can reach it
and no operator ever could. [`UNREACHED_ARMS`] is the allow-list for those.
It is empty and is kept as a gate: the first planted violation of this check
was one of those arms and the check said nothing.

The gate discipline is kept in full. [`tests`] contains a self-test that
plants a violation in a fixture and proves the reader reports it, another
that proves the reader does **not** report a clean fixture, and two that
aim at the specific misreadings a grep would make — because, in
`check-file-size.sh`'s words, a gate that has never been observed to fail
is not evidence of anything.

## Item notes

### `const DISPATCH_PAGES_SRC`

# Why there are two, and why the checker had to learn about it

`app::dispatch` reaches R2's 1,500-line limit, so the Pages tab's arms live
in `app::dispatch::pages`, behind a guard arm (`id if pages::handles(id)`).
The parent file does not *contain* those six commands anywhere a `syn` walk
of it can see.

A checker that reads only the parent reports `pages.delete`,
`pages.extract`, `pages.move_up`, `pages.move_down` and both rotates as
unreachable — which is exactly the behaviour this module's header argues
for over a `bash` grep: **it fails closed.** A grep would find the string
`"pages.delete"` in either file and say nothing.

The lesson worth keeping is that a checker which reads ONE file is a
checker with a shelf life: R2 guarantees that any file it reads will
eventually be split, so *"where is the routing table?"* is a question with a
growing answer. Adding a source here is the cheap half; the expensive half
would have been discovering the blindness from an operator pressing a dead
control.

### `const DISPATCH_MEASURE_SRC`

**The second source [`DISPATCH_PAGES_SRC`]'s own doc predicts**: a checker
which reads ONE file is a checker with a shelf life, because R2 guarantees
that any file it reads will eventually be split.

It fails closed again, and by name — six `measure.*` ids reported
unreachable the moment the arms move — which is the behaviour that
paragraph argues for over a `bash` grep. A grep would find the string
`"measure.set_scale"` in either file and say nothing.

⇒ **The next split will need a line here too**, and the failure that costs
nothing to fix is this one rather than an operator pressing a dead control.

### `const CONSTS_SRC`

One arm of the dispatcher is written `crate::shell::commands::FILE_RECENT
=> …` rather than as a literal, for the reason that constant's own doc
comment gives: the id is spelled in four places that must agree, and "a
typo in any of them produces silence — a menu that draws and reports
nothing — rather than an error."

So the reader resolves constant patterns instead of ignoring them, and it
resolves them **by parsing the file that defines them** rather than by
carrying a copy of the value. A copy would be a hand-maintained mirror of
exactly one entry, which is still the thing `DEFECTS.md` D5 forbids: *"a
hand-maintained list with a comment telling you to hand-maintain it has
already failed once."*

### `const SPLIT_DISPATCHER`

[`DISPATCHER`] is an inherent method on `PdfcerApp`; a file split out under
R2 has no `impl` block to hang one on, so its entry point is a plain
function. Two names rather than one loosened matcher, because "any function
containing a match" would start classifying helper functions as routing
tables the day somebody wrote one.

### `fn unclassifiable`

Failing rather than ignoring is the whole discipline: an arm the reader
cannot classify is an arm whose ids it would otherwise report as
unreachable *or* miss entirely, and neither silence is acceptable in a
check whose only interesting failure is a false pass.

### `fn find_routing_match`

Deliberately **not** a search for any `match` anywhere in the file.
`dispatch_command` contains four nested `match` expressions inside arm
bodies (the recent-file operand, the text-mark outcome, the page-move
refusal, the page-text failure), and one of them — the refusal — has string
literals on the *right* of its arrows. Reading the wrong one is the mistake
a grep makes; reading the right one is the reason this walks a tree.

### `fn guard_subject_fn`

Every guard arm in the dispatcher has the shape
`id if <path>(id).is_some()`, so this peels the wrappers — a method call, a
negation, a parenthesis — until it finds a call whose single argument is
the subject, and returns that call's last path segment.

Requiring the argument to be **exactly the subject** is what stops it
answering for an unrelated call in a more complicated guard. A guard that
does something this cannot read is an error at the call site above, not a
silent `None` treated as "claims nothing".

### `fn registry`

Against **this** rather than against [`super::super::catalog::all`]'s
literal text, for the reason every test in [`super::super::mapping`]
gives: it is the difference between asserting that the code agrees with
itself and asserting that the control exists.

### `fn every_registered_command_is_routed_or_argued`

The one assertion this module exists to make. A failure here means a
control is drawn, enabled and pressable and produces
`command-unimplemented` — which is what `file.save_copy` did for the
whole life of the project, agreed with by five surfaces and contradicted
by none.

### `fn no_scaffolded_command_is_also_planned`

`manifest::PLANNED` is for commands that are **not registered**: named
by `RIBBON_IA.md`, absent from this build, drawn nowhere. Everything
here is registered and drawn. An id in both lists would mean one of the
two is wrong about whether the command exists, and the registry
assertion in [`no_scaffolded_entry_is_stale`] already says which.

### `fn the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has`

The seam between the two halves of this module, asserted as a set
equality in both directions:

* a **new** guard arm in `dispatch.rs` that [`guard_claiming`] cannot
  run would otherwise report that arm's whole family unreachable, and
  the reader would go looking in the wrong place;
* a **deleted** guard arm would otherwise keep being vouched for by a
  mapping function that still exists, which is a false pass on the
  exact defect this module is about.

### `fn the_reader_found_the_routing_table`

A sanity check on the **reading**, not on the code: a `match id` with no
catch-all would not compile, so a run in which this is false means the
reader found something other than the routing table and every other
assertion in this module is measuring the wrong thing.

### `fn the_reader_finds_every_arm_shape`

Without this, assertion B below could pass by finding nothing at all —
which is the failure mode `run-all.sh`'s three-state model exists for,
arriving inside a test instead of inside a script.

### `fn a_deleted_arm_is_reported_unreachable`

The fixture is `CLEAN_FIXTURE` with the `"fx.literal"` arm deleted and
nothing else changed — which is exactly the shape of the real defect:
the command stays registered, the ribbon keeps drawing it, and only the
arm is gone.

### `fn the_reader_does_not_see_comments_or_body_strings`

The two false passes a grep produces. `"fx.in_a_comment"` is a quoted id
inside a `//` line — the exact shape of the doc comments in
`app::dispatch`, which quote ids constantly — and `"fx.in_a_body"` is a
string literal in executable code. A check that counted either would go
green over a command whose arm had been deleted while the prose about it
stayed.

### `fn the_reader_does_not_see_a_nested_match`

`dispatch_command` contains four nested `match` expressions inside arm
bodies. A text scan cannot tell their arrows from the outer ones, and
one of them has string literals on the right-hand side — so a grep would
credit the dispatcher with routing ids it has never heard of. The tree
walk visits the arms of one `match` and never descends into a body.

### `fn an_unreadable_arm_is_refused`

Failing closed is what keeps a future edit honest: a pattern nobody
taught this reader about must stop the suite and be classified, rather
than silently taking its ids out of the check.

### `fn a_source_with_no_dispatcher_is_refused`

The "zero files scanned" failure, closed here as an `Err`. In the real
module it cannot even arise: [`DISPATCH_SRC`] is an `include_str!` and a
missing dispatcher file is a compile error.

### `fn the_p3_tension_is_counted`

Not a rule — a **published number**, in the shape
`the_icon_coverage_split_adds_up_to_the_registry` established: an
arithmetic identity plus the two literals a reader actually consults, so
that shortening this list is a visible act rather than a silent one.

`RIBBON_IA.md` P3 says an unavailable capability renders **nothing**.
Every entry marked `P3` in its reason is a control this module's
author believes should not be drawn yet; removing one is a taxonomy
decision and is the operator's. The count moving *down* is the project
working.
