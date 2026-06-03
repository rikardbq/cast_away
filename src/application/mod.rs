use std::path::PathBuf;

use fcast_sender_sdk::{DeviceDiscovererEventHandler, device::{DeviceConnectionState, DeviceEventHandler, DeviceInfo, KeyEvent, MediaEvent, PlaybackState, Source}};
use serde::{Deserialize, Serialize};
use winit::event_loop::EventLoopProxy;

pub mod app;

#[derive(Deserialize)]
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

#[derive(Serialize)]
pub enum IpcPostMessageKind {
    DeviceDiscovered,
}

#[derive(Serialize)]
pub struct IpcPostMessage {
    kind: IpcPostMessageKind,
    data: Option<serde_json::Value>,
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
    ConnectToDevice(String),
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
