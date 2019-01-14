use soma::ops::{add, remove};

pub use self::common::*;

mod common;

#[test]
fn test_add_remove() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);
    let mut runtime = default_runtime();

    let repo_name = "simple-bof";
    assert!(add(
        &mut env,
        "https://github.com/PLUS-POSTECH/simple-bof.git",
        None
    )
    .is_ok());

    assert!(env.repo_manager().repo_exists(repo_name));
    let local_path = env
        .repo_manager()
        .get_repo(repo_name)
        .expect("added repository does not exist")
        .local_path();
    assert!(dir_contents_exists(local_path, &vec![".git"]));

    assert!(remove(&mut env, repo_name, &mut runtime).is_ok());
    assert!(!env.repo_manager().repo_exists(repo_name));
}

#[test]
fn test_add_with_name() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);

    let repo_name = "complicated-bof";
    assert!(add(
        &mut env,
        "https://github.com/PLUS-POSTECH/simple-bof.git",
        Some(repo_name),
    )
    .is_ok());

    assert!(env.repo_manager().repo_exists(repo_name));
    let local_path = env
        .repo_manager()
        .get_repo(repo_name)
        .expect("added repository does not exist")
        .local_path();
    assert!(dir_contents_exists(local_path, &vec![".git"]));
}
