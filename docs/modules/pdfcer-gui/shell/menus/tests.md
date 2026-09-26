# `shell::menus::tests` — the sweeps that keep the menu document honest


## The seam, and why it is a subject rather than a cut

[`super`] is a **document**: one function returning the menus pdfcer
defines, plus the prose arguing every row of every one of them. It changes
when a menu changes.

This is a **checker**. Every test here is a sweep over *whatever*
[`super::built_in`] happens to return — every command registered, every menu
non-empty in the state it is opened in, every id also reachable from the
ribbon, the whole thing round-tripping through RON. Not one of them names a
menu it was written for. They change when the *rules about menus* change,
which is a different rate and a different reason.

⇒ That is this project's own test for a seam, applied: two subjects, two
rates of change. It is the same cut [`crate::canvas::annotnodes`] and
[`crate::shell::commands::reach::guards`] already make, and it is why the
file that had to be split is the one that grew — the document grew a menu;
the checker did not grow anything.

## What did NOT move, and why

[`super::MenuHost`]. It is ~300 lines and looks like the other obvious cut,
and it stays because it is not a separate subject: `with_condition`'s
frame-ordering argument is about *when a menu is drawn*, which is the same
subject as *what is in it*, and a reader following a row from its
`visible_when` to the condition that corrects it would have to change files
mid-sentence. Splitting the tests off costs the reader nothing, because a
sweep is read on its own or not at all.
