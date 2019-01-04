use soma::ops::{add, fetch, pull};

pub use self::common::*;

mod common;

#[test]
fn test_fetch() {
    let temp_data_dir = tempdir();
    let temp_copy_dir = tempdir();

    let env = test_env(&temp_data_dir);

    let repo_name = "simple-bof";
    assert!(add(&env, "https://github.com/PLUS-POSTECH/simple-bof.git").is_ok());
    assert!(pull(&env, repo_name).is_ok());
    assert!(fetch(&env, repo_name, &temp_copy_dir).is_ok());

    expect_dir_contents(&temp_copy_dir, &vec!["simple-bof"]);
}
