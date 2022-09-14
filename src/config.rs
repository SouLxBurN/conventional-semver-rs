use serde::Deserialize;
use std::path::Path;
use std::{fs, io};

const CONFIG_PATH: &str = "conventional_release.toml";

#[derive(Deserialize, Debug)]
pub struct ConventionSemverConfig {
    pub v: Option<bool>,
    pub version_files: Option<Vec<VersionFileConfig>>,
}

#[derive(Deserialize, Debug)]
pub struct VersionFileConfig {
    pub v: Option<bool>,
    pub path: String,
    pub version_prefix: Option<String>,
    pub version_postfix: Option<String>,
}

impl ConventionSemverConfig {
    pub fn new(v: Option<bool>, version_files: Vec<VersionFileConfig>) -> Self {
        Self {
            v,
            version_files: Some(version_files)
        }
    }

    pub fn default() -> Self {
        Self {
            v: Some(false),
            version_files: None
        }
    }

    pub fn load_config() -> Result<Self, io::Error> {
        let pth = Path::new(CONFIG_PATH);
        let c_file = fs::read_to_string(pth).expect(&format!("{} not found.", CONFIG_PATH));
        let str = c_file.as_str();
        let cfg: Self = toml::from_str::<ConventionSemverConfig>(str).expect(&format!("Failed to parse {}", CONFIG_PATH));
        return Ok(cfg);
    }
}
