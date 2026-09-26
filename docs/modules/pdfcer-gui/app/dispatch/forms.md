# `app::dispatch::forms` — what the form-field commands do

One arm of [`super::PdfcerApp::dispatch_command`], in its own file. The
dispatcher is a routing table and this is a route; the **reasoning** below
is the reason it is not an arm, because it is about form fields rather than
about routing and would be unfindable among the dispatcher's other arms.

## The whole of what a form command does: it arms a tool

Nothing is authored here, and — unusually — nothing is authored on release
either. The placing gesture raises `Action::BeginFormField`, which opens
`crate::dialogs::formfield`, and the field exists once the operator presses
Add. `crate::canvas::formfield`'s header argues why that indirection is the
feature rather than an extra step: a stray form field is invisible on a
printed page and swallows every keystroke aimed near it, so a mis-drag must
cost nothing.

## Why the push button is refused here, in words

This file exists to hold one finding, and it is worth the space.

`edit.form_push_button` is `enabled_when("forms.push_button_runnable")`, a
condition nothing sets, so its ribbon item is greyed. That much is measured
rather than assumed — `egui_shell::ribbon::ctx::condition_holds` answers
`false` for an unset name.

**But `egui` refusing a click on a disabled widget is the entire mechanism
of greying.** Every other route into the dispatcher — a keyboard chord, the
QAT, a context menu, the `PDFCER_DIAG_INVOKE` harness seam — never touches
the ribbon at all. Driving the release binary with that id arms the tool and
traces `form-tool-armed kind=PushButton`. **An `enabled_when` is a drawing
instruction, not a rule**, and that holds for every command that carries one.

### Why there is no blanket guard at the top of `dispatch_command`

The obvious repair — refuse any command whose `enable` predicate is false —
is wrong, and the apply layer's own tests assert against it: *"the
dispatcher must not consult one. `undo.available` greys the control and the
apply arm declines an empty stack **in words** — both of which are somebody
else's job."*

**Greying is a hint; the worded decline is the answer.** A choke point that
swallowed the command would make `Ctrl+Z` on an empty stack do nothing at
all *and* say nothing at all — strictly worse than the status line it
produces, and the exact shape of the silent-control defect this project
keeps finding.

So enforcement lives where the words can live, which is the arm. What that
costs is one branch per command that needs it; what it buys is that an
operator who reaches a greyed capability by some other route is told why
rather than left pressing a key that does nothing.

## Item notes

### `fn arm`

`id` is carried only for the trace — it is recoverable from `kind`, but a
trace line that printed a reconstructed id would be a second opinion about
what the operator invoked, and the whole value of a trace is that it is a
record rather than an inference.

### `fn flatten`

# Flattening is NOT a destructive verb, and takes no blocking modal

It is one `EditSession` command and therefore one `Ctrl+Z`, and
`EditSession::flatten_fields` **appends** an overlay stream while leaving
existing content byte-verbatim — so under the default incremental save the
prior revision still holds the field values. Its irreversibility is
conditional on the save mode, not structural. That is why the panel's own
button carries delete-shaped weight — a rich, honest tooltip and one undo
step — rather than redaction's blocking modal, and the same reasoning
applies to this ribbon route unchanged. `text::forms`'
`forms_flatten_tooltip` is where the wording lives.

It is on the ribbon *as well as* in the Forms panel because a command
buried in a panel is reachable only by someone who already opened the panel.

# Why this raises the SAME action as the panel button, and takes no
extra gate

It pushes `FieldAction::Edit(FormEdit::Flatten)` — the identical intent,
through the identical apply path — so the two routes cannot become two
implementations. `Action::Command` makes that argument for command-to-command
routes and it is the same argument here.

And it deliberately adds **no mode gate**, though `arm` above has one.
The Edit tab is shown only in Edit, so the ribbon route is already
mode-scoped; a chord could reach further, and there is no chord. What
decided it is that the **panel** offers Flatten with no mode gate, in every
mode its dock is mounted in, and a second route with a stricter rule is the
disagreement this project refuses — two controls for one capability
answering differently, with the operator left to work out which one is
lying. If flattening should be Edit-only, that is one change in
`panels::forms` and this arm follows it; it is **recorded as an open
question rather than decided here**, because it is a scope call.

# The refusal is the strict gate, and it is asked in the same words

`flatten_refusal`, not `fill_refusal`. Flattening removes the form, which
is a structural change, and on the ordinary real-world shape — a certified
fillable form at `/P 2` — filling is permitted while flattening is refused.
`panels::forms` owns the statement of that distinction; this arm asks the
same question so the greyed panel button and the declining ribbon control
cannot disagree.
