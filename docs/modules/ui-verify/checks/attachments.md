# `ui-verify/checks/attachments`

`a_file_can_be_attached_and_taken_back_out` — **the whole round trip, through
the real binary, with no native dialog answered by hand.**

# What this proves


# Why the round trip rather than "a file was attached"

Because every step of this feature is **invisible on the page**. An embedded
file changes no pixel; a screenshot of a document with three attachments and
one with none are the same picture. So the only evidence available is what
the program says it did — and the failure this guards is the one where each
step reports success and the bytes never arrive.

Four claims, in order, each of which can be true while the next is false:

| # | claim | line |
|---|---|---|
| 1 | the operator's file reached the engine | `attach-file-read name=… bytes=N` |
| 2 | the engine wrote it into the document | `attach-file page=0 n=1 epoch=…` |
| 3 | **the panel reads it back out of the session** | `attachments-panel count=1` |
| 4 | **the bytes come back out byte-for-byte** | the saved copy compares equal |

Claim 4 is the one that cannot be faked. A build that stored the path
instead of the bytes, or that wrote a truncated stream, or that saved the
wrong attachment on a multi-row list, passes 1 to 3 and fails here. It is
also the only claim in this suite that compares **file contents** rather
than a trace line, and it is worth the exception: the subject of the feature
IS the bytes.

# The two seams, and why they are two

`PDFCER_DIAG_ATTACH_PATH` answers the *attach* picker and
`PDFCER_DIAG_ATTACHMENT_SAVE_PATH` answers the *save-a-copy* picker. Sharing
one variable would make exactly this check unwritable — the round trip needs
to name an input file and an output file in one session — which is why the
shell declares two rather than reusing `PDFCER_DIAG_SAVE_PATH`.

Both are answered by the application, not by synthetic input: a native
modal is a window this harness cannot reach, and every other picker in this
suite is answered the same way.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | open the panel on a document with no attachments | `attachments-panel count=0` |
| B | click **Attach file…** | `attach-file-read name=… bytes=N`, then `attach-file page=0 n=1` |
| C | read the census again | `count=1 document_level=1` |
| D | click **Save a copy…** | `attachment-saved bytes=N renamed=…` |
| E | compare the saved bytes with the file attached in B | identical |
| F | click **Remove** | `detach-file …`, then `count=0` |

# Phase A is a precondition, not a formality

`count=0` is asserted before anything is attached, so phase C's `count=1`
cannot be satisfied by a fixture that arrived carrying an attachment — the
same rule `a_note_can_be_written_onto_a_shape_that_exists` follows for
`with_note`. If the fixture does carry one, this SKIPS with the reason
rather than measuring the wrong thing.
