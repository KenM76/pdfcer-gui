# `handsign::typed` — a typed signature in a handwriting face

`OPERATOR_REQUESTS.md` **O269**, the *Type* tab: the operator types a name and
it is written in a handwriting face, as Acrobat Fill & Sign offers.

## The faces

`faces()` looks in the operating system's font folders for, in order:
Segoe Script (`segoesc.ttf`), Ink Free (`Inkfree.ttf`), Segoe Print
(`segoepr.ttf`). Each found file is proven usable by a probe subset of `Ab`
before it is offered: the subsetter refuses a face that is not TrueType or
whose licence bits forbid embedding. The list is computed once per run.

None usable: the Type tab is not drawn at all (R9). Nothing is bundled: the
faces are the operating system's, and embedding one into a signed form is the
use their licence bits govern, which the probe checks.

## The embed plan

`plan(face, name)` subsets exactly the characters of `name`. The subset tag is
keyed on the face file and the character set, so two names never share a tag
for different subsets (ISO 32000-1 §9.6.4). It fails when the face lacks a
character the name uses; the window checks this as the name is typed, so
**Place** is greyed with the subsetter's reason rather than refused after.

The name goes into the page through `AddTextRequest::with_embedded_face`, so
every reader shows the same face and nothing is substituted.

## The fit

`fit_typed(advance, ascent, descent, target)` is `handsign::fit`'s rule with
the face's full height (`ascent − descent`) standing in for a drawn mark's ink
height: size `min(0.95·W / advance, 0.95·2H / height)`; centred when the
placed height is `≤ 0.9H`, otherwise standing on the lower edge and rising
above the box; left inset `0.03W`. The result is a font size and the left end
of the baseline, y-down, in the box's units.

## The remembered copy

`typed-signature.txt` beside `settings.txt`: the face's family name on the
first line, the name on the second. A family name, not an index, because the
installed set differs between computers. A file with either line missing or
blank is ignored whole. It is written and deleted by the same
*Remember my signature on this computer* tick as the drawn copy.
