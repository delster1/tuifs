use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response};
use std::sync::Arc;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use hyper::server::conn::http1::Builder;
use hyper_util::rt::TokioIo;
use hyper::service::service_fn;
mod server;
use std::env;
use crate::server::Server;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3333").await.unwrap();

    let args: Vec<String> = env::args().collect();


    let mut server = Server::new("server1", 3333).await;
    let binding = &String::from("./storage");
    let storage_dir = args.get(1).unwrap_or(binding);

    println!("Server listening on http://127.0.0.1:{}", server.port);
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
                let req_body : hyper::body::Incoming = req.into_body();
                server.handle_addfile(req_body).await
            }
            "/getfiles" => {
                let whole_body = req.collect().await?.to_bytes();
                server.handle_getfiles(whole_body).await
            }

            _ => server.handle_std_request(),
        }
    }

