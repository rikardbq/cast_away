use ffmpeg_sidecar::{child::FfmpegChild, command::FfmpegCommand};
use mdns_sd::ServiceDaemon;
use rfd::FileDialog;
use rust_cast::{
    CastDevice, ChannelMessage,
    channels::{
        connection::ConnectionResponse,
        heartbeat::{HeartbeatChannel, HeartbeatResponse},
        media::Media,
        receiver::CastDeviceApp,
    },
};
use std::{
    fs,
    path::{MAIN_SEPARATOR_STR, Path},
    str::FromStr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    platform::windows::IconExtWindows,
    window::{Icon, Window, WindowAttributes, WindowId},
};
use wry::{WebView, WebViewBuilder};

use crate::{
    ASSETS_ROOT_DIR, DEFAULT_CAST_DESTINATION_ID, IPC_HANDLER_INIT_SCRIPT,
    application::{
        CastContext, DevEventHandler, DeviceDiscoveryEventHandler, DeviceEvent, DeviceInfo,
        IpcMethod, IpcPostMessage, IpcPostMessageKind, IpcRequest, IpcResponse, UserEvent,
        WebConfig,
    },
    get_application_root_dir,
};

pub struct App<'a> {
    event_proxy: EventLoopProxy<UserEvent>,
    web_config: WebConfig,
    cast_context: CastContext<'a>,
    devices: Vec<DeviceInfo>,
    mdns_daemon: Option<Arc<ServiceDaemon>>,
    window: Option<Window>,
    webview: Option<WebView>,
    window_attributes: Option<WindowAttributes>,
    initialization_script: Option<&'static str>,
    // testing stuff
    subshell: Arc<Mutex<Option<FfmpegChild>>>,
}

impl<'a> App<'a> {
    pub fn new(host: String, port: String, event_loop: &EventLoop<UserEvent>) -> Self {
        let event_proxy = event_loop.create_proxy();
        let devices: Vec<DeviceInfo> = Vec::new();

        Self {
            event_proxy,
            web_config: WebConfig::new(host, port),
            cast_context: CastContext::new(),
            devices,
            mdns_daemon: None,
            window: None,
            webview: None,
            window_attributes: None,
            initialization_script: None,
            subshell: Arc::new(Mutex::<Option<FfmpegChild>>::new(None)),
        }
    }
    pub fn set_window_attributes(&mut self, attributes: WindowAttributes) {
        self.window_attributes = Some(attributes);
    }
    pub fn set_initialization_script(&mut self, script: &'static str) {
        self.initialization_script = Some(script);
    }
    pub fn set_mdns_daemon(&mut self, mdns_daemon: Arc<ServiceDaemon>) {
        self.mdns_daemon = Some(mdns_daemon);
    }
    pub fn eval_script(&mut self, script: &str) -> Result<(), wry::Error> {
        self.webview.as_ref().unwrap().evaluate_script(script)
    }
}

impl<'a> ApplicationHandler<UserEvent> for App<'a> {
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
                        if let Some(mdns_daemon) = &self.mdns_daemon {
                            println!("ServiceDaemon exists, shutting down daemon!");
                            mdns_daemon
                                .shutdown()
                                .expect("Failed to shut down service daemon");
                        }
                        let mdns_daemon =
                            Arc::new(ServiceDaemon::new().expect("Failed to create mDNS daemon."));
                        self.set_mdns_daemon(mdns_daemon.clone());
                        CastContext::discover_devices(
                            mdns_daemon,
                            Arc::new(DeviceDiscoveryEventHandler::new(self.event_proxy.clone())),
                        );
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
                            let handle_clone = handle.clone();
                            // let streamable_gen_t_clone = Arc::clone(&self.streamable_gen_t);
                            // let subshell = Arc::clone(&self.subshell);
                            thread::spawn(move || {
                                let cache_dir = get_application_root_dir().join(Path::new("cache"));
                                if !Path::is_dir(&cache_dir.as_path()) {
                                    fs::create_dir_all(&cache_dir).unwrap();
                                }
                                let expected_out = &format!(
                                    "{}{}manifest.mpd",
                                    cache_dir.to_string_lossy(),
                                    MAIN_SEPARATOR_STR,
                                    // handle_clone
                                    //     .extension()
                                    //     .unwrap_or(&OsStr::new("mp4"))
                                    //     .to_string_lossy()
                                );
                                let subs = handle_clone
                                    .to_string_lossy()
                                    .replace("C:", "")
                                    .replace("\\", "/");
                                let mut subshell_guard = subshell.lock().unwrap();
                                *subshell_guard = Some(
                                    FfmpegCommand::new()
                                        .args([
                                            "-i",
                                            handle_clone.to_str().unwrap(),
                                            "-vf",
                                            &format!("subtitles={}:si=30", subs),
                                            // "-movflags",
                                            // "frag_keyframe+empty_moov+faststart",
                                            // DASH
                                            "-use_template",
                                            "1",
                                            "-use_timeline",
                                            "1",
                                            "-seg_duration",
                                            "6",
                                            "-f",
                                            "dash",
                                            // DASH END
                                            // HLS
                                            // "-f",
                                            // "hls",
                                            // "-hls_base_url",
                                            // "/hls/",
                                            // HLS END
                                            "-y",
                                        ])
                                        .arg(expected_out)
                                        .spawn()
                                        .unwrap(),
                                );
                                while !Path::is_file(Path::new(expected_out)) {
                                    std::thread::sleep(Duration::from_secs(1));
                                }
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
                                // subshell_guard.take().unwrap().wait().unwrap();
                                // let mut started = false;
                                // while streamable_gen_t_clone.load(Ordering::SeqCst) {
                                //     if !started {
                                //         started = true;
                                //     }
                                // }
                            });
                            // match infer::get_from_path(handle.clone()) {
                            //     Ok(res) => match res {
                            //         Some(type_) => {
                            //             event_proxy
                            //                 .send_event(UserEvent::CastLocal {
                            //                     media_type: type_,
                            //                     handle,
                            //                 })
                            //                 .unwrap();
                            //         }
                            //         None => println!("Unable to get file type"),
                            //     },
                            //     Err(err) => {
                            //         println!("Failed to infer type of file: {err}");
                            //     }
                            // };
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
            UserEvent::DeviceDiscovered(device_info) => {
                self.devices.push(device_info.clone());
                let response = IpcPostMessage {
                    kind: IpcPostMessageKind::DeviceDiscovered,
                    data: Some(serde_json::to_value(device_info).unwrap()),
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
                    match CastDevice::connect_without_host_verification(
                        device_info.address,
                        device_info.port,
                    ) {
                        Ok(cast_device) => {
                            if let Ok(_) =
                                cast_device.connection.connect(DEFAULT_CAST_DESTINATION_ID)
                            {
                                let proxy_clone = self.event_proxy.clone();
                                let status = cast_device.receiver.get_status().unwrap();
                                let app = status
                                    .applications
                                    .first()
                                    .expect("No application registered");
                                self.cast_context.set_cast_device(cast_device);
                                proxy_clone
                                    .send_event(UserEvent::DeviceConnected(
                                        CastDeviceApp::from_str(&app.app_id).unwrap(),
                                    ))
                                    .expect("Failed to send event DeviceConnected");

                                // comment out because this blocks and cannot be spawned in another thread or execution context due to implementation limits of cast_device type from rust-cast
                                // loop {
                                //     match cast_device.receive() {
                                //         Ok(message) => {
                                //             event_proxy_clone
                                //                 .send_event(UserEvent::DeviceMessage(message))
                                //                 .ok();
                                //         }
                                //         Err(err) => {
                                //             eprintln!("Cast receive error: {err}");
                                //             continue;
                                //         }
                                //     }
                                // }
                            }
                        }
                        Err(e) => {
                            eprintln!("Could not establish connection with Cast Device: {e:?}");
                        }
                    };
                }
            }
            UserEvent::DeviceConnected(device_app) => {
                println!("DEVICE CONNECTED");
                let cast_device = self.cast_context.cast_device.as_ref().unwrap();
                let app = cast_device.receiver.launch_app(&device_app).unwrap();
                println!(
                    "APPLICATIONS {:?}\n{}",
                    cast_device.receiver.get_status().unwrap().applications,
                    app.app_id
                );
                if let Ok(_) = cast_device.connection.disconnect(DEFAULT_CAST_DESTINATION_ID) {
                    if let Ok(_) = cast_device.connection.connect(&app.transport_id) {
                        let media = cast_device
                            .media
                            .load(
                                &app.transport_id,
                                &app.session_id,
                                &Media {
                                    duration: None,
                                    content_type: "video/mp4".to_string(),
                                    metadata: None,
                                    content_id: format!(
                                        "http://{}:{}/test/{}",
                                        local_ip_address::local_ip().unwrap().to_string(),
                                        self.web_config.port,
                                        "manifest.mpd"
                                    ),
                                    stream_type: rust_cast::channels::media::StreamType::Buffered,
                                },
                            )
                            .unwrap();
                        let media_sid = media.entries.first().unwrap().media_session_id;
                        cast_device
                            .media
                            .play(&app.transport_id, media_sid)
                            .unwrap();
                    }
                };
                // can assume device connection has been correctly done and now cast media
            }
            // this cannot run in separate thread for now, so comment out since it needs blocking steps to even function
            // UserEvent::DeviceMessage(message) => match message {
            //     ChannelMessage::Connection(response) => {
            //         println!("Connection message: {response:?}");
            //     }
            //     ChannelMessage::Heartbeat(response) => {
            //         println!("Heartbeat message: {response:?}");
            //     }
            //     ChannelMessage::Media(response) => {
            //         println!("Media message: {response:?}");
            //     }
            //     ChannelMessage::Receiver(response) => {
            //         println!("Receiver message: {response:?}");
            //     }
            //     ChannelMessage::Raw(response) => {
            //         println!("Raw message: {response:?}");
            //     }
            // },
            // TODO: see if rust-cast can look at device events
            // UserEvent::FromDevice { id, event } => {
            //     if id == self.current_device_id {
            //         match event {
            //             DeviceEvent::ConnectionStateChanged(state) => match state {
            //                 DeviceConnectionState::Disconnected => (),
            //                 DeviceConnectionState::Connecting => (),
            //                 DeviceConnectionState::Reconnecting => {
            //                     // self.eval_script(&format!(
            //                     //     "window.postMessage('{}');",
            //                     //     "connecting"
            //                     // ))
            //                     // .expect("Failed to evaluate script");
            //                 }
            //                 DeviceConnectionState::Connected { local_addr, .. } => {
            //                     self.local_adddress = local_addr;
            //                     // self.eval_script(&format!(
            //                     //     "window.postMessage('{}');",
            //                     //     "connected"
            //                     // ))
            //                     // .expect("Failed to evaluate script");
            //                     if let Some(active_device) = &self.active_device {
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
            //                 // self.eval_script(&format!(
            //                 //     "window.postMessage('{}');",
            //                 //     format!("volume_changed:{}", volume)
            //                 // ))
            //                 // .expect("Failed to evaluate script");
            //             }
            //             DeviceEvent::TimeChanged(time) => {
            //                 // self.eval_script(&format!(
            //                 //     "window.postMessage('{}');",
            //                 //     format!("time_changed:{}", time)
            //                 // ))
            //                 // .expect("Failed to evaluate script");
            //             }
            //             DeviceEvent::PlaybackStateChanged(state) => match state {
            //                 PlaybackState::Idle => (),
            //                 PlaybackState::Buffering => (),
            //                 PlaybackState::Playing => (),
            //                 PlaybackState::Paused => (),
            //             },
            //             DeviceEvent::DurationChanged(duration) => {
            //                 // self.eval_script(&format!(
            //                 //     "window.postMessage('{}');",
            //                 //     format!("duration_changed:{}", duration)
            //                 // ))
            //                 // .expect("Failed to evaluate script");
            //             }
            //             DeviceEvent::SpeedChanged(_) => (),
            //             DeviceEvent::SourceChanged(_source) => (),
            //         }
            //     }
            // }
            // UserEvent::CastLocal { media_type, handle } => {
            //     let matcher_type = media_type.matcher_type();
            //     // if !matches!(
            //     //     matcher_type,
            //     //     infer::MatcherType::Audio
            //     //         | infer::MatcherType::Image
            //     //         | infer::MatcherType::Video
            //     // ) {
            //     //     error!("Unsupported media type {matcher_type:?}");
            //     //     continue;
            //     // }
            //     let content_type = media_type.mime_type().to_string();
            //     println!("HELLO IM TRYING TO CAST A LOCAL FILE {}", content_type);
            //     match self.active_device.as_ref() {
            //         Some(active_device) => {
            //             if let Err(dev_err) = active_device.load(LoadRequest::Url {
            //                 content_type: String::from("application/dash+xml"),
            //                 url: format!(
            //                     "http://{}:{}/stream/{}",
            //                     local_ip_address::local_ip().unwrap().to_string(),
            //                     self.web_config.port,
            //                     "manifest.mpd"
            //                 ),
            //                 resume_position: None,
            //                 speed: None,
            //                 volume: None,
            //                 metadata: None,
            //                 request_headers: None,
            //             }) {
            //                 println!("{:?}", dev_err);
            //             }
            //         }
            //         None => println!("Not connected"),
            //     };
            // }
            // UserEvent::ChangeVolume(new_volume) => {
            //     if let Some(active_device) = self.active_device.as_ref() {
            //         active_device.change_volume(new_volume).unwrap();
            //     }
            // }
            // UserEvent::Seek(new_position) => {
            //     if let Some(active_device) = self.active_device.as_ref() {
            //         active_device.seek(new_position).unwrap();
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
