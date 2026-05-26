use cast_away::{
    HOST, KIOSK_SCRIPT, PORT, ROOT_DIR,
    application::app::{App, UserEvent},
    get_or_default_env,
};
use std::env;
use std::path::{MAIN_SEPARATOR_STR, PathBuf};

use actix_files::{Files, NamedFile};
use actix_web::{HttpRequest, HttpServer, Responder, web};
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, Window},
};

async fn media_handler(path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let file = path.into_inner().replace("<<", MAIN_SEPARATOR_STR);
    Ok(NamedFile::open(file)?
        .use_etag(false)
        .use_last_modified(false)
        .customize()
        .insert_header(("Cache-Control", "no-store")))
}

async fn index(_req: HttpRequest) -> actix_web::Result<impl Responder> {
    let path: PathBuf = format!("./{ROOT_DIR}/index.html").parse().unwrap();
    Ok(NamedFile::open(path)?
        .use_etag(false)
        .use_last_modified(false)
        .customize()
        .insert_header(("Cache-Control", "no-store")))
}

// #[actix_web::main]
#[tokio::main]
async fn main() {
    let srv_host = get_or_default_env("SRV_HOST", HOST);
    let srv_port = get_or_default_env("SRV_PORT", PORT);
    let srv_root = get_or_default_env("SRV_ROOT", ROOT_DIR);

    let web_srv = tokio::spawn({
        HttpServer::new(move || {
            actix_web::App::new()
                .route("/", web::get().to(index))
                .route("/media/{file_name}", web::get().to(media_handler))
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
