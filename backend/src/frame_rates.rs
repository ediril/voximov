use crate::{
    cancellation::Cancellation,
    ffmpeg::{self, Tool},
    model::FrameRate,
    storage::Store,
    video_export::VideoInput,
};
use anyhow::Result;

pub fn probe(store: &Store, asset_id: &str) -> Result<FrameRate> {
    let (path, base, length) = store.location(asset_id, false)?;
    let input = VideoInput { path, base, length };
    let format = ffmpeg::probe(
        &ffmpeg::locate(Tool::Ffprobe),
        &input.url(),
        &Cancellation::default(),
    )?;
    Ok(format.frame_rate)
}

pub fn probe_imported(store: &mut Store, asset_id: &str) -> Option<FrameRate> {
    store.probed_frame_rates.insert(asset_id.to_owned());
    probe(store, asset_id)
        .inspect_err(|error| tracing::warn!(%error, "Could not read the video frame rate"))
        .ok()
}

pub fn resolve(store: &mut Store) {
    let Some(asset) = store.project.frame_rate_asset() else {
        return;
    };
    if asset.frame_rate.is_some() || store.probed_frame_rates.contains(&asset.id) {
        return;
    }
    let asset_id = asset.id.clone();
    if let Some(rate) = probe_imported(store, &asset_id) {
        record(store, &asset_id, rate);
    }
}

fn record(store: &mut Store, asset_id: &str, rate: FrameRate) {
    let projects = std::iter::once(&mut store.project)
        .chain(store.history.iter_mut())
        .chain(store.redo.iter_mut());
    for project in projects {
        for asset in project
            .assets
            .iter_mut()
            .filter(|asset| asset.id == asset_id && asset.kind == "video")
        {
            asset.frame_rate = Some(rate);
        }
    }
    if let Err(error) = store.persist() {
        tracing::warn!(%error, "Could not store the video frame rate");
    }
}
