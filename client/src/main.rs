mod app;
mod httpclient;

use app::App;
use color_eyre::Result;
use httpclient::{CustomHTTPClient, IpAndPort};
use ratatui::DefaultTerminal;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?; // Setup error handling

    let default = IpAndPort::default();
    let client = CustomHTTPClient::new(&default.to_string()).await.unwrap();

    let mut terminal = ratatui::init();
    let app_result = App::new(client).run(&mut terminal);

    if let Err(err) = ratatui::try_restore() {
        eprintln!(
            "Failed to restore the terminal, run reset or restart terminal :/ : {}",
            err
        );
    }

    app_result?;
    Ok(())
}

