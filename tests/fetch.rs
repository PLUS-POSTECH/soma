use soma::ops::{add, fetch};

pub use self::common::*;

mod common;

#[test]
fn test_fetch() {
    let temp_data_dir = tempdir();
    let temp_copy_dir = tempdir();

    let env = test_env(&temp_data_dir);

    let repo_name = "simple-bof";
    assert!(add(&env, "https://github.com/PLUS-POSTECH/simple-bof.git").is_ok());
    assert!(fetch(&env, repo_name, &temp_copy_dir).is_ok());

    assert!(temp_copy_dir.path().join("simple-bof").is_file());
}
