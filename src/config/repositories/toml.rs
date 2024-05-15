use std::{fs::File, io::Read};

use crate::config::Config;

pub struct TomlConfigManager;

impl TomlConfigManager {
    pub fn from_file(path: &str) -> Config {
        let mut buf = String::new();
        let mut file = File::open(path).unwrap();

        file.read_to_string(&mut buf).unwrap();

        let config: Config = toml::from_str(&buf).unwrap();

        config
    }
}
