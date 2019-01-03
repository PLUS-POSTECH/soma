use tempfile::tempdir;

use soma::error::Result as SomaResult;
use soma::ops::add;

mod common;

#[test]
fn test_add() -> SomaResult<()> {
    let temp_dir = tempdir()?;
    let env = common::test_env(&temp_dir)?;

    let repo_name = "simple-bof";
    assert!(add(&env, "https://github.com/PLUS-POSTECH/simple-bof.git").is_ok());
    assert!(env.data_dir().repo_root_path().join(repo_name).is_dir());

    let repo_index = env
        .data_dir()
        .read_repo_index()
        .expect("failed to read repository index");
    assert!(repo_index.contains_key(repo_name));

    Ok(())
}
