#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub triggers: Vec<String>,
    pub output: String,
    pub baka: u64,
}

pub fn parse_line(line: &str) -> Option<Entry> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let close = line.find(']')?;
    let triggers = parse_triggers(&line[1..close])?;
    if triggers.is_empty() {
        return None;
    }
    let rest = line[close + 1..].trim();
    let rest = rest.strip_prefix('=').map(str::trim).unwrap_or(rest);
    let idx = rest.rfind(',')?;
    let output = rest[..idx].trim();
    if output.is_empty() {
        return None;
    }
    let baka_tail = rest[idx + 1..].trim();
    let num = baka_tail.strip_prefix("baka")?.trim();
    let baka: u64 = num.parse().ok()?;
    Some(Entry {
        triggers,
        output: output.to_string(),
        baka,
    })
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
mod tests {
    use super::*;

    #[test]
    fn parses_basic_line() {
        let e = parse_line("['et'] = the, baka 1").unwrap();
        assert_eq!(e.triggers, vec!["et"]);
        assert_eq!(e.output, "the");
        assert_eq!(e.baka, 1);
    }

    #[test]
    fn parses_multi_trigger_line() {
        let e = parse_line("['hnt', 'hlnt'] = then, baka 1").unwrap();
        assert_eq!(e.triggers, vec!["hnt", "hlnt"]);
        assert_eq!(e.output, "then");
    }

    #[test]
    fn parses_backtick_and_double_quotes() {
        let e = parse_line("[`dy`, `dfy`] = body, baka 1").unwrap();
        assert_eq!(e.triggers, vec!["dy", "dfy"]);
        let e = parse_line("[\"'dn\"] = Don't, baka 1").unwrap();
        assert_eq!(e.triggers, vec!["'dn"]);
        assert_eq!(e.output, "Don't");
    }

    #[test]
    fn handles_extra_whitespace() {
        let e = parse_line("  [ 'et' ]  =  the ,  baka  2  ").unwrap();
        assert_eq!(e.triggers, vec!["et"]);
        assert_eq!(e.output, "the");
        assert_eq!(e.baka, 2);
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(parse_line("[] = foo, baka 1").is_none());
        assert!(parse_line("garbage").is_none());
        assert!(parse_line("['et'] = no baka here").is_none());
        assert!(parse_line("['et'] = , baka xyz").is_none());
        assert!(parse_line("").is_none());
        assert!(parse_line("[et] = foo, baka 1").is_none());
    }

    #[test]
    fn preserves_output_case() {
        let e = parse_line("['et'] = The, baka 2").unwrap();
        assert_eq!(e.output, "The");
    }
}