use bytes::Bytes;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

// use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::Response;
// use rand::Rng;
// use std::collections::HashMap;
// use std::net::IpAddr;
// use std::sync::Arc;
// use url::form_urlencoded;
pub struct Server {
    pub name: String,
    pub port: u16,
    storage_dir: String,
}

impl Server {
    pub async fn new(name: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            port: port,
            storage_dir: "./storage".to_string(),
        }
    }

    pub fn set_storage_dir(&mut self, storage_dir: &str) -> std::io::Result<()> {
        self.storage_dir = storage_dir.to_string();

        let storage_path = Path::new(&self.storage_dir);
        if !storage_path.exists() {
            fs::create_dir(storage_path).unwrap();
            println!("Created storage directory at {:?}", storage_path);
        } else {
            println!("Storage directory already exists at {:?}", storage_path);
        }
        Ok(())

    }
}

impl Server {

    pub async fn handle_addfile(
        &self,
        req_bytes: Bytes,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let response_body = "to be implemented";
        Ok(hyper::Response::builder()
            .status(200)
            .body(Full::from(Bytes::from(response_body)))
            .unwrap())
    }

    pub async fn handle_addfolder(
        &self,
        req_bytes: Bytes,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let response_body = "to be implemented";
        Ok(hyper::Response::builder()
            .status(200)
            .body(Full::from(Bytes::from(response_body)))
            .unwrap())
    }

    pub async fn handle_getfiles(
        &self,
        req_bytes: Bytes,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let paths = fs::read_dir(&self.storage_dir).unwrap();
        let mut paths_list : Vec<String> = Vec::new();
        for path in paths{
            paths_list.push(path.unwrap().path().display().to_string());
        }
        let response_body = serde_json::to_string(&paths_list).unwrap();
        Ok(hyper::Response::builder()
            .status(200)
            .body(Full::from(Bytes::from(response_body)))
            .unwrap())
    }

    pub fn handle_std_request(&self) -> Result<Response<Full<Bytes>>, hyper::Error> {
        return Ok(hyper::Response::builder()
            .status(404)
            .body(Full::from(Bytes::from("Not Found")))
            .unwrap());
    }


}
