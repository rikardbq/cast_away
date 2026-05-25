use std::{path::PathBuf, sync::Arc};

use fcast_sender_sdk::{
    DeviceDiscovererEventHandler, IpAddr,
    context::CastContext,
    device::{
        CastingDevice, DeviceConnectionState, DeviceEventHandler, DeviceFeature, DeviceInfo,
        EventSubscription, KeyEvent, LoadRequest, MediaEvent, PlaybackState, Source,
    },
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    platform::windows::IconExtWindows,
    window::{Icon, Window, WindowAttributes, WindowId},
};
use wry::{WebView, WebViewBuilder};

use crate::{IPC_HANDLER_INIT_SCRIPT, ROOT_DIR};

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
    // testing stuff
    devices: Vec<DeviceInfo>,
    active_device: Option<Arc<dyn CastingDevice>>,
    current_device_id: usize,
    local_adddress: IpAddr,
}

impl App {
    pub fn new(host: String, port: String, event_loop: &EventLoop<UserEvent>) -> Self {
        let cast_context = CastContext::new().unwrap();
        let event_proxy = event_loop.create_proxy();

        let devices: Vec<DeviceInfo> = Vec::new();
        let active_device: Option<Arc<dyn CastingDevice>> = None;
        let current_device_id: usize = 0;
        let local_adddress = IpAddr::v4(127, 0, 0, 1);

        Self {
            event_proxy: event_proxy,
            web_config: WebConfig::new(host, port),
            cast_context,
            window: None,
            webview: None,
            window_attributes: None,
            initialization_script: None,
            // testing stuff
            devices,
            active_device,
            current_device_id,
            local_adddress,
        }
    }
    pub fn set_window_attributes(&mut self, attributes: WindowAttributes) {
        self.window_attributes = Some(attributes);
    }
    pub fn set_initialization_script(&mut self, script: &'static str) {
        self.initialization_script = Some(script);
    }
    pub fn eval_script(&mut self, script: &str) -> Result<(), wry::Error> {
        self.webview.as_ref().unwrap().evaluate_script(script)
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
                let result = match req.method {
                    IpcMethod::ListFiles => {
                        let files: Vec<String> = std::fs::read_dir(".")
                            .unwrap()
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect();

                        Some(serde_json::to_value(files).unwrap())
                    }
                    IpcMethod::DiscoverDevices => {
                        let discovery_event_handler =
                            DiscoveryEventHandler::new(self.event_proxy.clone());
                        self.cast_context
                            .start_discovery(Arc::new(discovery_event_handler));

                        None
                    }
                    IpcMethod::ConnectToDevice => {
                        let device_name = req.params.get("device_name").unwrap();
                        self.event_proxy
                            .send_event(UserEvent::Connect(
                                serde_json::from_value::<String>(device_name.clone()).unwrap(),
                            ))
                            .unwrap();
                        None
                    }
                    IpcMethod::RequestCastLocal => {
                        let file_path = FileDialog::new()
                            .add_filter(
                                "Media",
                                &[
                                    "png", "jpg", "jpeg", "avif", "mkv", "mp4", "webm", "flac",
                                    "opus", "mp3", "mka", "m4a", "wav", "ogg", "vorbis", "apng",
                                    "gif", "webp",
                                ],
                            )
                            .add_filter("All", &["*"])
                            .pick_file();
                        if let Some(handle) = file_path {
                            let event_proxy = self.event_proxy.clone();
                            match infer::get_from_path(handle.clone()) {
                                Ok(res) => match res {
                                    Some(type_) => {
                                        println!("{type_}");
                                        event_proxy
                                            .send_event(UserEvent::CastLocal {
                                                media_type: type_,
                                                handle,
                                            })
                                            .unwrap();
                                    }
                                    None => println!("Unable to get file type"),
                                },
                                Err(err) => {
                                    println!("Failed to infer type of file: {err}");
                                }
                            };
                        }

                        None
                    }
                    _ => Some(serde_json::json!({"error": "unknown method"})),
                };
                let response = IpcResponse { id: req.id, result };
                let json = serde_json::to_string(&response).unwrap();
                self.eval_script(&format!("window.ipc_handler.responseHandler({});", json))
                    .expect("Failed to evaluate script");
            }
            UserEvent::DeviceAvailable(device_info) => {
                self.devices.push(device_info.clone());
                self.eval_script(&format!("window.postMessage('{}');", device_info.name))
                    .expect("Failed to evaluate script");
            }
            UserEvent::Connect(device_name) => {
                if let Some(device_info) = self
                    .devices
                    .iter()
                    .find(|device| device.name == device_name)
                    .cloned()
                {
                    let device = self.cast_context.create_device_from_info(device_info);
                    device
                        .connect(
                            None,
                            Arc::new(DevEventHandler::new(
                                self.event_proxy.clone(),
                                self.current_device_id,
                            )),
                            1000,
                        )
                        .unwrap();
                    self.active_device = Some(device);
                }
            }
            UserEvent::FromDevice { id, event } => {
                if id == self.current_device_id {
                    match event {
                        DeviceEvent::ConnectionStateChanged(state) => match state {
                            DeviceConnectionState::Disconnected => (),
                            DeviceConnectionState::Connecting => (),
                            DeviceConnectionState::Reconnecting => {
                                self.eval_script(&format!(
                                    "window.postMessage('{}');",
                                    "connecting"
                                ))
                                .expect("Failed to evaluate script");
                            }
                            DeviceConnectionState::Connected { local_addr, .. } => {
                                self.local_adddress = local_addr;
                                self.eval_script(&format!(
                                    "window.postMessage('{}');",
                                    "connected"
                                ))
                                .expect("Failed to evaluate script");
                                if let Some(active_device) = &self.active_device {
                                    if active_device
                                        .supports_feature(DeviceFeature::MediaEventSubscription)
                                    {
                                        let _ = active_device
                                            .subscribe_event(EventSubscription::MediaItemEnd);
                                    }
                                }
                            }
                        },
                        DeviceEvent::VolumeChanged(volume) => {
                            self.eval_script(&format!(
                                "window.postMessage('{}');",
                                format!("volume_changed:{}", volume)
                            ))
                            .expect("Failed to evaluate script");
                        }
                        DeviceEvent::TimeChanged(time) => {
                            self.eval_script(&format!(
                                "window.postMessage('{}');",
                                format!("time_changed:{}", time)
                            ))
                            .expect("Failed to evaluate script");
                        }
                        DeviceEvent::PlaybackStateChanged(state) => match state {
                            PlaybackState::Idle => (),
                            PlaybackState::Buffering => (),
                            PlaybackState::Playing => (),
                            PlaybackState::Paused => (),
                        },
                        DeviceEvent::DurationChanged(duration) => {
                            self.eval_script(&format!(
                                "window.postMessage('{}');",
                                format!("duration_changed:{}", duration)
                            ))
                            .expect("Failed to evaluate script");
                        }
                        DeviceEvent::SpeedChanged(_) => (),
                        DeviceEvent::SourceChanged(source) => (),
                    }
                }
            }
            UserEvent::CastLocal { media_type, handle } => {
                let matcher_type = media_type.matcher_type();
                // if !matches!(
                //     matcher_type,
                //     infer::MatcherType::Audio
                //         | infer::MatcherType::Image
                //         | infer::MatcherType::Video
                // ) {
                //     error!("Unsupported media type {matcher_type:?}");
                //     continue;
                // }
                let content_type = media_type.mime_type().to_string();
                match self.active_device.as_ref() {
                    Some(active_device) => {
                        let address = local_ip_address::local_ip().unwrap();
                        active_device
                            .load(LoadRequest::Url {
                                content_type,
                                url: format!(
                                    "http://{}:{}/media/{}",
                                    address.to_string(),
                                    self.web_config.port,
                                    handle.to_str().unwrap().replace("\\", "<<")
                                ),
                                resume_position: None,
                                speed: None,
                                volume: None,
                                metadata: None,
                                request_headers: None,
                            })
                            .unwrap();
                    }
                    None => println!("Not connected"),
                };
            }
            UserEvent::ChangeVolume(new_volume) => {
                if let Some(active_device) = self.active_device.as_ref() {
                    active_device.change_volume(new_volume).unwrap();
                }
            }
            UserEvent::Seek(new_position) => {
                if let Some(active_device) = self.active_device.as_ref() {
                    active_device.seek(new_position).unwrap();
                }
            }
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
