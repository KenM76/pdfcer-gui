//! The discovery file: how a client finds a running instance's pipe.
//!
//! One file per process, `<dir>/<pid>.txt`, holding `key=value` lines:
//! `pipe`, `pid`, `app` and `document` (empty when none is open). Written at
//! start, rewritten when the document changes, deleted on drop. A process that
//! crashes leaves its file behind; [`list`] skips it because no server holds
//! its pipe.

use std::path::{Path, PathBuf};

/// The file this process owns.
#[derive(Debug)]
pub(crate) struct Discovery {
    path: PathBuf,
    pipe: String,
    app: String,
}

impl Discovery {
    pub(crate) fn create(dir: &Path, pipe: &str, app: &str) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let this = Self {
            path: dir.join(format!("{}.txt", std::process::id())),
            pipe: pipe.to_owned(),
            app: app.to_owned(),
        };
        this.write(None)?;
        Ok(this)
    }

    pub(crate) fn write(&self, document: Option<&Path>) -> std::io::Result<()> {
        let body = format!(
            "pipe={}\npid={}\napp={}\ndocument={}\n",
            self.pipe,
            std::process::id(),
            self.app,
            document
                .map(|d| d.display().to_string())
                .unwrap_or_default()
        );
        // Write-then-rename so a client never reads half a file.
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, body)?;
        std::fs::rename(&tmp, &self.path)
    }
}

impl Drop for Discovery {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// One entry read back by a client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    /// The pipe path to open.
    pub pipe: String,
    /// The process id.
    pub pid: u32,
    /// The application name.
    pub app: String,
    /// The open document's path; empty when none.
    pub document: String,
}

/// Every live instance in `dir`, newest first. Unreadable files, and files left
/// by a process whose pipe no longer exists, are skipped.
pub fn list(dir: &Path) -> Vec<Instance> {
    read_all(dir)
        .into_iter()
        .filter(|i| native_pipe::exists(&i.pipe))
        .collect()
}

/// Every readable discovery file in `dir`, newest first, live or not.
fn read_all(dir: &Path) -> Vec<Instance> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<(std::time::SystemTime, Instance)> = read
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "txt"))
        .filter_map(|e| {
            let text = std::fs::read_to_string(e.path()).ok()?;
            let modified = e.metadata().and_then(|m| m.modified()).ok()?;
            Some((modified, parse(&text)?))
        })
        .collect();
    found.sort_by_key(|f| std::cmp::Reverse(f.0));
    found.into_iter().map(|(_, i)| i).collect()
}

fn parse(text: &str) -> Option<Instance> {
    let field = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key)?.strip_prefix('='))
            .map(str::to_owned)
    };
    Some(Instance {
        pipe: field("pipe")?,
        pid: field("pid")?.parse().ok()?,
        app: field("app").unwrap_or_default(),
        document: field("document").unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_written_file_lists_and_disappears_on_drop() {
        let dir = std::env::temp_dir().join(format!("command-remote-disc-{}", std::process::id()));
        let d = Discovery::create(&dir, r"\\.\pipe\x", "demo").expect("create");
        d.write(Some(Path::new(r"C:\a b.pdf"))).expect("write");
        let all = read_all(&dir);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].pipe, r"\\.\pipe\x");
        assert_eq!(all[0].document, r"C:\a b.pdf");
        assert_eq!(all[0].pid, std::process::id());
        assert!(
            list(&dir).is_empty(),
            "no server holds the pipe, so it is not live"
        );
        drop(d);
        assert!(read_all(&dir).is_empty());
        let _ = std::fs::remove_dir(&dir);
    }
}
