use std::path::Path;

use fs_extra::dir;
use matches::assert_matches;
use remove_dir_all::remove_dir_all;

use soma::docker::{self, image_from_repo_exists};
use soma::ops::{add, build, clean, update};
use soma::prelude::*;

pub use self::common::*;

mod common;

fn dir_copy(from: impl AsRef<Path>, to: impl AsRef<Path>) {
    if to.as_ref().exists() {
        remove_dir_all(to.as_ref()).expect("Failed to remove the directory");
    }

    let mut copy_options = dir::CopyOptions::new();
    copy_options.copy_inside = true;
    dir::copy(from.as_ref(), to.as_ref(), &copy_options).expect("Failed to copy the directory");
}

#[test]
fn test_update() {
    let project_dir = tempdir();

    let (_, mut data_dir) = temp_data_dir();
    let mut env = test_env(&mut data_dir);
    let mut runtime = default_runtime();

    dir_copy("test_repo/ab", project_dir.path());

    assert!(add(
        &mut env,
        &project_dir.path().as_os_str().to_string_lossy(),
        Some("test")
    )
    .is_ok());

    assert!(build(&env, "test.a", &mut runtime).is_ok());
    assert!(build(&env, "test.b", &mut runtime).is_ok());

    // image "test.b" should be removed before updating
    dir_copy("test_repo/a", project_dir.path());

    assert_matches!(
        update(&env, "test", &mut runtime).map_err(error_downcast),
        Err(Ok(SomaError::UnsupportedUpdate))
    );

    assert!(clean(&env, "test.b", &mut runtime).is_ok());
    assert!(update(&env, "test", &mut runtime).is_ok());

    // update should not fail when there is no removed problem
    dir_copy("test_repo/abc", project_dir.path());
    assert!(update(&env, "test", &mut runtime).is_ok());

    assert!(clean(&env, "test.a", &mut runtime).is_ok());

    let images = runtime.block_on(docker::list_images(&env)).unwrap();
    assert!(!image_from_repo_exists(&images, "test"));
}
