use crate::parser;
use serde::Deserialize;
use std::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub hostname: String,
    pub username: String, 
    pub password: String,
    pub version: String
}

#[derive(Clone)]
pub struct WebdavClient {
    pub config: Option<Config>,
    pub reqwest_client: reqwest::Client
}

impl WebdavClient {
    pub fn new() -> WebdavClient {
        WebdavClient { config: None, reqwest_client: reqwest::Client::new() }
    }

    pub fn set_config(&mut self, config_file: &str) {
        let config: Config = parser::parse_config(config_file);

        self.config = Some(Config {
            hostname: config.hostname,
            username: config.username,
            password: config.password,
            version: config.version
        });
    }

    pub fn upload_file_blocking(&self, system_file_path: &str, file: &str) {
        let file_fd = File::open(system_file_path).unwrap();

        let url = format!(
            "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}",
            self.config.as_ref().unwrap().hostname,
            self.config.as_ref().unwrap().version,
            file
        );

        let client = reqwest::blocking::Client::new();
        let _res = client.put(url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .body(file_fd)
            .send()
            .unwrap()
            .text()
            .unwrap();
    }

    // delete in the future
    pub async fn upload_file(&self, system_file_path: &str, file: &str) {
        let file_fd = tokio::fs::File::open(system_file_path).await.unwrap();

        let stream = FramedRead::new(file_fd, BytesCodec::new());
        let body = reqwest::Body::wrap_stream(stream);

        let url = format!(
            "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}",
            self.config.as_ref().unwrap().hostname,
            self.config.as_ref().unwrap().version,
            file
        );

        let _upload_response = self.reqwest_client.put(url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .body(body)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
    }

    pub async fn unzip_zip(&self, zip_name: &str) {
        let mut form_data = std::collections::HashMap::new();
        form_data.insert("method", "UNZIP");

        let url = format!(
            "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}",
            self.config.as_ref().unwrap().hostname,
            self.config.as_ref().unwrap().version,
            zip_name
        );

        let _unzip_response = self.reqwest_client.post(url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .form(&form_data)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
    }

    // delete in the future
    pub async fn delete_zip(&self, zip_name: &str) {
        let url = format!(
            "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}",
            self.config.as_ref().unwrap().hostname,
            self.config.as_ref().unwrap().version,
            zip_name
        );

        let _delete_zip_response = self.reqwest_client.delete(url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
    }

    pub fn delete(&self, path: &str) {
        let url = format!(
            "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}",
            self.config.as_ref().unwrap().hostname,
            self.config.as_ref().unwrap().version,
            path
        );

        let client = reqwest::blocking::Client::new();
        let _delete_response = client.delete(&url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .send()
            .unwrap()
            .text()
            .unwrap();
    }

    pub fn create_directory(&self, directory_path: &str) {
        let url = format!(
            "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}",
            self.config.as_ref().unwrap().hostname,
            self.config.as_ref().unwrap().version,
            directory_path
        );

        let client = reqwest::blocking::Client::new();
        let _response = client.request(reqwest::Method::from_bytes(b"MKCOL").expect("Could not generate MKCOL method"), url)
            .basic_auth(self.config.as_ref().unwrap().username.clone(), Some(self.config.as_ref().unwrap().password.clone()))
            .send()
            .unwrap()
            .text()
            .unwrap();
    }

    pub async fn send_cartridge(&self, cartridges_parent_path: &str, name: &str) {
        self.upload_file(
            format!("{}/{}.zip",
                &cartridges_parent_path,
                name
            ).as_str(),
            format!("{}.zip", &name).as_str()
        ).await;

        self.unzip_zip(format!("{}.zip", &name).as_str()).await;

        self.delete_zip(format!("{}.zip", &name).as_str()).await;
    }
}
