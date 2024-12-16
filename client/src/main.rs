#![allow(unused)] // THIS IS STUPID BUT IM DOING IT
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use std::fmt;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::rt::TokioIo;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, ListState, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::{io as other_io, net::IpAddr};
use tokio::io::{self, AsyncWriteExt as _};
use tokio::net::TcpStream;
// we use a struct to store the state of our application and stuff
#[derive(Debug, Default)]
pub struct App<'a> {
    pub title: &'a str,
    pub server_files: StatefulList<&'a str>,
    client: CustomHTTPClient,
    exit: bool,
}

#[derive(Debug)]
pub struct IpAndPort {
    ip: IpAddr,
    port: u16,
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

pub struct CustomHTTPClient {
    sender: hyper::client::conn::http1::SendRequest<Empty<Bytes>>,
}
impl CustomHTTPClient {
    pub async fn new(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let url = address.parse::<hyper::Uri>()?;

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

        Ok(CustomHTTPClient {sender})
    }
}


impl Default for CustomHTTPClient {
    fn default() -> Self {
        // Use a default address, e.g., localhost:3333
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

#[derive(Debug, Default)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<'a> App<'a> {
    pub fn new(client : CustomHTTPClient) -> Self {
        Self {
            title: "tuifs",
            server_files: StatefulList::default(),
            client,
            exit: false,
        }
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("handle events failed");
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed\n{key_event:#?}")),

            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('u') => self.upload_file(),
            KeyCode::Char('d') => self.download_file(),
            KeyCode::Char('s') => self.set_server_location(),
            KeyCode::Char('g') => self.get_server_files(),
            _ => {}
        }
        Ok(())
    }

    fn upload_file(&mut self) {
        println!("uploading file");
    }

    fn download_file(&mut self) {
        println!("downloading file");
    }

    fn set_server_location(&mut self) {
        println!("setting server location");
    }

    fn get_server_files(&mut self) {
        println!("getting server files");
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

// rendering ui requires passing a Frame to draw(), Frames have render_widget(), which renders any
// type implementing the widget trait, here, we implement the Widget trait for the App struct
impl<'a> Widget for &App<'a> {
    
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" This will eventually be a tuifs B)".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec!["Values: ".into()])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {

    // eventually ask user for ip and port before initial launch B)
    let default : IpAndPort = Default::default();
    let sender = CustomHTTPClient::new(&default.to_string()).await.unwrap();
    
    color_eyre::install();
    let mut terminal = ratatui::init();
    let app_result = App::new(sender).run(&mut terminal);
    if let Err(err) = ratatui::try_restore() {
        eprintln!(
            "failed to restore the terminal, run reset or restart terminal :/ : {}",
            err
        );
    } // putting this here ensures any errors are propagated to user

    app_result
}
