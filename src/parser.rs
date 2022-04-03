use std::collections;
use crate::webdav;

pub fn parse_config(config_file: &str) -> webdav::Config {
    let file = std::fs::read_to_string(config_file).unwrap();
    let config: webdav::Config = serde_json::from_str(&file).unwrap();

    config
} 



pub fn parse_args(args: &Vec<String>) -> collections::HashMap<&str, String> {
    // path
    // flags
    let mut arguments = collections::HashMap::new();

    if args.len() <= 1 {
        panic!("No path provided");
    }
    
    arguments.insert("path", args[1].clone());

    let s: Vec<&String> = args.iter().skip(2).collect();

    for q in s {
        println!("{}", q);
    }

    arguments
}
