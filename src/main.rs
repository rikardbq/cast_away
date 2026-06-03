// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cast_away::{
    ASSETS_ROOT_DIR, B64, HOST, KIOSK_SCRIPT, PORT, application::UserEvent, application::app::App,
    get_application_root_dir, get_or_default_env,
};
use ffmpeg_sidecar::paths::sidecar_path;
use std::env;
use std::path::PathBuf;
use tokio_util::codec::{BytesCodec, FramedRead};

use actix_files::{Files, NamedFile};
use actix_web::{HttpRequest, HttpResponse, HttpServer, Responder, web};
use futures::StreamExt;
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, Window},
};

async fn stream_handler(path: web::Path<String>) -> impl Responder {
    let file_path = B64::decode_str(&path.into_inner());
    println!("Stream handler file_path: {:?}", file_path);
    let file = tokio::fs::File::open(file_path).await.unwrap();
    let stream = FramedRead::new(file, BytesCodec::new()).map(|r| r.map(|b| b.freeze()));
    HttpResponse::Ok()
        // .append_header(("Content-Type", "video/*"))
        .append_header(("Cache-Control", "no-cache"))
        .streaming(stream)
}

async fn index(_req: HttpRequest) -> actix_web::Result<impl Responder> {
    let path: PathBuf = format!(
        "{}/{}/index.html",
        get_application_root_dir().to_string_lossy(),
        ASSETS_ROOT_DIR
    )
    .parse()
    .unwrap();
    Ok(NamedFile::open(path)?
        .use_etag(false)
        .use_last_modified(false)
        .customize()
        .insert_header(("Cache-Control", "no-store")))
}

// #[actix_web::main]
#[tokio::main]
async fn main() {
    println!("{}", sidecar_path().unwrap().to_string_lossy());
    let srv_host = get_or_default_env("SRV_HOST", HOST);
    let srv_port = get_or_default_env("SRV_PORT", PORT);
    let srv_root = get_or_default_env("SRV_ROOT", ASSETS_ROOT_DIR);

    let web_srv = tokio::spawn({
        HttpServer::new(move || {
            actix_web::App::new()
                .route("/", web::get().to(index))
                .route("/stream/{file_name}", web::get().to(stream_handler))
                .service(Files::new("/", format!("./{srv_root}")))
        })
        .bind(format!("0.0.0.0:{srv_port}"))
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
}
