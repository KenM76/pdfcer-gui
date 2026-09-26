# `pdfcer-gui-base/acrobat/tests`

## Item notes

### `struct Machine`

`app_paths` is keyed by executable name — `"Acrobat.exe"` — exactly as
[`Registrations::app_path`] is asked. `present` is the set of paths that
exist; **a path not in it does not exist**, which is how "the registry
names an Acrobat that has been uninstalled" is expressed without creating
and deleting real files.

### `fn a_machine_with_no_acrobat_offers_no_viewer_and_therefore_no_button`

R9, and the reason the whole capability is expressed as a `visible_when`
rather than as an `enabled_when`: *an unavailable capability renders
nothing; greying is reserved for **temporarily** unavailable and is always
explained on hover.* A machine with no Acrobat is not temporarily anything.

The decision the ribbon makes is `resolve(...).is_some()`, so this is the
button-visibility test in the only form it has outside a window.

### `fn pro_beats_reader_when_both_are_installed`

`OPERATOR_REQUESTS.md` O122: *"acrobat reader or pro depending on what is
installed"*, decided in favour of Pro because Pro is the superset and is
what somebody who owns both reaches for.

### `fn pro_beats_reader_even_when_reader_is_the_registered_handler`

The ordering that makes [`Source::rank`] a tie-break rather than the first
sort key. Sorting by source first would answer *Reader*, on the reasoning
that `App Paths` is the better-quality registration — which is true and
beside the point, because the operator asked for Pro when Pro is there.

### `fn a_configured_path_beats_discovery_and_never_falls_back_to_it`

The escape hatch of O122 point 4, and the two halves are equally
load-bearing. Beating discovery is what makes the setting mean anything at
all. *Not falling back* is what stops a person who deliberately pointed
pdfcer at their second installation from being silently sent to their
first — which would undo the setting with nothing on screen saying so.

### `fn a_stale_registration_is_not_offered`

An uninstall that leaves its `App Paths` key behind is ordinary. Offering
the button anyway would produce a control that is present, enabled, and
does nothing when pressed — R9's failure reached from the other direction.

### `fn a_configured_path_is_honoured_even_if_its_name_is_not_one_we_know`

The deliberate asymmetry with discovery: [`super::discover::edition_of`]
is a **filter** on what the registry offers and a **label** on what the
operator typed. Somebody who points this setting at a renamed executable,
a launcher script wrapper or a portable install has answered the question
the filter exists to ask, and refusing them would make the escape hatch
narrower than the thing it is an escape from.

### `fn launching_passes_the_document_to_the_viewer`

Thin, and worth having anyway: it is the assertion that the two paths are
not transposed. Inside a [`Launcher`] both are paths, so an implementation
that started the document and handed it the program compiles and runs, and
fails only on a real machine — where it would try to execute a PDF.
