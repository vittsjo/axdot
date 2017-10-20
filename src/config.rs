extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use std;
use std::io::prelude::*;

pub enum ConfigFormat {
    JSON,
    YAML,
}

impl ConfigFormat {
    pub fn from_ext(ext: &str) -> Option<ConfigFormat> {
        match ext {
            "json" => Some(ConfigFormat::JSON),
            "yaml" | "yml" => Some(ConfigFormat::YAML),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default)]
    pub directories: std::vec::Vec<String>,

    #[serde(default)]
    pub files: std::vec::Vec<String>,

    #[serde(default)]
    pub links: std::collections::HashMap<String, String>,

    #[serde(default)]
    pub copy: std::collections::HashMap<String, String>,

    #[serde(default)]
    pub commands: std::vec::Vec<std::vec::Vec<String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            directories: vec![],
            files: vec![],
            links: std::collections::HashMap::new(),
            copy: std::collections::HashMap::new(),
            commands: vec![vec![]],
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn to_yaml_string(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }

    pub fn load_file(path: &std::path::Path) -> std::result::Result<Config, String> {
        match std::fs::File::open(path) {
            Ok(file) => {
                match path.extension() {
                    None => Err(format!("{:?} has extension name", path)),
                    Some(ext) => {
                        match ext.to_str() {
                            None => Err(format!("{:?} is not valid Unicode string", ext)),
                            Some(ext) => {
                                match ConfigFormat::from_ext(ext) {
                                    Some(ConfigFormat::JSON) => {
                                        match serde_json::from_reader(file) {
                                            Ok(config) => Ok(config),
                                            Err(e) => Err(e.to_string()),
                                        }
                                    }
                                    Some(ConfigFormat::YAML) => {
                                        match serde_yaml::from_reader(file) {
                                            Ok(config) => Ok(config),
                                            Err(e) => Err(e.to_string()),
                                        }
                                    }
                                    None => Err(format!("{:?} is not valid Unicode string", ext)),
                                }
                            }
                        }
                    }                    
                }
            }
            Err(e) => Err(e.to_string()),
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
                    }.as_bytes(),
                )
            }            
        }
    }
}
