use std::{
    fs,
    collections::HashMap
};

fn check_forbidden(forbidden_paths: &Vec<&str>, path: &str) -> bool {
    for forbidden_path in forbidden_paths {
        if path.contains(forbidden_path) {
            return true;
        } 
    }

    return false;
}

pub fn walk_directories(path: &std::path::Path, cartridges_metadata: &mut HashMap<String, String>, forbidden_paths: &Vec<&str>) -> std::io::Result<()> {
    let path_str = path.to_str().unwrap();

    if path.is_dir() && !check_forbidden(&forbidden_paths, path_str) {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry.file_name().to_str().unwrap().contains(".project") {
                let cartridge_name = path.clone()
                    .to_str()
                    .unwrap()
                    .split("/")
                    .last();

                cartridges_metadata.insert(cartridge_name.unwrap().to_string(), path.to_str().unwrap().to_string());
                continue;
            }

            let _ = walk_directories(&entry_path, cartridges_metadata, forbidden_paths);
        }
    }

    Ok(())
}

pub fn sanitize_webdav_path(system_file_path: &str) -> String {
    let system_file_path_split: Vec<&str> = system_file_path.split("/").collect();

    let index = system_file_path_split.iter().rev().position(|&x| x == "cartridge").unwrap();

    let webdav_path: String  = 
        system_file_path_split
        .into_iter()
        .rev()
        .take(index + 2)
        .rev()
        .collect::<Vec<&str>>()
        .join("/");

    webdav_path
}
