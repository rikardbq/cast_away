use std::{
    fs,
    path::{MAIN_SEPARATOR_STR, Path},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use fcast_sender_sdk::{
    IpAddr,
    context::CastContext,
    device::{
        CastingDevice, DeviceConnectionState, DeviceFeature, DeviceInfo, EventSubscription,
        LoadRequest, PlaybackState,
    },
};
use ffmpeg_sidecar::{child::FfmpegChild, command::FfmpegCommand};
use rfd::FileDialog;
use serde_json::json;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    platform::windows::IconExtWindows,
    window::{Icon, Window, WindowAttributes, WindowId},
};
use wry::{WebView, WebViewBuilder};

use crate::{
    ASSETS_ROOT_DIR, B64, IPC_HANDLER_INIT_SCRIPT,
    application::{
        DevEventHandler, DeviceEvent, DiscoveryEventHandler, IpcMethod, IpcPostMessage,
        IpcPostMessageKind, IpcRequest, IpcResponse, UserEvent, WebConfig,
    },
    get_application_root_dir,
};

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
    // streamable_gen_t: Arc<AtomicBool>,
    // old_handle_f: Option<PathBuf>,
    subshell: Arc<Mutex<Option<FfmpegChild>>>,
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
            // streamable_gen_t: Arc::new(AtomicBool::new(true)),
            // old_handle_f: None,
            subshell: Arc::new(Mutex::<Option<FfmpegChild>>::new(None)),
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
                Icon::from_path(
                    &format!(
                        "{}/{}/testicon.ico",
                        get_application_root_dir().to_string_lossy(),
                        ASSETS_ROOT_DIR
                    ),
                    None,
                )
                .unwrap(),
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
                            .send_event(UserEvent::ConnectToDevice(
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
                            let subshell = Arc::clone(&self.subshell);
                            {
                                if let Some(mut shell) = subshell.lock().unwrap().take() {
                                    if let Err(_) = shell.quit() {
                                        let _ = shell.kill();
                                    }
                                }
                            }
                            let event_proxy = self.event_proxy.clone();
                            // let streamable_gen_t_clone = Arc::clone(&self.streamable_gen_t);
                            // let subshell = Arc::clone(&self.subshell);
                            // thread::spawn(move || {
                            //     let cache_dir = get_application_root_dir().join(Path::new("cache"));
                            //     if !Path::is_dir(&cache_dir.as_path()) {
                            //         fs::create_dir_all(&cache_dir).unwrap();
                            //     }
                            //     let expected_out = &format!(
                            //         "{}{}manifest.mpd",
                            //         cache_dir.to_string_lossy(),
                            //         MAIN_SEPARATOR_STR,
                            //         // handle_clone
                            //         //     .extension()
                            //         //     .unwrap_or(&OsStr::new("mp4"))
                            //         //     .to_string_lossy()
                            //     );
                            //     let subs = handle_clone
                            //         .to_string_lossy()
                            //         .replace("C:", "")
                            //         .replace("\\", "/");
                            //     let mut subshell_guard = subshell.lock().unwrap();
                            //     *subshell_guard = Some(
                            //         FfmpegCommand::new()
                            //             .args([
                            //                 "-i",
                            //                 handle_clone.to_str().unwrap(),
                            //                 "-vf",
                            //                 &format!("subtitles={}:si=30", subs),
                            //                 // "-movflags",
                            //                 // "frag_keyframe+empty_moov+faststart",
                            //                 // DASH
                            //                 "-use_template",
                            //                 "1",
                            //                 "-use_timeline",
                            //                 "1",
                            //                 "-seg_duration",
                            //                 "6",
                            //                 "-f",
                            //                 "dash",
                            //                 // DASH END
                            //                 // HLS
                            //                 // "-f",
                            //                 // "hls",
                            //                 // "-hls_base_url",
                            //                 // "/hls/",
                            //                 // HLS END
                            //                 "-y",
                            //             ])
                            //             .arg(expected_out)
                            //             .spawn()
                            //             .unwrap(),
                            //     );
                            //     while !Path::is_file(Path::new(expected_out)) {
                            //         std::thread::sleep(Duration::from_secs(1));
                            //     }
                            //     match infer::get_from_path(handle.clone()) {
                            //         Ok(res) => match res {
                            //             Some(type_) => {
                            //                 event_proxy
                            //                     .send_event(UserEvent::CastLocal {
                            //                         media_type: type_,
                            //                         handle,
                            //                     })
                            //                     .unwrap();
                            //             }
                            //             None => println!("Unable to get file type"),
                            //         },
                            //         Err(err) => {
                            //             println!("Failed to infer type of file: {err}");
                            //         }
                            //     };
                            //     // subshell_guard.take().unwrap().wait().unwrap();
                            //     // let mut started = false;
                            //     // while streamable_gen_t_clone.load(Ordering::SeqCst) {
                            //     //     if !started {
                            //     //         started = true;
                            //     //     }
                            //     // }
                            // });
                            match infer::get_from_path(handle.clone()) {
                                Ok(res) => match res {
                                    Some(type_) => {
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
                            // self.streamable_gen_t.store(true, Ordering::SeqCst);
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
                let response = IpcPostMessage {
                    kind: IpcPostMessageKind::DeviceDiscovered,
                    data: Some(json!({
                        "name": device_info.name,
                        // "protocol": format!("{:?}", device_info.protocol),
                        "address": format!("{:?}", device_info.addresses.first().unwrap()),
                        "port": device_info.port
                    })),
                };
                self.eval_script(&format!(
                    "window.postMessage({});",
                    serde_json::to_string(&response).unwrap()
                ))
                .expect("Failed to evaluate script");
            }
            UserEvent::ConnectToDevice(device_name) => {
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
                                // self.eval_script(&format!(
                                //     "window.postMessage('{}');",
                                //     "connecting"
                                // ))
                                // .expect("Failed to evaluate script");
                            }
                            DeviceConnectionState::Connected { local_addr, .. } => {
                                self.local_adddress = local_addr;
                                // self.eval_script(&format!(
                                //     "window.postMessage('{}');",
                                //     "connected"
                                // ))
                                // .expect("Failed to evaluate script");
                                if let Some(active_device) = &self.active_device {
                                    // idle screen
                                    if let Err(err) = active_device.load(LoadRequest::Image {
                                        content_type: "image/*".to_string(),
                                        url: format!(
                                            "http://{}:{}/idle",
                                            local_ip_address::local_ip().unwrap().to_string(),
                                            self.web_config.port,
                                        ),
                                        metadata: None,
                                        request_headers: None,
                                    }) {
                                        println!("{err:?}");
                                    };
                                    // idle screen end
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
                            // self.eval_script(&format!(
                            //     "window.postMessage('{}');",
                            //     format!("volume_changed:{}", volume)
                            // ))
                            // .expect("Failed to evaluate script");
                        }
                        DeviceEvent::TimeChanged(time) => {
                            // self.eval_script(&format!(
                            //     "window.postMessage('{}');",
                            //     format!("time_changed:{}", time)
                            // ))
                            // .expect("Failed to evaluate script");
                        }
                        DeviceEvent::PlaybackStateChanged(state) => match state {
                            PlaybackState::Idle => (),
                            PlaybackState::Buffering => (),
                            PlaybackState::Playing => (),
                            PlaybackState::Paused => (),
                        },
                        DeviceEvent::DurationChanged(duration) => {
                            // self.eval_script(&format!(
                            //     "window.postMessage('{}');",
                            //     format!("duration_changed:{}", duration)
                            // ))
                            // .expect("Failed to evaluate script");
                        }
                        DeviceEvent::SpeedChanged(_) => (),
                        DeviceEvent::SourceChanged(_source) => (),
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
                println!("HELLO IM TRYING TO CAST A LOCAL FILE {}", content_type);
                match &self.active_device {
                    Some(active_device) => {
                        if let Err(dev_err) = active_device.load(LoadRequest::Url {
                            content_type,
                            url: format!(
                                "http://{}:{}/stream/{}",
                                local_ip_address::local_ip().unwrap().to_string(),
                                self.web_config.port,
                                B64::encode_str(&handle.to_string_lossy())
                            ),
                            resume_position: None,
                            speed: None,
                            volume: None,
                            metadata: None,
                            request_headers: None,
                        }) {
                            println!("{:?}", dev_err);
                        }
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
                let subshell = Arc::clone(&self.subshell);
                if let Some(mut shell) = subshell.lock().unwrap().take() {
                    if let Err(_) = shell.quit() {
                        let _ = shell.kill();
                    }
                }
                event_loop.exit();
            }
            _ => (),
        }
    }
}
