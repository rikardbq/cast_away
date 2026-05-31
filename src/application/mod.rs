use std::path::PathBuf;

use fcast_sender_sdk::{
    DeviceDiscovererEventHandler,
    device::{
        DeviceConnectionState, DeviceEventHandler, DeviceInfo, KeyEvent, MediaEvent, PlaybackState,
        Source,
    },
};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use rust_cast::{
    CastDevice,
    channels::{
        media::{Media, StreamType},
        receiver::CastDeviceApp,
    },
};
use serde::{Deserialize, Serialize};
use winit::event_loop::EventLoopProxy;

pub mod app;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IpcMethod {
    ListFiles,
    DiscoverDevices,
    ConnectToDevice,
    DisconnectFromDevice,
    RequestCastLocal,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize)]
struct IpcRequest {
    id: u64,
    method: IpcMethod,
    params: serde_json::Value,
}

#[derive(Serialize)]
struct IpcResponse {
    id: u64,
    result: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum DeviceEvent {
    ConnectionStateChanged(DeviceConnectionState),
    VolumeChanged(f64),
    TimeChanged(f64),
    PlaybackStateChanged(PlaybackState),
    DurationChanged(f64),
    SpeedChanged(f64),
    SourceChanged(Source),
}

#[derive(Debug)]
pub enum UserEvent {
    ExecEval(String),
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
    CastLocal {
        media_type: infer::Type,
        handle: PathBuf,
    },
    ChangeVolume(f64),
    Seek(f64),
}

// new tech start
#[derive(Debug)]
pub enum CastEvent {
    DeviceDiscovered(Option<String>)
}

pub struct DeviceDiscoveryEventHandler {
    event_proxy: EventLoopProxy<CastEvent>,
}

impl DeviceDiscoveryEventHandler {
    pub fn new(event_proxy: EventLoopProxy<CastEvent>) -> Self {
        Self { event_proxy }
    }
    pub fn device_discovered(&self, device_info: Option<String>) {
        let event_proxy = self.event_proxy.clone();
        event_proxy
            .send_event(CastEvent::DeviceDiscovered(device_info))
            .expect("Failed to send event");
    }
}
// new tech end

struct DiscoveryEventHandler {
    event_proxy: EventLoopProxy<UserEvent>,
}

impl DiscoveryEventHandler {
    pub fn new(event_proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { event_proxy }
    }
}

impl DeviceDiscovererEventHandler for DiscoveryEventHandler {
    fn device_available(&self, device_info: DeviceInfo) {
        let event_proxy = self.event_proxy.clone();
        event_proxy
            .send_event(UserEvent::DeviceAvailable(device_info))
            .expect("Failed to send event");
    }

    fn device_removed(&self, device_name: String) {
        let event_proxy = self.event_proxy.clone();
        event_proxy
            .send_event(UserEvent::DeviceRemoved(device_name))
            .expect("Failed to send event");
    }

    fn device_changed(&self, device_info: DeviceInfo) {
        let event_proxy = self.event_proxy.clone();
        event_proxy
            .send_event(UserEvent::DeviceChanged(device_info))
            .expect("Failed to send event");
    }
}

struct DevEventHandler {
    event_proxy: EventLoopProxy<UserEvent>,
    id: usize,
}

impl DevEventHandler {
    pub fn new(event_proxy: EventLoopProxy<UserEvent>, id: usize) -> Self {
        Self { event_proxy, id }
    }

    fn send_event(&self, event: DeviceEvent) {
        let id = self.id;
        let event_proxy = self.event_proxy.clone();
        if let Err(err) = event_proxy.send_event(UserEvent::FromDevice { id, event }) {
            println!("Failed to send event: {err}");
        }
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

pub struct WebConfig {
    hostname: String,
    port: usize,
}

impl WebConfig {
    pub fn new(hostname: String, port: String) -> Self {
        Self {
            hostname,
            port: port.parse::<usize>().unwrap(),
        }
    }
    pub fn set_hostname(&mut self, hostname: String) {
        self.hostname = hostname;
    }
    pub fn set_port(&mut self, port: usize) {
        self.port = port;
    }
}

const SERVICE_TYPE: &str = "_googlecast._tcp.local.";

fn discover() -> Option<(String, u16)> {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon.");
    let receiver = mdns
        .browse(SERVICE_TYPE)
        .expect("Failed to browse mDNS services.");

    while let Ok(event) = receiver.recv() {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                let mut addresses = info
                    .get_addresses()
                    .iter()
                    .map(|address| address.to_string())
                    .collect::<Vec<_>>();
                println!(
                    "{}{}",
                    "Resolved a new service: ",
                    format!("{} ({})", info.get_fullname(), addresses.join(", "))
                );

                // Based on mDNS crate code we should have at least one address available.
                return Some((addresses.remove(0), info.get_port()));
            }
            other_event => {
                println!(
                    "{}{}",
                    "Received other service event: ",
                    format!("{:?}", other_event)
                );
            }
        }
    }
    None
}

fn play_media(
    device: &CastDevice,
    app_to_run: &CastDeviceApp,
    media: String,
    media_type: String,
    media_stream_type: StreamType,
) {
    let app = device.receiver.launch_app(app_to_run).unwrap();

    device
        .connection
        .connect(app.transport_id.as_str())
        .unwrap();

    let status = device
        .media
        .load(
            app.transport_id.as_str(),
            app.session_id.as_str(),
            &Media {
                content_id: media,
                content_type: media_type,
                stream_type: media_stream_type,
                duration: None,
                metadata: None,
            },
        )
        .unwrap();

    for i in 0..status.entries.len() {
        println!("{}{}{}", "Media#", i.to_string(), ": ");
        println!(
            "{} {}",
            "Playback rate:",
            status.entries[i].playback_rate.to_string()
        );
        println!(
            "{} {}",
            "Player state:",
            status.entries[i].player_state.to_string()
        );

        if let Some(time) = status.entries[i].current_time {
            println!("{} {}", "Current time:", time.to_string());
        }

        if let Some(ref media) = status.entries[i].media {
            println!("{} {}", "Content Id:", media.content_id.as_str());
            println!("{} {}", "Stream type:", media.stream_type.to_string());
            println!("{} {}", "Content type:", media.content_type.as_str());

            if let Some(duration) = media.duration {
                println!("{} {}", "Duration:", duration.to_string());
            }
        }
    }
}
