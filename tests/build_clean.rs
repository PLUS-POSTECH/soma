use soma::docker;
use soma::docker::{image_exists, image_from_prob_exists, image_from_repo_exists};
use soma::ops::{add, build, clean};

pub use self::common::*;

mod common;

#[test]
fn test_build_clean() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);
    let mut runtime = default_runtime();

    assert!(add(&mut env, SIMPLE_BOF_GIT, None).is_ok());

    let prob_query = "simple-bof";
    let problem = env
        .repo_manager()
        .search_prob(prob_query)
        .expect("Problem not found");
    let repo_name = problem.repo_name();
    let image_name = problem.docker_image_name(env.username());

    assert!(build(&env, prob_query, &mut runtime).is_ok());
    let images = runtime.block_on(docker::list_images(&env)).unwrap();
    assert!(image_exists(&images, &image_name));
    assert!(image_from_repo_exists(&images, repo_name));
    assert!(image_from_prob_exists(&images, &problem));

    assert!(clean(&env, prob_query, &mut runtime).is_ok());
    let images = runtime.block_on(docker::list_images(&env)).unwrap();
    assert!(!image_exists(&images, &image_name));
    assert!(!image_from_repo_exists(&images, repo_name));
    assert!(!image_from_prob_exists(&images, &problem));
}

#[test]
fn test_build_clean_list() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);
    let mut runtime = default_runtime();

    assert!(add(&mut env, BATA_LIST_GIT, None).is_ok());

    let prob_query = "r0pbaby";
    let problem = env
        .repo_manager()
        .search_prob(prob_query)
        .expect("Problem not found");
    let repo_name = problem.repo_name();
    let image_name = problem.docker_image_name(env.username());

    assert!(build(&env, prob_query, &mut runtime).is_ok());
    let images = runtime.block_on(docker::list_images(&env)).unwrap();
    assert!(image_exists(&images, &image_name));
    assert!(image_from_repo_exists(&images, repo_name));
    assert!(image_from_prob_exists(&images, &problem));

    assert!(clean(&env, prob_query, &mut runtime).is_ok());
    let images = runtime.block_on(docker::list_images(&env)).unwrap();
    assert!(!image_exists(&images, &image_name));
    assert!(!image_from_repo_exists(&images, repo_name));
    assert!(!image_from_prob_exists(&images, &problem));
}
