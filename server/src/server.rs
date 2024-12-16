use bytes::Bytes;
use chrono::Utc;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::{body, Request, Response};
use rand::Rng;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use url::form_urlencoded;
pub struct Server {
    pub name: String,
    pub port: u16,
}

fn get_unix_time_as_bigint() -> i64 {
    Utc::now().timestamp_millis() // Get the current Unix time in milliseconds
}
impl Server {
    pub async fn new(name: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            port: port,
        }
    }

    pub fn handle_std_request(&self) -> Result<Response<Full<Bytes>>, hyper::Error> {
        return Ok(hyper::Response::builder()
            .status(404)
            .body(Full::from(Bytes::from("Not Found")))
            .unwrap());
    }

}

impl Server {
    pub async fn handle_addfile(
        &self,
        req_bytes: Bytes,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let response_body = "File added successfully";
        Ok(hyper::Response::builder()
            .status(200)
            .body(Full::from(Bytes::from(response_body)))
            .unwrap())
    }

    pub async fn handle_getfile(
        &self,
        req_bytes: Bytes,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let response_body = "temporary response body";
        Ok(hyper::Response::builder()
            .status(200)
            .body(Full::from(Bytes::from(response_body)))
            .unwrap())
    }
}
