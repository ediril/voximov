use crate::{
    media, mixer,
    model::{Asset, Clip, MAX_UPLOAD, SAMPLE_RATE, Track, TrackKind, id},
    native_files,
    storage::Store,
    video_export,
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn snapshot(store: &Store) -> Value {
    json!({"project":store.project,"config":store.config,"duration":store.project.duration(),"frameRate":store.project.frame_rate(),"canUndo":!store.history.is_empty(),"canRedo":!store.redo.is_empty()})
}
pub(crate) fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key].as_str().with_context(|| {
        crate::i18n::formatted_message("error.fieldMissing", &[("key", key.into())])
    })
}
pub(crate) fn number(value: &Value, key: &str) -> Result<f64> {
    let n = value[key].as_f64().with_context(|| {
        crate::i18n::formatted_message("error.fieldMissing", &[("key", key.into())])
    })?;
    ensure!(n.is_finite(), crate::i18n::message("error.number"));
    Ok(n)
}

pub struct Decoded {
    pub assets: Vec<Asset>,
    was_dirty: bool,
    had_root: bool,
}

impl Decoded {
    pub fn discard(&self, store: &mut Store) -> Result<()> {
        for asset in &self.assets {
            let _ = fs::remove_file(store.source(&asset.id));
            let _ = fs::remove_file(store.pcm(&asset.id));
        }
        if !self.was_dirty {
            store.config.dirty = false;
            store.persist()?;
            if !self.had_root {
                let _ = fs::remove_dir_all(store.root());
            }
        }
        Ok(())
    }
}

pub fn decode_source(store: &mut Store, source: &Path, name: &str, kind: &str) -> Result<Decoded> {
    let size = fs::metadata(source)?.len();
    ensure!(size <= MAX_UPLOAD, crate::i18n::message("error.fileLimit"));
    let was_dirty = store.config.dirty;
    let had_root = store.root().exists();
    store.begin_changes()?;
    let asset_id = id();
    let owned = store.source(&asset_id);
    let decoded = media::import(
        source,
        &owned,
        &store.pcm(&asset_id),
        Asset {
            id: asset_id.clone(),
            name: name.into(),
            kind: kind.into(),
            size,
            duration: 0.,
            frames: 0,
            sample_rate: SAMPLE_RATE,
            waveform: vec![],
            peak_frames: 256,
            frame_rate: None,
        },
        &store.progress,
    );
    match decoded {
        Ok(assets) => Ok(Decoded {
            assets,
            was_dirty,
            had_root,
        }),
        Err(error) => {
            let _ = fs::remove_file(&owned);
            let _ = fs::remove_file(store.pcm(&asset_id));
            if !was_dirty {
                store.config.dirty = false;
                store.persist()?;
                if !had_root {
                    let _ = fs::remove_dir_all(store.root());
                }
            }
            Err(error)
        }
    }
}

pub fn import_source(
    store: &mut Store,
    source: PathBuf,
    name: String,
    kind: String,
    target: Option<String>,
    position: f64,
) -> Result<()> {
    ensure!(
        position.is_finite() && position >= 0.,
        crate::i18n::message("error.position")
    );
    let mut decoded = decode_source(store, &source, &name, &kind)?;
    if kind == "video"
        && let Some(asset) = decoded.assets.first_mut()
    {
        asset.frame_rate = crate::frame_rates::probe_imported(store, &asset.id);
    }
    let assets = decoded.assets.clone();
    let result = (|| {
        let mut assets = assets.into_iter();
        let asset = assets
            .next()
            .context(crate::i18n::message("error.noDecodableAudio"))?;
        let mut project = store.project.clone();
        let index = project
            .tracks
            .iter()
            .position(|t| {
                !t.locked
                    && if kind == "video" {
                        t.kind == TrackKind::Video
                    } else {
                        target.as_ref().is_some_and(|id| id == &t.id) && t.kind == TrackKind::Audio
                    }
            })
            .unwrap_or_else(|| {
                if kind == "video" {
                    project
                        .tracks
                        .insert(0, Track::new(TrackKind::Video, "track.video"));
                    0
                } else {
                    let track = project.audio_track();
                    project.tracks.push(track);
                    project.tracks.len() - 1
                }
            });
        let track = &mut project.tracks[index];
        if kind == "video" && asset.frames == 0 && track.name == "track.video" {
            track.name = "track.videoPreview".into();
        }
        let start = track
            .clips
            .iter()
            .map(|c| c.start + c.duration)
            .fold(position, f64::max);
        track.clips.push(Clip {
            id: id(),
            asset_id: Some(asset.id.clone()),
            name,
            start,
            offset: 0.,
            duration: asset.duration,
            script: None,
        });
        project.assets.push(asset);
        for (stream_index, audio) in assets.enumerate() {
            let track_name = Path::new(&audio.name)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let mut track = Track::new(TrackKind::Audio, &track_name);
            track.mute = stream_index > 0;
            track.clips.push(Clip {
                id: id(),
                asset_id: Some(audio.id.clone()),
                name: audio.name.clone(),
                start,
                offset: 0.,
                duration: audio.duration,
                script: None,
            });
            project.tracks.insert(index + stream_index + 1, track);
            project.assets.push(audio);
        }
        project.validate()?;
        Ok::<_, anyhow::Error>(project)
    })();
    if result.is_err() {
        decoded.discard(store)?;
    }
    store.replace(result?)
}

fn project_name(store: &Store) -> String {
    store
        .config
        .project_file
        .as_deref()
        .and_then(|path| path.file_stem())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| store.project.name.clone())
}

pub fn audio_destination(chosen: PathBuf) -> (PathBuf, &'static str) {
    let extension = chosen
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("mp3") => (chosen, "mp3"),
        Some("wav") => (chosen, "wav"),
        _ => {
            let mut path = chosen.into_os_string();
            path.push(".wav");
            (path.into(), "wav")
        }
    }
}

fn check_export_destination(store: &Store, destination: &Path) -> Result<()> {
    ensure!(
        destination.is_absolute(),
        crate::i18n::message("error.exportPath")
    );
    ensure!(
        store.config.project_file.as_deref() != Some(destination)
            && !destination.starts_with(store.root()),
        crate::i18n::message("error.exportOverwritesProject")
    );
    Ok(())
}

pub fn execute(store: &mut Store, value: &Value) -> Result<Value> {
    crate::frame_rates::resolve(store);
    let result = run(store, value);
    crate::frame_rates::resolve(store);
    result
}

fn run(store: &mut Store, value: &Value) -> Result<Value> {
    let command = text(value, "command")?;
    let language = value["language"].as_str().unwrap_or("en");
    match command {
        "snapshot" => {}
        "preferences_patch" => {
            store.set_language(text(value, "preference")?)?;
        }
        "time_preferences_patch" => {
            store.set_time_display(serde_json::from_value(value["timeDisplay"].clone())?)?;
        }
        "speech_preferences_patch" => {
            store.set_speech_voice(text(value, "voice")?)?;
        }
        "audio_preferences_patch" => {
            store.set_audio_devices(text(value, "inputDevice")?, text(value, "outputDevice")?)?;
        }
        "preset_save" => {
            store.save_preset(
                text(value, "name")?,
                serde_json::from_value(value["effects"].clone())?,
                value["fxBypass"].as_bool().unwrap_or(false),
            )?;
        }
        "regions_edit" | "clips_paste" => return crate::regions::execute(store, value),
        "working_directory" => {
            let path = if let Some(path) = value["path"].as_str() {
                Some(PathBuf::from(path))
            } else {
                native_files::directory(language)?
            };
            if let Some(path) = path {
                store.change_directory(path)?;
            }
        }
        "save" => {
            let destination = if let Some(path) = value["path"].as_str() {
                Some(PathBuf::from(path))
            } else if !value["saveAs"].as_bool().unwrap_or(false)
                && store.config.project_file.is_some()
            {
                store.config.project_file.clone()
            } else {
                native_files::project(true, language)?
            };
            if let Some(destination) = destination {
                if value["saveAs"].as_bool().unwrap_or(false) {
                    store.save_with_options(&destination, true)?;
                } else {
                    store.save(&destination)?;
                }
                return Ok(json!({"saved":true,"path":destination,"snapshot":snapshot(store)}));
            }
            return Ok(json!({"saved":false}));
        }
        "open" => {
            let source = if let Some(path) = value["path"].as_str() {
                Some(PathBuf::from(path))
            } else {
                native_files::project(false, language)?
            };
            if let Some(source) = source {
                store.load(&source)?;
            }
        }
        "import_native" | "import_paths" => {
            let paths = if command == "import_native" {
                native_files::media(language)?.unwrap_or_default()
            } else {
                value["paths"]
                    .as_array()
                    .context(crate::i18n::message("error.filesMissing"))?
                    .iter()
                    .map(|path| {
                        path.as_str()
                            .map(PathBuf::from)
                            .context(crate::i18n::message("error.path"))
                    })
                    .collect::<Result<Vec<_>>>()?
            };
            ensure!(
                paths.len() <= 128,
                crate::i18n::message("error.importFileCount")
            );
            let progress = store.progress.clone();
            let count = paths.len();
            let imported = (|| {
                for (index, source) in paths.into_iter().enumerate() {
                    store.progress = progress.range(index as f64 / count as f64, 1. / count as f64);
                    let name = source
                        .file_name()
                        .context(crate::i18n::message("error.filenameMissing"))?
                        .to_string_lossy()
                        .into_owned();
                    let extension = source
                        .extension()
                        .and_then(|x| x.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let kind = if ["mp4", "mov", "mkv", "webm", "m4v"].contains(&extension.as_str())
                    {
                        "video"
                    } else {
                        "audio"
                    };
                    import_source(
                        store,
                        source,
                        name,
                        kind.into(),
                        value["trackId"].as_str().map(str::to_owned),
                        number(value, "position")?,
                    )?;
                }
                Ok::<_, anyhow::Error>(())
            })();
            store.progress = progress;
            imported?;
        }
        "export" => {
            let name = if let Some(track_id) = value["trackId"].as_str() {
                store
                    .project
                    .tracks
                    .iter()
                    .find(|track| track.id == track_id)
                    .context(crate::i18n::message("error.trackMissing"))?
                    .name
                    .clone()
            } else {
                project_name(store)
            };
            let chosen = if let Some(path) = value["path"].as_str() {
                Some(PathBuf::from(path))
            } else {
                native_files::export(language, &name)?
            };
            let Some(chosen) = chosen else {
                return Ok(json!({"saved":false}));
            };
            let (destination, format) = audio_destination(chosen);
            check_export_destination(store, &destination)?;
            let temporary = destination.with_extension(format!("{}.partial", id()));
            let rendered = mixer::render(store, &temporary, value["trackId"].as_str(), format);
            if let Err(error) = rendered {
                let _ = fs::remove_file(&temporary);
                return Err(error);
            }
            crate::filesystem::replace_file(&temporary, &destination)?;
            return Ok(json!({"saved":true,"path":destination,"format":format}));
        }
        "export_video" => {
            let ticket = text(value, "ticket")?;
            ensure!(
                !ticket.is_empty() && ticket.len() <= 128,
                crate::i18n::formatted_message("error.fieldMissing", &[("key", "ticket".into())])
            );
            let _operation = store.cancellation.begin(ticket);
            ensure!(
                cfg!(target_os = "macos"),
                crate::i18n::message("error.videoExportUnsupported")
            );
            ensure!(
                video_export::has_video(&store.project),
                crate::i18n::message("error.noExportVideo")
            );
            let destination = if let Some(path) = value["path"].as_str() {
                Some(PathBuf::from(path))
            } else {
                native_files::video(language, &project_name(store))?
            };
            let Some(destination) = destination else {
                return Ok(json!({"saved":false}));
            };
            check_export_destination(store, &destination)?;
            return Ok(match video_export::export(store, &destination)? {
                video_export::Outcome::Saved => json!({"saved":true,"path":destination}),
                video_export::Outcome::Cancelled => json!({"saved":false,"cancelled":true}),
            });
        }
        "undo" => {
            if let Some(project) = store.history.pop() {
                store.begin_changes()?;
                store.redo.push(store.project.clone());
                store.project = project;
                store.persist()?;
            }
        }
        "redo" => {
            if let Some(project) = store.redo.pop() {
                store.begin_changes()?;
                store.history.push(store.project.clone());
                store.project = project;
                store.persist()?;
            }
        }
        _ => crate::editing::execute(store, value)?,
    }
    Ok(snapshot(store))
}
