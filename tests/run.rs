use soma::docker;
use soma::docker::{
    container_exists, container_from_repo_exists, image_exists, image_from_repo_exists,
};
use soma::ops::{add, build, run};

pub use self::common::*;

mod common;

#[test]
fn test_run() {
    // disabled for Windows CI
    if option_env!("WINDOWSCI").is_none() {
        let temp_data_dir = tempdir();

        let env = test_env(&temp_data_dir);

        let repo_name = "test-simple-bof";
        let image_name = docker::image_name(repo_name);
        let mut runtime = default_runtime();

        assert!(add(
            &env,
            "https://github.com/PLUS-POSTECH/simple-bof.git",
            Some(repo_name)
        )
        .is_ok());

        assert!(build(&env, repo_name).is_ok());
        let images = runtime.block_on(docker::list_images(&env)).unwrap();
        assert!(image_exists(&images, &image_name));
        assert!(image_from_repo_exists(&images, repo_name));

        let container_id = run(&env, repo_name, 31337, &mut runtime).unwrap();
        let containers = runtime.block_on(docker::list_containers(&env)).unwrap();
        assert!(container_exists(&containers, &container_id));
        assert!(container_from_repo_exists(&containers, repo_name));

        // Cleanup
        assert!(runtime.block_on(docker::stop(&env, &container_id)).is_ok());
        assert!(runtime
            .block_on(docker::remove_container(&env, &container_id))
            .is_ok());
        let containers = runtime.block_on(docker::list_containers(&env)).unwrap();
        assert!(!container_exists(&containers, &container_id));
        assert!(!container_from_repo_exists(&containers, repo_name));

        assert!(runtime
            .block_on(docker::remove_image(&env, &image_name))
            .is_ok());
        let images = runtime.block_on(docker::list_images(&env)).unwrap();
        assert!(!image_exists(&images, &image_name));
        assert!(!image_from_repo_exists(&images, repo_name));
    }
}
