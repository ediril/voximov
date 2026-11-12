#![cfg_attr(all(windows, feature = "desktop"), windows_subsystem = "windows")]
mod application;
mod archive;
#[cfg(all(feature = "desktop", target_os = "linux"))]
mod audio_devices;
#[cfg(all(feature = "desktop", target_os = "linux"))]
mod audio_output;
mod cancellation;
mod commands;
mod controller;
#[cfg(feature = "desktop")]
mod desktop;
mod dsp;
mod editing;
mod ffmpeg;
mod filesystem;
mod frame_rates;
mod generator;
#[cfg(test)]
mod generator_tests;
mod i18n;
mod import_reader;
mod indexed_archive;
#[cfg(test)]
mod integration_tests;
mod kokoro;
mod lifecycle;
mod loudness;
#[cfg(test)]
mod loudness_tests;
mod media;
#[cfg(test)]
mod media_tests;
mod mixer;
mod model;
mod mp4_timing;
mod native_files;
#[cfg(test)]
mod performance_tests;
#[cfg(test)]
mod preferences_tests;
mod progress;
mod pronunciation;
#[cfg(test)]
mod pronunciation_tests;
mod regions;
#[cfg(test)]
mod regions_tests;
mod runtime;
#[cfg(test)]
mod script_tests;
mod server;
#[cfg(test)]
mod server_tests;
mod session;
mod speech;
#[cfg(test)]
mod split_tests;
mod state;
mod storage;
#[cfg(test)]
mod storage_tests;
mod video_export;
#[cfg(all(test, target_os = "macos"))]
mod video_export_ffmpeg_tests;
#[cfg(test)]
mod video_export_tests;
mod video_source;
mod voice_model;
#[cfg(test)]
mod voice_model_tests;

fn main() -> anyhow::Result<()> {
    #[cfg(all(feature = "desktop", target_os = "linux"))]
    audio_output::initialize_environment()?;
    #[cfg(all(feature = "desktop", target_os = "linux"))]
    audio_devices::initialize_environment();
    run()
}

#[tokio::main]
async fn run() -> anyhow::Result<()> {
    application::run().await
}
