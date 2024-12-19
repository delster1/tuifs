use bytes::Bytes;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::env;
// use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::Response;
// use rand::Rng;
// use std::collections::HashMap;
// use std::net::IpAddr;
// use std::sync::Arc;
// use url::form_urlencoded;
use std::path::{PathBuf};
pub struct Server {
    pub name: String,
    pub port: u16,
    storage_dir: PathBuf,
}
fn get_current_working_dir() -> String {
    let res = env::current_dir();
    match res {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(_) => "FAILED".to_string()
    }
}

impl Server {
    pub async fn new(name: &str, port: u16) -> Self {
        let storage_dir = Server::get_default_storage_path();
        Self {
            name: name.to_string(),
            port,
            storage_dir,
        }
    }
 /// Get default storage path in `server/storage`
    fn get_default_storage_path() -> PathBuf {
        // Find the executable's directory and resolve "server/storage" relative to it
        let exe_dir = env::current_exe()
            .expect("Failed to find the executable path")
            .parent()
            .expect("Executable path has no parent")
            .to_path_buf();

        // Traverse up to ensure it's relative to `server/`
        let server_dir = exe_dir
            .ancestors()
            .nth(2) // Traverse up two levels to reach `server/`
            .expect("Failed to resolve server directory")
            .to_path_buf();

        let storage_path = server_dir.join("storage");
        
        // Create the directory if it doesn't exist
        if !storage_path.exists() {
            fs::create_dir_all(&storage_path).expect("Failed to create storage directory");
            println!("Created storage directory at {:?}", storage_path);
        }

        storage_path
    }

    /// Set a custom storage directory
    pub fn set_storage_dir(&mut self, storage_dir: &str) -> std::io::Result<()> {
        let path = Path::new(storage_dir).to_path_buf();
        if !path.exists() {
            fs::create_dir_all(&path)?;
            println!("Created custom storage directory at {:?}", path);
        } else {
            println!("Custom storage directory already exists at {:?}", path);
        }
        self.storage_dir = path;
        Ok(())
    }
}

impl Server {

    pub async fn handle_addfile(
        &self,
        req_body: hyper::body::Incoming,
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
        println!("Recieved getfiles request, sending \n{:?}", paths_list);
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
