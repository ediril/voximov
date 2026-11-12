use crate::commands::{number, text};
use crate::{
    model::{Clip, MIN_SCRIPT_SLOT, Project, SAMPLE_RATE, Script, Track, TrackKind, id},
    storage::Store,
};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;

const SAMPLE: f64 = 1. / SAMPLE_RATE as f64;
const EDGE_TOLERANCE: f64 = 1e-9;

pub fn split_position(project: &Project, position: f64) -> f64 {
    let snapped = project
        .frame_rate()
        .map_or(position, |rate| rate.nearest_boundary(position));
    (snapped * SAMPLE_RATE as f64).round() / SAMPLE_RATE as f64
}

fn split_track(track: &mut Track, position: f64) -> bool {
    let mut split = false;
    let mut index = 0;
    while index < track.clips.len() {
        let original = &track.clips[index];
        let cut = position - original.start;
        if original.script.is_none()
            && cut >= SAMPLE - EDGE_TOLERANCE
            && original.duration - cut >= SAMPLE - EDGE_TOLERANCE
        {
            let tail = Clip {
                id: id(),
                start: position,
                offset: original.offset + cut,
                duration: original.duration - cut,
                ..original.clone()
            };
            track.clips[index].duration = cut;
            track.clips.insert(index + 1, tail);
            split = true;
            index += 1;
        }
        index += 1;
    }
    split
}

pub fn execute(store: &mut Store, value: &Value) -> Result<()> {
    let command = text(value, "command")?;
    match command {
        "asset_remove" => {
            let asset_id = text(value, "assetId")?;
            let mut project = store.project.clone();
            ensure!(
                project.assets.iter().any(|asset| asset.id == asset_id),
                crate::i18n::message("error.sourceMissing")
            );
            let uses_asset = |clip: &Clip| clip.asset_id.as_deref() == Some(asset_id);
            ensure!(
                !project
                    .tracks
                    .iter()
                    .any(|track| track.locked && track.clips.iter().any(uses_asset)),
                crate::i18n::message("error.assetOnLockedTrack")
            );
            for track in &mut project.tracks {
                track
                    .clips
                    .retain(|clip| clip.script.is_some() || !uses_asset(clip));
                for clip in track.clips.iter_mut().filter(|clip| uses_asset(clip)) {
                    clip.asset_id = None;
                    if let Some(script) = &mut clip.script {
                        script.take = None;
                    }
                }
            }
            project.assets.retain(|asset| asset.id != asset_id);
            store.replace(project)?;
        }
        "track_add" => {
            let mut project = store.project.clone();
            let track = project.audio_track();
            project.tracks.push(track);
            store.replace(project)?;
        }
        "track_remove" => {
            let track_id = text(value, "trackId")?;
            let mut project = store.project.clone();
            let index = project
                .tracks
                .iter()
                .position(|track| track.id == track_id)
                .context(crate::i18n::message("error.trackMissing"))?;
            ensure!(
                !project.tracks[index].locked,
                crate::i18n::message("error.trackLocked")
            );
            project.tracks.remove(index);
            store.replace(project)?;
        }
        "track_patch" => {
            let track_id = text(value, "trackId")?;
            let mut project = store.project.clone();
            let index = project
                .tracks
                .iter()
                .position(|t| t.id == track_id)
                .context(crate::i18n::message("error.trackMissing"))?;
            let patch = value["patch"]
                .as_object()
                .context(crate::i18n::message("error.patchMissing"))?;
            let mut serialized = serde_json::to_value(&project.tracks[index])?;
            for (key, val) in patch {
                ensure!(
                    [
                        "name",
                        "mute",
                        "solo",
                        "locked",
                        "armed",
                        "fxBypass",
                        "volume",
                        "pan",
                        "effects",
                        "speechSpeed"
                    ]
                    .contains(&key.as_str()),
                    crate::i18n::message("error.trackField")
                );
                serialized[key] = val.clone();
            }
            let updated: Track = serde_json::from_value(serialized)?;
            if updated.armed {
                for t in &mut project.tracks {
                    t.armed = false;
                }
            }
            project.tracks[index] = updated;
            store.replace(project)?;
        }
        "clip_place" => {
            let mut project = store.project.clone();
            let asset = project
                .assets
                .iter()
                .find(|a| Some(a.id.as_str()) == value["assetId"].as_str())
                .context(crate::i18n::message("error.sourceMissing"))?
                .clone();
            let track = project
                .tracks
                .iter_mut()
                .find(|t| Some(t.id.as_str()) == value["trackId"].as_str())
                .context(crate::i18n::message("error.trackMissing"))?;
            ensure!(
                !track.locked && (asset.kind == "video") == (track.kind == TrackKind::Video),
                crate::i18n::message("error.selectCompatibleTrack")
            );
            track.clips.push(Clip {
                id: id(),
                asset_id: Some(asset.id),
                name: asset.name,
                start: number(value, "position")?.max(0.),
                offset: 0.,
                duration: asset.duration,
                script: None,
            });
            store.replace(project)?;
        }
        "split_at" => {
            let requested = number(value, "position")?;
            ensure!(
                (0. ..=2_000_000.).contains(&requested),
                crate::i18n::message("error.position")
            );
            let mut project = store.project.clone();
            let position = split_position(&project, requested);
            let mut split = false;
            for track in project.tracks.iter_mut().filter(|track| !track.locked) {
                split |= split_track(track, position);
            }
            if split {
                store.replace(project)?;
            }
        }
        "clip_edit" | "clip_remove" => {
            let mut project = store.project.clone();
            let assets = project.assets.clone();
            let track = project
                .tracks
                .iter_mut()
                .find(|t| Some(t.id.as_str()) == value["trackId"].as_str())
                .context(crate::i18n::message("error.trackMissing"))?;
            ensure!(!track.locked, crate::i18n::message("error.trackLocked"));
            let index = track
                .clips
                .iter()
                .position(|c| Some(c.id.as_str()) == value["clipId"].as_str())
                .context(crate::i18n::message("error.clipMissing"))?;
            if command == "clip_remove" {
                track.clips.remove(index);
            } else {
                let clip = &mut track.clips[index];
                let delta = number(value, "delta")?;
                match text(value, "mode")? {
                    "move" => clip.start = (clip.start + delta).max(0.),
                    "left" if clip.script.is_some() => {
                        let d = delta.clamp(-clip.start, (clip.duration - MIN_SCRIPT_SLOT).max(0.));
                        clip.start += d;
                        clip.duration -= d;
                    }
                    "right" if clip.script.is_some() => {
                        clip.duration = (clip.duration + delta).max(MIN_SCRIPT_SLOT);
                    }
                    "left" => {
                        let d = delta.clamp(
                            -clip.start.min(clip.offset),
                            (clip.duration - 1. / SAMPLE_RATE as f64).max(0.),
                        );
                        clip.start += d;
                        clip.offset += d;
                        clip.duration -= d;
                    }
                    "right" => {
                        let duration = assets
                            .iter()
                            .find(|a| clip.asset_id.as_deref() == Some(a.id.as_str()))
                            .context(crate::i18n::message("error.sourceMissing"))?
                            .duration;
                        clip.duration = (clip.duration + delta)
                            .clamp(1. / SAMPLE_RATE as f64, duration - clip.offset);
                    }
                    _ => bail!(crate::i18n::message("error.editTool")),
                }
            }
            store.replace(project)?;
        }
        "script_add" => {
            let mut project = store.project.clone();
            let track = project
                .tracks
                .iter_mut()
                .find(|t| Some(t.id.as_str()) == value["trackId"].as_str())
                .context(crate::i18n::message("error.trackMissing"))?;
            ensure!(
                !track.locked,
                crate::i18n::message("error.selectCompatibleTrack")
            );
            ensure!(
                track.kind == TrackKind::Audio,
                crate::i18n::message("error.scriptAudioTrack")
            );
            let clip_id = text(value, "clipId")?;
            uuid::Uuid::parse_str(clip_id)?;
            track.clips.push(Clip {
                id: clip_id.into(),
                asset_id: None,
                name: String::new(),
                start: number(value, "position")?.max(0.),
                offset: 0.,
                duration: number(value, "duration")?.max(MIN_SCRIPT_SLOT),
                script: Some(Script {
                    text: String::new(),
                    take: None,
                }),
            });
            store.replace(project)?;
        }
        "script_patch" => {
            let mut project = store.project.clone();
            let track = project
                .tracks
                .iter_mut()
                .find(|t| Some(t.id.as_str()) == value["trackId"].as_str())
                .context(crate::i18n::message("error.trackMissing"))?;
            ensure!(!track.locked, crate::i18n::message("error.trackLocked"));
            let script = track
                .clips
                .iter_mut()
                .find(|c| Some(c.id.as_str()) == value["clipId"].as_str())
                .and_then(|clip| clip.script.as_mut())
                .context(crate::i18n::message("error.clipMissing"))?;
            script.text = text(value, "text")?.trim().into();
            store.replace(project)?;
        }
        "project_voice" => {
            let mut project = store.project.clone();
            let voice = value["voice"].as_str();
            ensure!(
                voice.is_none_or(crate::voice_model::is_voice),
                crate::i18n::message("error.scriptVoice")
            );
            project.voice = voice.map(Into::into);
            store.replace(project)?;
        }
        _ => bail!(crate::i18n::message("error.editCommand")),
    }
    Ok(())
}
