use std::env;

use ffmpeg_sidecar::{
    download::{
        check_latest_version, download_ffmpeg_package, ffmpeg_download_url,
        unpack_ffmpeg_without_extras,
    },
    paths::sidecar_dir,
    version::ffmpeg_version_with_path,
};

fn build_print(message: &str) {
    println!("cargo:warning={}", message);
}

fn main() {
    let ffmpeg_sidecar = env::var("FFMPEG_SIDECAR").unwrap_or(String::from("false"));
    if ffmpeg_sidecar == "true" || ffmpeg_sidecar == "1" {
        match check_latest_version() {
            Ok(version) => build_print(&format!("Latest available version: {version}")),
            Err(_) => build_print("Skipping version check on this platform"),
        }

        let download_url = ffmpeg_download_url().unwrap();
        let destination = sidecar_dir().unwrap().join("../../");

        build_print(&format!("Downloading from: {:?}", download_url));
        let archive_path = download_ffmpeg_package(download_url, &destination).unwrap();

        build_print(&format!("Downloaded package: {:?}", archive_path));
        build_print("Extracting...");
        unpack_ffmpeg_without_extras(&archive_path, &destination).unwrap();
        let version = ffmpeg_version_with_path(destination.join("ffmpeg")).unwrap();

        build_print(&format!("FFmpeg version: {version}"));
        build_print("Done!");
    } else {
        build_print("FFmpeg is not being included");
    }
}
