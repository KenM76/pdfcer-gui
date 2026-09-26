# `text::status::waiting` — **sentences about the program being busy**

One function today. It is its own module rather than a paragraph in
[`super`] because it is a different **species** of sentence from everything
that catalog holds, and the distinction is the one `app::status`'s own header
spends forty lines making.

Every other line in that half of the bar describes **an event**: what a fill
inferred, what a move had to change, what a command declined to do. Each is
keyed on the document's edit epoch and retires when the document moves past
it.

This describes **a state** — *the picture you are looking at is not the
answer yet* — which is live for as long as its condition holds, needs no
retirement rule, and can appear for one edit and not the next depending only
on how hard the page was to draw.

★ Filed apart so that the second sentence of this species, when it comes, is
written beside the first and under the same rules rather than filed by
whichever group looked closest.


[`line_weights_off`] — *the canvas is deliberately not showing what will
print* — is a state exactly as [`page_catching_up`] is: live while a toggle
is on, retiring by itself when it goes off, with no epoch and nothing to
clear. It was written here without argument because the paragraph above had
already made it.

⚠ **They are not the same KIND of state, and the difference decides the
wording.** The first is *temporary and self-correcting* — wait, and the
picture catches up. The second is *chosen and permanent* — it will not
resolve, because the operator asked for it, and it ends only when he ends
it. So the first reassures and the second must NOT: it is rule 4's
disclosure obligation, and a soothing sentence there would be the sneaky
half of that rule wearing a friendly face.
