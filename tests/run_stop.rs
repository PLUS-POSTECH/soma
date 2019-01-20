use soma::docker;
use soma::docker::{
    container_exists, container_from_repo_exists, image_exists, image_from_repo_exists,
};
use soma::ops::{add, build, clean, run, stop};

pub use self::common::*;

mod common;

#[test]
fn test_run_stop() {
    // disabled for Windows CI
    if option_env!("WINDOWSCI").is_none() {
        let (_, mut data_dir) = temp_data_dir();
        let mut env = test_env(&mut data_dir);

        let repo_name = "test-simple-bof";
        let image_name = docker::image_name(repo_name);
        let mut runtime = default_runtime();

        assert!(add(
            &mut env,
            "https://github.com/PLUS-POSTECH/simple-bof.git",
            Some(repo_name),
        )
        .is_ok());

        assert!(build(&env, repo_name, &mut runtime).is_ok());
        let images = runtime.block_on(docker::list_images(&env)).unwrap();
        assert!(image_exists(&images, &image_name));
        assert!(image_from_repo_exists(&images, repo_name));

        let container_id = run(&env, repo_name, 31337, &mut runtime).unwrap();
        let containers = runtime.block_on(docker::list_containers(&env)).unwrap();
        assert!(container_exists(&containers, &container_id));
        assert!(container_from_repo_exists(&containers, repo_name));

        // Cleanup
        assert!(stop(&env, repo_name, &mut runtime).is_ok());
        let containers = runtime.block_on(docker::list_containers(&env)).unwrap();
        assert!(!container_exists(&containers, &container_id));
        assert!(!container_from_repo_exists(&containers, repo_name));

        assert!(clean(&env, repo_name, &mut runtime).is_ok());
        let images = runtime.block_on(docker::list_images(&env)).unwrap();
        assert!(!image_exists(&images, &image_name));
        assert!(!image_from_repo_exists(&images, repo_name));
    }
}
