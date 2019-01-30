use soma::docker;
use soma::docker::{
    container_exists, container_from_prob_exists, image_exists, image_from_repo_exists,
};
use soma::ops::{add, build, clean, run, stop};

pub use self::common::*;

mod common;

#[test]
fn test_run_stop() {
    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);
    let mut runtime = default_runtime();

    assert!(add(
        &mut env,
        "https://github.com/PLUS-POSTECH/simple-bof.git",
        None,
    )
    .is_ok());

    let prob_query = "simple-bof";
    let problem = env
        .repo_manager()
        .search_prob(prob_query)
        .expect("Problem not found");
    let prob_name = problem.prob_name();
    let image_name = problem.docker_image_name(env.username());

    assert!(build(&env, prob_query, &mut runtime).is_ok());
    let images = runtime.block_on(docker::list_images(&env)).unwrap();
    assert!(image_exists(&images, &image_name));
    assert!(image_from_repo_exists(&images, problem.repo_name()));

    let container_id = run(&env, prob_query, 31337, &mut runtime).unwrap();
    let containers = runtime.block_on(docker::list_containers(&env)).unwrap();
    assert!(container_exists(&containers, &container_id));
    assert!(container_from_prob_exists(&containers, &problem));

    // Problem container should be running exclusively
    assert!(run(&env, prob_query, 31337, &mut runtime).is_err());

    // Cleanup
    assert!(stop(&env, prob_query, &mut runtime).is_ok());
    let containers = runtime.block_on(docker::list_containers(&env)).unwrap();
    assert!(!container_exists(&containers, &container_id));
    assert!(!container_from_prob_exists(&containers, &problem));

    assert!(clean(&env, prob_query, &mut runtime).is_ok());
    let images = runtime.block_on(docker::list_images(&env)).unwrap();
    assert!(!image_exists(&images, &image_name));
    assert!(!image_from_repo_exists(&images, &prob_name));
}
