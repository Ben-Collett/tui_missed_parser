use super::*;

fn parser() -> Parser {
    Parser::from_template(DEFAULT_MESSAGE).unwrap()
}

fn template(t: &str) -> Parser {
    Parser::from_template(t).unwrap()
}

#[test]
fn parses_basic_line() {
    let e = parser().parse_line("['et'] = the 1").unwrap();
    assert_eq!(e.triggers, vec!["et"]);
    assert_eq!(e.output, "the");
    assert_eq!(e.baka, 1);
}

#[test]
fn parses_multi_trigger_line() {
    let e = parser().parse_line("['hnt', 'hlnt'] = then 1").unwrap();
    assert_eq!(e.triggers, vec!["hnt", "hlnt"]);
    assert_eq!(e.output, "then");
}

#[test]
fn parses_backtick_and_double_quotes() {
    let e = parser().parse_line("[`dy`, `dfy`] = body 1").unwrap();
    assert_eq!(e.triggers, vec!["dy", "dfy"]);
    let e = parser().parse_line("[\"'dn\"] = Don't 1").unwrap();
    assert_eq!(e.triggers, vec!["'dn"]);
    assert_eq!(e.output, "Don't");
}

#[test]
fn parses_user_defined_format() {
    let p = template("$triggers = $chord, baka");
    let e = p.parse_line("['et'] = the, baka 1").unwrap();
    assert_eq!(e.triggers, vec!["et"]);
    assert_eq!(e.output, "the");
    assert_eq!(e.baka, 1);
}

#[test]
fn parses_non_default_message_layout() {
    let p = template("trigger: $triggers out: $chord");
    let e = p
        .parse_line("trigger: ['x', 'y'] out: some text 3")
        .unwrap();
    assert_eq!(e.triggers, vec!["x", "y"]);
    assert_eq!(e.output, "some text");
    assert_eq!(e.baka, 3);
}

#[test]
fn parses_chord_before_triggers() {
    let p = template("$chord <- $triggers");
    let e = p.parse_line("hello <- ['h', 'j'] 2").unwrap();
    assert_eq!(e.triggers, vec!["h", "j"]);
    assert_eq!(e.output, "hello");
    assert_eq!(e.baka, 2);
}

#[test]
fn handles_extra_whitespace() {
    let e = parser().parse_line("  [ 'et' ]  =  the  2  ").unwrap();
    assert_eq!(e.triggers, vec!["et"]);
    assert_eq!(e.output, "the");
    assert_eq!(e.baka, 2);
}

#[test]
fn output_may_contain_trailing_digits() {
    let e = parser().parse_line("['w'] = win2 3").unwrap();
    assert_eq!(e.output, "win2");
    assert_eq!(e.baka, 3);
}

#[test]
fn rejects_malformed_lines() {
    let p = parser();
    assert!(p.parse_line("[] = foo 1").is_none());
    assert!(p.parse_line("garbage").is_none());
    assert!(p.parse_line("['et'] = no count here").is_none());
    assert!(p.parse_line("['et'] =  baka xyz").is_none());
    assert!(p.parse_line("").is_none());
    assert!(p.parse_line("[et] = foo 1").is_none());
    assert!(p.parse_line("['et'] = ").is_none());
}

#[test]
fn preserves_output_case() {
    let e = parser().parse_line("['et'] = The 2").unwrap();
    assert_eq!(e.output, "The");
}

#[test]
fn requires_both_placeholders() {
    assert!(Parser::from_template("$triggers only").is_err());
    assert!(Parser::from_template("$chord only").is_err());
    assert!(Parser::from_template("no placeholders").is_err());
    assert!(Parser::from_template("$triggers = $chord").is_ok());
}
