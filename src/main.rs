use cast_away::{
    HOST, KIOSK_SCRIPT, PORT, ROOT_DIR,
    application::app::{App, UserEvent, WebConfig}, get_or_default_env,
};
use std::env;
use std::path::PathBuf;

use actix_files::{Files, NamedFile};
use actix_web::{HttpRequest, HttpServer, web};
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, Window},
};
// FCAST STUFF
use fcast_sender_sdk::{
    context::CastContext,
    device::{
        CastingDevice, DeviceConnectionState, DeviceEventHandler, DeviceFeature, DeviceInfo,
        EventSubscription, KeyEvent, LoadRequest, MediaEvent, PlaybackState, ProtocolType, Source,
    },
    DeviceDiscovererEventHandler, IpAddr,
};

use tokio::{
    runtime::Runtime,
    sync::mpsc::{channel, Receiver, Sender},
};
// FCAST STUFF END

async fn index(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = format!("./{ROOT_DIR}/index.html").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

// FCAST STUFF
#[derive(Debug)]
enum DeviceEvent {
    ConnectionStateChanged(DeviceConnectionState),
    VolumeChanged(f64),
    TimeChanged(f64),
    PlaybackStateChanged(PlaybackState),
    DurationChanged(f64),
    SpeedChanged(f64),
    SourceChanged(Source),
}

#[derive(Debug)]
enum Event {
    Quit,
    DeviceAvailable(DeviceInfo),
    DeviceRemoved(String),
    DeviceChanged(DeviceInfo),
    Connect(String),
    Disconnect,
    FromDevice {
        id: usize,
        event: DeviceEvent,
    },
    /// User requested that a local file should be casted
    CastLocalRequested,
    CastLocal {
        media_type: String,
        handle: String,
    },
    ChangeVolume(f64),
    Seek(f64),
}

struct DiscoveryEventHandler {
    event_tx: Sender<Event>,
}

impl DiscoveryEventHandler {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self { event_tx }
    }
}

impl DeviceDiscovererEventHandler for DiscoveryEventHandler {
    fn device_available(&self, device_info: DeviceInfo) {
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            event_tx
                .send(Event::DeviceAvailable(device_info))
                .await
                .unwrap();
        });
    }

    fn device_removed(&self, device_name: String) {
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            event_tx
                .send(Event::DeviceRemoved(device_name))
                .await
                .unwrap();
        });
    }

    fn device_changed(&self, device_info: DeviceInfo) {
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            event_tx
                .send(Event::DeviceChanged(device_info))
                .await
                .unwrap();
        });
    }
}

struct DevEventHandler {
    event_tx: Sender<Event>,
    id: usize,
}

impl DevEventHandler {
    pub fn new(event_tx: Sender<Event>, id: usize) -> Self {
        Self { event_tx, id }
    }

    fn send_event(&self, event: DeviceEvent) {
        let id = self.id;
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            if let Err(err) = event_tx.send(Event::FromDevice { id, event }).await {
                println!("Failed to send event: {err}");
            }
        });
    }
}

impl DeviceEventHandler for DevEventHandler {
    fn connection_state_changed(&self, state: DeviceConnectionState) {
        self.send_event(DeviceEvent::ConnectionStateChanged(state));
    }

    fn volume_changed(&self, volume: f64) {
        self.send_event(DeviceEvent::VolumeChanged(volume));
    }

    fn time_changed(&self, time: f64) {
        self.send_event(DeviceEvent::TimeChanged(time));
    }

    fn playback_state_changed(&self, state: PlaybackState) {
        self.send_event(DeviceEvent::PlaybackStateChanged(state));
    }

    fn duration_changed(&self, duration: f64) {
        self.send_event(DeviceEvent::DurationChanged(duration));
    }

    fn speed_changed(&self, speed: f64) {
        self.send_event(DeviceEvent::SpeedChanged(speed));
    }

    fn source_changed(&self, source: Source) {
        self.send_event(DeviceEvent::SourceChanged(source));
    }

    fn key_event(&self, _event: KeyEvent) {}

    fn media_event(&self, event: MediaEvent) {
        println!("Media event: {event:?}");
    }

    fn playback_error(&self, message: String) {
        println!("Playback error: {message}");
    }
}
// FCAST STUFF END

// #[actix_web::main]
#[tokio::main]
async fn main() {
    // use app event_loop_proxy to perform the propagation of the events that are managed by FCAST device event handler
    // as seen in the example code for "FCAST STUFF" the events are sent with "event_tx", similarly the events will be
    // sent with my event_loop_proxy and managed inside the App user_event function
    let srv_host = get_or_default_env("SRV_HOST", HOST);
    let srv_port = get_or_default_env("SRV_PORT", PORT);
    let srv_root = get_or_default_env("SRV_ROOT", ROOT_DIR);

    let web_srv = tokio::spawn({
        HttpServer::new(move || {
            actix_web::App::new()
                .route("/", web::get().to(index))
                .service(Files::new("/", format!("./{srv_root}")))
        })
        .bind(format!("{srv_host}:{srv_port}"))
        .unwrap()
        .run()
    });

    let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new(srv_host, srv_port, &event_loop);
    let cli_args: Vec<String> = env::args().collect();
    cli_args.iter().for_each(|x| match x.as_str() {
        "--kiosk" => {
            let fullscreen = Some(Fullscreen::Borderless(None));
            app.set_window_attributes(Window::default_attributes().with_fullscreen(fullscreen));
            app.set_initialization_script(KIOSK_SCRIPT);
        }
        _ => (),
    });

    event_loop.run_app(&mut app).unwrap();
    web_srv.abort();

    // FCAST STUFF
    // let cast_context = CastContext::new().unwrap();

    // let discovery_event_handler = DiscoveryEventHandler::new(event_tx.clone());
    // cast_context.start_discovery(Arc::new(discovery_event_handler));
    // FCAST STUFF END

}
