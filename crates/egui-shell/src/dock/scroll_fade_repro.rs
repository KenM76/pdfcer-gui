//! **Headless reproduction of the right-dock wobble** — an exact-size
//! `egui::Panel` holding a solid-style `ScrollArea` whose scroll bar is
//! fading in.
//!
//! Written 2026-09-09 after `dock/overflow_probe.rs` named the overflowing
//! widget on its first read: the Comments list's vertical scroll bar, whose
//! rect ran to `1400.4` and then `1400.1` on consecutive frames — a decay,
//! so an animation. This file asks egui the question directly, with no
//! application in the way: **does a solid-style scroll bar fading in push
//! an exact-size panel's rect past the window edge?** Every number printed
//! here is egui's own, from a `Context` driven with a fixed clock, so the
//! per-frame profile is reproducible to the digit.
//!
//! The test is not a regression guard — it is the measurement the RAG
//! entry cites. It asserts only the fact the probe saw, so that an egui
//! upgrade that fixes the behaviour turns it red and tells the next reader
//! that the workaround in `Dock::show` can go.

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2, vec2};

    /// One frame of the reproduction: a 1400×900 window, a right panel of
    /// exact width 320, a solid 10-pt scroll bar, and `rows` labels of 20 pt
    /// each in a vertical `ScrollArea`. Returns the panel's response rect.
    fn frame(ctx: &egui::Context, t: f64, rows: usize) -> Rect {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1400.0, 900.0))),
            time: Some(t),
            ..Default::default()
        };
        ctx.begin_pass(input);
        let panel_rect = {
            let ctx = ctx.clone();
            let shown = egui::Panel::right(egui::Id::new("repro-right"))
                .exact_size(320.0)
                .resizable(false)
                .show_separator_line(false)
                .frame(egui::Frame::NONE)
                .show(
                    &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::new("repro-root"),
                        egui::UiBuilder::new().max_rect(ctx.content_rect()),
                    ),
                    |ui| {
                        let mut scroll = egui::style::ScrollStyle::solid();
                        scroll.bar_width = 10.0;
                        ui.style_mut().spacing.scroll = scroll;
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for i in 0..rows {
                                let (_, r) = ui.allocate_space(vec2(ui.available_width(), 20.0));
                                let _ = (i, r);
                            }
                        });
                    },
                );
            shown.response.rect
        };
        let _ = ctx.end_pass();
        panel_rect
    }

    /// The content is short for five frames (no bar), then long (bar fades
    /// in over `animation_time`). Every frame's panel rect is printed; the
    /// assertion is the one fact the driven probe saw.
    #[test]
    fn a_solid_bar_fading_in_overflows_an_exact_size_panel() {
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|s| s.animation_time = 0.1);
        let mut worst = 0.0_f32;
        let mut profile = Vec::new();
        for n in 0..40 {
            let t = n as f64 / 60.0;
            let rows = if n < 5 { 5 } else { 100 };
            let r = frame(&ctx, t, rows);
            let overflow = r.max.x - 1400.0;
            profile.push(format!(
                "frame {n:2} rows {rows:3} panel [{:.2} .. {:.2}] overflow {overflow:+.3}",
                r.min.x, r.max.x
            ));
            worst = worst.max(overflow);
        }
        eprintln!("{}", profile.join("\n"));
        assert!(
            worst > 0.05,
            "no overflow measured; egui may have fixed this — retire the workaround. Profile:\n{}",
            profile.join("\n")
        );
    }
}
