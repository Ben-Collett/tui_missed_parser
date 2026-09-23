use serde::Deserialize;
use serde_json::Value;

use crate::parse::Entry;

#[derive(Debug, Deserialize)]
struct RawEntry {
    triggers: Vec<String>,
    chord: String,
    count: u64,
}

impl RawEntry {
    fn into_entry(self) -> Option<Entry> {
        if self.triggers.is_empty() || self.chord.is_empty() {
            return None;
        }
        Some(Entry {
            triggers: self.triggers,
            output: self.chord,
            baka: self.count,
        })
    }
}

pub fn parse_json(text: &str) -> Result<(Vec<Entry>, usize), String> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| format!("invalid json: {e}"))?;
    let mut entries = Vec::new();
    let mut skipped = 0usize;
    match value {
        Value::Array(items) => {
            for item in items {
                match serde_json::from_value::<RawEntry>(item) {
                    Ok(raw) => match raw.into_entry() {
                        Some(e) => entries.push(e),
                        None => skipped += 1,
                    },
                    Err(_) => skipped += 1,
                }
            }
        }
        Value::Object(_) => match serde_json::from_value::<RawEntry>(value) {
            Ok(raw) => match raw.into_entry() {
                Some(e) => entries.push(e),
                None => skipped += 1,
            },
            Err(_) => skipped += 1,
        },
        _ => return Err("expected a JSON array or object".to_string()),
    }
    Ok((entries, skipped))
}

#[cfg(test)]
#[path = "tests/json.rs"]
mod tests;
