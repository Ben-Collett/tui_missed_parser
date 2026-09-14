use super::*;

#[test]
fn message_from_config_reads_message() {
    let text = "[notification]\nmessage = \"$triggers => $chord\"\n";
    assert_eq!(
        message_from_config(text).unwrap().as_deref(),
        Some("$triggers => $chord")
    );
}

#[test]
fn message_from_config_defaults_when_absent() {
    assert_eq!(message_from_config("").unwrap(), None);
    assert_eq!(
        message_from_config("[general]\nmode = \"charachorder\"\n").unwrap(),
        None
    );
    assert_eq!(
        message_from_config("[notification]\ntitle = \"hi\"\n").unwrap(),
        None
    );
}

#[test]
fn message_from_config_errors_on_non_string() {
    let text = "[notification]\nmessage = 5\n";
    assert!(message_from_config(text).is_err());
}

#[test]
fn message_from_config_errors_on_bad_toml() {
    assert!(message_from_config("not = = valid").is_err());
}

#[test]
fn expands_tilde_paths() {
    assert_eq!(expand_path("~", "/home/u"), PathBuf::from("/home/u"));
    assert_eq!(
        expand_path("~/x/y", "/home/u"),
        PathBuf::from("/home/u/x/y")
    );
    assert_eq!(
        expand_path("/abs/path", "/home/u"),
        PathBuf::from("/abs/path")
    );
}
