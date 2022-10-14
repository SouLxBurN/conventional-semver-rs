use std::collections::HashMap;
use std::str::FromStr;
use clap::once_cell::sync::Lazy;
use super::VersionFileConfig;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum FilePresets {
    CARGOTOML,
    PACKAGEJSON
}

impl FromStr for FilePresets {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<FilePresets, crate::Error> {
        match s {
            "Cargo.toml" => Ok(FilePresets::CARGOTOML),
            "package.json" => Ok(FilePresets::PACKAGEJSON),
            _ => {
                Err(crate::Error::PresetError{bad_preset: String::from(s)})
            },
        }
    }
}

pub static PRESETS: Lazy<HashMap<FilePresets, VersionFileConfig>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(FilePresets::CARGOTOML, VersionFileConfig{
        v: false,
        path: String::from("Cargo.toml"),
        version_prefix: Some(String::from("version = \"")),
        version_postfix: Some(String::from("\"[^,]")),
        preset: Some(String::from("Cargo.toml")),
    });
    m.insert(FilePresets::PACKAGEJSON, VersionFileConfig{
        v: false,
        path: String::from("package.json"),
        version_prefix: Some(String::from("\"version\": \"")),
        version_postfix: Some(String::from("\",")),
        preset: Some(String::from("package.json")),
    });
    m
});
