use soma::ops::add;

pub use self::common::*;

mod common;

#[test]
fn test_add() {
    let temp_dir = tempdir();
    let env = test_env(&temp_dir);

    let repo_name = "simple-bof";
    assert!(add(&env, "https://github.com/PLUS-POSTECH/simple-bof.git").is_ok());
    assert!(env.data_dir().repo_path(repo_name).is_dir());

    let repo_index = env
        .data_dir()
        .read_repo_index()
        .expect("failed to read repository index");
    assert!(repo_index.contains_key(repo_name));
}
