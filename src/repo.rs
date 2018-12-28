use std::fs::File;
use std::io::Read;

use failure::Error;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub name: String,
    pub executable: Vec<FileConfig>,
    pub readonly: Vec<FileConfig>,
    pub binary: BinaryConfig,
}

#[derive(Deserialize)]
pub struct FileConfig {
    pub path: String,
    pub public: Option<bool>,
}

#[derive(Deserialize)]
pub struct BinaryConfig {
    pub os: String,
    pub entry: String,
}

fn read_file_contents(file_name: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(file_name)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn load_config(config_file_name: &str) -> Result<Config, Error> {
    Ok(toml::from_slice(&read_file_contents(config_file_name)?)?)
}
