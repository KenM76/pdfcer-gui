# `ui-verify/profile`

## Item notes

### `const SETTINGS_HEADINGS_LEGACY`

# Provenance, stated in full because a calibrated number is only as good as
its provenance


The calibration is [`Calibration::Image`], so these fractions can only be
used against that image. They are **not** valid against a live window of
the old GUI: the crop is a sub-rectangle of the dialog, not the client
area, so the same fractions would sample the wrong part of a live capture.
Driving the live old binary to its Settings dialog needs its own
calibration pass, and until somebody does that pass the live-mode check
SKIPs saying so — which is the honest report, and is not the same as a pass.

### `fn legacy_profile_names_the_pre_rename_gui`

# Why this test exists


* the exe path came to name a binary in the ENGINE repository, whose
  `Pass 247.0` had just deleted the only GUI crate it ever had --
  a path that can never exist;
* the diagnostic environment variable came to name one the old binary
  does not read, which leaves its tracing OFF;
* the trace prefix came to name one the old binary never prints, which
  parses to an EMPTY trace.

Each of the last two is silent. An empty trace and a build that said
nothing are the same bytes, so the falsification suite would have
reported "the old build does not exhibit the defect" -- the exact
inversion the suite exists to prevent -- with every gate green.

This asserts the shape rather than the spelling: the four fields must
carry the old stem and must NOT carry the new one. It is deliberately
a test and not a comment, because a comment is what was there.
