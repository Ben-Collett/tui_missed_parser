use super::*;

fn sample() -> &'static str {
    r#"[
  { "triggers": ["gost"], "chord": "something", "count": 7 },
  { "triggers": ["gl"], "chord": "long", "count": 6 },
  { "triggers": ["hit"], "chord": "thing", "count": 3 }
]"#
}

#[test]
fn parses_array_of_objects() {
    let (entries, skipped) = parse_json(sample()).unwrap();
    assert_eq!(skipped, 0);
    assert_eq!(entries.len(), 3);
    assert_eq!(
        entries[0],
        Entry {
            triggers: vec!["gost".to_string()],
            output: "something".to_string(),
            baka: 7
        }
    );
}

#[test]
fn parses_single_object_top_level() {
    let (entries, skipped) =
        parse_json(r#"{ "triggers": ["gl"], "chord": "long", "count": 6 }"#).unwrap();
    assert_eq!(skipped, 0);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].output, "long");
    assert_eq!(entries[0].baka, 6);
}

#[test]
fn multi_trigger_object_keeps_list() {
    let (entries, _) =
        parse_json(r#"{ "triggers": ["hnt", "hlnt"], "chord": "then", "count": 1 }"#).unwrap();
    assert_eq!(entries[0].triggers, vec!["hnt", "hlnt"]);
}

#[test]
fn strings_are_used_verbatim() {
    let (entries, _) =
        parse_json(r#"{ "triggers": [" a "], "chord": " the ", "count": 2 }"#).unwrap();
    assert_eq!(entries[0].triggers, vec![" a "]);
    assert_eq!(entries[0].output, " the ");
}

#[test]
fn extra_fields_are_ignored() {
    let (entries, skipped) = parse_json(
        r#"{ "triggers": ["et"], "chord": "the", "count": 1, "extra": true }"#,
    )
    .unwrap();
    assert_eq!(skipped, 0);
    assert_eq!(entries.len(), 1);
}

#[test]
fn counts_skipped_bad_objects() {
    let text = r#"[
      { "triggers": ["a"], "chord": "good", "count": 1 },
      { "triggers": [], "chord": "empty trigs", "count": 1 },
      { "triggers": ["b"], "chord": "", "count": 1 },
      { "triggers": ["c"], "chord": "neg", "count": -1 },
      { "triggers": ["d"], "chord": "float", "count": 2.5 },
      { "triggers": ["e"], "chord": "str", "count": "7" },
      { "triggers": "not an array", "chord": "x", "count": 1 },
      { "chord": "no triggers", "count": 1 },
      "not an object"
    ]"#;
    let (entries, skipped) = parse_json(text).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(skipped, 8);
}

#[test]
fn rejects_scalar_top_level() {
    assert!(parse_json("\"just a string\"").is_err());
    assert!(parse_json("42").is_err());
    assert!(parse_json("true").is_err());
    assert!(parse_json("null").is_err());
}

#[test]
fn rejects_malformed_json() {
    assert!(parse_json("[{").is_err());
    assert!(parse_json("").is_err());
}

#[test]
fn accepts_empty_array_and_empty_object() {
    let (entries, skipped) = parse_json("[]").unwrap();
    assert_eq!(entries.len(), 0);
    assert_eq!(skipped, 0);

    let (entries, skipped) = parse_json("{}").unwrap();
    assert_eq!(entries.len(), 0);
    assert_eq!(skipped, 1);
}
