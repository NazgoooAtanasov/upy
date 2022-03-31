use crate::webdav;

pub fn parse_config(config_file: &str) -> webdav::Config {
    let file = std::fs::read_to_string(config_file).unwrap();
    let config: webdav::Config = serde_json::from_str(&file).unwrap();

    config
} 
