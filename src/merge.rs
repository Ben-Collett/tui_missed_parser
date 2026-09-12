use std::collections::HashMap;

use crate::parse::Entry;

#[derive(Debug, Clone)]
pub struct TriggerInfo {
    pub display: String,
    pub count: u64,
}

#[derive(Debug, Clone)]
pub struct WordEntry {
    pub output: String,
    pub triggers: Vec<TriggerInfo>,
    pub total: u64,
}

pub fn canonical(trigger: &str) -> String {
    let mut chars: Vec<char> = trigger.chars().collect();
    chars.sort_unstable();
    chars.into_iter().collect()
}

pub fn merge_all(entries: &[Entry]) -> Vec<WordEntry> {
    let mut by_trig: HashMap<String, HashMap<String, (String, u64)>> = HashMap::new();
    let mut totals: HashMap<String, u64> = HashMap::new();

    for e in entries {
        let map = by_trig.entry(e.output.clone()).or_default();
        for t in &e.triggers {
            let key = canonical(t);
            let slot = map.entry(key).or_insert_with(|| (t.clone(), 0));
            slot.1 += e.baka;
        }
        *totals.entry(e.output.clone()).or_insert(0) += e.baka;
    }

    let mut words: Vec<WordEntry> = by_trig
        .into_iter()
        .map(|(output, map)| {
            let mut triggers: Vec<TriggerInfo> = map
                .into_values()
                .map(|(display, count)| TriggerInfo { display, count })
                .collect();
            triggers.sort_by_key(|a| std::cmp::Reverse(a.count));
            let total = totals.remove(&output).unwrap_or(0);
            WordEntry {
                output,
                triggers,
                total,
            }
        })
        .collect();

    words.sort_by(|a, b| {
        b.total.cmp(&a.total).then_with(|| {
            a.output
                .to_lowercase()
                .cmp(&b.output.to_lowercase())
                .then_with(|| a.output.cmp(&b.output))
        })
    });

    words
}

#[cfg(test)]
#[path = "tests/merge.rs"]
mod tests;