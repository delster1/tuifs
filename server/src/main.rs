use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use hyper::server::conn::http1::Builder;
use hyper_util::rt::TokioIo;
use hyper::service::{service_fn, Service};
mod server;
use crate::server::Server;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = Server::new("server1", 3333).await;
    let listener = TcpListener::bind("127.0.0.1:3333").await.unwrap();
    println!("Server listening on http://127.0.0.1:3333");

    // Wrap `Server` in an `Arc` for shared ownership
    let server_arc = Arc::new(server);
    loop {
        let (stream, _) = listener.accept().await?; // Use tokio::net::TcpStream
        let server_arc = Arc::clone(&server_arc);
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle_request(req, Arc::clone(&server_arc))),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
        req: Request<hyper::body::Incoming>,
        server: Arc<Server>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        match req.uri().path() {
            "/addfile" => {
                let whole_body = req.collect().await?.to_bytes();
                server.handle_addfile(whole_body).await
            }
            "/getfile" => {
                let whole_body = req.collect().await?.to_bytes();
                server.handle_getfile(whole_body).await
            }

            _ => server.handle_std_request(),
        }
    }

