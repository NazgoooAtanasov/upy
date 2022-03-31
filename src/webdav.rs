use crate::parser;
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Deserialize)]
pub struct Config {
    pub hostname: String,
    pub username: String, 
    pub password: String,
    pub version: String
}

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

    pub async fn upload_file(&self, system_file_path: &str, zip_file_name: &str) {
        let file = File::open(system_file_path).await.unwrap();

        let stream = FramedRead::new(file, BytesCodec::new());
        let body = reqwest::Body::wrap_stream(stream);

        let url = format!(
            "https://{}/on/demandware.servlet/webdav/Sites/Cartridges/{}/{}.zip",
            self.config.as_ref().unwrap().hostname,
            self.config.as_ref().unwrap().version,
            zip_file_name
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

    pub async fn send_cartridge(&self, cartridges_parent_path: &str, name: &str) {
        self.upload_file(format!("{}/{}.zip", &cartridges_parent_path, name).as_str(), &name).await;
        self.unzip_zip(format!("{}.zip", &name).as_str()).await;
        self.delete_zip(format!("{}.zip", &name).as_str()).await;
    }
}
