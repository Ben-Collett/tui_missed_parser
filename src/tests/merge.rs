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