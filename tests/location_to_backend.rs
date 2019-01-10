use soma::ops::location_to_backend;
use soma::repo::backend::Backend;

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
    assert!(if let Err(_) = location_to_backend(location) {
        true
    } else {
        false
    });
}

#[test]
fn location_to_backend_success() {
    test_parse_git(
        "https://github.com/PLUS-POSTECH/simple-bof.git",
        "simple-bof",
    );
    // TODO: git through other protocols
    // test_parse_git("git@github.com:PLUS-POSTECH/simple-bof.git", "simple-bof");
    test_parse_local("hooks", "hooks");
}

#[test]
fn location_to_backend_fail() {
    test_parse_fail("not_existing");
}
