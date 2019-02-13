use matches::assert_matches;
use soma::ops::{add, remove};

use soma::prelude::*;

pub use self::common::*;

mod common;

#[test]
fn test_add_remove() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);
    let mut runtime = default_runtime();

    let repo_name = SIMPLE_BOF_REPO_NAME;
    assert!(add(&mut env, SIMPLE_BOF_GIT, None).is_ok());

    assert!(env.repo_manager().repo_exists(repo_name));
    let local_path = env
        .repo_manager()
        .get_repo(repo_name)
        .expect("Added repository does not exist")
        .path();
    assert!(dir_contents_exists(local_path, &[".git"]));

    assert!(remove(&mut env, repo_name, &mut runtime).is_ok());
    assert!(!env.repo_manager().repo_exists(repo_name));
}

#[test]
fn test_add_with_name() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);

    let repo_name = "complicated-bof";
    assert!(add(&mut env, SIMPLE_BOF_GIT, Some(repo_name)).is_ok());

    assert!(env.repo_manager().repo_exists(repo_name));
    let local_path = env
        .repo_manager()
        .get_repo(repo_name)
        .expect("Added repository does not exist")
        .path();
    assert!(dir_contents_exists(local_path, &[".git"]));
}

#[test]
fn test_prob_search() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);
    let mut runtime = default_runtime();

    let repo_name_1 = "bof1";
    let repo_name_2 = "bof2";

    assert!(add(&mut env, SIMPLE_BOF_GIT, Some(repo_name_1)).is_ok());

    assert!(add(&mut env, SIMPLE_BOF_GIT, Some(repo_name_2)).is_ok());

    assert_matches!(
        env.repo_manager()
            .search_prob("simple-bof")
            .map_err(error_downcast),
        Err(Ok(SomaError::ProblemQueryAmbiguous))
    );
    assert!(remove(&mut env, repo_name_1, &mut runtime).is_ok());

    assert!(env.repo_manager().search_prob("simple-bof").is_ok());
    assert!(remove(&mut env, repo_name_2, &mut runtime).is_ok());

    assert_matches!(
        env.repo_manager()
            .search_prob("simple-bof")
            .map_err(error_downcast),
        Err(Ok(SomaError::ProblemNotFound))
    );
}

#[test]
fn test_soma_list() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);
    let mut runtime = default_runtime();

    let repo_name = BATA_LIST_REPO_NAME;
    assert!(add(&mut env, BATA_LIST_GIT, None).is_ok());

    assert!(env.repo_manager().repo_exists(repo_name));
    let local_path = env
        .repo_manager()
        .get_repo(repo_name)
        .expect("Added repository does not exist")
        .path();
    assert!(dir_contents_exists(local_path, &[".git"]));

    assert!(env.repo_manager().search_prob("xkcd").is_ok());
    assert!(env.repo_manager().search_prob("r0pbaby").is_ok());
    assert!(env
        .repo_manager()
        .search_prob(&format!("{}.babyecho", BATA_LIST_REPO_NAME))
        .is_ok());
    assert_matches!(
        env.repo_manager()
            .search_prob("simple-bof")
            .map_err(error_downcast),
        Err(Ok(SomaError::ProblemNotFound))
    );

    assert!(remove(&mut env, repo_name, &mut runtime).is_ok());
    assert!(!env.repo_manager().repo_exists(repo_name));
}
