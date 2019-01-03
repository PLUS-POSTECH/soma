use soma::ops::parse_repo_location;
use soma::repo::backend::Backend;

fn test_parse_git(url: &str, expected_repo_name: &str) {
    assert!(
        if let Ok((repo_name, Backend::GitBackend(_))) = parse_repo_location(url) {
            assert_eq!(repo_name, expected_repo_name);
            true
        } else {
            false
        }
    );
}

fn test_parse_local(url: &str, expected_repo_name: &str) {
    assert!(
        if let Ok((repo_name, Backend::LocalBackend(_))) = parse_repo_location(url) {
            assert_eq!(repo_name, expected_repo_name);
            true
        } else {
            false
        }
    );
}

fn test_parse_fail(url: &str) {
    assert!(if let Err(_) = parse_repo_location(url) {
        true
    } else {
        false
    });
}

#[test]
fn parse_repo_location_success() {
    test_parse_git(
        "https://github.com/PLUS-POSTECH/simple-bof.git",
        "simple-bof",
    );
    // TODO: git through other protocols
    // test_parse_git("git@github.com:PLUS-POSTECH/simple-bof.git", "simple-bof");
    test_parse_local("hooks", "#hooks");
}

#[test]
fn parse_repo_location_fail() {
    test_parse_fail("not_existing");
}
