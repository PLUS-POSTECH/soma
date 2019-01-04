use soma::docker;
use soma::ops::{add, pull, run};

pub use self::common::*;

mod common;

#[test]
fn test_run() {
    let temp_data_dir = tempdir();

    let env = test_env(&temp_data_dir);

    let repo_name = "simple-bof";
    let expected_image_name = format!("soma/{}", repo_name);
    let mut runtime = default_runtime();

    assert!(add(&env, "https://github.com/PLUS-POSTECH/simple-bof.git").is_ok());
    assert!(pull(&env, repo_name).is_ok());
    let container_id = run(&env, repo_name, &mut runtime).unwrap();

    let images = runtime.block_on(docker::list_images(&env)).unwrap();
    let containers = runtime.block_on(docker::list_containers(&env)).unwrap();
    expect_image_exists(&images, &expected_image_name, false);
    expect_image_from_repo_exists(&images, repo_name, false);
    expect_container_exists(&containers, &container_id, false);
    expect_container_from_repo_exists(&containers, repo_name, false);

    // Cleanup
    assert!(runtime.block_on(docker::stop(&env, &container_id)).is_ok());
    assert!(runtime
        .block_on(docker::remove_container(&env, &container_id))
        .is_ok());
    expect_container_exists(&containers, &container_id, true);
    expect_container_from_repo_exists(&containers, repo_name, true);

    assert!(runtime
        .block_on(docker::remove_image(&env, &expected_image_name))
        .is_ok());
    expect_image_exists(&images, &expected_image_name, true);
    expect_image_from_repo_exists(&images, repo_name, true);
}
