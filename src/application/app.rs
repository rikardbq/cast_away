use serde::{Deserialize, Serialize};
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    platform::windows::IconExtWindows,
    window::{Icon, Window, WindowAttributes, WindowId},
};
use wry::{WebView, WebViewBuilder};
// use x_win::get_open_windows;

use crate::{HOST, IPC_HANDLER_INIT_SCRIPT, PORT, ROOT_DIR};

#[derive(Deserialize)]
struct IpcRequest {
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Serialize)]
struct IpcResponse {
    id: u64,
    result: serde_json::Value,
}

#[derive(Debug)]
pub enum UserEvents {
    ExecEval(String),
}

pub struct WebConfig {
    hostname: String,
    port: usize,
}

impl WebConfig {
    pub fn default() -> Self {
        WebConfig {
            hostname: HOST.to_string(),
            port: PORT.parse::<usize>().unwrap(),
        }
    }
    pub fn set_hostname(&mut self, hostname: String) {
        self.hostname = hostname;
    }
    pub fn set_port(&mut self, port: usize) {
        self.port = port;
    }
}

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    webview: Option<WebView>,
    event_loop_proxy: Option<EventLoopProxy<UserEvents>>,
    window_attributes: Option<WindowAttributes>,
    initialization_script: Option<&'static str>,
    web_config: Option<WebConfig>,
}

impl App {
    pub fn set_event_loop_proxy(&mut self, proxy: EventLoopProxy<UserEvents>) {
        self.event_loop_proxy = Some(proxy);
    }
    pub fn set_window_attributes(&mut self, attributes: WindowAttributes) {
        self.window_attributes = Some(attributes);
    }
    pub fn set_initialization_script(&mut self, script: &'static str) {
        self.initialization_script = Some(script);
    }
    pub fn set_web_config(&mut self, web_config: WebConfig) {
        self.web_config = Some(web_config);
    }
}

impl ApplicationHandler<UserEvents> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let web_config = match &self.web_config {
            Some(conf) => conf,
            None => &WebConfig::default(),
        };
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
                web_config.hostname, web_config.port
            ))
            .with_initialization_script(IPC_HANDLER_INIT_SCRIPT);

        if let Some(proxy) = &self.event_loop_proxy {
            let proxy = proxy.clone();
            webview_builder = webview_builder.with_ipc_handler(move |req| {
                let msg = req.body().to_string();
                proxy
                    .send_event(UserEvents::ExecEval(msg))
                    .expect("Failed to send event");
            })
        }

        if let Some(script) = self.initialization_script {
            webview_builder = webview_builder.with_initialization_script(script);
        }

        let webview = webview_builder.build(&window).unwrap();

        self.window = Some(window);
        self.webview = Some(webview);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvents) {
        match event {
            UserEvents::ExecEval(msg) => {
                let req: IpcRequest = serde_json::from_str(&msg).unwrap();

                let result = match req.method.as_str() {
                    "list_files" => {
                        let files: Vec<String> = std::fs::read_dir(".")
                            .unwrap()
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect();

                        serde_json::to_value(files).unwrap()
                    }
                    // "get_windows" => {
                    //     let open_windows = get_open_windows().expect("Error showing windows");
                    //     let titles: Vec<String> = open_windows.iter().map(|x| x.title.clone()).collect();
                    //     serde_json::to_value(titles).unwrap()
                    
                    // }
                    _ => serde_json::json!({"error": "unknown method"}),
                };

                let response = IpcResponse { id: req.id, result };
                let json = serde_json::to_string(&response).unwrap();

                self.webview
                    .as_ref()
                    .unwrap()
                    .evaluate_script(&format!("window.ipc_handler.responseHandler({});", json))
                    .unwrap();
            } // _ => ()
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
