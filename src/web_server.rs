use std::{sync::Mutex, thread};

use anyhow::{Result, anyhow};
use rouille::{Request, Response, Server, extension_to_mime, router, try_or_400};

use crate::{config::ClimateSettings, events::{Event, EventSender}};

static WEB: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/web");

pub fn start_server<S>(settings: ClimateSettings, event_sender: S) -> Result<()>
    where S: EventSender + Send + Sync + 'static
{
    let handler = Mutex::new(RequestHandler::new(settings, event_sender));

    let server = Server::new("0.0.0.0:8000", move |request| {
        router!(request,
            (GET) (/api/settings) => {
                handler.lock().unwrap().get_settings()
            },
            (POST) (/api/settings) => {
                handler.lock().unwrap().post_settings(request)
            },
            _ => asset_response(request.url())
        )
    });

    let server = server.map_err(|e| anyhow!(e))?;

    thread::spawn(|| {
        server.run();
    });

    Ok(())
}

struct RequestHandler<S> {
    event_sender: S,
    settings: ClimateSettings,
}

impl<S: EventSender + Send + Sync + 'static> RequestHandler<S> {
    fn new(settings: ClimateSettings, event_sender: S) -> Self {
        Self {
            settings,
            event_sender,
        }
    }

    fn get_settings(&self) -> Response {
        Response::json(&self.settings)
    }

    fn post_settings(&mut self, request: &Request) -> Response {
        self.settings = try_or_400!(rouille::input::json_input(request));
        self.event_sender.send_event(Event::SettingsUpdate(self.settings.clone())).unwrap();
        Response::empty_204()
    }
}

fn asset_response(url: String) -> Response {
    let path = if url == "/" {
        "index.html"
    } else {
        url.strip_prefix("/").unwrap()
    };

    if let Some(file) = WEB.get_file(path) {
        let ext = file.path().extension().unwrap().to_str().unwrap();
        Response::from_data(
            extension_to_mime(ext),
            file.contents()
        )
    } else {
        Response::empty_404()
    }
}
