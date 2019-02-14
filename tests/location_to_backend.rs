use soma::ops::location_to_backend;
use soma::repository::Backend;

pub use self::common::*;

mod common;

fn test_parse_git(location: &str, expected_repo_name: &str) {
    assert!(
        if let Ok((repo_name, Backend::GitBackend(_))) = location_to_backend(location) {
            assert_eq!(repo_name, expected_repo_name);
            true
        } else {
            false
        }
    );
}

fn test_parse_local(location: &str, expected_repo_name: &str) {
    assert!(
        if let Ok((repo_name, Backend::LocalBackend(_))) = location_to_backend(location) {
            assert_eq!(repo_name, expected_repo_name);
            true
        } else {
            false
        }
    );
}

fn test_parse_fail(location: &str) {
    assert!(location_to_backend(location).is_err());
}

#[test]
fn location_to_backend_success() {
    test_parse_git(SIMPLE_BOF_GIT, SIMPLE_BOF_REPO_NAME);
    test_parse_git(BATA_LIST_GIT, BATA_LIST_REPO_NAME);
    // TODO: git through other protocols
    // test_parse_git("git@github.com:PLUS-POSTECH/simple-bof.git", SIMPLE_BOF_REPO_NAME);
    test_parse_local("hooks", "hooks");
}

#[test]
fn location_to_backend_fail() {
    test_parse_fail("not_existing");
}
