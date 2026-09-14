use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub triggers: Vec<String>,
    pub output: String,
    pub baka: u64,
}

pub const DEFAULT_MESSAGE: &str = "$triggers = $chord";

pub struct Parser {
    re: Regex,
}

impl Parser {
    pub fn from_template(template: &str) -> Result<Self, String> {
        if !template.contains("$triggers") {
            return Err(
                "message format does not define $triggers, cannot parse the log".to_string(),
            );
        }
        if !template.contains("$chord") {
            return Err("message format does not define $chord, cannot parse the log".to_string());
        }
        Ok(Parser {
            re: build_regex(template)?,
        })
    }

    pub fn parse_line(&self, line: &str) -> Option<Entry> {
        let caps = self.re.captures(line.trim())?;
        let mut triggers_raw = caps.name("triggers")?.as_str().trim();
        if let Some(inner) = triggers_raw
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
        {
            triggers_raw = inner;
        }
        let triggers = parse_triggers(triggers_raw)?;
        if triggers.is_empty() {
            return None;
        }
        let output = caps.name("chord")?.as_str().trim();
        if output.is_empty() {
            return None;
        }
        let baka = caps.name("count")?.as_str().parse::<u64>().ok()?;
        Some(Entry {
            triggers,
            output: output.to_string(),
            baka,
        })
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Triggers,
    Chord,
}

fn build_regex(template: &str) -> Result<Regex, String> {
    let mut pattern = String::from("^");
    let mut rest = template;
    let mut triggers_seen = 0usize;
    let mut chord_seen = 0usize;
    loop {
        let trig = rest.find("$triggers");
        let chord = rest.find("$chord");
        let (pos, kind) = match (trig, chord) {
            (Some(t), Some(c)) if t <= c => (t, Kind::Triggers),
            (_, Some(c)) => (c, Kind::Chord),
            (Some(t), None) => (t, Kind::Triggers),
            (None, None) => break,
        };
        pattern.push_str(&regex::escape(&rest[..pos]));
        match kind {
            Kind::Triggers => {
                if triggers_seen == 0 {
                    pattern.push_str("(?P<triggers>.*?)");
                } else {
                    pattern.push_str("(.*?)");
                }
                triggers_seen += 1;
                rest = &rest[pos + "$triggers".len()..];
            }
            Kind::Chord => {
                if chord_seen == 0 {
                    pattern.push_str("(?P<chord>.*?)");
                } else {
                    pattern.push_str("(.*?)");
                }
                chord_seen += 1;
                rest = &rest[pos + "$chord".len()..];
            }
        }
    }
    pattern.push_str(&regex::escape(rest));
    pattern.push_str(r"\s*?(?P<count>\d+)\s*$");
    Regex::new(&pattern).map_err(|e| format!("invalid message format: {e}"))
}

fn parse_triggers(inner: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut chars = inner.chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        let Some(q) = chars.next() else { break };
        if q != '\'' && q != '"' && q != '`' {
            return None;
        }
        let mut content = String::new();
        let mut closed = false;
        while let Some(c) = chars.next() {
            if c == '\\' {
                content.push(chars.next()?);
                continue;
            }
            if c == q {
                closed = true;
                break;
            }
            content.push(c);
        }
        if !closed {
            return None;
        }
        out.push(content);
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        match chars.peek() {
            None => break,
            Some(',') => {
                chars.next();
            }
            _ => return None,
        }
    }
    Some(out)
}

#[cfg(test)]
#[path = "tests/parse.rs"]
mod tests;
