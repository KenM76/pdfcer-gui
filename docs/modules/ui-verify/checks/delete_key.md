# `ui-verify/checks/delete_key`

`delete_key_after_canvas_click` — the regression test for **D1**.

# The defect

Reported by the operator as *"I can't even click on an object and delete it
by hitting the delete key."* It is real, it is not a discoverability
problem, and the selection half works perfectly — which is what makes it so
confusing to use.

The causal chain, from `DEFECTS.md` D1, verified against source:

1. **Click-select works with no gating.** The canvas hit-tests and assigns
   the selection. The object visibly selects.
2. **The canvas grabs egui keyboard focus on every click**:
   `if image_response.clicked() { image_response
   .request_focus(); }`. Deliberate and reasonable — the canvas was meant to
   be a real Tab stop rather than an inert image. Because the widget is
   recreated every frame its id stays live, so the focus never lapses.
3. **The keyboard guard tests the wrong thing**:
   `let typing = ctx.egui_wants_keyboard_input();`. In egui 0.35 that is
   **not** "a text field is focused" — verified in the vendored source at
   `egui-0.35.0/src/context.rs:2884`, it is `self.memory(|m| m.focused().is_some())`, i.e.
   *any* focused widget, the canvas included. The doc comment directly above
   it promises the opposite. This is an egui API footgun, not a careless
   read.
4. **So the binding is never installed.** `if (!tool_active ||
   canvas_delete_target) && !typing` — the first half is satisfied, `typing`
   is `true` from step 3, and the branch never runs.
5. **The deletion logic downstream is correct and simply unreachable.**

The same guard also kills `PageDown`/`PageUp`, `Home`/`End` and `[`/`]`
from the same click, so keyboard page navigation dies with it.

# Why the existing test could not catch it

`collect_keyboard_actions` has exactly one test, and it builds a bare
`egui::Context::default()` with **no widgets**. Therefore `memory.focused()`
is `None`, therefore `typing` is always `false`, therefore the single
property that breaks in the real application is *structurally absent from
the only harness that exercises the function*. Object deletion is covered at
the `Action` level and never through the key.

The regression is self-declared in its own commit message: *"a focused text
field keeps its unmodified keys — analysis-confirmed, NOT empirically
verified."*

# How this check detects it

By doing what the operator did, through the operating system:

1. convert a **document point** to a screen point (never a literal screen
   coordinate — see [`crate::coords`]);
2. click it with the real cursor, so egui's focus machinery runs exactly as
   it does for a person — the reason [`crate::input`] rejects in-process
   injection for this check specifically;
3. **assert the selection is non-empty** — the precondition, without which
   the next step's failure would be a mystery rather than a defect;
4. press `Delete` with a real keystroke;
5. **assert the object count dropped.**

Against the current old binary, steps 1–3 succeed and step 5 does not: no
deletion is traced, because the key was suppressed at step 4. That is a
FAIL, and it is the acceptance evidence for this harness.

# Two oracles for step 5, and the check says which one it used

**The page object count**, when the binary reports one (`objects n=…`).
Read after the click, read again after the key, compared. This is the
preferred oracle for one reason: it measures *the property the check is
about* — did the object leave the page — rather than the verb that was
meant to change it. It is also indifferent to a deletion implemented by
some future command nobody remembered to add a trace call to.

**The `delete-objects` event**, when there is no count. Weaker, in the
specific way described below, and used only as the fallback.

Every failure string from this check begins by naming which of the two it
used. That costs one clause and removes the question a reader would
otherwise have to answer by reading this file: *is "no deletion traced" a
statement about the page, or about a trace call site?*

# The one subtle judgement in this file

The fallback oracle's evidence is **the absence of a `delete-objects` trace
line**. Absence is normally weak
evidence and this crate refuses it elsewhere. It is admissible here, and
only here, for three stated reasons:

* the event is **in the binary's vocabulary** — the code path exists and
  traces unconditionally when it runs, so its absence means the path was
  not taken, not that the binary cannot report;
* the harness has already **established the precondition** in step 3, so it
  is known to have got as far as a live selection;
* the harness **pressed the key itself**, so it is known that the key event
  was delivered to the foreground window.

Remove any one of those and this becomes a SKIP. That is why step 3 is an
assertion and not a convenience, and why a hit test that finds nothing
produces a SKIP rather than a FAIL: the harness cannot then distinguish "the
application is broken" from "the harness aimed at empty page", and filing
the former when it is the latter is precisely the retracted-false-defect
outcome `crate::coords` documents.

## Item notes

### `fn drive`

The three-way return is the SKIP/FAIL/PASS rule made structural:
`Err` is a precondition that was absent (SKIP), `Ok(Some(_))` is an
assertion that did not hold (FAIL), `Ok(None)` is a pass. A check author
who reaches for `?` gets a SKIP, which is the safe default — the unsafe
default would be a pass.
