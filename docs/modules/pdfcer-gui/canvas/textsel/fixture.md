# `canvas::textsel::fixture` — the rotated-text page these rules are tested on

Test-only. Builds `fixtures/rotated-text.pdf`: one US-Letter page carrying
the same sentence set five times, at 0°, 90°, 180°, 270° and 30°.

## Why a synthetic page and not the operator's own drawing

The report that started this work names a real file — `SW41177.pdf`, whose
title block carries a vertical SolidWorks path stamp — and that file is what
every claim here was *measured* against. It is deliberately **not committed**
as a fixture: it is a customer drawing, and the standing rule about
SolidWorks-derived work product is that it does not enter a repository that
could be published. It stays at `D:\Dev\temp\pdfcer\SW41177.pdf` and is used
by hand and by `ui-verify --pdf`.

So the committed fixture has to stand in for it, and standing in means
reproducing the *mechanism* rather than the appearance:


## The generator is a test, and the fixture is committed

[`regenerate`] is `#[ignore]`d and writes the file; the fixture is checked
in beside it. That is the convention `crate::ocr::fixture` established and
the reasons are its reasons: a fixture generated at test time is a fixture
whose bytes no one has ever looked at, and `ui-verify` needs a path on disk
that exists before `cargo test` has run.

## Why the bytes are written by hand

`pdfcer-core` can author text, and using it here would make the fixture a
product of the same engine whose extraction is under test — so a change that
moved both would still pass. Hand-written content streams state the text
matrices literally, which is the one thing this fixture is *for*: `0 1 -1 0
100 300 Tm` is the input, in the file, readable, and not the output of
anything.

## Item notes

### `const LINES`

`[a b c d e f]` is §9.4.2's `Tm`. The rotation lives in `a b c d`; `e f` is
where the string starts. Every rotated entry is **capitals**, for the reason
in the module header.

### `fn regenerate_the_rotated_text_fixture`

`#[ignore]`d: it writes into the repository, and a test that edits the
tree it is run from should be an act rather than a side effect. Run it
deliberately —
`cargo test -p pdfcer-gui regenerate_the_rotated_text_fixture -- --ignored`
— and commit what it produces.

### `fn the_committed_fixture_matches_its_generator`

Without this, the generator and the file could drift and every test that
reads the fixture would still pass — while the header above, which
explains the fixture in terms of the generator's `LINES` table, would
have quietly become fiction.
