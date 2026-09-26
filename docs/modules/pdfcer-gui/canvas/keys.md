# `canvas::keys` — the keys the canvas owns, and who gets Escape


## Tab, and why it is guarded on the armed tool rather than on a mode

Tab advances the **snap cycle** while a measure tool is armed
([`crate::canvas::measure::cycle_snap`]) — the operator's way of saying
*"the other candidate"* when an endpoint and a midpoint are a few pixels
apart and pointing cannot separate them.

Tab is `egui`'s focus key, so the guard matters more here than for the
other two. It is the **armed tool**, not a mode and not a capability:
with no measure tool armed the branch is false, the key is untouched, and
keyboard navigation of every panel and dialog behaves exactly as it did.
A mode-shaped guard would have been wrong in both directions — Review and
Edit both offer the measure tools, and in neither of them should Tab stop
moving focus when the operator is not measuring.

## Many claimants for Escape, one press, one effect

Decision 025's L1 is that Escape ascends **exactly one rung** rather than
collapsing the ladder, and the same discipline governs everything else that
would like the key. They are ordered *"retire the most transient thing
first"*, and rung 3 has two occupants that cannot both be present:

| # | claimant | who decides | how it says it took the key |
|---|---|---|---|
| 0 | a **form field being typed into** | [`crate::canvas::forms`] | [`crate::canvas::forms::escape_spent`]: `true` when a draft was committed |
| 1 | a **drag in flight**, including a markup band | [`crate::canvas::gesture::GestureState::update`] — the only thing that knows whether there is one | [`crate::canvas::gesture::GestureOutcome::Cancelled`], arriving here as `escape_consumed` |
| 2 | a **guide drag in flight** | [`crate::canvas::guides::cancel_drag`] | its return value: `true` when there was one |
| 3a | a **measure pick**, a **markup vertex run**, a **text draft** or a **pending placement** in progress | [`crate::canvas::measure::abandon`] / [`crate::canvas::markup::vertex::abandon`] / [`crate::canvas::textedit::settle`] / [`crate::dialogs::placing`] | its return value: `true` when there was one. Only the text draft **writes** what it retires |
| 3b | an **armed markup or measure tool** | [`crate::canvas::tool::disarm_markup`] / [`crate::canvas::tool::disarm_measure`] | its return value: `true` when there was one armed |
| — | the **armed text tool** is deliberately **not** a claimant — see below | — | — |
| 4 | an **armed region zoom** | [`crate::canvas::zoom::disarm_region_zoom`] | its return value: `true` when there was something to retire |
| 5 | the **selection ladder**, or the **text selection** | [`crate::canvas::selection::SelectionState::escape`] / clearing [`crate::canvas::textsel::TextSelection`] | it is last, so it acts only when none above did |

### Why the text selection shares rung 5 rather than taking a sixth

The two occupants of rung 5 can both be present. In **Edit**, an operator
can marquee some objects with the select tool, arm the text tool, sweep a
line, and hold both selections at once —
[`crate::canvas::textsel::takes_the_press`] gives a press its text meaning
on a disjunction, not on the single `Capabilities::edit_content` flag that
once made the two mutually exclusive by construction. `canvas::textsel` §3
records that exclusivity as a matter of **precedence** now.

The ordering within the rung is therefore real, and it is
**the text selection first.** Three reasons, in weight order:

1. **It is the more transient**, which is this table's own rule. A text range
   is made by the gesture the operator is performing right now and is
   destroyed by the very next edit anyway (`textsel` §7's epoch rule); an
   object selection survives edits, navigation, zoom and the mode change that
   hid it.
2. **It is what the operator just did.** Reaching this state requires arming
   a tool and sweeping, and the press that follows means the thing that press
   could plausibly be about.
3. **The content ladder has rungs and a text range has none**, which is the
   asymmetry the paragraph below already describes: clearing the text
   selection costs the operator one re-sweep, while ascending the object
   ladder loses the sub-path or node they had descended into. Given a
   choice, spend the press on the cheaper loss.

A rung of its own is not the answer. It would put a *precedence* between two
things that are the same act
— "clear what is selected" — expressed twice because this shell has two kinds
of selectable thing; and it would make a mode in which only one of them can
exist (Read, Review) look as though it had two rungs to climb. The rung is
"the selection"; which selection is answered by what is actually there.

Both branches obey L1 identically: one press clears, and a **second** press
then does nothing, because there is nothing left. That is deliberately
unlike the content ladder, whose first press *ascends* and whose second
clears — a text range has no rungs to ascend, since there is no larger unit
than the sweep the operator made and no smaller one they have descended into.

### Why the armed TEXT tool takes no rung at all

Rung 3b retires an armed markup or measure tool, so the obvious symmetry is a
third call beside them. It is deliberately absent, and the decision is
recorded here because its absence looks exactly like an omission.

**What rung 3b is actually for.** Both tools it retires paint a **crosshair**
that promises a gesture which *writes to the document*, and a mis-armed pen is
costly enough that the operator needs a universal way out of it. The text tool
paints an I-beam and promises a selection; it authors nothing (see
[`crate::canvas::tool::retire_forbidden`], which permits it in every mode for
the same reason). Nothing about it needs an emergency exit.

**What the rung would collide with.** Escape at rung 5 already means *clear
the selection* while this tool is armed, because clearing a text selection is
what rung 5's first branch does. A rung *below* that — which is where the
transience rule would put it, the tool being less transient than the range —
would make the second press silently move the operator from **sweeping text**
to **marqueeing objects** in Edit: a change of what the primary button means,
delivered by the key they pressed to clear something. One press, one effect
is satisfied; *one press, one **expected** effect* is not.

**What the reference applications do**, under the standing instruction to
match Inkscape, Acrobat and SolidWorks: Inkscape's Escape in the text tool
deselects and **stays in the tool**; Acrobat's Escape changes no tool; only
SolidWorks exits the active command. Two of three keep the tool, which is
what ships — and it is also the answer that needs no code.

The route out is the control that armed it: `view.tool_text` is a toggle, so
pressing it again returns to the select tool
([`crate::canvas::tool::toggle_text`]). That is the same *the button is
pressed, so pressing it un-presses it* rule the four markup buttons follow —
rung 3b is the **extra** affordance those tools get, not their only one.

### Why a measure pick is TWO rungs rather than one

A markup **band** is a drag, so abandoning it and retiring the pen are
already separated by the table: the drag is claimant 1, the tool is claimant
3b. A measure pick is a sequence of **clicks**, so there is no drag for
claimant 1 to cancel — and yet a linear dimension with point A taken and
point B not is unmistakably a gesture in flight. Without 3a, one Escape
would put the tool down *and* silently discard that pick: two effects from
one press, which is exactly what decision 025's L1 forbids.

**The same argument admits a second occupant to 3a**, and the fact that
it needs no new reasoning is the point. PolyLine and Polygon
are also gestured by clicks
([`crate::canvas::markup::vertex`]), so a run of three vertices with the
fourth not yet placed is in exactly the position a half-taken linear pick is.
[`crate::canvas::markup::vertex::abandon`] therefore sits beside
[`crate::canvas::measure::abandon`] rather than taking a rung of its own:
the two cannot both be in progress, because a measure tool and a markup tool
cannot both be armed, so this is **one claimant expressed as two calls** —
which is precisely the arrangement rung 3b already uses for the two `disarm`
functions.

It also means the sentence *"a markup is a drag"* is now only three-quarters
true, and the quarter that is not is why the rung was needed. The band kinds
and Ink are drags; the two vertex kinds are not.

So the pick is retired first and the tool second, which is also the order
the transience rule gives — a half-taken pick is the more transient of the
two, and it is the thing the operator is most likely to have meant.
Pressing Escape twice puts the tool down; pressing it once corrects a
mis-aimed first click without leaving the tool.

### Why a focused form field is rung 0, and why its rung is unlike every
other

It is numbered 0 rather than 1 because it does not merely *outrank* the
others — while it is live, **none of them can see the key at all**. A
focused field is an `egui::TextEdit`, so `Context::text_edit_focused` is
true, and that predicate is the first line of this very function
(`DEFECTS.md` D1's guard) as well as the gate `interact` builds the gesture
machine's `cancel` flag from. So the exclusion is **mechanical**, not
ordered, and there is no version of this table in which a marquee and a
half-typed field compete for one press.

What the rung is actually for is the *other* direction, and it is a real
hazard rather than a formality. `egui`'s own `TextEdit` surrenders focus on
Escape, and it does so **before** this function runs — so by the time the
guard above is asked, `text_edit_focused` is already false and the key
falls straight through to the selection ladder. One press would commit the
draft *and* ascend a rung: exactly the double effect L1 forbids, and exactly
the shape the `escape_consumed` flag was invented for one row down.
[`crate::canvas::forms::escape_spent`] is the same report-rather-than-
re-derive contract, read once and cleared by the reading.

### Where the markup tool sits, and why the transience rule does not
settle it

Row 1 needed **no change at all**, and that is the first thing to notice: a
markup band is a [`DragKind`](crate::canvas::gesture::DragKind), so a markup
drag in flight is already the *drag in flight* claimant, cancelled by the
existing branch, with no new mechanism and no second rule. Abandoning it
authors nothing — the annotation is only written by
[`crate::app::actions::Action::CommitMarkup`], which the release raises and
a cancellation never does.

Retiring the armed **tool** is a different act, and it needed a row. Its
placement is the one judgement call here, because the table's own rule does
not decide it: an armed markup tool is *less* transient than an armed region
zoom, not more. The zoom arming is a one-shot, spent by the very next drag;
the markup tool is a mode the operator stays in while they draw five
rectangles. On transience alone it would sit **below** the zoom.

It sits above it, and the deciding argument is the one the guide-versus-zoom
row already makes in the paragraph below: *"there is no reading of that press
under which they meant a zoom they armed earlier and have not used."* That
is true here twice over, and the second reason is mechanical rather than a
matter of intent — **while the markup tool is armed, the region zoom cannot
be reached at all.** [`crate::canvas::gesture::press_kind`] gives an armed
markup tool the primary drag unconditionally, so the armed zoom is inert for
as long as the pen is down. Spending the operator's Escape on retiring
something inert, while they are looking at an armed Rectangle button that
does not un-press, is one press with no visible effect — and the effect it
*does* have is on a control that lives on a different ribbon tab, where they
cannot see it.

So the ordering rule is unchanged and is simply not the one that applies to
this pair. What applies is the rule underneath it: **retire the thing the
operator could have meant.**

### Why a guide drag outranks an armed region zoom

Both are "something in flight", so the tie is broken by the rule itself:
**retire the most transient thing first.** A guide drag ends the moment
the pointer is released, and it is following the pointer *now*; an armed
region zoom persists across frames waiting for a drag that has not started.
An operator dragging a guide who presses Escape means the guide, and there
is no reading of that press under which they meant a zoom they armed
earlier and have not used.

A guide drag also does **not** reach [`crate::canvas::gesture`] — see that
module's header for why: a drag that did would move a guide *and* the
selection. So claimant 1 cannot speak for it, which is exactly why it needs
a row of its own rather than folding into `escape_consumed`.

Each claimant reports whether it took the key rather than the caller
guessing, because the caller cannot know: whether a drag exists is the
gesture machine's private state and whether a zoom is armed is the zoom
module's. A version that re-derived either here would be the version that
cancels a drag **and** ascends a rung — which is the defect the whole
arrangement exists to prevent, and which an operator experiences as losing
the sub-path they were editing every time they abandon a mis-aimed drag.
