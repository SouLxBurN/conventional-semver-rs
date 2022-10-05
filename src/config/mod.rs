use serde::Deserialize;
use std::path::Path;
use std::{fs, io};

const CONFIG_PATH: &str = "conventional_release.toml";

#[derive(Deserialize, Debug)]
pub struct ConventionalSemverConfig {
    #[serde(default = "ConventionalSemverConfig::default_v")]
    pub v: bool,
    pub version_files: Option<Vec<VersionFileConfig>>,
    #[serde(default = "CommitSignature::default_sig")]
    pub commit_signature: CommitSignature,
}

impl ConventionalSemverConfig {
    fn default_v() -> bool {
        false
    }
}

#[derive(Deserialize, Debug)]
pub struct CommitSignature {
    #[serde(default = "CommitSignature::default_sig_name")]
    pub name: String,
    #[serde(default = "CommitSignature::default_sig_email")]
    pub email: String,
}

impl CommitSignature {
    fn default_sig_name() -> String {
        String::from("rs-release")
    }

    fn default_sig_email() -> String {
        String::from("rs-release@rust.com")
    }

    fn default_sig() -> CommitSignature {
        CommitSignature {
            name: Self::default_sig_name(),
            email: Self::default_sig_email(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct VersionFileConfig {
    #[serde(default = "ConventionalSemverConfig::default_v")]
    pub v: bool,
    pub path: String,
    pub version_prefix: Option<String>,
    pub version_postfix: Option<String>,
}

impl ConventionalSemverConfig {
    pub fn new(v: bool, commit_signature: CommitSignature, version_files: Vec<VersionFileConfig>) -> Self {
        Self {
            v,
            commit_signature,
            version_files: Some(version_files)
        }
    }

    pub fn default() -> Self {
        Self {
            v: false,
            commit_signature: CommitSignature::default_sig(),
            version_files: None
        }
    }

    pub fn load_config() -> Result<Self, io::Error> {
        let pth = Path::new(CONFIG_PATH);
        match fs::read_to_string(pth) {
            Ok(c_file) => {
                let str = c_file.as_str();
                Ok(toml::from_str::<ConventionalSemverConfig>(str)?)
            },
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    eprintln!("convention_release.toml not found, using default configuration");
                    Ok(Self::default())
                } else {
                    Err(err)
                }
            }
        }
    }
}
