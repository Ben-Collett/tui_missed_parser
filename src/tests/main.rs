use super::*;

fn pair(out: &str, trig: &str) -> (String, String) {
    (out.to_string(), trig.to_string())
}

#[test]
fn buffer_matches_spec_example() {
    let body = format_buffer(&[pair("fill", "fil"), pair("the", "et")]);
    assert_eq!(body, "fill fil\nthe  et");
}

#[test]
fn buffer_sorts_case_insensitive() {
    let body = format_buffer(&[
        pair("The", "T"),
        pair("and", "ae"),
        pair("the", "et"),
    ]);
    assert_eq!(body, "and ae\nThe T\nthe et");
}

#[test]
fn buffer_pads_to_longest_output_with_no_trailing_space() {
    let body = format_buffer(&[
        pair("the", "et"),
        pair("something", "gost"),
    ]);
    assert_eq!(body, "something gost\nthe       et");
}

#[test]
fn clamp_offset_keeps_cursor_visible() {
    assert_eq!(clamp_offset(0, 0, 10, 100), 0);
    assert_eq!(clamp_offset(0, 7, 10, 100), 0);
    assert_eq!(clamp_offset(0, 12, 10, 100), 3);
    assert_eq!(clamp_offset(200, 200, 10, 20), 10);
    assert_eq!(clamp_offset(5, 0, 10, 100), 0);
    assert_eq!(clamp_offset(0, 3, 10, 5), 0);
}