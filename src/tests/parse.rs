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