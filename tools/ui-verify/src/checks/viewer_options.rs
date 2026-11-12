//! Reaching a 3D viewer control that lives in its options menu.
//!
//! The viewer keeps its views, look, navigation, window and picture
//! settings behind one glyph at the picture's top-left
//! (`dialogs::model3d::options::REGION_OPTIONS`), on tabs. A control in that
//! menu is drawn only while the menu is open on its tab, so a check that
//! wants one opens the menu and walks the tabs to it. The menu stays open
//! while its switches are used, and covers part of the picture until it is
//! closed.

use crate::checks::driving::declared_in;
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::LRect;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;

/// The options glyph: `dialogs::model3d::options::REGION_OPTIONS`.
pub const OPTIONS: &str = "model3d.options";

/// `name`'s rect and viewport, opening the options menu when it is not
/// drawn; `None` when it is not drawn with the menu open either.
pub fn reach(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<Option<(LRect, Option<String>)>> {
    if let Some(found) = declared_in(&session.trace()?, ui_rect, name) {
        return Ok(Some(found));
    }
    let Some((glyph, vp)) = declared_in(&session.trace()?, ui_rect, OPTIONS) else {
        return Ok(None);
    };
    if declared_in(&session.trace()?, ui_rect, MENU).is_none() {
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(glyph))?;
        session.settle(20);
    }
    // The menu shows one tab at a time and opens on the last one shown:
    // look there first, then on each tab in turn.
    if let Some(found) = scroll_to(session, pointer, ui_rect, name)? {
        return Ok(Some(found));
    }
    for tab in TABS {
        let Some((rect, tvp)) = declared_in(&session.trace()?, ui_rect, tab) else {
            return Ok(None);
        };
        pointer.click_in(session, tvp.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(12);
        if let Some(found) = scroll_to(session, pointer, ui_rect, name)? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

/// The options menu's tabs: `dialogs::model3d::options::REGION_TAB` and
/// each tab's word.
const TABS: [&str; 6] = [
    "model3d.options.tab.view",
    "model3d.options.tab.look",
    "model3d.options.tab.moving",
    "model3d.options.tab.picture",
    "model3d.options.tab.measure",
    "model3d.options.tab.controls",
];

/// `name` on the open tab, wheeling its rows down until it shows or they
/// end: a row scrolled out of the menu publishes no rect. A tab opens
/// scrolled to the top.
fn scroll_to(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<Option<(LRect, Option<String>)>> {
    for _ in 0..SCROLLS {
        let trace = session.trace()?;
        if let Some(found) = declared_in(&trace, ui_rect, name) {
            return Ok(Some(found));
        }
        let Some((menu, mvp)) = declared_in(&trace, ui_rect, MENU) else {
            return Ok(None);
        };
        pointer.wheel_in(session, mvp.as_deref(), WindowPoint::centre_of(menu), -1.0)?;
        session.settle(12);
    }
    Ok(declared_in(&session.trace()?, ui_rect, name))
}

/// Wheel notches [`reach`] spends looking down a scrolled menu, one at a
/// time: a short menu's rows show in a band narrower than three notches.
const SCROLLS: usize = 18;

/// Click `name`, opening the options menu first when it lives there;
/// `false` when it is drawn nowhere.
pub fn press(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<bool> {
    let Some((rect, vp)) = reach(session, pointer, ui_rect, name)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    close(session, pointer, ui_rect)?;
    Ok(true)
}

/// The open menu's frame: `dialogs::model3d::options::REGION_MENU`.
pub const MENU: &str = "model3d.options.menu";

/// Close the options menu by its glyph when it is open, so the picture
/// under it takes the pointer again.
pub fn close(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, MENU).is_none() {
        return Ok(());
    }
    if let Some((glyph, vp)) = declared_in(&trace, ui_rect, OPTIONS) {
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(glyph))?;
        session.settle(20);
    }
    Ok(())
}

/// The viewer's picture: `dialogs::model3d::REGION_IMAGE`.
const IMAGE: &str = "model3d.image";
/// How far the glyph may sit from the picture's top-left corner, points.
const CORNER: f32 = 64.0;

/// The sentence for an options glyph that is missing or away from the
/// picture's top-left corner; `None` when it is there.
pub fn misplaced(session: &Session, ui_rect: &str) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some((glyph, _)) = declared_in(&trace, ui_rect, OPTIONS) else {
        return Ok(Some(format!("★★★ no `{OPTIONS}` glyph over the picture.")));
    };
    let Some((image, _)) = declared_in(&trace, ui_rect, IMAGE) else {
        return Ok(Some(format!(
            "no `{IMAGE}` region: the viewer drew no picture."
        )));
    };
    let inside = image.contains_rect(glyph);
    let left = glyph.min.x - image.min.x;
    let top = glyph.min.y - image.min.y;
    Ok((!inside || left > CORNER || top > CORNER).then(|| {
        format!(
            "★★★ the options glyph {glyph:?} is not at the top-left of the picture {image:?} \
             (left gap {left}, top gap {top})."
        )
    }))
}
