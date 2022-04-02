use std::{
    collections::HashMap,
    process::Command
};

use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;

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

    let mut threads: Vec<std::thread::JoinHandle<()>> = Vec::new();

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

        let webdav_client_clone = webdav_client.clone();
        let thread = std::thread::spawn(move || {
            let (tx, rx) = channel();
            let mut watcher = watcher(tx, Duration::from_secs(0)).unwrap();
            watcher.watch(path, RecursiveMode::Recursive).unwrap();

            loop {
                match rx.recv() {
                    Ok(DebouncedEvent::Write(path)) | Ok(DebouncedEvent::Create(path)) => {
                        webdav_client_clone.upload_file_blocking(
                            path.to_str().unwrap(),
                            &directories::sanitize_webdav_path(path.to_str().unwrap())
                        );
                    },
                    Ok(DebouncedEvent::Remove(path)) => println!("delete {}", path.to_str().unwrap()),
                    Ok(_event) => {},
                    Err(e) => println!("err {:?}", e)
                }
            };
        });

        threads.push(thread);
    }

    for thread in threads {
        thread.join().expect("Error in joining thread");
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filepath = &args[1];

    manage(filepath).await;
}
