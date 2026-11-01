//! Reaching a 3D viewer control that lives in its options menu.
//!
//! The viewer keeps its views, projection, axes, mesh fit, navigation and
//! window switches behind one glyph at the picture's top-left
//! (`dialogs::model3d::options::REGION_OPTIONS`). A control in that menu is
//! drawn only while the menu is open, so a check that wants one opens the
//! menu first. The menu stays open while its switches are used, and covers
//! part of the picture until it is closed.

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
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(glyph))?;
    session.settle(20);
    Ok(declared_in(&session.trace()?, ui_rect, name))
}

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
