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
mod tests {
    use super::*;

    fn entry(triggers: &[&str], output: &str, baka: u64) -> Entry {
        Entry {
            triggers: triggers.iter().map(|s| s.to_string()).collect(),
            output: output.to_string(),
            baka,
        }
    }

    #[test]
    fn merges_triggers_for_same_output() {
        let words = merge_all(&[
            entry(&["et"], "the", 1),
            entry(&["t"], "the", 2),
        ]);
        assert_eq!(words.len(), 1);
        let w = &words[0];
        assert_eq!(w.output, "the");
        assert_eq!(w.total, 3);
        assert_eq!(w.triggers.len(), 2);
        assert_eq!(w.triggers[0].display, "t");
        assert_eq!(w.triggers[0].count, 2);
        assert_eq!(w.triggers[1].display, "et");
        assert_eq!(w.triggers[1].count, 1);
    }

    #[test]
    fn anagram_triggers_merge() {
        let words = merge_all(&[
            entry(&["te"], "the", 1),
            entry(&["et"], "the", 1),
        ]);
        let w = &words[0];
        assert_eq!(w.triggers.len(), 1);
        assert_eq!(w.triggers[0].display, "te");
        assert_eq!(w.triggers[0].count, 2);
        assert_eq!(w.total, 2);
    }

    #[test]
    fn merging_is_case_sensitive() {
        let words = merge_all(&[
            entry(&["et"], "the", 1),
            entry(&["T"], "The", 2),
        ]);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].output, "The");
        assert_eq!(words[1].output, "the");
    }

    #[test]
    fn sorts_by_frequency_then_alphabetically() {
        let words = merge_all(&[
            entry(&["a"], "zebra", 2),
            entry(&["b"], "apple", 5),
            entry(&["c"], "mango", 5),
            entry(&["d"], "fig", 1),
        ]);
        let outputs: Vec<&str> = words.iter().map(|w| w.output.as_str()).collect();
        assert_eq!(outputs, vec!["apple", "mango", "zebra", "fig"]);
    }

    #[test]
    fn triple_counts_once_per_trigger_in_line() {
        let words = merge_all(&[entry(&["hnt", "hlnt"], "then", 4)]);
        let w = &words[0];
        assert_eq!(w.total, 4);
        assert_eq!(w.triggers.iter().map(|t| t.count).sum::<u64>(), 8);
    }
}