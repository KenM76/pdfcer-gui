# `ui-verify/sys/win32`

The Windows implementation of the platform API.

Everything here is a thin, documented wrapper over one Win32 call. The
interesting decisions are recorded at the function that embodies them; the
cross-cutting ones are here.

## Why `GetDC(NULL)` + `BitBlt` and not `PrintWindow`

[`capture_screen`] photographs the **composited desktop**, not the window's
own device context. The window's DC is the obvious choice and it is wrong
for this application: eframe renders through glow/wgpu, and a
GPU-composited surface frequently comes back **blank** from a
`PrintWindow`/`BitBlt` of the window DC. A blank capture is the worst
possible failure here, because it is indistinguishable from a real one at
the call site — the file exists, the call succeeded, and only a human
looking at the PNG can tell it is not evidence. This project's predecessor
recorded exactly that, twice, and recorded a plausible-but-invented cause
being attached to it before the real one was found.

The consequence of reading the desktop is that whatever is *in front of*
the window is what gets photographed. Hence [`raise_window`], and hence the
near-uniformity guard in [`crate::pixels::region_not_uniform`], which is
the mechanical version of "a human looked at it".

## Why the window search is by process id

Not by title, and not by class. Titles change with the open document; class
names are winit's business and not a contract. The process id is the one
thing the harness knows for certain, because it launched the process. It
also guarantees the harness can never drive a window belonging to an
instance the operator opened for their own work — a hazard the predecessor
scripts hit hard enough to write four paragraphs about.
