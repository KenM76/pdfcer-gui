# `app::pickstore` — the selection filter, on disk

One question, answered the way [`crate::app::persistence`] answers it for
the dock layout: *where does the operator's selection filter live, and when
is it written?*

## Why a second module rather than a second type in `persistence`

`app::persistence` owns one type, [`crate::app::persistence::LayoutStore`],
and is shaped around it: its whole write-scheduling apparatus exists for a
problem this file does not have. A second, differently-scheduled store in
there would put two unrelated write policies under one header.

## Why there is no debounce here, when the layout needs one

The one substantive difference between the two stores, stated because the
absence otherwise reads as an oversight.

`LayoutStore` debounces because a splitter drag reports a change **on every
frame of the gesture** — a two-second drag is a hundred and twenty change
notifications describing one operator decision, and writing each one would
be a hundred and twenty file writes for one intent.

A selection filter cannot do that. Its only inputs are discrete clicks on
eleven checkboxes and two buttons; there is no continuous gesture that can
produce one. So a change here is already exactly one operator decision, and
the correct number of writes for one decision is one, immediately. Adding a
delay would buy nothing and would introduce a window in which pulling the
power cord loses a choice the operator has already seen take effect on the
canvas.

## Where the file lives

`<the settings directory>/select-filter.txt` — beside `settings.txt` and
`layout.ron`, with the directory resolved by asking `pdfcer-core` rather
than computing it here:

```text
pdfcer_core::settings::resolve_store()      →  StoreLocation { path, kind }
                       .directory()        →  <exe dir>/userdata      (Portable)
                                           or the platform config dir (PlatformFallback)
                                           or None                    (None)
```

Deferring to that call is what keeps a portable install portable: an
operator who copies the folder to a memory stick takes their filter with
them, because it never was anywhere else. A second implementation here
would be a second answer to *"where is the profile?"*, and the two would
disagree the first time either moved.

## Failure is silent, in one direction only, and that is deliberate

**Loading never fails.** A missing file, an unreadable one, a directory
that does not exist, or a line full of tokens from a newer build all yield a
working filter. A shell that refused to start — or that opened with
everything unselectable — because a preferences file was unreadable would be
trading a total failure for a cosmetic one.

The distinction that makes this safe is the one
[`crate::canvas::pick::PickFilter::from_tokens`] cannot make for itself:

| on disk | means | yields |
|---|---|---|
| no file at all | this operator has never touched the filter | [`PickFilter::default`] — everything the shell can pick |
| a file, with tokens | these are the classes they chose | exactly those |
| a file, **empty** | they switched everything off | [`PickFilter::none`] — and the status bar says so |

Row three is the one worth guarding. Collapsing "an empty file" into "no
file" would silently overrule a deliberate choice every restart, and the
operator would never find out why their filter kept resetting.

**Saving may fail, and says so.** A read-only install directory is a real
condition, and [`save`] returns the error rather than swallowing it, so a
caller can decide whether it is worth telling anyone. Today's caller does
not tell anyone, which is a judgement about *this* preference rather than
about errors: losing a selection filter across a restart is an
inconvenience, and a modal about a preferences file at the moment the
operator clicked a checkbox would be worse than the thing it reports.

## Item notes

### `fn an_empty_file_means_nothing_selectable_not_never_configured`

An operator who switched every class off and quit must get that back,
not a helpfully-restored default. If this ever fails, the shell has
started overruling a deliberate choice once per restart, and the
operator's report will be "my filter keeps resetting" with no way for
them to see why.

### `const FILTER_FILE`

A plain text file rather than RON, because the whole content is a single
line of space-separated words. RON would add a schema wrapper, a parser
dependency and a version field to serialise eleven booleans that already
have a stable textual form — and would make the file unreadable by the one
tool most likely to be pointed at it, which is a person with a text editor
trying to work out why their canvas stopped selecting things.

### `fn path`

`None` is not an error: `pdfcer-core` reports it for a build with no
writable profile location at all, and the correct behaviour then is to run
with defaults and save nothing.

### `fn load_from`

The `Err` arm and the `Ok` arm are deliberately **not** merged. They mean
different things — "you have never set this" against "these are your
settings" — and only the former may be answered with the default. See the
module header's table.

### `fn save`

Returns `Ok(false)` when this install has nowhere to write — which is a
successful no-op rather than a failure, and is distinguished from
`Ok(true)` so a caller can tell "saved" from "there is no profile".

Creates the directory if it is missing, because on a fresh portable install
the first thing that wants to persist anything is whatever the operator
touches first, and that may well be this.
