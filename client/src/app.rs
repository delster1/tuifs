/// This file contains the main application logic for the TUI file sharing application.
/// - The App struct contains the main state for the application and related functions
/// in this file:
/// - run: main start point
/// - handle_key_event: handles key events based on current screen state
/// - upload/download server files
/// - server configuration backend
use hyper::header::HeaderValue;
use crate::httpclient::CustomHTTPClient;
use tokio::io::AsyncWriteExt;
use crate::statefullist::StatefulList;
use tokio::fs::File;
use crate::ui::ui;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use futures::executor::block_on;
///
use http_body_util::BodyExt;
use http_body_util::{combinators::BoxBody, Empty, Full, StreamBody};
use hyper::body::{Body, Bytes, Frame};
use hyper::Request;
use ratatui::{
    // layout::Rect,
    // style::Stylize,
    // symbols::border,
    // text::{Line, Text},
    // widgets::{Block, ListState, Paragraph, StatefulWidget, Widget},
    DefaultTerminal, // , Frame,
};
use serde_json as serrde_json;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::{borrow::BorrowMut, fmt};

pub enum CurrentScreen {
    Start, // Main screen - Menu and stuff
    ServerFiles,
    Uploading,   // screen while a file is uploading - should show success.
    Downloading, // screen while a file is downloading - should show success.
    Configuring, // screen for configuring the server location, download location, upload location
}

pub enum CurrentlyConfiguring {
    DownloadLocation,
    ServerLocation,
    UploadLocation,
}

impl Default for CurrentScreen {
    fn default() -> Self {
        CurrentScreen::Start
    }
}

impl fmt::Debug for CurrentScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for CurrentlyConfiguring {
    fn default() -> Self {
        CurrentlyConfiguring::ServerLocation
    }
}

impl fmt::Debug for CurrentlyConfiguring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Default)]
pub struct App<'a> {
    pub title: &'a str,
    pub input: String,
    pub server_files: StatefulList<String>,
    pub client: Option<CustomHTTPClient>,
    pub exit: bool,
    pub current_screen: CurrentScreen,
    pub currently_configuring: Option<CurrentlyConfiguring>,
    download_location: String,
}

impl<'a> App<'a> {
    pub fn new(client: Option<CustomHTTPClient>) -> Self {
        Self {
            title: "tuifs",
            input: String::new(),
            server_files: StatefulList::default(),
            client,
            exit: false,
            current_screen: CurrentScreen::Start,
            currently_configuring: None,
            download_location: String::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if self.client.is_some() {
            self.get_server_files();
        } else {
            self.current_screen = CurrentScreen::Configuring;
            self.currently_configuring = Some(CurrentlyConfiguring::ServerLocation);
        }
        while !self.exit {
            terminal.draw(|frame| ui(frame, self.borrow_mut()))?;
            if let Event::Key(key_event) = event::read()? {
                self.handle_key_event(key_event).unwrap();
            }
        }
        Ok(())
    }

    fn upload_file(&mut self, filepath: String) -> Result<()> {
        let file_path: PathBuf = PathBuf::from(&filepath);
        let file_data = fs::metadata(&file_path)?;
        if file_data.is_dir() {
            // eventually add code to upload directories
            println!("Cannot upload directories");
            return Err(
                io::Error::new(io::ErrorKind::Other, "Cannot upload directories (yet)").into(),
            );
        }

        let res = block_on(self.client.as_mut().unwrap().send_file(file_path)).unwrap();

        Ok(())
    }

    fn download_file(&mut self) {
        let uri = format!("http://{}/downloadfile", self.client.as_ref().unwrap().address);
        
        let file_name = self.server_files.items.get(self.server_files.state.selected().unwrap()).unwrap();
        let file_value = HeaderValue::from_str(file_name).unwrap();

        let mut req: Request<BoxBody<Bytes, std::io::Error>> = Default::default();

        *req.uri_mut() = uri.parse().unwrap();
        req.headers_mut().insert("file", file_value);
        
        let response = block_on(self.client.as_mut().unwrap().send_request(req)).unwrap();
        let (_, body) = response.into_parts();

        let body = body.collect();
        let body = block_on(body).unwrap().to_bytes();

        let mut file_path: PathBuf = PathBuf::from(&self.download_location);
        file_path.push(format!("{}", file_name));

        let mut file = block_on(tokio::fs::File::create(file_path)).unwrap();
        block_on(file.write_all(&body)).unwrap();
    }

    fn set_server_location(&mut self) {
        // println!("setting server location");
    }

    fn get_server_files(&mut self) {
        let uri = format!("http://{}/getfiles", self.client.as_ref().unwrap().address);
        let mut req: Request<BoxBody<Bytes, std::io::Error>> = Default::default();
        *req.uri_mut() = uri.parse().unwrap();
        // println!("sending request{:?}", req);
        let response = block_on(self.client.as_mut().unwrap().send_request(req)).unwrap();
        let (_, body) = response.into_parts();

        let body = body.collect();
        let body = block_on(body).unwrap().to_bytes();

        let server_files: Vec<String> = serrde_json::from_slice(&body).unwrap();

        self.server_files.items = server_files;
        self.server_files.state.select(Some(0));
        *self.server_files.state.offset_mut() = 0;
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

// all event handlers here, can eventually move this to a separate file if necessary
impl<'a> App<'a> {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.current_screen {
            CurrentScreen::Start => self.handle_start_screen(key_event)?,
            CurrentScreen::ServerFiles => self.handle_server_files_screen(key_event)?,
            CurrentScreen::Uploading => self.handle_uploading_screen(key_event)?,
            CurrentScreen::Downloading => self.handle_downloading_screen(key_event)?,
            CurrentScreen::Configuring => self.handle_configuring_screen(key_event)?,
        }

        Ok(())
    }

    fn handle_uploading_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => {
                self.current_screen = CurrentScreen::Start;
            }
            KeyCode::Char('c') => {
                self.current_screen = CurrentScreen::Configuring;
                self.set_server_location();
            }
            KeyCode::Char('g') => {
                self.current_screen = CurrentScreen::ServerFiles;
                self.get_server_files();
            }
            _ => {}
        }
        Ok(())
    }

    fn change_download_location(&mut self, location: String) {
        self.download_location = location;
    }
    fn handle_configuring_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Esc => {
                self.input = String::new();
                self.current_screen = CurrentScreen::Start;
            }
            KeyCode::Char('\n') | KeyCode::Enter => {
                if let Some(editing) = &self.currently_configuring {
                    match editing {
                        CurrentlyConfiguring::DownloadLocation => {
                            self.change_download_location(self.input.clone());
                            self.download_file();
                            self.currently_configuring = None;
                            self.input = String::new();
                            self.current_screen = CurrentScreen::ServerFiles;
                        }
                        CurrentlyConfiguring::ServerLocation => {
                            self.client =
                                Some(block_on(CustomHTTPClient::new(&self.input)).unwrap());
                            self.currently_configuring = None;
                            self.input = String::new();
                            self.get_server_files();
                            self.current_screen = CurrentScreen::ServerFiles;
                        }
                        CurrentlyConfiguring::UploadLocation => {
                            let upload_output = self.upload_file(self.input.clone()); // this
                                                                                      // function has an async block
                            self.get_server_files();
                            match upload_output {
                                Ok(_) => {
                                    self.currently_configuring = None;
                                    self.input = String::new();
                                    self.current_screen = CurrentScreen::Uploading;
                                }
                                Err(e) => {
                                    self.input = format!("Error Uploading: {:?}", e);
                                }
                            }
                        }
                    }
                } else {
                    self.current_screen = CurrentScreen::ServerFiles;
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_downloading_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => {
                self.current_screen = CurrentScreen::Start;
            }
            KeyCode::Char('c') => {
                self.current_screen = CurrentScreen::Configuring;
                self.set_server_location();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_server_files_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('d') | KeyCode::Enter | KeyCode::Char('\n') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::DownloadLocation);
            }
            KeyCode::Char('c') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::ServerLocation);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.server_files.previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.server_files.next();
            }
            KeyCode::Esc => {
                self.currently_configuring = None;
                self.current_screen = CurrentScreen::Start;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_start_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('u') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::UploadLocation);
            }
            KeyCode::Char('d') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::DownloadLocation);
            }
            KeyCode::Char('c') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::ServerLocation);
            }
            KeyCode::Char('g') => {
                self.current_screen = CurrentScreen::ServerFiles;
                self.get_server_files();
            }
            _ => {}
        }
        Ok(())
    }
}
