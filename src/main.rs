use cast_away::{
    HOST, KIOSK_SCRIPT, PORT, ROOT_DIR,
    application::app::{App, UserEvents, WebConfig},
};
use std::env;
use std::path::PathBuf;

use actix_files::{Files, NamedFile};
use actix_web::{HttpRequest, HttpServer, web};
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, Window},
};

const SERVICE_TYPE: &str = "_googlecast._tcp.local.";
const DEFAULT_DESTINATION_ID: &str = "receiver-0";

fn get_or_default_env(env_var: &str, default: &str) -> String {
    env::var(env_var).unwrap_or(default.to_string())
}

async fn index(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = format!("./{ROOT_DIR}/index.html").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

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
                    Green.paint("Resolved a new service: "),
                    Red.paint(format!(
                        "{} ({})",
                        info.get_fullname(),
                        addresses.join(", ")
                    ))
                );

                // Based on mDNS crate code we should have at least one address available.
                return Some((addresses.remove(0), info.get_port()));
            }
            other_event => {
                println!(
                    "{}{}",
                    Green.paint("Received other service event: "),
                    Red.paint(format!("{:?}", other_event))
                );
            }
        }
    }
    None
}

// #[actix_web::main]
#[tokio::main]
async fn main() {
    let srv_host = get_or_default_env("SRV_HOST", HOST);
    let srv_port = get_or_default_env("SRV_PORT", PORT);
    let srv_root = get_or_default_env("SRV_ROOT", ROOT_DIR);

    // let web_srv = tokio::spawn({
    //     HttpServer::new(move || {
    //         actix_web::App::new()
    //             .route("/", web::get().to(index))
    //             .service(Files::new("/", format!("./{srv_root}")))
    //     })
    //     .bind(format!("{srv_host}:{srv_port}"))
    //     .unwrap()
    //     .run()
    // });

    // let mut app = App::default();
    // let mut web_config = WebConfig::default();
    // let event_loop: EventLoop<UserEvents> = EventLoop::with_user_event().build().unwrap();
    // let proxy = event_loop.create_proxy();
    // app.set_event_loop_proxy(proxy);

    // web_config.set_hostname(srv_host);
    // web_config.set_port(srv_port.parse::<usize>().unwrap());
    // app.set_web_config(web_config);

    // let cli_args: Vec<String> = env::args().collect();
    // cli_args.iter().for_each(|x| match x.as_str() {
    //     "--kiosk" => {
    //         let fullscreen = Some(Fullscreen::Borderless(None));
    //         app.set_window_attributes(Window::default_attributes().with_fullscreen(fullscreen));
    //         app.set_initialization_script(KIOSK_SCRIPT);
    //     }
    //     _ => (),
    // });

    // event_loop.run_app(&mut app).unwrap();
    // web_srv.abort();

    let (address, port) = match args.flag_address {
        Some(address) => (address, args.flag_port),
        None => {
            println!("Cast Device address is not specified, trying to discover...");
            discover().unwrap_or_else(|| {
                println!("No Cast device discovered, please specify device address explicitly.");
                std::process::exit(1);
            })
        }
    };

    let cast_device = match CastDevice::connect_without_host_verification(address, port) {
        Ok(cast_device) => cast_device,
        Err(err) => panic!("Could not establish connection with Cast Device: {:?}", err),
    };

    cast_device
        .connection
        .connect(DEFAULT_DESTINATION_ID.to_string())
        .unwrap();
    cast_device.heartbeat.ping().unwrap();
}
