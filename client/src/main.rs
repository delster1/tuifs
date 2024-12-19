mod app;
mod ui;
mod httpclient;
mod statefullist;

use app::App;
use httpclient::{CustomHTTPClient, IpAndPort};

use color_eyre::Result;
use std::env;
#[tokio::main]
async fn main() -> Result<()> {

    let mut default_serverlocation = IpAndPort::default();

    let args : Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() > 1 {
         default_serverlocation = IpAndPort::new(args[1].clone());
    }
    color_eyre::install()?; // Setup error handling

    let client = CustomHTTPClient::new(&default_serverlocation.to_string()).await.unwrap();

    let mut terminal = ratatui::init();

    let app_result = App::new(Some(client)).run(&mut terminal);

    if let Err(err) = ratatui::try_restore() {
        eprintln!(
            "Failed to restore the terminal, run reset or restart terminal :/ : {}",
            err
        );
    }

    app_result?;
    Ok(())
}

