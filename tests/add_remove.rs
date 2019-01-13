use soma::ops::{add, remove};

pub use self::common::*;

mod common;

#[test]
fn test_add_remove() {
    let temp_dir = tempdir();
    let env = test_env(&temp_dir);
    let mut runtime = default_runtime();

    let repo_name = "simple-bof";
    assert!(add(&env, "https://github.com/PLUS-POSTECH/simple-bof.git", None).is_ok());
    {
        let repo_index = env
            .data_dir()
            .read_repo_index()
            .expect("failed to read repository index");
        assert!(repo_index.contains_key(repo_name));

        let repository = repo_index.get(repo_name).unwrap();
        let local_path = repository.local_path();
        assert!(dir_contents_exists(local_path, &vec![".git"]));

        assert!(env.data_dir().repo_exists(repo_name));
        assert_eq!(
            &env.data_dir().repo_path(repo_name),
            repository.local_path()
        );
    }

    assert!(remove(&env, repo_name, &mut runtime).is_ok());
    {
        let repo_index = env
            .data_dir()
            .read_repo_index()
            .expect("failed to read repository index");
        assert!(!repo_index.contains_key(repo_name));
        assert!(!env.data_dir().repo_exists(repo_name));
    }
}

#[test]
fn test_add_with_name() {
    let temp_dir = tempdir();
    let env = test_env(&temp_dir);

    let repo_name = "complicated-bof";
    assert!(add(
        &env,
        "https://github.com/PLUS-POSTECH/simple-bof.git",
        Some(repo_name)
    )
    .is_ok());
    assert!(env.data_dir().repo_path(repo_name).is_dir());

    let repo_index = env
        .data_dir()
        .read_repo_index()
        .expect("failed to read repository index");
    assert!(repo_index.contains_key(repo_name));

    let repository = repo_index.get(repo_name).unwrap();
    let local_path = repository.local_path();
    assert!(dir_contents_exists(local_path, &vec![".git"]));
}
