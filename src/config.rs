use serde::Deserialize;
use std::path::Path;
use std::{fs, io};

#[derive(Deserialize, Debug)]
pub struct ConventionSemverConfig {
    pub lead_v: bool,
    pub version_files: Vec<VersionFileConfig>,
}

#[derive(Deserialize, Debug)]
pub struct VersionFileConfig {
    pub path: String,
    pub version_prefix: String,
    pub version_postfix: String,
}

impl ConventionSemverConfig {
    pub fn new(lead_v: bool, version_files: Vec<VersionFileConfig>) -> Self {
        Self { lead_v, version_files }
    }

    pub fn load_config() -> Result<Self, io::Error> {
        let pth = Path::new(".conventional_release.toml");
        let c_file = fs::read_to_string(pth).expect(".conventional_release.toml not found.");
        let str = c_file.as_str();
        let cfg: Self = toml::from_str::<ConventionSemverConfig>(str).expect("toml parsing failed");

        return Ok(cfg);
    }
}
