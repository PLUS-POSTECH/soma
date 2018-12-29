use std::fs;
use std::path::PathBuf;

use question::{Answer, Question};

pub fn dir_path() -> PathBuf {
    if let Some(dir) = std::env::var_os("SOMA_DATA_DIR") {
        return dir.into();
    }

    let mut home = dirs::home_dir().expect("could not find home directory");
    home.push(".soma");
    home
}

pub fn initialize() -> bool {
    let path = dir_path();
    if !path.exists() {
        if let Answer::NO = Question::new(&format!(
            "It seems that you are using soma for the first time. The data directory will be initialized at {:?}. [y/n]\n",
            path.as_os_str(),
        ))
        .confirm()
        {
            return false;
        }
        fs::create_dir_all(&path).expect("failed to create soma data directory");
    }
    true
}
