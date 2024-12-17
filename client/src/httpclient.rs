use hyper::body::Bytes;

use http_body_util::Full;
use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use http_body_util::Empty;
use std::fmt;
use std::net::IpAddr;
use hyper::client::conn::http1::SendRequest;
use color_eyre::Result;
use hyper::{Request, Response};

pub struct CustomHTTPClient {
    pub address: String,
    pub sender: SendRequest<Empty<Bytes>>,
}

impl CustomHTTPClient {
    pub async fn new(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let url = address.parse::<hyper::Uri>()?;
        let address_clone = address.to_string();
        // Get the host and port
        let host = url.host().expect("uri has no host");
        let port = url.port_u16().unwrap_or(80);
        let address = format!("{}:{}", host, port);

        // Open a TCP connection
        let stream = TcpStream::connect(address).await?;
        let io = TokioIo::new(stream);

        // Create the Hyper client
        let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        // Spawn the connection to poll in the background
        tokio::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection failed: {:?}", err);
            }
        });

        Ok(CustomHTTPClient {address: address_clone, sender })
    }

    pub async fn send_request(&mut self, request: Request<Empty<Bytes>>) -> Result<hyper::body::Bytes> {
        let mut res = self.sender.send_request(request).await?;
        let res_bytes = res.collect().await?.to_bytes();
        Ok(res_bytes)
    }

}

impl Default for CustomHTTPClient {
    fn default() -> Self {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Self::new("127.0.0.1:3333"))
            .expect("Failed to create default HttpClient")
    }
}

impl fmt::Debug for CustomHTTPClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomHTTPClient").finish()
    }
}

#[derive(Debug)]
pub struct IpAndPort {
    pub ip: IpAddr,
    pub port: u16,
}

impl Default for IpAndPort {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".parse().unwrap(),
            port: 3333,
        }
    }
}

impl fmt::Display for IpAndPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

