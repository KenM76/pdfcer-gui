# `pdfcer-gui-base/sign/timestamp`

The RFC 3161 time-stamping server a PAdES B-T signature asks, and whether this
build can ask one at all.

# Contract

- `available()` is `cfg!(feature = "timestamp")` and is the only way a surface
  learns the capability exists (R8). The Sign window draws its server field
  exactly when it is true; a build without the feature has no field, not a
  field that fails.
- `requested(field)` trims the field; blank is `None`, meaning an ordinary B-B
  signature. Anything else is a request for B-T, and a malformed address is
  refused by the transport by name rather than signed without the timestamp.
- `authority_for(url)` is `None` without the feature. `sign::prepare` turns
  that into `PrepareFailure::TimestampUnavailable`, reachable only from a
  remembered preference in a stripped build.
- `HttpAuthority` is the engine's `TimestampAuthority` over
  `pdfcer_fetch::post_time_stamp_query_with`, bounded by `TIMEOUT` (30 s). A
  server that accepts the connection and never answers costs the bound, not
  forever.

# Why the failure is never downgraded

A requested timestamp that fails surfaces as `SignApplyError::Timestamp` and
nothing is written. Retrying as B-B would hand the operator an untimestamped
signature he asked to be timestamped, and the difference is invisible in the
file until a verifier looks. The sentence names the remedy: clear the field.

# Network contact is explicit

The field defaults to empty and is filled only by what the operator typed or
last signed with (`Prefs::sign_timestamp_server`, saved when a signing is
sent). pdfcer never proposes a server. What is sent is the RFC 3161 request —
a hash of the signature value — never the document.

# Known limit

The round trip runs on the UI thread, so the window stops repainting for up
to `TIMEOUT` while the server answers. Moving it to a worker needs the signing
action split around `EditSession::sign_with_timestamp`, which holds `&mut` to
the session for the whole call.
