use std::sync::Arc;

use fcast_sender_sdk::{DeviceDiscovererEventHandler, context::CastContext, device::{DeviceConnectionState, DeviceEventHandler, DeviceInfo, KeyEvent, MediaEvent, PlaybackState, Source}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    platform::windows::IconExtWindows,
    window::{Icon, Window, WindowAttributes, WindowId},
};
use wry::{WebView, WebViewBuilder};

use crate::{IPC_HANDLER_INIT_SCRIPT, ROOT_DIR};

#[derive(Deserialize)]
struct IpcRequest {
    id: u64,
    method: String,
    params: serde_json::Value, // remember to check later...
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
    CastLocalRequested,
    CastLocal {
        media_type: String,
        handle: String,
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

pub struct App {
    event_proxy: EventLoopProxy<UserEvent>,
    web_config: WebConfig,
    cast_context: CastContext,
    window: Option<Window>,
    webview: Option<WebView>,
    window_attributes: Option<WindowAttributes>,
    initialization_script: Option<&'static str>,
}

impl App {
    pub fn new(host: String, port: String, event_loop: &EventLoop<UserEvent>) -> Self {
        let cast_context = CastContext::new().unwrap();
        let event_proxy = event_loop.create_proxy();

        Self {
            event_proxy: event_proxy,
            web_config: WebConfig::new(host, port),
            cast_context,
            window: None,
            webview: None,
            window_attributes: None,
            initialization_script: None,
        }
    }
    pub fn set_window_attributes(&mut self, attributes: WindowAttributes) {
        self.window_attributes = Some(attributes);
    }
    pub fn set_initialization_script(&mut self, script: &'static str) {
        self.initialization_script = Some(script);
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_attr = self
            .window_attributes
            .to_owned()
            .or(Some(Window::default_attributes()))
            .unwrap()
            .with_window_icon(Some(
                Icon::from_path(&format!("{ROOT_DIR}/testicon.ico"), None).unwrap(),
            ));
        let window = event_loop.create_window(win_attr).unwrap();
        let mut webview_builder = WebViewBuilder::new()
            .with_url(format!(
                "http://{}:{}",
                self.web_config.hostname, self.web_config.port
            ))
            .with_initialization_script(IPC_HANDLER_INIT_SCRIPT);

        let proxy_clone = self.event_proxy.clone();
        webview_builder = webview_builder.with_ipc_handler(move |req| {
            let msg = req.body().to_string();
            proxy_clone
                .send_event(UserEvent::ExecEval(msg))
                .expect("Failed to send event");
        });
        if let Some(script) = self.initialization_script {
            webview_builder = webview_builder.with_initialization_script(script);
        }
        let webview = webview_builder.build(&window).unwrap();
        self.window = Some(window);
        self.webview = Some(webview);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::ExecEval(msg) => {
                let req: IpcRequest = serde_json::from_str(&msg).unwrap();
                let result = match req.method.as_str() {
                    "list_files" => {
                        let files: Vec<String> = std::fs::read_dir(".")
                            .unwrap()
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect();

                        Some(serde_json::to_value(files).unwrap())
                    }
                    "discover_devices" => {
                        let discovery_event_handler = DiscoveryEventHandler::new(self.event_proxy.clone());
                        self.cast_context.start_discovery(Arc::new(discovery_event_handler));

                        None
                    }
                    _ => Some(serde_json::json!({"error": "unknown method"})),
                };
                let response = IpcResponse { id: req.id, result };
                let json = serde_json::to_string(&response).unwrap();
                self.webview
                    .as_ref()
                    .unwrap()
                    .evaluate_script(&format!("window.ipc_handler.responseHandler({});", json))
                    .unwrap();
            }
            UserEvent::DeviceAvailable(device_info) => {
                println!("{}", device_info.name);
                self.webview
                    .as_ref()
                    .unwrap()
                    .evaluate_script(&format!("window.postMessage('{}');", device_info.name))
                    .unwrap();
            }
            // UserEvent::Connect(device_name) => {
            //     if let Some(device_info) = devices
            //         .iter()
            //         .find(|device| device.name == device_name)
            //         .cloned()
            //     {
            //         let device = self.cast_context.create_device_from_info(device_info);
            //         device.connect(
            //             None,
            //             Arc::new(DevEventHandler::new(
            //                 self.event_tx.clone(),
            //                 current_device_id,
            //             )),
            //             1000,
            //         )?;
            //         active_device = Some(device);
            //     }
            // }
            // UserEvent::FromDevice { id, event } => {
            //     if id == current_device_id {
            //         match event {
            //             DeviceEvent::ConnectionStateChanged(state) => match state {
            //                 DeviceConnectionState::Disconnected => (),
            //                 DeviceConnectionState::Connecting => (),
            //                 DeviceConnectionState::Reconnecting => {
            //                     self.ui_weak.upgrade_in_event_loop(|ui| {
            //                         ui.global::<Bridge>().set_state(State::Connecting);
            //                     })?;
            //                 }
            //                 DeviceConnectionState::Connected { local_addr, .. } => {
            //                     local_adddress = local_addr;
            //                     self.ui_weak.upgrade_in_event_loop(|ui| {
            //                         ui.global::<Bridge>().invoke_connected();
            //                     })?;
            //                     if let Some(active_device) = &active_device {
            //                         if active_device
            //                             .supports_feature(DeviceFeature::MediaEventSubscription)
            //                         {
            //                             let _ = active_device
            //                                 .subscribe_event(EventSubscription::MediaItemEnd);
            //                         }
            //                     }
            //                 }
            //             },
            //             DeviceEvent::VolumeChanged(volume) => {
            //                 self.ui_weak.upgrade_in_event_loop(move |ui| {
            //                     ui.global::<Bridge>().set_volume(volume as f32);
            //                 })?
            //             }
            //             DeviceEvent::TimeChanged(time) => {
            //                 self.ui_weak.upgrade_in_event_loop(move |ui| {
            //                     ui.global::<Bridge>().set_playback_position(time as f32);
            //                 })?
            //             }
            //             DeviceEvent::PlaybackStateChanged(state) => match state {
            //                 PlaybackState::Idle => (),
            //                 PlaybackState::Buffering => (),
            //                 PlaybackState::Playing => (),
            //                 PlaybackState::Paused => (),
            //             },
            //             DeviceEvent::DurationChanged(duration) => {
            //                 self.ui_weak.upgrade_in_event_loop(move |ui| {
            //                     ui.global::<Bridge>().set_playback_duration(duration as f32);
            //                 })?
            //             }
            //             DeviceEvent::SpeedChanged(_) => (),
            //             DeviceEvent::SourceChanged(source) => (),
            //         }
            //     }
            // }
            _ => (),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            _ => (),
        }
    }
}
