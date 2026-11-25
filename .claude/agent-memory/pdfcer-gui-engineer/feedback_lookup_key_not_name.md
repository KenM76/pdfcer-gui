---
name: lookup-key-not-name
description: An engine object built from a key need not carry that key back; dispatching on its name field silently matched nothing
metadata:
  type: feedback
---

An object built from a lookup key may not keep that key as its `name`. `Profile::builtin("pdfx-4").name` is `"PDF/X-4"`, the claim. The Preflight panel matched validators on `profile.name` against `profile_name()` keys, so no standard run ever reached its validator. The first cut shipped with no verdict at all. The unit tests were green and the code read correctly; only a driven check exposed it.

**Why:** a match on the wrong field fails silently. The `else` arm, which runs the plain preflight, still produces sensible-looking findings.

**How to apply:** when dispatching on a value derived from an engine object, carry the key the operator picked. Do not re-derive it from a display field. Pin it with a test that loops over every key and asserts each one reaches its branch. See [[feedback_unit_tests_call_verb]].
