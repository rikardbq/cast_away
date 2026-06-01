use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use fcast_sender_sdk::device;
use fcast_sender_sdk::{
    DeviceDiscovererEventHandler,
    device::{
        DeviceConnectionState, DeviceEventHandler, KeyEvent, MediaEvent, PlaybackState, Source,
    },
};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use rust_cast::ChannelMessage;
use rust_cast::channels::connection::ConnectionResponse;
use rust_cast::channels::heartbeat::HeartbeatResponse;
use rust_cast::{
    CastDevice,
    channels::{
        media::{Media, StreamType},
        receiver::CastDeviceApp,
    },
};
use serde::{Deserialize, Serialize};
use winit::event_loop::EventLoopProxy;

use crate::MDNS_SERVICE_TYPE;

pub mod app;

#[derive(Deserialize)]
pub enum IpcMethod {
    ListFiles,
    DiscoverDevices,
    ConnectToDevice,
    DisconnectFromDevice,
    RequestCastLocal,
    Heartbeat,
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
    DeviceDiscovered(DeviceInfo),
    ConnectToDevice(String),
    DeviceConnected,
    DeviceMessage(ChannelMessage),
    Quit,
    DeviceAvailable(device::DeviceInfo),
    DeviceRemoved(String),
    DeviceChanged(device::DeviceInfo),
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
pub struct DeviceDiscoveryEventHandler {
    event_proxy: EventLoopProxy<UserEvent>,
}

impl DeviceDiscoveryEventHandler {
    pub fn new(event_proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { event_proxy }
    }
    pub fn device_discovered(&self, device_info: DeviceInfo) {
        let event_proxy = self.event_proxy.clone();
        event_proxy
            .send_event(UserEvent::DeviceDiscovered(device_info))
            .expect("Failed to send event");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    // td: String,
    // s_td: Option<String>,
    name: String,
    address: String,
    port: u16,
    // eventual TXTProps we need to extract
}

impl DeviceInfo {
    pub fn new(name: String, address: String, port: u16) -> Self {
        Self {
            name,
            address,
            port,
        }
    }
}

pub struct CastContext<'a> {
    cast_device: Option<CastDevice<'a>>,
}

impl<'a> CastContext<'a> {
    pub fn new() -> Self {
        Self { cast_device: None }
    }
    pub fn set_cast_device(&mut self, cast_device: CastDevice<'a>) {
        self.cast_device = Some(cast_device);
    }
    pub fn discover_devices(
        mdns_daemon: Arc<ServiceDaemon>,
        event_handler: Arc<DeviceDiscoveryEventHandler>,
    ) {
        thread::spawn(move || {
            let receiver = mdns_daemon
                .browse(MDNS_SERVICE_TYPE)
                .expect("Failed to browse mDNS services.");
            while let Ok(event) = receiver.recv_timeout(Duration::from_secs(10)) {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        if let Some(address) = info
                            .get_addresses()
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<String>>()
                            .first()
                        {
                            println!("{}", info.get_properties());
                            event_handler.device_discovered(DeviceInfo::new(
                                info.get_property_val_str("fn")
                                    .unwrap_or(info.get_fullname())
                                    .to_string(),
                                address.clone(),
                                info.get_port(),
                            ));
                        } else {
                            continue;
                        }
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
            println!("exit while!");
        });
    }
    pub fn device_listen(
        cast_device: CastDevice,
        event_proxy: Arc<&EventLoopProxy<UserEvent>>,
    ) {
        loop {
            match cast_device.receive() {
                Ok(message) => {
                    event_proxy
                        .send_event(UserEvent::DeviceMessage(message))
                        .ok();
                }
                Err(err) => {
                    eprintln!("Cast receive error: {err}");
                    break;
                }
            }
        }
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
    fn device_available(&self, device_info: device::DeviceInfo) {
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

    fn device_changed(&self, device_info: device::DeviceInfo) {
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
