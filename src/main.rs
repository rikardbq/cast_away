// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cast_away::{
    ASSETS_ROOT_DIR, HOST, KIOSK_SCRIPT, PORT, application::UserEvent, application::app::App,
    get_application_root_dir, get_or_default_env,
};
use ffmpeg_sidecar::paths::sidecar_path;
use std::path::{MAIN_SEPARATOR_STR, PathBuf};
use std::{env, path::Path};
use tokio_util::codec::{BytesCodec, FramedRead};

use actix_files::{Files, NamedFile};
use actix_web::{HttpRequest, HttpResponse, HttpServer, Responder, web};
use futures::StreamExt;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window},
};

async fn test_handler() -> 
actix_web::Result<
impl Responder
> 
{
    let file_path = get_application_root_dir()
        .join(Path::new("ffmpeg_stuff/Arcane.S02E01.Heavy.Is.the.Crown.1080p.NF.WEB-DL.DDP5.1.Atmos.H.264-FLUX.mkv"));
    // .replace("<<", MAIN_SEPARATOR_STR);
    println!("Stream handler file_path: {:?}", file_path);
    Ok(NamedFile::open(file_path)?
        .use_etag(false)
        .use_last_modified(false)
        .customize()
        // .insert_header(("Content-Type", "video/mp4"))
        .insert_header(("Cache-Control", "no-store")))
    // let file = tokio::fs::File::open(file_path).await.unwrap();
    // let stream = FramedRead::new(file, BytesCodec::new()).map(|r| r.map(|b| b.freeze()));
    // HttpResponse::Ok()
    //     .append_header(("Content-Type", "video/mp4"))
    //     .append_header(("Cache-Control", "no-cache"))
    //     .streaming(stream)
}
async fn test_subs_handler() -> 
actix_web::Result<
impl Responder
> 
{
    let file_path = get_application_root_dir()
        .join(Path::new("ffmpeg_stuff/subs.srt"));
    // .replace("<<", MAIN_SEPARATOR_STR);
    println!("Stream handler file_path: {:?}", file_path);
    Ok(NamedFile::open(file_path)?
        .use_etag(false)
        .use_last_modified(false)
        .customize()
        // .insert_header(("Content-Type", "video/mp4"))
        .insert_header(("Cache-Control", "no-store")))
    // let file = tokio::fs::File::open(file_path).await.unwrap();
    // let stream = FramedRead::new(file, BytesCodec::new()).map(|r| r.map(|b| b.freeze()));
    // HttpResponse::Ok()
    //     .append_header(("Content-Type", "video/mp4"))
    //     .append_header(("Cache-Control", "no-cache"))
    //     .streaming(stream)
}

async fn media_handler(path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let file = path.into_inner().replace("<<", MAIN_SEPARATOR_STR);
    Ok(NamedFile::open(file)?
        .use_etag(false)
        .use_last_modified(false)
        .customize()
        // .insert_header(("Content-Type", "video/mp4"))
        .insert_header(("Cache-Control", "no-store")))
}

async fn hls_handler(path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let p = path.into_inner();
    let file_path = get_application_root_dir()
        .join(Path::new("cache"))
        .join(Path::new(&p));

    println!("HLS handler file_path: {:?}", file_path);

    if file_path.ends_with(".m3u8") {
        Ok(NamedFile::open(format!("{}", file_path.to_string_lossy()))?
            .use_etag(false)
            .use_last_modified(false)
            .customize()
            .insert_header(("Content-Type", "application/vnd.apple.mpegurl"))
            // .insert_header(("Content-Type", "application/x-mpegURL"))
            .insert_header(("Cache-Control", "no-cache")))
    } else {
        Ok(NamedFile::open(format!("{}", file_path.to_string_lossy()))?
            .use_etag(false)
            .use_last_modified(false)
            .customize()
            .insert_header(("Content-Type", "video/mp2t")))
        // .insert_header(("Cache-Control", "no-cache")))
    }
}

async fn dash_handler(path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let p = path.into_inner();
    let file_path = get_application_root_dir()
        .join(Path::new("cache"))
        .join(Path::new(&p));

    println!("DASH handler file_path: {:?}", file_path);

    if p.ends_with(".mpd") {
        Ok(NamedFile::open(format!("{}", file_path.to_string_lossy()))?
            .use_etag(false)
            .use_last_modified(false)
            .customize()
            .insert_header(("Content-Type", "application/dash+xml"))
            .insert_header(("Cache-Control", "no-cache")))
    } else {
        Ok(NamedFile::open(format!("{}", file_path.to_string_lossy()))?
            .use_etag(false)
            .use_last_modified(false)
            .customize()
            .insert_header(("Content-Type", "video/iso.segment")))
        // .insert_header(("Content-Type", "video/mp4")))
        // .insert_header(("Cache-Control", "no-cache")))
    }
}

async fn stream_video(path: web::Path<String>) -> impl Responder {
    let p = path.into_inner();
    let file_path = get_application_root_dir()
        .join(Path::new("cache"))
        .join(Path::new(&p));
    // .replace("<<", MAIN_SEPARATOR_STR);
    println!("Stream handler file_path: {:?}", file_path);
    if p.ends_with(".mpd") {
        let file = tokio::fs::File::open(file_path).await.unwrap();
        let stream = FramedRead::new(file, BytesCodec::new()).map(|r| r.map(|b| b.freeze()));
        HttpResponse::Ok()
            .append_header(("Content-Type", "application/dash+xml"))
            .append_header(("Cache-Control", "no-cache"))
            .streaming(stream)
    } else {
        let file = tokio::fs::File::open(get_application_root_dir().join(Path::new(&p)))
            .await
            .unwrap();
        let stream = FramedRead::new(file, BytesCodec::new()).map(|r| r.map(|b| b.freeze()));
        HttpResponse::Ok()
            .append_header(("Content-Type", "video/iso.segment"))
            .streaming(stream)
    }
}

async fn index(_req: HttpRequest) -> actix_web::Result<impl Responder> {
    let path: PathBuf = format!("./{ASSETS_ROOT_DIR}/index.html").parse().unwrap();
    Ok(NamedFile::open(path)?
        .use_etag(false)
        .use_last_modified(false)
        .customize()
        .insert_header(("Cache-Control", "no-store")))
}

// #[actix_web::main]
#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install rustls provider");
    println!("{}", sidecar_path().unwrap().to_string_lossy());
    let srv_host = get_or_default_env("SRV_HOST", HOST);
    let srv_port = get_or_default_env("SRV_PORT", PORT);
    let srv_root = get_or_default_env("SRV_ROOT", ASSETS_ROOT_DIR);

    let web_srv = tokio::spawn({
        HttpServer::new(move || {
            actix_web::App::new()
                .route("/", web::get().to(index))
                .route("/test", web::get().to(test_handler))
                .route("/test_subs", web::get().to(test_subs_handler))
                .route("/media/{file_name}", web::get().to(media_handler))
                .route("/hls/{file_name}", web::get().to(hls_handler))
                .route("/dash/{file_name}", web::get().to(dash_handler))
                .route("/stream/{file_name}", web::get().to(stream_video))
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

    // event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
    web_srv.abort();
}
