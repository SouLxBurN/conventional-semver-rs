use serde::Deserialize;
use std::path::Path;
use std::{fs, io};
use std::str::FromStr;

mod presets;
use presets::FilePresets;

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
    fn default_path() -> String {
        String::from("")
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
        String::from("conventional-semver-rs")
    }

    fn default_sig_email() -> String {
        String::from("conventional-semver-rs@github.com")
    }

    fn default_sig() -> CommitSignature {
        CommitSignature {
            name: Self::default_sig_name(),
            email: Self::default_sig_email(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct VersionFileConfig {
    #[serde(default = "ConventionalSemverConfig::default_v")]
    pub v: bool,
    #[serde(default = "ConventionalSemverConfig::default_path")]
    pub path: String,
    pub version_prefix: Option<String>,
    pub version_postfix: Option<String>,
    pub preset: Option<String>,
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

    pub fn load_config() -> Result<Self, crate::Error> {
        let pth = Path::new(CONFIG_PATH);
        match fs::read_to_string(pth) {
            Ok(c_file) => {
                let str = c_file.as_str();
                let mut config = toml::from_str::<ConventionalSemverConfig>(str)?;
                if config.version_files.is_some() {
                    for mut f in config.version_files.as_mut().unwrap().iter_mut() {
                        if let Some(pre) = &f.preset{
                            let preset = FilePresets::from_str(&pre)?;
                            let cp = presets::PRESETS.get(&preset).expect("Preset not part of preset map");
                            f.v = cp.v.clone();
                            f.path = cp.path.clone();
                            f.version_prefix = cp.version_prefix.clone();
                            f.version_postfix = cp.version_postfix.clone();
                            f.preset = cp.preset.clone();
                            ()
                        } else if f.path == "" {
                            return Err(crate::Error::InvalidConfigError{
                                reason: String::from("version_file path cannot be blank, without a preset")
                            })
                        }
                    };
                }
                Ok(config)
            },
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    eprintln!("convention_release.toml not found, using default configuration");
                    Ok(Self::default())
                } else {
                    Err(err.into())
                }
            }
        }
    }
}
