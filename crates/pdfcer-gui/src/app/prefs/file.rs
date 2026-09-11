//! # `app::prefs::file` — the on-disk format
//!
//! The parser and the writer, and nothing else. Split out of `prefs/mod.rs`
//! under **R2** on 2026-09-09, when the *Zoom* control's `find_zoom_on_jump`
//! key took that file to 1,502 lines — two over the ceiling.
//!
//! ## Why these two and not some other seam
//!
//! [`Prefs::parse`] and [`Prefs::write_to_string`] are **two spellings of one
//! vocabulary**. Every key exists twice: once as an arm of the parser's match
//! and once as a `push_str` in the writer, and the pair has to agree about the
//! key's name, its spelling of each value, and its default. The crate already
//! treats them as one subject — `the_writer_emits_no_key_the_parser_rejects`
//! and `every_preference_round_trips_through_the_file` each assert a property
//! of the *pair*, not of either half — so the file boundary now says what the
//! tests already said.
//!
//! The practical consequence, and the reason to prefer this seam over "move
//! the writer only": **adding a preference is one edit to one file.** A split
//! that put the two halves in different files would make the commonest change
//! to this module a two-file change, and a two-file change is how a writer
//! comes to emit a key its own parser rejects.
//!
//! ## What stayed in `mod.rs`
//!
//! The [`Prefs`](super::Prefs) struct itself, its `Default`, the nested
//! preference groups, [`PrefNote`](super::PrefNote), `load`, `save` and
//! `seed_view`. Those are the *model* and the *lifecycle*; this file is only
//! the transcription between the model and a text file.
//!
//! A second inherent `impl Prefs` block here is legal because this module is
//! in the same crate as the type — the methods are the same public API at the
//! same paths they were at before the split, and no caller changed.

use super::*;

impl Prefs {
    /// Parse, with per-key recovery.
    ///
    /// # The `match` is the file format
    ///
    /// There is no key table, no `HashMap` and no derive: every key this build
    /// understands is an arm below, and the `_` arm reports everything else as
    /// [`PrefNote::UnknownKey`] and **keeps it in the file**. That last part is
    /// what makes it safe for an operator to run two versions of pdfcer out of
    /// one `userdata` folder — the older one does not delete the newer one's
    /// settings on its next Save, because [`Self::write_to_string`] writes what
    /// this build knows and the loader never rewrites on load.
    ///
    /// The honest limit of that: an unknown key survives until the operator
    /// presses Save in the older build, which writes a fresh file from the
    /// fields it has. Preserving unknown lines across a *write* would mean
    /// carrying them on `Prefs`, and a struct holding values it cannot use is
    /// worse than the narrow case it protects.
    #[must_use]
    pub fn parse(text: &str) -> (Self, Vec<PrefNote>) {
        let mut prefs = Self::default();
        let mut notes = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((key, value)) = trimmed.split_once('=') else {
                notes.push(PrefNote::Malformed { line });
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            if key.is_empty() {
                notes.push(PrefNote::Malformed { line });
                continue;
            }
            match key {
                // ui-text-exempt: a file KEY, parsed out of preferences.txt.
                "page_cache" => match PageCache::from_key(value) {
                    Some(c) => prefs.page_cache = c,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "render_quality" => match RenderQuality::from_key(value) {
                    Some(q) => prefs.render_quality = q,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // ui-text-exempt: a file KEY, matched literally.
                //
                // ★ Taken verbatim, with no validation against the engine's
                // standard list. A hand-edited or newer-pdfcer id that this
                // build does not know is not an error: `preset::live_choice`
                // asks `still_holds`, which answers `false` for an unknown id
                // and falls back to the derived reading. Rejecting it here
                // would turn "a standard this build has not heard of" into a
                // parse note, which is the wrong report — nothing is wrong with
                // the file.
                //
                // Empty means "none chosen", so a file written by a build that
                // had no choice to record round-trips to `None` rather than to
                // `Some("")`.
                "chosen_standard" => {
                    prefs.chosen_standard =
                        (!value.trim().is_empty()).then(|| value.trim().to_owned());
                }
                // ui-text-exempt: a file KEY, matched literally.
                // ★ Trimmed, and an all-whitespace value is the same as
                // absent: a name of three spaces would write a `/T` that
                // renders as an empty author column in every reviewer UI,
                // which is worse than no key at all because it claims one.
                "author_name" => prefs.author_name = value.trim().to_owned(),
                // ui-text-exempt: a file KEY, matched literally.
                // ★ Trimmed, like its neighbour and for a related reason: a
                // path with a trailing space is a path that does not exist, and
                // the failure would present as "the setting does nothing".
                // `resolve` trims again at the point of use, because this file
                // is not the only way the value arrives.
                "acrobat_path" => prefs.acrobat_path = value.trim().to_owned(),
                // ui-text-exempt: a file KEY, matched literally.
                // ★ Trimmed for its neighbour's reason exactly: a path with a
                // trailing space is a path that does not exist, and the failure
                // presents as "the trust setting does nothing" rather than as
                // "that file is not there". `crate::trust::locate` trims again,
                // because this file is not the only way the value arrives.
                "acrobat_trust_store_path" => {
                    prefs.acrobat_trust_store_path = value.trim().to_owned();
                }
                // ui-text-exempt: a file KEY, matched literally.
                // ★ A REPEATED key: every occurrence appends. That is why this
                // arm pushes where every other arm assigns, and it is the one
                // place the file's grammar is not "one key, one value".
                // `fonts::add` applies the cap and the duplicate rule, so a
                // hand-edited file with twenty entries is bounded the same way
                // the picker is.
                "font_folder" => {
                    if let Some(path) = fonts::parse_one(value) {
                        fonts::add(&mut prefs.font_folders, &path);
                    }
                }
                // ui-text-exempt: a file KEY, parsed out of preferences.txt.
                "use_os_fonts" => match value.trim() {
                    // ui-text-exempt: file VALUES, parsed not displayed.
                    "true" => prefs.use_os_fonts = true,
                    "false" => prefs.use_os_fonts = false,
                    // ★ Reported rather than silently defaulted, and it keeps
                    // the operator's OLD value: a hand-edited `use_os_fonts =
                    // yes` is somebody trying to switch it ON, and a parser
                    // that answered by turning it off would be the opposite of
                    // what they wrote, with no sentence anywhere.
                    _ => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "max_zoom_percent" => match value.parse::<f32>() {
                    Ok(pct) if pct.is_finite() => {
                        let clamped = pct.clamp(MIN_MAX_ZOOM_PERCENT, MAX_MAX_ZOOM_PERCENT);
                        if (clamped - pct).abs() > f32::EPSILON {
                            notes.push(PrefNote::Clamped {
                                key: key.to_owned(),
                                value: value.to_owned(),
                                line,
                            });
                        }
                        prefs.max_zoom_percent = clamped;
                    }
                    // ★ A non-finite value is a BadValue rather than a clamp.
                    // `inf` would propagate into a scroll extent and blank the
                    // canvas, and reporting it as "clamped" would imply the
                    // operator wrote something reasonable.
                    _ => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "zoom_settle_ms" => match value.parse::<u64>() {
                    Ok(ms) => {
                        let clamped = ms.clamp(MIN_SETTLE_MS, MAX_SETTLE_MS);
                        if clamped != ms {
                            notes.push(PrefNote::Clamped {
                                key: key.to_owned(),
                                value: value.to_owned(),
                                line,
                            });
                        }
                        prefs.zoom_settle_ms = clamped;
                    }
                    Err(_) => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "ui_scale" => match value.parse::<f32>() {
                    // ★ `is_finite` first, and it is not defensive padding.
                    // `"nan"` and `"inf"` both parse successfully as `f32`, so
                    // without this a hand-edited `ui_scale = nan` would reach
                    // `normalise_ui_scale`, where `clamp` propagates NaN rather
                    // than rejecting it — and a NaN zoom factor is a window
                    // that draws nothing. It is reported as a bad value, which
                    // is what it is, rather than clamped to an end the operator
                    // did not name.
                    Ok(raw) if raw.is_finite() => {
                        let scale = chrome::normalise_ui_scale(raw);
                        // Reported when the file's value is not one the control
                        // can produce — see `normalise_ui_scale` on why the
                        // rounding happens at all. The epsilon is a tenth of a
                        // step, comfortably finer than any difference that
                        // matters and coarse enough that float noise from the
                        // round trip does not raise a note on a clean file.
                        if (scale - raw).abs() > UI_SCALE_STEP / 10.0 {
                            notes.push(PrefNote::Clamped {
                                key: key.to_owned(),
                                value: value.to_owned(),
                                line,
                            });
                        }
                        prefs.ui_scale = scale;
                    }
                    _ => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // ★ An unreadable token leaves the DEFAULT in place and is
                // REPORTED, exactly as every sibling arm does. Swallowing it
                // silently was the first version and was wrong: a token this
                // build cannot read is either a file from a newer build or a
                // hand-edit with a typo, and both are worth a note. The load
                // still succeeds, so one bad line never costs the operator
                // every other setting in the file.
                "paste_chords" => match PasteChords::from_key(value) {
                    Some(o) => prefs.paste_chords = o,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // ★ O80. The absent key and the present-but-unparseable key
                // are different: absent leaves `None` (he has not said), and
                // a bad value is a note, exactly as every other key here does
                // it — a typo in a hand-edited file must not silently become
                // a preference.
                "default_page_display" => match crate::viewer::PageDisplay::from_id(value) {
                    Some(d) => prefs.default_page_display = Some(d),
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // ★ `opening::bool_from_key`, which is the file's existing
                // vocabulary — `true`/`false` and nothing else. A key that also
                // accepted `yes` would be a second dialect in one file, and the
                // strictness is deliberate: that function's own header records
                // why a lenient reading here is worse than a reported bad
                // value.
                "shade_form_fields" => match opening::bool_from_key(value) {
                    Some(on) => prefs.shade_form_fields = on,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "wheel_paging" => match WheelPaging::from_key(value) {
                    Some(w) => prefs.wheel_paging = w,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "opening_fit" => match OpeningFit::from_key(value) {
                    Some(f) => prefs.opening_fit = f,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // The three overlays share one parse shape and differ only in
                // which field they land in, so the destination is picked first
                // and the reading is written once. Three near-identical arms is
                // how the fourth overlay gets a subtly different parser.
                // O70. Its own arm rather than joining the three-key arm
                // below, because it is not one of the three overlays and a
                // reader meeting it inside that pattern would go looking for a
                // fourth chrome field.
                "smart_select" => match opening::bool_from_key(value) {
                    Some(on) => prefs.smart_select = on,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // O163, 2026-09-09. Its own arm for the same reason
                // `smart_select` has one: it is not a member of any of the
                // key families below, and a reader meeting it inside a shared
                // pattern would go looking for a family it does not have.
                "find_zoom_on_jump" => match opening::bool_from_key(value) {
                    Some(on) => prefs.find_zoom_on_jump = on,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // ui-text-exempt: a file KEY, matched literally.
                // O173, 2026-09-10. Its own arm for its neighbour's reason: it
                // belongs to no key family, and a reader meeting it inside a
                // shared pattern would go looking for the family it does not
                // have. A bad value is a note rather than a silent `false`,
                // because silently suppressing a question is indistinguishable
                // from never having had one.
                "ask_default_app" => match opening::bool_from_key(value) {
                    Some(on) => prefs.ask_default_app = on,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // The two auto-hide settings, 2026-09-05. One arm each rather
                // than a shared pattern: they are two independent surfaces and
                // a reader meeting one inside a joint arm would reasonably
                // expect the other to move with it.
                "ribbon_auto_hide" => match opening::bool_from_key(value) {
                    Some(on) => prefs.ribbon_auto_hide = on,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "rail_auto_hide" => match opening::bool_from_key(value) {
                    Some(on) => prefs.rail_auto_hide = on,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "show_rulers" | "show_grid" | "show_guides" => {
                    let target = match key {
                        "show_rulers" => &mut prefs.chrome.rulers,
                        "show_grid" => &mut prefs.chrome.grid,
                        // Exhaustive by the arm's own pattern; the compiler
                        // cannot see that, and a `_` here would silently absorb
                        // a fourth overlay added to the pattern above and never
                        // given a field.
                        _ => &mut prefs.chrome.guides,
                    };
                    match opening::bool_from_key(value) {
                        Some(on) => *target = on,
                        None => notes.push(PrefNote::BadValue {
                            key: key.to_owned(),
                            value: value.to_owned(),
                            line,
                        }),
                    }
                }
                // ★★ The print group — thirteen keys, delegated whole.
                //
                // ⚠ This is the ONE family whose parser is not an arm in this
                // match, and the reason is the rule this file's header states
                // rather than an exception to it: the parser and the writer
                // must stay together, and for this group they live together in
                // `printing.rs` beside the type, its defaults and its token
                // vocabulary. Adding a print preference is still one edit to
                // one file. See `printing::parse_key`'s own header for the
                // full argument.
                //
                // The `_` arm below is reached only when this returns
                // `NotMine`, so an unknown key is still reported exactly once
                // and a bad print value is reported against its own key rather
                // than as a spelling mistake.
                //
                // ★ A SECOND delegated family joined it on 2026-09-11:
                // `off_page.<mode>`, whose key is a PREFIX and so cannot be a
                // literal arm above at all. The two are chained rather than
                // nested — `offpage` first, falling through to `printing` on
                // `NotMine` — because both speak the same `KeyOutcome` and the
                // single `match` below stays the one place a note is made. A
                // second `match` per family is how one of them comes to report
                // a bad value as an unknown key.
                _ => {
                    let outcome = match offpage::parse_key(&mut prefs.off_page, key, value) {
                        printing::KeyOutcome::NotMine => {
                            printing::parse_key(&mut prefs.print, key, value)
                        }
                        mine => mine,
                    };
                    match outcome {
                        printing::KeyOutcome::Accepted => {}
                        printing::KeyOutcome::BadValue => notes.push(PrefNote::BadValue {
                            key: key.to_owned(),
                            value: value.to_owned(),
                            line,
                        }),
                        printing::KeyOutcome::NotMine => notes.push(PrefNote::UnknownKey {
                            key: key.to_owned(),
                            line,
                        }),
                    }
                }
            }
        }
        (prefs, notes)
    }

    /// The file's whole text.
    ///
    /// Commented, because the file is meant to be opened in a text editor and
    /// a bare `render_quality = faster` tells an operator nothing about what
    /// else they could write. Same posture as the engine's store, which spends
    /// a comment block per key for exactly this reason.
    #[must_use]
    pub fn write_to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "# pdfcer display preferences\n\
             #\n\
             # How pdfcer draws, as distinct from how it reads and writes PDFs —\n\
             # those live in settings.txt beside this file. Plain text, one\n\
             # `key = value` per line, # for comments. An unknown key is reported\n\
             # and kept, not deleted, and a value pdfcer cannot read falls back for\n\
             # that key alone.\n\
             #\n\
             # KEEP THIS FOLDER when you update pdfcer.\n\
             \n\
             # How sharply a page is drawn: faster | normal | sharper\n\
             # faster  = three quarter scale. Softer lines, quicker on a big sheet.\n\
             # normal  = one pixel per screen pixel. The shipped answer.\n\
             # sharper = one and a half times. For small text over fine linework.\n",
        );
        // ui-text-exempt: a file KEY, written into preferences.txt and parsed
        // back out of it. Never displayed — the operator meets this setting as
        // "How sharply pages are drawn" in the Settings window.
        out.push_str("render_quality = ");
        out.push_str(self.render_quality.key());
        out.push('\n');
        out.push_str(
            "\n\
             # How much memory pdfcer may use to remember pages it has already\n\
             # drawn, so that scrolling back to one does not draw it again:\n\
             #   small   = about 190 MB. What pdfcer used before 2026-08-19.\n\
             #   medium  = about 490 MB.\n\
             #   large   = about 980 MB. The shipped answer.\n\
             #   maximum = about 1950 MB. A whole drawing set kept resident.\n\
             # Pages furthest from the one you are looking at are dropped first.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("page_cache = ");
        out.push_str(self.page_cache.key());
        out.push('\n');
        out.push_str(
            "\n\
             # How long a zoom must stop changing before the page is redrawn\n\
             # sharply, in milliseconds. 20 to 1000. Lower feels more immediate\n\
             # and redraws more; higher swallows a whole wheel gesture.\n",
        );
        out.push_str(
            "\n\
             # The highest zoom you can reach, as a percentage. 800 is the\n\
             # shipped default and is what earlier versions allowed.\n\
             #\n\
             # Above roughly 1000% pdfcer stops drawing the whole page and draws\n\
             # only what is on screen, because a whole-page image would exceed\n\
             # what can be rasterized. Panning is free below that point and\n\
             # costs a redraw above it -- so this is also the dial for trying\n\
             # the two out against each other.\n\
             # 10 to 1000000000000.\n",
        );
        out.push_str(&fonts::write_block(&self.font_folders));
        out.push_str(&fonts::write_os_flag(self.use_os_fonts));
        // ui-text-exempt: a file KEY, as above.
        out.push_str("max_zoom_percent = ");
        out.push_str(&format_percent(self.max_zoom_percent));
        out.push('\n');
        out.push_str(
            "\n\
             # How long a zoom must stop changing before the page is redrawn\n\
             # sharply, in milliseconds. 20 to 1000. Lower feels more immediate\n\
             # and redraws more; higher swallows a whole wheel gesture.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("zoom_settle_ms = ");
        out.push_str(&self.zoom_settle_ms.to_string());
        out.push('\n');
        out.push_str(
            "\n\
             # How big pdfcer's own menus, buttons and labels are drawn, as a\n\
             # MULTIPLIER on your Windows display setting -- not a replacement\n\
             # for it. 0.8 to 2.0, in steps of 0.05. A value of 1 means exactly\n\
             # what Windows asked for. Changes the program, never the page.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("ui_scale = ");
        // Two decimals: the step is 0.05, so two places represent every value
        // the control can produce exactly and none that it cannot. The default
        // `f32` formatting would write `1` for 1.0 and `1.1500001` for a value
        // that arrived through a slider, and the second of those is a number no
        // operator should have to read in a file they are invited to edit.
        out.push_str(&format!("{:.2}", self.ui_scale));
        out.push('\n');
        out.push_str(
            // ui-text-exempt: file comments, never displayed in the UI.
            "\n\
             # The rendering standard you picked in Settings, if you picked one.\n\
             # Blank means none. This records WHAT YOU ASKED FOR; the settings\n\
             # it implies are written above and are what actually renders. Most\n\
             # of the PDF/X and PDF/A standards ask a renderer for exactly the\n\
             # same thing, so this is the only place your particular choice is\n\
             # kept.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("chosen_standard = ");
        out.push_str(self.chosen_standard.as_deref().unwrap_or(""));
        out.push('\n');
        out.push_str(
            "\n\
             # Your name, written into every comment you author (the PDF calls\n\
             # it the annotation's title). Blank leaves comments anonymous,\n\
             # which is legal and is what pdfcer did before this existed.\n\
             # It goes into files you send to other people, so it is yours to\n\
             # set rather than something pdfcer guesses from your Windows login.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("author_name = ");
        out.push_str(&self.author_name);
        out.push('\n');
        out.push_str(
            "\n\
             # Where Acrobat is — OPERATOR_REQUESTS.md O122. Leave this blank\n\
             # and pdfcer asks Windows itself, preferring Acrobat Pro over\n\
             # Acrobat Reader. Fill it in with the full path to the program to\n\
             # point at a particular installation: a portable copy, a second\n\
             # version, or one Windows has not been told about. If nothing is\n\
             # found and nothing is set here, the Acrobat button beside\n\
             # Read / Review / Edit is simply not shown.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("acrobat_path = ");
        out.push_str(&self.acrobat_path);
        out.push('\n');
        out.push_str(
            "\n\
             # Where Acrobat's downloaded trust list is — the AATL + EU Trusted\n\
             # Lists file Acrobat writes as addressbook.acrodata. Leave this\n\
             # blank and pdfcer looks in the usual per-user locations. Fill it\n\
             # in to point at a particular store. This is only a LOCATION:\n\
             # whether pdfcer may read it at all is acrobat_trust_store in\n\
             # settings.txt, which is off until you turn it on.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("acrobat_trust_store_path = ");
        out.push_str(&self.acrobat_trust_store_path);
        out.push('\n');
        out.push_str(
            "\n\
             # ---------------------------------------------------------------\n\
             # What you see when a document first opens. Both of these apply to\n\
             # the NEXT document opened, not to the one already on screen.\n\
             # ---------------------------------------------------------------\n\
             \n\
             # How the first page is sized: page | width | height | actual\n\
             # page   = the whole page fits the window. The shipped answer.\n\
             # width  = the full width fits; the bottom may run off screen.\n\
             # height = the full height fits; the side may run off screen.\n\
             # actual = one page point per screen point, whatever that shows.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("opening_fit = ");
        out.push_str(self.opening_fit.key());
        out.push('\n');
        out.push_str(
            // ui-text-exempt: file comments, never displayed in the UI.
            "\n\
             # What the mouse wheel does on a single page: scroll | flip\n\
             # scroll = move within the sheet. The shipped answer.\n\
             # flip   = turn to the next or previous page.\n\
             # Ignored under a continuous display mode, where the wheel\n\
             # scrolls the whole document by definition.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("paste_chords = ");
        out.push_str(self.paste_chords.key());
        out.push('\n');
        // ui-text-exempt: settings-file COMMENT text. Read in a text editor,
        // never rendered by this program.
        out.push_str(
            "# shade_form_fields: wash the fillable fields so you can see what\n\
             # accepts typing, the way Acrobat does. On screen only - it never\n\
             # reaches a print, an export or a saved file. true or false.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("shade_form_fields = ");
        out.push_str(if self.shade_form_fields {
            "true"
        } else {
            "false"
        });
        out.push('\n');
        // ui-text-exempt: a file KEY, as above.
        out.push_str("wheel_paging = ");
        out.push_str(self.wheel_paging.key());
        out.push('\n');
        // ★★ O80. Written only when he has stated one — an absent key is how
        // "fall through to the per-mode default" is spelled on disk, and
        // emitting a value for `None` would make the file claim a preference
        // nobody expressed. The comment goes above it either way, so somebody
        // reading the file finds out the setting exists even when it is unset.
        out.push_str(
            "\n\
             # default_page_display: how a document this program has never\n\
             # opened is laid out. Values:\n\
             #   single | continuous | facing | facing_continuous\n\
             # A document you HAVE opened remembers its own arrangement and\n\
             # ignores this. Leave the line out entirely to let each mode\n\
             # choose -- Read opens continuous, everything else single page.\n",
        );
        if let Some(display) = self.default_page_display {
            // ui-text-exempt: a file KEY, as above.
            out.push_str("default_page_display = ");
            out.push_str(display.id());
            out.push('\n');
        }
        out.push_str(
            "\n\
             # smart_select: true | false. With this on, clicking a drawing\n\
             # that was placed as one piece -- a title block, a stamped\n\
             # detail -- selects the whole piece, and double-clicking goes\n\
             # inside it. With it off a click selects the individual line\n\
             # under the pointer, which is how pdfcer behaved before\n\
             # 2026-08-31.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("smart_select = ");
        out.push_str(opening::bool_key(self.smart_select));
        out.push('\n');
        out.push_str(
            "\n\
             # find_zoom_on_jump: true | false. With this on, going to a\n\
             # search hit re-applies Fit page or Fit width to whatever page\n\
             # the hit is on -- so on a set whose sheets are different sizes,\n\
             # the zoom changes as you step through the results. With it off\n\
             # the page still changes and the hit is still highlighted, but\n\
             # the view keeps the size you were reading at and the zoom\n\
             # readout shows a percentage instead of a fit. The checkbox is\n\
             # in the find bar's Options menu, named Zoom.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("find_zoom_on_jump = ");
        out.push_str(opening::bool_key(self.find_zoom_on_jump));
        out.push('\n');
        out.push_str(
            "\n\
             # ask_default_app: true | false. With this on, pdfcer offers\n\
             # once, on startup, to add itself to the list of programs\n\
             # Windows uses for PDF files. Ticking Do not ask me again in\n\
             # that offer sets this to false. It only governs the\n\
             # QUESTION -- the button that does it stays at the top of\n\
             # Settings either way, so nothing is lost by turning the\n\
             # offer off. Note that no program can make itself the\n\
             # default PDF viewer on Windows 10 or 11: pdfcer can only\n\
             # put itself in the list, and Windows then asks you to\n\
             # confirm.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("ask_default_app = ");
        out.push_str(opening::bool_key(self.ask_default_app));
        out.push('\n');
        out.push_str(
            "\n\
             # ribbon_auto_hide / rail_auto_hide: true | false. With one of\n\
             # these on, that strip stays out of the way until you move the\n\
             # pointer onto it, and then it appears OVER the drawing rather\n\
             # than pushing it down or sideways -- so nothing you were about\n\
             # to click moves. The ribbon keeps its row of tab names either\n\
             # way, and the rail keeps a narrow marked edge, so there is\n\
             # always somewhere to put the pointer to get the strip back.\n",
        );
        // ui-text-exempt: file KEYS, as above.
        out.push_str("ribbon_auto_hide = ");
        out.push_str(opening::bool_key(self.ribbon_auto_hide));
        out.push('\n');
        out.push_str("rail_auto_hide = "); // ui-text-exempt: a file KEY, as above.
        out.push_str(opening::bool_key(self.rail_auto_hide));
        out.push('\n');
        out.push_str(
            "\n\
             # Which overlays are already switched on: true | false.\n\
             # Rulers take a strip off the top and left of the drawing area.\n\
             # Guides are dragged OUT OF a ruler, so placing one needs both\n\
             # show_guides and show_rulers on.\n",
        );
        // ui-text-exempt: a file KEY, as above. Three keys, written together
        // under one comment block because they are one setting in the window
        // and a reader meeting them apart would not know they interlock.
        for (key, value) in [
            ("show_rulers", self.chrome.rulers),
            ("show_grid", self.chrome.grid),
            ("show_guides", self.chrome.guides),
        ] {
            out.push_str(key);
            // ui-text-exempt: the file format's own `key = value` separator,
            // never displayed. The three single-key writes above spell it into
            // their key literal; a loop cannot, so it is its own push.
            out.push_str(" = ");
            out.push_str(opening::bool_key(value));
            out.push('\n');
        }

        // The print group's whole block — see `printing::write_block`, which
        // sits beside the parser that reads it back.
        printing::write_block(&self.print, &mut out);

        // Off-page display, one answer per ribbon mode. Same arrangement and
        // same reason: `offpage::write_block` sits beside the parser that
        // reads it back. Its comment block is written even when the operator
        // has answered nothing, because a preference nobody can discover is a
        // preference nobody has.
        offpage::write_block(&self.off_page, &mut out);

        out
    }
}
