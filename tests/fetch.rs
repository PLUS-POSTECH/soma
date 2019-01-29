use soma::ops::{add, fetch};

pub use self::common::*;

mod common;

#[test]
fn test_fetch() {
    let temp_copy_dir = tempdir();

    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);

    let repo_name = "simple-bof";
    assert!(add(
        &mut env,
        "https://github.com/PLUS-POSTECH/simple-bof.git",
        None
    )
    .is_ok());
    assert!(fetch(&mut env, repo_name, &temp_copy_dir).is_ok());

    expect_dir_contents(&temp_copy_dir, &["simple-bof"]);
}
