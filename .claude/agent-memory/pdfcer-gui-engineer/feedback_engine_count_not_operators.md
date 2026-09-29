---
name: feedback-engine-count-not-operators
description: A receipt quoting the engine verb's own count can read 0 in exactly the case the command exists for — count what the operator cares about
metadata:
  type: feedback
---

A receipt should count the outcome the operator cares about, not what the engine verb reports. The purge-passwords receipt quoted `PasswordPurgeOutcome::fields_purged`. On the one fixture built for the feature's reason to exist (a value only in a superseded revision), the verb purged 0 fields: the current field was already empty, and the single-version rewrite removed the value. The receipt would have said "0 fields cleared" while removing the password.

**Why:** an engine verb counts its own act; a pipeline (scan, verb, rewrite) has several acts, and the operator's outcome can be done entirely by a different stage than the one whose count you quoted.

**How to apply:** when a command chains stages, derive the receipt's headline number from a before/after measurement (here, the scan counts) and drive a fixture shaped like the command's motivating case, not a convenient one. Related: [[a-check-whose-input-is-chosen-for-convenience-tests-the-assertion]], [[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]].
