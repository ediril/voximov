use crate::{
    commands,
    model::{Asset, Clip, FrameRate, SAMPLE_RATE, Script, id},
    storage::Store,
};
use anyhow::Result;
use serde_json::json;
use std::path::Path;

fn asset(kind: &str, duration: f64, frame_rate: Option<FrameRate>) -> Asset {
    Asset {
        id: id(),
        name: format!("{kind}.source"),
        kind: kind.into(),
        size: 0,
        duration,
        frames: 0,
        sample_rate: SAMPLE_RATE,
        waveform: vec![],
        peak_frames: 256,
        frame_rate,
    }
}

fn clip(asset: &Asset, start: f64, duration: f64) -> Clip {
    Clip {
        id: id(),
        asset_id: Some(asset.id.clone()),
        name: asset.name.clone(),
        start,
        offset: 0.,
        duration,
        script: None,
    }
}

fn script(start: f64, duration: f64) -> Clip {
    Clip {
        id: id(),
        asset_id: None,
        name: String::new(),
        start,
        offset: 0.,
        duration,
        script: Some(Script {
            text: "Hello".into(),
            take: None,
        }),
    }
}

const NTSC: FrameRate = FrameRate {
    numerator: 30000,
    denominator: 1001,
};

fn studio(directory: &Path, frame_rate: Option<FrameRate>) -> Result<Store> {
    let mut store = Store::open(directory.join("config"), Some(directory.join("work")))?;
    let video = asset("video", 20., frame_rate);
    let voice = asset("audio", 20., None);
    let music = asset("audio", 20., None);
    let mut project = store.project.clone();
    project.tracks[0].clips.push(clip(&video, 0., 10.));
    project.tracks[1].clips.push(clip(&voice, 1., 8.));
    project.tracks[1].clips.push(script(9.5, 4.));
    project.tracks[2].clips.push(clip(&music, 2., 10.));
    project.assets = vec![video, voice, music];
    store.replace(project)?;
    Ok(store)
}

fn split(store: &mut Store, position: f64) -> Result<()> {
    commands::execute(store, &json!({"command":"split_at","position":position}))?;
    Ok(())
}

fn edges(store: &Store, track: usize) -> Vec<(f64, f64, f64)> {
    store.project.tracks[track]
        .clips
        .iter()
        .map(|clip| (clip.start, clip.offset, clip.duration))
        .collect()
}

#[test]
fn splits_every_crossing_clip_on_every_track_in_one_undo_step() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut store = studio(directory.path(), None)?;
    let history = store.history.len();
    split(&mut store, 5.)?;
    assert_eq!(edges(&store, 0), [(0., 0., 5.), (5., 5., 5.)]);
    assert_eq!(edges(&store, 1)[..2], [(1., 0., 4.), (5., 4., 4.)]);
    assert_eq!(edges(&store, 2), [(2., 0., 3.), (5., 3., 7.)]);
    assert_eq!(store.history.len(), history + 1);
    let tail = &store.project.tracks[0].clips[1];
    assert_ne!(tail.id, store.project.tracks[0].clips[0].id);
    assert_eq!(tail.asset_id, store.project.tracks[0].clips[0].asset_id);
    commands::execute(&mut store, &json!({"command":"undo"}))?;
    assert_eq!(edges(&store, 0), [(0., 0., 10.)]);
    assert_eq!(edges(&store, 1).len(), 2);
    assert_eq!(edges(&store, 2), [(2., 0., 10.)]);
    Ok(())
}

#[test]
fn locked_tracks_and_script_clips_are_left_whole() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut store = studio(directory.path(), None)?;
    let mut project = store.project.clone();
    project.tracks[2].locked = true;
    store.replace(project)?;
    let history = store.history.len();
    split(&mut store, 11.)?;
    assert_eq!(store.history.len(), history);
    assert_eq!(edges(&store, 1)[1], (9.5, 0., 4.));
    assert_eq!(edges(&store, 2), [(2., 0., 10.)]);
    split(&mut store, 9.)?;
    assert_eq!(edges(&store, 0), [(0., 0., 9.), (9., 9., 1.)]);
    assert_eq!(edges(&store, 1).len(), 2);
    assert_eq!(edges(&store, 2), [(2., 0., 10.)]);
    Ok(())
}

#[test]
fn clip_edges_within_one_sample_are_not_split() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut store = studio(directory.path(), None)?;
    let sample = 1. / SAMPLE_RATE as f64;
    let mut project = store.project.clone();
    project.tracks[2].clips[0].start = 2. + sample / 2.;
    store.replace(project)?;
    split(&mut store, 2. + sample)?;
    assert_eq!(edges(&store, 0).len(), 2);
    assert_eq!(edges(&store, 2).len(), 1);
    let history = store.history.len();
    split(&mut store, 12.)?;
    assert_eq!(store.history.len(), history);
    assert_eq!(edges(&store, 2).len(), 1);
    split(&mut store, 2. + 2. * sample)?;
    assert_eq!(edges(&store, 2).len(), 2);
    assert!((edges(&store, 2)[0].2 - 1.5 * sample).abs() < 1e-12);
    Ok(())
}

#[test]
fn split_positions_snap_to_the_nearest_video_frame() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut store = studio(directory.path(), Some(NTSC))?;
    split(&mut store, 4.01)?;
    let frame = (4.01_f64 * 30000. / 1001.).round();
    let expected = (frame * 1001. / 30000. * SAMPLE_RATE as f64).round() / SAMPLE_RATE as f64;
    assert_eq!(frame, 120.);
    assert_eq!(store.project.tracks[0].clips[1].start, expected);
    assert_eq!(store.project.tracks[1].clips[1].start, expected);
    assert_eq!(store.project.tracks[2].clips[1].start, expected);
    assert_eq!(store.project.tracks[0].clips[0].duration, expected);
    Ok(())
}

#[test]
fn the_project_frame_rate_comes_from_the_first_clip_on_the_top_video_track() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut store = studio(
        directory.path(),
        Some(FrameRate {
            numerator: 25,
            denominator: 1,
        }),
    )?;
    let later = asset("video", 5., Some(NTSC));
    let mut project = store.project.clone();
    project.tracks[0].clips.insert(0, clip(&later, 12., 5.));
    project.assets.push(later);
    store.replace(project)?;
    let snapshot = commands::execute(&mut store, &json!({"command":"snapshot"}))?;
    assert_eq!(
        snapshot["frameRate"],
        json!({"numerator":25,"denominator":1})
    );
    let mut project = store.project.clone();
    project.tracks[0].clips[1].start = 13.;
    project.tracks[0].clips[1].duration = 1.;
    project.tracks[0].clips[0].start = 0.;
    store.replace(project)?;
    assert_eq!(store.project.frame_rate(), Some(NTSC));
    let mut project = store.project.clone();
    project.tracks[0].clips.clear();
    store.replace(project)?;
    let snapshot = commands::execute(&mut store, &json!({"command":"snapshot"}))?;
    assert!(snapshot["frameRate"].is_null());
    Ok(())
}

#[test]
fn frame_rates_must_be_positive_and_plausible() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let store = studio(directory.path(), Some(NTSC))?;
    for (numerator, denominator) in [(0, 1), (30, 0), (1, 2), (241, 1), (u32::MAX, 1)] {
        let mut project = store.project.clone();
        project.assets[0].frame_rate = Some(FrameRate {
            numerator,
            denominator,
        });
        assert!(project.validate().is_err(), "{numerator}/{denominator}");
    }
    let mut project = store.project.clone();
    project.assets[1].frame_rate = Some(NTSC);
    assert!(project.validate().is_err());
    let mut saved = serde_json::to_value(&store.project)?;
    assert_eq!(
        saved["assets"][0]["frameRate"],
        json!({"numerator":30000,"denominator":1001})
    );
    assert!(saved["assets"][1].get("frameRate").is_none());
    saved["assets"][0]
        .as_object_mut()
        .unwrap()
        .remove("frameRate");
    let old: crate::model::Project = serde_json::from_value(saved)?;
    old.validate()?;
    assert_eq!(old.assets[0].frame_rate, None);
    Ok(())
}
