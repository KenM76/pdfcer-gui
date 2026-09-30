//! The wire format: one request line in, one framed reply out.
//!
//! **Request:** a UTF-8 line, `verb arg arg …`. Arguments split on whitespace;
//! an argument containing spaces is wrapped in double quotes, with `\"` and
//! `\\` as the only escapes.
//!
//! **Reply:** a status line, `ok` or `err <code> <message>`, then zero or more
//! body lines, then a line holding a single `.`. A body line starting with `.`
//! is sent with one more `.` in front (SMTP dot-stuffing), so the terminator
//! is unambiguous. `<code>` is one lowercase word a client may branch on;
//! `<message>` is for a person.

/// A parsed request line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The first word, lowercased.
    pub verb: String,
    /// The remaining arguments, unquoted.
    pub args: Vec<String>,
}

/// Split a request line. `None` for a blank line or an unterminated quote.
pub fn parse(line: &str) -> Option<Line> {
    let mut words = Vec::new();
    let mut chars = line.trim().chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let mut word = String::new();
        if c == '"' {
            chars.next();
            let mut closed = false;
            while let Some(c) = chars.next() {
                match c {
                    '"' => {
                        closed = true;
                        break;
                    }
                    '\\' if matches!(chars.peek(), Some('"' | '\\')) => {
                        word.push(chars.next().unwrap_or('\\'));
                    }
                    other => word.push(other),
                }
            }
            if !closed {
                return None;
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                word.push(c);
                chars.next();
            }
        }
        words.push(word);
    }
    let mut words = words.into_iter();
    let verb = words.next()?.to_lowercase();
    Some(Line {
        verb,
        args: words.collect(),
    })
}

/// Quote `arg` if it needs it, so [`parse`] returns it unchanged.
pub fn quote(arg: &str) -> String {
    if !arg.is_empty() && !arg.chars().any(|c| c.is_whitespace() || c == '"') {
        return arg.to_owned();
    }
    let mut out = String::with_capacity(arg.len() + 2);
    out.push('"');
    for c in arg.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// A reply to one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    /// `None` for success; the error code otherwise.
    pub error: Option<String>,
    /// The error's message, or a one-line success summary (may be empty).
    pub message: String,
    /// Body lines, without terminators.
    pub body: Vec<String>,
}

impl Reply {
    /// Success with a summary.
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            error: None,
            message: message.into(),
            body: Vec::new(),
        }
    }

    /// Failure. `code` is one lowercase word.
    pub fn err(code: &str, message: impl Into<String>) -> Self {
        Self {
            error: Some(code.to_owned()),
            message: message.into(),
            body: Vec::new(),
        }
    }

    /// Append body lines.
    #[must_use]
    pub fn with_body(mut self, lines: impl IntoIterator<Item = String>) -> Self {
        self.body.extend(lines);
        self
    }

    /// Whether this reply is a success.
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    /// The framed bytes sent on the wire.
    pub fn encode(&self) -> String {
        let mut out = match &self.error {
            None if self.message.is_empty() => "ok".to_owned(),
            None => format!("ok {}", one_line(&self.message)),
            Some(code) => format!("err {code} {}", one_line(&self.message)),
        };
        out.push('\n');
        for line in &self.body {
            if line.starts_with('.') {
                out.push('.');
            }
            out.push_str(&one_line(line));
            out.push('\n');
        }
        out.push_str(".\n");
        out
    }
}

fn one_line(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}

/// Read one framed reply from a client's reader: the inverse of [`Reply::encode`].
///
/// # Errors
/// An I/O error, or `UnexpectedEof` when the stream ends mid-reply.
pub fn read_reply(reader: &mut impl std::io::BufRead) -> std::io::Result<Reply> {
    let eof = || std::io::Error::from(std::io::ErrorKind::UnexpectedEof);
    let mut status = String::new();
    if reader.read_line(&mut status)? == 0 {
        return Err(eof());
    }
    let status = status.trim_end_matches(['\r', '\n']);
    let mut reply = if let Some(rest) = status.strip_prefix("err ") {
        let (code, message) = rest.split_once(' ').unwrap_or((rest, ""));
        Reply::err(code, message)
    } else {
        Reply::ok(status.strip_prefix("ok").unwrap_or(status).trim_start())
    };
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Err(eof());
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line == "." {
            return Ok(reply);
        }
        reply
            .body
            .push(line.strip_prefix('.').unwrap_or(line).to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_arguments_round_trip() {
        let path = r#"C:\a b\say "hi".pdf"#;
        let line = parse(&format!("RUN file.open {}", quote(path))).expect("parses");
        assert_eq!(line.verb, "run");
        assert_eq!(line.args, vec!["file.open".to_owned(), path.to_owned()]);
        assert_eq!(parse("   "), None);
        assert_eq!(parse(r#"run "unclosed"#), None);
    }

    #[test]
    fn a_body_line_that_looks_like_the_terminator_survives() {
        let reply = Reply::err("busy", "two\nlines").with_body([
            ".".to_owned(),
            "..x".to_owned(),
            "plain".to_owned(),
        ]);
        let wire = reply.encode();
        assert!(wire.starts_with("err busy two lines\n"));
        let back = read_reply(&mut wire.as_bytes()).expect("reads");
        assert_eq!(back.error.as_deref(), Some("busy"));
        assert_eq!(back.body, vec![".", "..x", "plain"]);
        let ok = read_reply(&mut Reply::ok("").encode().as_bytes()).expect("reads");
        assert!(ok.is_ok() && ok.message.is_empty() && ok.body.is_empty());
    }
}
