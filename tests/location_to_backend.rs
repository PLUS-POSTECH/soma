use soma::repository::backend::location_to_backend;

pub use self::common::*;

mod common;

fn test_parse(location: &str, expected_repo_name: &str) {
    let (repo_name, _) = location_to_backend(location).expect("failed to parse the location");
    assert_eq!(repo_name, expected_repo_name);
}

fn test_parse_fail(location: &str) {
    assert!(location_to_backend(location).is_err());
}

#[test]
fn location_to_backend_success() {
    test_parse(SIMPLE_BOF_GIT, SIMPLE_BOF_REPO_NAME);
    test_parse(BATA_LIST_GIT, BATA_LIST_REPO_NAME);
    // TODO: git through other protocols
    // test_parse_git("git@github.com:PLUS-POSTECH/simple-bof.git", SIMPLE_BOF_REPO_NAME);

    test_parse("ci", "ci");
    test_parse("tests/common", "common");
}

#[test]
fn location_to_backend_fail() {
    test_parse_fail("not_existing");
}
