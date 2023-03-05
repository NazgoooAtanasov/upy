// @TODO: research the `Result` return type, in order to use the `?` instead of constantly unsing unwraps
// @TODO: create a sane way of not looking into common directories like `node_modules`, `.git`, etc.
use std::collections::HashMap;
use std::fs;
use std::path;
use std::thread;
use std::sync;
use serde::Deserialize;
use std::io::{Read, Write};

type Cartridges = Vec<String>;

struct Uploader {
    f_watch: bool,
    f_upload: bool,
    working_dir: String,
    arguments: HashMap<String, String>,
    cartridges: Cartridges
}

impl Uploader {
    fn new() -> Self {
        return Self {
            f_upload: true,
            f_watch: true,
            working_dir: String::new(),
            arguments: HashMap::new(),
            cartridges: Cartridges::default()
        }
    }

    fn parse_args(mut self, command_args: std::env::Args) -> Self {
        let mut expect_value = false;
        let mut expect_value_buffer = String::new();
        for (n, arg) in command_args.skip(1).enumerate() {
            if n == 0 {
                self.working_dir = arg;
            } else {
                match arg.as_str() {
                    "-w" | "-u" => {
                        self.arguments.insert(arg, String::new());
                    },

                    "--config" => {
                        expect_value = true;
                        expect_value_buffer = arg;
                    }

                    anything => {
                        if expect_value && !expect_value_buffer.is_empty() {
                            self.arguments.insert(expect_value_buffer.clone(), String::from(anything));
                            expect_value = false;
                            expect_value_buffer = String::new();
                        }
                    }
                }
            }
        }

        return self;
    }

    fn set_flags(mut self) -> Self {
        for (flag, _value) in self.arguments.clone() {
            match flag.as_str() {
                "-w" => {
                    self.f_watch = true;
                }

                "-u" => {
                    self.f_upload = true;
                }

                _ => panic!("Flag {flag} not supported")
            }
        }

        return self;
    }

    fn get_cartridges(&self, working_dir: String) -> Cartridges {
        let mut cartridges = Cartridges::default();

        let is_project = fs::read_dir(path::Path::new(&working_dir))
            .expect(format!("Problem with opening passed dir {}", working_dir).as_str())
            .map(|x| {
                return x.unwrap();
            })
            .any(|x| {
                return x.path().to_str().unwrap().contains(".project");
            });

        if is_project {
            cartridges.push(working_dir);
        } else {
            fs::read_dir(path::Path::new(&working_dir)) // refetching the whole dir for now. research how to reuse an iterator for above an here.
                .expect(format!("Problem with opening passed dir {}", working_dir).as_str())
                .map(|x| {
                    return x.unwrap();
                })
                .filter(|x| {
                    return x.path().is_dir();
                })
                .for_each(|x| {
                    cartridges.append(&mut self.get_cartridges(x.path().to_str().unwrap().to_string()));
                });
        }

        return cartridges;
    }

    fn set_cartridges(mut self) -> Self {
        self.cartridges = self.get_cartridges(self.working_dir.clone());
        return self;
    }
}


type ZipFiles = HashMap<String, String>;
struct ZipHanlder {
    options: zip::write::FileOptions,
    zip_files: ZipFiles,
    output_dir: String
}

impl ZipHanlder {
    fn new() -> Self {
        return Self {
            zip_files: ZipFiles::default(),
            output_dir: String::from("./outdir"),
            options: zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o755)
        };
    }

    fn reset_outdir(self) -> Self {
        let old_zips = fs::read_dir(path::Path::new(self.output_dir.as_str())).unwrap();
        for zip in old_zips {
            fs::remove_file(zip.unwrap().path()).unwrap();
        }
        return self;
    }

    fn zip_entries(
        &self,
        zip: &mut zip::ZipWriter<fs::File>,
        options: &zip::write::FileOptions,
        current_cartridge_name: &String,
        current_cartridge_path: &String,
        entries: fs::ReadDir
    ) -> zip::result::ZipResult<()> {
        let entries = entries.map(|x| { return x.unwrap(); });
        let mut buffer = Vec::new();

        for entry in entries {
            let path = entry.path();
            let mut name = path
                .to_str()
                .unwrap()
                .replace(current_cartridge_path, "");
            name.insert_str(0, current_cartridge_name);

            if path.is_file() {
                zip.start_file(name, *options)?;
                let mut f = fs::File::open(path)?;

                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
                buffer.clear();
            } else {
                zip.add_directory(name, *options)?;

                let new_entries = fs::read_dir(path)?;
                self.zip_entries(zip, options, current_cartridge_name, current_cartridge_path, new_entries)?;
            }
        }

        return Ok(());
    }

    fn zip(mut self, cartridges: &Cartridges) -> zip::result::ZipResult<Self> {
        for cartridge in cartridges.iter() {
            let cartridge_name = cartridge
                .split("/")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .clone(); // ???

            // check for outdirs existance before writing to it
            let zip_file_path_str = String::from(format!("{}/{}.zip", self.output_dir, cartridge_name));
            let zip_file_path = path::Path::new(zip_file_path_str.as_str());
            let zip_file = fs::File::create(zip_file_path).unwrap();
            let mut zip = zip::ZipWriter::new(zip_file);

            let cartridge_dir_path = path::Path::new(cartridge);
            let dir_entries = fs::read_dir(cartridge_dir_path);

            if let Ok(entries) = dir_entries {
                self.zip_entries(&mut zip, &self.options, &String::from(cartridge_name), &cartridge, entries)?;
                self.zip_files.insert(String::from(cartridge_name), zip_file_path_str);
                zip.finish().unwrap();
            }
        }

        return Ok(self);
    }
}

#[derive(Default, Deserialize, Clone)]
struct DWConfig {
    hostname: String,
    username: String,
    password: String,
    version: String
}

#[derive(Clone)]
struct Demandware {
    config: DWConfig,
}

fn request(dw: &Demandware, method: reqwest::Method, url: String) -> reqwest::blocking::RequestBuilder {
    let url = format!(
        "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}", 
        dw.config.hostname,
        dw.config.version, 
        url
    );

    return reqwest::blocking::Client::new()
        .request(method, url)
        .basic_auth(dw.config.username.clone(), Some(dw.config.password.clone()));
}

impl Demandware {
    fn new() -> Self {
        let mut s = Self {
            config: DWConfig::default(),
        };

        s.parse_config();
        // s.remote_clear(); @TODO: implement remote_clear

        return s;
    }

    fn parse_config(&mut self) -> &Self {
        let dw_config = fs::read_to_string("./dw.json");

        if let Err(err) = dw_config {
            panic!("Can not read config file, {}", err);
        }

        let config: Result<DWConfig, serde_json::Error> = serde_json::from_str(dw_config.unwrap().as_str());

        if let Err(err) = config {
            panic!("Can not parse config file, {}", err);
        }

        self.config = config.unwrap();
        return self;
    }

    fn remote_send_zip(&self, path: String, name: String) -> Result<(), reqwest::Error> {
        let file_fd = fs::File::open(path).unwrap();
        request(self, reqwest::Method::PUT, format!("{}.zip", name))
            .body(file_fd)
            .send()?;
        return Ok(());
    }

    fn remote_unzip(&self, name: String) -> Result<(), reqwest::Error> {
        let mut body = HashMap::new();
        body.insert("method", "UNZIP");
        request(self, reqwest::Method::POST, format!("{}.zip", name))
            .form(&body)
            .send()?;
        return Ok(());
    }

    fn remote_remove(&self, name: String) -> Result<(), reqwest::Error> {
        request(self, reqwest::Method::DELETE, format!("{}.zip", name))
            .send()?;
        return Ok(());
    }

    // @TODO: implement remote_clear
    // fn remote_clear(&self) -> &Self {
    //     return self;
    // }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uploader: Uploader = Uploader::new()
        .parse_args(std::env::args())
        .set_flags()
        .set_cartridges();

    let zip_handler: ZipHanlder = ZipHanlder::new()
        .reset_outdir()
        .zip(&uploader.cartridges)?;

    let demandware = sync::Arc::new(Demandware::new());
    let mut running_threads: Vec<thread::JoinHandle<Result<(), reqwest::Error>>> = Vec::new();

    for (name, path) in zip_handler.zip_files {
        let demandware = sync::Arc::clone(&demandware);

        running_threads.push(thread::spawn(move || {
            // upload
            demandware.remote_send_zip(path.clone(), name.clone())?;

            // unzip
            demandware.remote_unzip(name.clone())?;

            // delete
            demandware.remote_remove(name.clone())?;

            return Ok(());
        }));
    }

    for t in running_threads {
        t.join().unwrap()?;
    }

    return Ok(());
}
