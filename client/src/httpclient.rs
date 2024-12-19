use color_eyre::Result;
use futures::stream;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full, StreamBody};
use hyper::body::{Body, Bytes, Frame};
use hyper::client::conn::http1::SendRequest;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use map_ok::MapOk;
use std::fmt;
use std::io::Error;
use std::net::IpAddr;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::net::TcpStream;
use tokio_util::io::ReaderStream;
pub struct CustomHTTPClient {
    pub address: String,
    pub sender: SendRequest<BoxBody<Bytes, Error>>,
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

        Ok(CustomHTTPClient {
            address: address_clone,
            sender,
        })
    }

    pub async fn send_my_request<B>(&mut self, request: Request<B>) -> Result<hyper::body::Bytes>
    where
        B: Body + Send + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        BoxBody<hyper::body::Bytes, Error>: From<B>,
    {
        let request = request.map(Into::into);
        let res = self.sender.send_request(request).await?;
        let res_bytes = res.collect().await?.to_bytes();
        Ok(res_bytes)
    }

    pub async fn send_file(&mut self, filepath: PathBuf) -> Result<hyper::body::Bytes>
where {
        let file = File::open(filepath).await;
        if file.is_err() {
            eprintln!("ERROR: Unable to open file.");
        }

        let uri = format!("http://{}/addfile", self.address);
        let file: File = file.unwrap();

        // Wrap to a tokio_util::io::ReaderStream
        let reader_stream = ReaderStream::new(file);
        // Convert to http_body_util::BoxBody
        let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
        let boxed_body = stream_body.boxed();

        // Send request
        let request = Request::builder().uri(uri).body(boxed_body).unwrap();

        let res_bytes = self.send_my_request(request).await.unwrap();

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

impl IpAndPort {
    pub fn new(ip_and_port: String) -> Self {
        let parts: Vec<&str> = ip_and_port.split(':').collect();
        let ip = parts[0].parse().unwrap();
        let port = parts[1].parse().unwrap();
        Self { ip, port }
    }
}
