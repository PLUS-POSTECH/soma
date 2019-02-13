use soma::ops::{add, fetch};

pub use self::common::*;

mod common;

#[test]
fn test_fetch1() {
    let temp_copy_dir = tempdir();

    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);

    assert!(add(&mut env, SIMPLE_BOF_GIT, None).is_ok());
    assert!(fetch(&env, "simple-bof", &temp_copy_dir).is_ok());

    expect_dir_contents(&temp_copy_dir, &["simple-bof"]);
}

#[test]
fn test_fetch2() {
    let temp_copy_dir = tempdir();

    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);

    assert!(add(&mut env, BATA_LIST_GIT, None).is_ok());
    assert!(fetch(&env, "xkcd", &temp_copy_dir).is_ok());

    expect_dir_contents(&temp_copy_dir, &["xkcd"]);
}

#[test]
fn test_fetch3() {
    let temp_copy_dir = tempdir();

    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);

    assert!(add(&mut env, BATA_LIST_GIT, None).is_ok());
    assert!(fetch(
        &env,
        &format!("{}.r0pbaby", BATA_LIST_REPO_NAME),
        &temp_copy_dir
    )
    .is_ok());

    expect_dir_contents(&temp_copy_dir, &["r0pbaby"]);
}
