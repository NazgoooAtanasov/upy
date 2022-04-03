use crate::parser;
use serde::Deserialize;
use std::fs::File;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub hostname: String,
    pub username: String, 
    pub password: String,
    pub version: String,
    pub cartridge: Vec<String>
}

#[derive(Clone)]
pub struct WebdavClient {
    pub config: Option<Config>,
    pub reqwest_client: reqwest::blocking::Client
}

impl WebdavClient {
    pub fn new() -> WebdavClient {
        WebdavClient { config: None, reqwest_client: reqwest::blocking::Client::new() }
    }

    pub fn set_config(&mut self, config_file: &str) {
        let config: Config = parser::parse_config(config_file);

        self.config = Some(Config {
            hostname: config.hostname,
            username: config.username,
            password: config.password,
            version: config.version,
            cartridge: config.cartridge
        });
    }

    fn generate_url(&self, path: &str) -> String {
        format!(
            "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}",
            self.config.as_ref().unwrap().hostname,
            self.config.as_ref().unwrap().version,
            path
        )
    }

    pub fn upload_file(&self, system_file_path: &str, file: &str) {
        let file_fd = File::open(system_file_path).unwrap();

        let url = self.generate_url(file);

        let _res = self.reqwest_client.put(url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .body(file_fd)
            .send()
            .expect("Problem with sending upload file request");
    }

    pub fn unzip_zip(&self, zip_name: &str) {
        let mut form_data = std::collections::HashMap::new();
        form_data.insert("method", "UNZIP");

        let url = self.generate_url(zip_name);

        let _unzip_response = self.reqwest_client.post(url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .form(&form_data)
            .send()
            .expect("Problem with sending unzipping file request");
    }

    pub fn delete(&self, path: &str) {
        let url = self.generate_url(path);

        let client = reqwest::blocking::Client::new();
        let _delete_response = client.delete(&url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .send()
            .expect("Problem with sending delete file request");
    }

    pub fn create_directory(&self, directory_path: &str) {
        let url = self.generate_url(directory_path);

        let client = reqwest::blocking::Client::new();
        let _response = client.request(reqwest::Method::from_bytes(b"MKCOL").expect("Could not generate MKCOL method"), url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .send()
            .expect("Problem with creating folder request");
    }

    pub fn send_cartridge(&self, cartridges_parent_path: &str, name: &str) {
        self.upload_file(
            format!("{}/{}.zip",
                &cartridges_parent_path,
                name
            ).as_str(),
            format!("{}.zip", &name).as_str()
        );

        self.unzip_zip(format!("{}.zip", &name).as_str());

        self.delete(format!("{}.zip", &name).as_str());
    }
}
