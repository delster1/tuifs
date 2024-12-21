use futures::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, StreamBody};
use hyper::body::{Body, Bytes, Frame};
use hyper::client::conn::http1::SendRequest;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use map_ok::MapOk;
use std::error::Error;
use std::fmt;
use std::net::IpAddr;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::net::TcpStream;
use tokio_util::io::ReaderStream;

pub struct CustomHTTPClient {
    pub address: String,
    pub sender: SendRequest<BoxBody<Bytes, std::io::Error>>,
}

impl CustomHTTPClient {
    /// Creates a new HTTP client
    pub async fn new(address: &str) -> Result<Self, Box<dyn Error>> {
        let url = address.parse::<hyper::Uri>()?;
        let address_clone = address.to_string();

        // Get the host and port
        let host = url.host().ok_or("Missing host in URL")?;
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

    /// Generic function to send a request
    pub async fn send_request<B>(&mut self, request: Request<B>) -> Result<Response<hyper::body::Incoming>, Box<dyn Error>>
    where
        B: Body + Send + 'static,
        BoxBody<Bytes, std::io::Error>: From<B>,
        B::Error: Into<Box<dyn Error + Send + Sync>>,
    {
        let request = request.map(Into::into);
        let response: Response<hyper::body::Incoming> = self.sender.send_request(request).await?;
        Ok(response)
    }

    pub async fn send_file(
        &mut self,
        filepath: PathBuf,
    ) -> Result<Response<hyper::body::Incoming>, Box<dyn Error>>
where {
        let file = File::open(&filepath).await;
        if file.is_err() {
            // eprintln!("ERROR: Unable to open file.");
        }
        
        let file_name = filepath.file_name().unwrap().to_str().unwrap();
        let file_type = filepath.extension().unwrap().to_str().unwrap();

        let uri = format!("http://{}/addfile", self.address);
        let file: File = file.unwrap();

        // Wrap to a tokio_util::io::ReaderStream
        let reader_stream = ReaderStream::new(file);
        // Convert to http_body_util::BoxBody

        let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
        let boxed_body = stream_body.boxed();

        // Send request
        let request = Request::builder().uri(uri).header("file_name",file_name).header("file_type",file_type).body(boxed_body).unwrap();

        let res = self.send_request(request).await.unwrap();
        // println!("{:?}",res_bytes);
        Ok(res)
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
