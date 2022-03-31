use std::{
    collections::HashMap,
    process::Command
};

mod webdav;
mod parser;
mod directories;

async fn manage(filepath: &str) {
    let path = std::path::Path::new(filepath);

    let mut webdav_client = webdav::WebdavClient::new();
    webdav_client.set_config(format!("{}/dw.json", filepath).as_str());
    println!("{}", webdav_client.config.as_ref().unwrap().version);

    let mut cartridges_metadata: HashMap<String, String> = HashMap::new();

    let forbidden_paths: Vec<&str> = vec!["node_modules", "target", "build-suite", "git"];

    let _ = directories::walk_directories(path, &mut cartridges_metadata, &forbidden_paths);

    for (name, path) in cartridges_metadata {
        let mut cartridge_parent_path_vec: Vec<&str> = path.split("/").collect();
        cartridge_parent_path_vec.pop();

        let cartridges_parent_path = cartridge_parent_path_vec.join("/");

        println!("Uploading [{}]", name);

        let _out = Command::new("sh")
            .current_dir(&cartridges_parent_path)
            .arg("-c")
            .arg(format!("zip {}.zip {} -r", name, name))
            .output();

        webdav_client.send_cartridge(&cartridges_parent_path, &name).await;
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filepath = &args[1];

    manage(filepath).await;
}
