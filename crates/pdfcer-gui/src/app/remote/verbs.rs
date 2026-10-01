//! Each verb a connected program may send. Replies are for programs: `key=value`
//! body lines, tab-separated rows, and error codes without spaces.

use command_remote::{Reply, Request};
use eframe::egui;

use super::super::PdfcerApp;
use super::super::actions::Action;
use super::super::state::Status;
use crate::app::save::has_unsaved_edits;
use pdfcer_core::vector::VectorObject;
use pdfcer_gui_base::canvastarget::TargetId;

const HELP: &str = include_str!("help.txt");
const MAX_REPEAT: usize = 100;
const DEFAULT_DPI: f32 = 96.0;
const MAX_DPI: f32 = 600.0;

/// How a verb is answered.
enum Answer {
    /// Now.
    Now(Reply),
    /// After this frame's actions apply, with the state they produced.
    Later(String),
}

/// Answer `request` now, or queue it to be answered after the frame's actions.
pub(super) fn handle(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    request: Request,
    actions: &mut Vec<Action>,
) {
    let args = request.args.as_slice();
    let answer = match request.verb.as_str() {
        "help" => Ok(Answer::Now(
            Reply::ok("").with_body(HELP.lines().map(str::to_owned)),
        )),
        "state" => Ok(Answer::Now(state_reply(app, ""))),
        "commands" => Ok(Answer::Now(commands(app, ctx))),
        "history" => history(app).map(Answer::Now),
        "objects" => objects(app).map(Answer::Now),
        "select" => select(app, args).map(|()| Answer::Now(state_reply(app, ""))),
        "render" => render(app, args).map(Answer::Now),
        "run" => run(app, ctx, args, actions).map(Answer::Later),
        verb @ ("undo" | "redo") => {
            step(app, ctx, verb, args, actions).map(|n| Answer::Later(n.to_string()))
        }
        "page" => page(app, args, actions).map(|n| Answer::Later(n.to_string())),
        "open" => match args {
            [path] => {
                actions.push(Action::Open(path.into()));
                Ok(Answer::Later(String::new()))
            }
            _ => Err(usage()),
        },
        _ => Err(Reply::err("unknown-verb", "send help")), // ui-text-exempt: wire reply
    };
    let reply = match answer {
        Ok(Answer::Later(message)) => {
            app.remote.deferred.push((request, message));
            return;
        }
        Ok(Answer::Now(reply)) | Err(reply) => reply,
    };
    if let Some(remote) = app.remote.remote.as_mut() {
        remote.respond(request, reply);
    }
}

fn usage() -> Reply {
    Reply::err("usage", "send help") // ui-text-exempt: wire reply
}

fn no_document() -> Reply {
    Reply::err("no-document", "open a PDF first") // ui-text-exempt: wire reply
}

/// `ok <message>` and the state the program is now in.
pub(super) fn state_reply(app: &PdfcerApp, message: &str) -> Reply {
    let mut body = vec![format!("documents={}", app.document_count())];
    body.push(format!(
        "mode={}",
        app.ribbon.mode().unwrap_or("-") // ui-text-exempt: wire token
    ));
    if let Status::Open(doc) = &app.status {
        let path = app
            .active_path()
            .map(|p| super::absolute(p).display().to_string());
        body.push(format!("document={}", path.unwrap_or_default()));
        body.push(format!(
            "page={}/{}",
            doc.view.page_index + 1,
            doc.pages.len()
        ));
        body.push(format!("zoom={:.0}", doc.view.zoom * 100.0));
        body.push(format!("unsaved={}", has_unsaved_edits(doc)));
        let selected = doc.selection.object_indices_on(doc.view.page_index);
        body.push(format!("selected={}", join(&selected)));
        body.push(format!("undo_depth={}", doc.session.undo_depth()));
    }
    Reply::ok(message).with_body(body)
}

fn join(indices: &[usize]) -> String {
    indices
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// Every registered command, with whether it can run now.
fn commands(app: &PdfcerApp, ctx: &egui::Context) -> Reply {
    let conditions = app.conditions(ctx);
    let mode = app.ribbon.mode();
    let mut body: Vec<String> = app
        .commands
        .iter()
        .map(|c| {
            let state = if !pdfcer_gui_base::modecapability::offers_command(
                app.shell.as_ref(),
                mode,
                &c.id,
            ) {
                "hidden"
            } else if c.is_enabled(&conditions) {
                "enabled"
            } else {
                "disabled"
            };
            format!("{}\t{state}\t{}", c.id, c.label)
        })
        .collect();
    body.sort();
    Reply::ok("").with_body(body)
}

/// Dispatch a command exactly as its ribbon button would.
fn run(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    args: &[String],
    actions: &mut Vec<Action>,
) -> Result<String, Reply> {
    let [id] = args else {
        return Err(usage());
    };
    let Some(command) = app.commands.get(id) else {
        return Err(Reply::err("unknown-command", "send commands")); // ui-text-exempt: wire reply
    };
    if !pdfcer_gui_base::modecapability::offers_command(app.shell.as_ref(), app.ribbon.mode(), id) {
        return Err(Reply::err("hidden", "the current mode does not offer it")); // ui-text-exempt: wire reply
    }
    if !command.is_enabled(&app.conditions(ctx)) {
        return Err(Reply::err("disabled", "not available in the current state")); // ui-text-exempt: wire reply
    }
    app.dispatch_command(ctx, id, actions);
    Ok(id.clone())
}

/// `undo [n]` / `redo [n]`, through the same commands as Ctrl+Z / Ctrl+Y.
fn step(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    verb: &str,
    args: &[String],
    actions: &mut Vec<Action>,
) -> Result<usize, Reply> {
    let n = match args.first() {
        None => 1,
        Some(n) => n.parse::<usize>().map_err(|_| usage())?,
    }
    .min(MAX_REPEAT);
    let id = format!("edit.{verb}");
    for _ in 0..n {
        run(app, ctx, std::slice::from_ref(&id), actions)?;
    }
    Ok(n)
}

fn history(app: &PdfcerApp) -> Result<Reply, Reply> {
    let Status::Open(doc) = &app.status else {
        return Err(no_document());
    };
    let kind = |k: Option<_>| k.map_or_else(|| "-".to_owned(), |k| format!("{k:?}"));
    // Newest first, so the Nth entry is what `undo N` reaches back to.
    let kinds = |ks: &mut dyn Iterator<Item = _>| {
        let all: Vec<String> = ks.map(|k| format!("{k:?}")).collect();
        if all.is_empty() {
            "-".to_owned()
        } else {
            all.join(",")
        }
    };
    let session = &doc.session;
    Ok(Reply::ok("").with_body(vec![
        format!("undo_depth={}", session.undo_depth()),
        format!("undo_top={}", kind(session.undo_kind())),
        format!("redo_top={}", kind(session.redo_kind())),
        format!("redo_depth={}", session.redo_depth()),
        format!("undo_kinds={}", kinds(&mut session.undo_kinds())),
        format!("redo_kinds={}", kinds(&mut session.redo_kinds())),
    ]))
}

fn page(app: &PdfcerApp, args: &[String], actions: &mut Vec<Action>) -> Result<usize, Reply> {
    let Status::Open(doc) = &app.status else {
        return Err(no_document());
    };
    let n: usize = args
        .first()
        .and_then(|a| a.parse().ok())
        .ok_or_else(usage)?;
    if n == 0 || n > doc.pages.len() {
        return Err(Reply::err(
            "no-such-page",
            format!("1..{}", doc.pages.len()),
        ));
    }
    actions.push(Action::GoToPage(n - 1));
    Ok(n)
}

/// The current page's objects, indexed as `select` takes them.
fn objects(app: &PdfcerApp) -> Result<Reply, Reply> {
    let Status::Open(doc) = &app.status else {
        return Err(no_document());
    };
    let Some(provider) = doc.page_objects() else {
        return Err(Reply::err("no-objects", "the page did not decompose")); // ui-text-exempt: wire reply
    };
    let body: Vec<String> = provider
        .page_objects()
        .objects
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let kind = match o {
                VectorObject::Path(_) => "path",
                VectorObject::Text(_) => "text",
                VectorObject::Image(_) => "image",
            };
            let b = o.page_bbox();
            format!(
                "{i}\t{kind}\t{:.1},{:.1},{:.1},{:.1}",
                b.min.x, b.min.y, b.max.x, b.max.y
            )
        })
        .collect();
    Ok(Reply::ok("").with_body(body))
}

/// Replace the selection on the current page.
fn select(app: &mut PdfcerApp, args: &[String]) -> Result<(), Reply> {
    let Status::Open(doc) = &mut app.status else {
        return Err(no_document());
    };
    let arg = args.first().ok_or_else(usage)?;
    let page = doc.view.page_index;
    if arg == "none" {
        doc.selection.clear();
        return Ok(());
    }
    let count = doc
        .page_objects()
        .map_or(0, |p| p.page_objects().objects.len());
    let mut ids = Vec::new();
    for part in arg.split(',') {
        let i: usize = part.trim().parse().map_err(|_| usage())?;
        if i >= count {
            return Err(Reply::err("no-such-object", format!("0..{count}")));
        }
        ids.push(TargetId::Object(i as u64));
    }
    doc.selection.marquee(page, &ids, false);
    Ok(())
}

/// Render a page, as saved content would render, to a PNG in the temp folder.
fn render(app: &PdfcerApp, args: &[String]) -> Result<Reply, Reply> {
    use crate::app::settings::SettingsExt;
    let Status::Open(doc) = &app.status else {
        return Err(no_document());
    };
    let page = match args.first() {
        None => doc.view.page_index,
        Some(n) => match n.parse::<usize>() {
            Ok(n) if (1..=doc.pages.len()).contains(&n) => n - 1,
            _ => {
                return Err(Reply::err(
                    "no-such-page",
                    format!("1..{}", doc.pages.len()),
                ));
            }
        },
    };
    let dpi = match args.get(1) {
        None => DEFAULT_DPI,
        Some(d) => d
            .parse::<f32>()
            .ok()
            .filter(|d| (1.0..=MAX_DPI).contains(d))
            .ok_or_else(usage)?,
    };
    let mut options = doc
        .settings
        .render_options()
        .with_backdrop(pdfcer_render::PageBackdrop::White);
    options.annotations = doc.annotations_visible();
    options.layers = doc.layer_visibility();
    let view = doc.session.view();
    let failed = |e: String| Reply::err("render-failed", &e);
    let rendered = pdfcer_render::render_page_with_view(
        &view,
        &doc.pages[page],
        pdfcer_gui_base::imageexport::scale_for(dpi),
        &options,
    )
    .map_err(|e| failed(e.to_string()))?;
    let png = pdfcer_render::export::encode_png(&rendered.pixmap, Some(dpi))
        .map_err(|e| failed(e.to_string()))?;
    let dir = std::env::temp_dir().join("pdfcer-remote");
    let path = dir.join(format!("{}-page{}.png", std::process::id(), page + 1));
    std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(&path, png))
        .map_err(|e| failed(e.to_string()))?;
    Ok(Reply::ok(path.display().to_string()))
}
