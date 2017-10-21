extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

use std;
use std::io::prelude::*;
use serde::Serialize;

pub enum ConfigFormat {
    JSON,
    YAML,
    TOML,
}

impl ConfigFormat {
    pub fn from_str(s: &str) -> Option<ConfigFormat> {
        match s.to_lowercase().as_str() {
            "json" => Some(ConfigFormat::JSON),
            "yaml" | "yml" => Some(ConfigFormat::YAML),
            "toml" => Some(ConfigFormat::TOML),
            _ => None,
        }
    }

    pub fn from_path(path: &std::path::Path) -> Option<ConfigFormat> {
        let ext = match path.extension() {
            Some(ext) => ext.to_str().unwrap_or(&""),
            None => {
                return None;
            } 
        };
        Self::from_str(ext)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ArchiveFormat {
    Zip,
    Rar,
    Tar,
    TarGz,
    TarBz2,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Archive {
    format: ArchiveFormat,
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default)]
    pub commands: std::vec::Vec<std::vec::Vec<String>>,

    #[serde(default)]
    pub directories: std::vec::Vec<String>,

    #[serde(default)]
    pub files: std::vec::Vec<String>,

    #[serde(default, serialize_with = "ordered_map")]
    pub links: std::collections::HashMap<String, String>,

    #[serde(default, serialize_with = "ordered_map")]
    pub copy: std::collections::HashMap<String, String>,

    #[serde(default, serialize_with = "ordered_string_archive_map")]
    pub archives: std::collections::HashMap<String, Archive>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            directories: vec![],
            files: vec![],
            links: std::collections::HashMap::new(),
            copy: std::collections::HashMap::new(),
            archives: std::collections::HashMap::new(),
            commands: vec![vec![]],
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn to_yaml_string(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }

    pub fn load_file(path: &std::path::Path) -> std::result::Result<Config, String> {

        let mut file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        match ConfigFormat::from_path(path) {
            Some(ConfigFormat::YAML) => serde_yaml::from_reader(file).map_err(|e| e.to_string()),
            Some(ConfigFormat::JSON) => serde_json::from_reader(file).map_err(|e| e.to_string()),
            Some(ConfigFormat::TOML) => {
                let mut buf = String::new();
                match file.read_to_string(&mut buf) {
                    Ok(_) => {}
                    Err(e) => return Err(e.to_string()),
                }

                toml::from_str(&mut buf).map_err(|e| e.to_string())
            }
            None => {
                return Err(format!("Failed to detect format from {:?} ", path));
            }
        }
    }

    pub fn save_file(&self, path: &std::path::Path, format: ConfigFormat) -> std::io::Result<()> {
        match std::fs::File::create(path) {
            Err(e) => Err(e),
            Ok(mut file) => {
                file.write_all(
                    match format {
                        ConfigFormat::JSON => self.to_json_string(),
                        ConfigFormat::YAML => self.to_yaml_string(),
                        ConfigFormat::TOML => self.to_toml_string(),
                    }.as_bytes(),
                )
            }
        }
    }
}

fn ordered_string_archive_map<S>(
    value: &std::collections::HashMap<String, Archive>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: std::collections::BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

fn ordered_map<S>(
    value: &std::collections::HashMap<String, String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: std::collections::BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
