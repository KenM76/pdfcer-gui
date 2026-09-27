# `printspooler::device` — what a printer IS, and how it is configured

## The seam this file is on the other side of

[`super`] is the adapter for **the job**: which pages, at what size, in
what order, placed where on a sheet. This file is the adapter for **the
device**: which printers exist, what each one can do, which sheets it
offers, and what its driver currently holds.


## What is still true of both halves

**This module and its parent are the only files in the crate that name
`pdfcer_print`.** Everything else in [`crate::dialogs::print`] — the three
tabs, the preview, the footer — is written against the mirrored types
here. That is what confined "make printing work" to one module in
August 2026, and it is worth keeping: see [`super`]'s header for the full
reasoning, including why no arithmetic is ever mirrored.

## The rule that governs every capability query here

**A query that answers "I do not know" is not a query that answered
"no".** `pdfcer-print` was explicit about this when it declined this
project's proposal to gate the tray control on a `bool`:

> *"`DC_BINS` on Microsoft Print to PDF returns nothing at all, while
> that same device's `dmDefaultSource` is already `DMBIN_FORMSOURCE` — it
> picks by form by default. A bool would have collapsed 'the driver said
> nothing' into 'no', and told the operator a device cannot do the thing
> it was already doing."*

So [`FormSourceSupport`] has three states and not two, and the shell's
reading of them is the inverse of R83's usual direction: **`NotListed`
and `Unknown` still get the control**, with the disclosure. R83 forbids
offering an affordance the hardware *cannot* honour; it does not forbid
offering one the driver merely declined to advertise.

Contrast [`DeviceFeatures::supports_duplex`], which is a genuine
capability answer and *is* gated: `DC_DUPLEX` returning zero means the
device is simplex, and no setting in the dialog will change that.

## Item notes

### `struct DeviceFeatures`

Maps to `pdfcer_print::DeviceFeatures`. Read **once**, when the dialog
opens: asking a driver this question sixty times a second while a dialog
sits open would be rude to a service other applications share.

### `fn list_printers`

Called **once**, when the dialog opens — enumerating printers touches the
spooler, and doing it per frame while a dialog sits open would be rude to
a service other applications share. [`super::PrintDialog::new`] is the
only caller and it stores the result.

# Errors

[`Unavailable::Spooler`] when the spooler could not be queried at all,
which on a non-Windows target is always (`PrintError::Unsupported`).

**An empty `Vec` is `Ok`, not an error.** A machine with no printers
installed is a normal machine; see [`Unavailable`]'s own documentation for
why the type has nowhere to put that case.

### `fn device_features`

Consulted **before** offering the duplex control at all (R83), never
after. [`crate::dialogs::print::PrintDialog::refresh_device`] calls it once per change
of the selected printer — which is the fix for a defect the old shell
still carries: it read features only for the *initially* selected device
and never again, so switching printers left the duplex control gated on
the previous one's capabilities.

# Errors

[`Unavailable::Spooler`] when the driver would not answer. The caller
falls back to [`DeviceFeatures::default`] — `supports_duplex: false` —
which is the safe direction: a device that cannot describe itself gets no
duplex control, rather than a control that may silently do nothing.

### `struct PaperForm`

# Why the id and the name are both carried

[`Self::id`] is what a job is addressed with — `dmPaperSize`, an integer
the driver defined — and [`Self::name`] is what the operator recognises.
Neither substitutes for the other: two drivers can use different names for
the same standard id (`"A4"` and `"A4 210 x 297 mm"`), and a *vendor*
driver can use the same name for different ids across models. The combo
shows the name and sends the id, which is the only pairing that survives
both.

# Why the size is carried as well as the name

Because a driver's name for a roll or a custom form is frequently not a
size at all — `"Roll Paper 24in"`, `"User Defined"`, `"Custom"` — and the
operator choosing between two of those needs the dimensions.

**This is the PHYSICAL sheet, not the printable area.** The engine makes
the same distinction on `PrinterCaps` and for the same reason: fitting a
page to the physical size produces a page whose edges the hardware crops.
Nothing in this shell plans against this value — planning reads the
geometry [`super::plan`] gets back from `printer_caps_for`, which is the
printable area for *this* sheet. This one is for the label only.

### `enum FormSourceSupport`

Maps to `pdfcer_print::FormSourceSupport`. **Three states, and the third is
the whole point** — see this module's header for the measurement that
killed the `bool` version of this field.

### `struct DriverConfig`

# What this actually is, and why the shell must not look inside

A Windows `DEVMODE`: a public header pdfcer understands, followed by a
**driver-private tail** in a format only that one driver knows. The
engine measured the tail on this machine — 5,208 bytes for Microsoft
Print to PDF, 7,972 for both EPSONs, 920 for the XPS writer. On the
EPSONs *97 % of a real `DEVMODE` is data pdfcer cannot interpret*, and it
carries media type, print quality, colour handling, stapling, output bin
and everything else the vendor's own dialog offers.

So this type has no fields the shell reads and no way to construct one:
it comes from the driver, it goes back to the driver, and the only thing
the shell may know about it is [`Self::summary`]. A shell that unpacked it
would be re-implementing a format it does not have.

# Why it exists at all rather than the dialog just holding the bytes

Because a `DEVMODE` belongs to **one device**. Handing one driver's
configuration to another is not a degraded result, it is an undefined one,
and the engine refuses it by name (`PrintError::Configuration`). Wrapping
it keeps that fact visible at the seam, and the dialog clears the field
with the rest of its per-device cache whenever the selection changes.

### `fn engine`

`pub(super)` and not `pub(crate)`: [`super::plan`] and [`super::spool`]
are the only callers, and widening this would let a `pdfcer_print` type
escape into a third file — which is the property this module exists to
hold.

### `struct ConfigSummary`

Maps to the three fields of `pdfcer_print::ConfigurationSummary` this shell
has a use for. The engine's version carries five more — orientation,
duplex, tray, form name, device name — and they are deliberately **not**
mirrored: a field nothing reads is a field that can quietly acquire the
wrong units or stop being filled, and the dialog's own controls are
authoritative for every one of them (see [`super::to_engine_settings`] on
which members pdfcer asserts over whatever the configuration held).

### `fn printer_forms`

Called on a change of the selected printer, alongside [`device_features`],
and stored. The list is what the paper combo is drawn from; an empty list
or a refusal leaves the combo showing only "from the printer's own
settings", which is honest — pdfcer cannot name a sheet the driver would
not enumerate.

# Errors

[`Unavailable::Spooler`] when the driver would not answer. The caller
falls back to an empty list.

### `fn edit_printer_configuration`

# Why there is no silent "read the current settings" call beside this

There was one, briefly, on the theory that the FIRST press of this button
should resume from the device's current configuration. It should not, and
it already does: `DocumentProperties` with `DM_IN_PROMPT` and no input
buffer starts the dialog from the printer's own settings, which is
precisely what an operator expects the first time they open it. Passing a
separately-fetched copy would have been the same value by a longer route.

`start_from` earns its place on the SECOND press: it resumes from what the
first press produced, so an operator reopening the dialog to change one
thing does not silently lose the rest.

# `Ok(None)` is Cancel, and it is not a failure

The engine is explicit: *"that is the operator declining, and a shell that
showed an error for it would be scolding them for using the dialog
correctly."* The caller keeps whatever configuration it already had and
says nothing.

# The parent handle

`parent` is this application's own top-level window, as a raw `HWND` cast
to `isize`. Passing `None` is legal and produces an **unowned** modal
dialog, which can fall behind the main window — a modal the operator
cannot see and cannot dismiss, with the application apparently frozen
behind it. So the shell passes its handle.

# This call BLOCKS the frame, for as long as the operator takes

It is a nested modal message loop belonging to the driver, run from inside
our own event loop. egui stops painting until it returns. That is
acceptable here and would not be for anything on the canvas: it is one
button, pressed deliberately, whose entire purpose is a window the
operator is about to interact with, and the alternative — running it on
another thread — hands a foreign modal a parent it does not own.

# Errors

[`Unavailable::Device`], for the same reason as
[`printer_configuration`].
