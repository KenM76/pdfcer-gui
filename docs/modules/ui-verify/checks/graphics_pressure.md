# `ui-verify/checks/graphics_pressure`

**The graphics-pressure instrument is read by something** — the dial in
front of O219 and O221, which until now nobody stood in front of.

# What this file is for

`crate`-side, `render::pressure` publishes two diagnostic lines:

* `gl-max-texture-side side=N` — the device's real single-axis texture
  limit, traced whenever it changes;
* `gl-pressure oom=… count=… other=… truncated=… {clean|blamed …|unattributed=…}`
  — one line per frame on which a GL error was drained, carrying what the
  failure could and could not be pinned on.

Both were written for the operator's most-reported symptom — O219, *"the
view goes blank and when I zoom in a little more I get the error"* — and
before this file **neither was consumed by any check in this harness**. A
`grep` for either name across `tools/ui-verify/src/` returned nothing. They
were documented, gated by `check-trace-names`, emitted every frame, and read
by nobody. Emitting a measurement and making a measurement are different
acts, and only one of them can be gated from the emitting side.

# The assertion, and the mechanism that makes it load-bearing

`egui`'s `InputState` `Default` sets `max_texture_side` to **2048**, and
`RawInput::max_texture_side` is an `Option` that begins `None`;
`InputState::begin_pass` folds it in with `unwrap_or(self.max_texture_side)`.
⇒ **if the backend never supplies the figure, the field stays 2048 for the
life of the process** and reads as a perfectly plausible device limit.

So a check that merely asserted *"a `gl-max-texture-side` line exists"*
would be satisfied by a build whose instrument is wired to a constant. The
assertion here is on the **standing** value — the last line, since
`diag::trace_on_change` emits only on a change — and it must clear
[`CREDIBLE_FLOOR`]. On any device this shell can be used on, the sequence is
two lines: `side=2048` before the backend has spoken and the real figure
after.

⚠ The trade that buys that: a device whose limit genuinely **is** 2048 goes
red here, and the report would blame the shell for the truth. That is
accepted deliberately — the whole-page raster tier's budget admits an edge
of `pdfcer_render::MAX_PIXMAP_EDGE`, which is far above 2048, so such a
device cannot draw a zoomed page at all and "the instrument is stuck" is by
a wide margin the likelier reading of a 2048. The failure text says both, so
a reader on such a machine is not sent hunting.

# Why it is off the desktop and sends nothing

The verdict is entirely trace-borne — not one pixel is read and not one
keystroke or click is sent — so the window goes where a human cannot see it
and **this check can run while the operator is working**. That matters more
than usual here: it is the first rung of the document-count ladder O221
needs, and a rung that competes for the machine is a rung that only runs on
a night nobody is using it.

# What this does NOT reach

* **It does not provoke pressure.** One document, fit zoom, no gestures, is
  the ladder's **control**: `gl-pressure` is expected to be silent. Its
  readings are therefore *reported and not asserted* — an assertion on them
  would be an assertion about how much graphics memory was free on the
  machine that happened to run the sweep.
* **It says nothing about O221.** The standing hypothesis — that N open
  documents each claim the whole of `render::strip::MAX_CACHED_TEXELS`
  against one card — is a hypothesis with a citation and not a measurement.
  Measuring it needs three and six documents in separate processes, which
  this harness cannot yet arrange: it has exactly two fixture slots and
  reaches a second document through a one-path environment seam.
* ⚠ **It is evidence only from a release build.** In a debug build
  `egui_glow`'s own `check_for_gl_error!` runs after every GL call and
  **clears** the flag, so `gl-pressure` can never fire and its silence here
  would mean nothing at all. `render::pressure`'s own header carries the
  same warning; it is repeated because this is the consuming end, and the
  consuming end is where a silence gets mistaken for a clean reading.
