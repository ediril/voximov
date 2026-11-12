use crate::{
    commands,
    media::wav_header,
    mixer::Mixer,
    model::{Clip, Project, SAMPLE_RATE, Script, Take, TrackKind, id},
    storage::Store,
};
use anyhow::Result;
use serde_json::{Value, json};
use std::{fs::File, io::Write, path::Path};

fn new_store(path: &Path) -> Result<Store> {
    Store::open(path.join("config"), Some(path.join("work")))
}

fn add_script(store: &mut Store, track: usize, position: f64, duration: f64) -> Result<String> {
    let clip_id = id();
    commands::execute(
        store,
        &json!({"command":"script_add","trackId":store.project.tracks[track].id,"clipId":clip_id,"position":position,"duration":duration}),
    )?;
    Ok(clip_id)
}

fn run(store: &mut Store, command: Value) -> Result<Value> {
    commands::execute(store, &command)
}

fn import_tone(store: &mut Store, path: &Path, seconds: usize) -> Result<String> {
    let mut file = File::create(path)?;
    file.write_all(&wav_header(48000 * seconds as u64))?;
    for _ in 0..48000 * seconds {
        file.write_all(&[0, 32, 0, 32])?;
    }
    drop(file);
    commands::import_source(
        store,
        path.to_path_buf(),
        "take.wav".into(),
        "audio".into(),
        None,
        0.,
    )?;
    Ok(store.project.assets.last().unwrap().id.clone())
}

fn script_clip(store: &Store, clip_id: &str) -> Clip {
    store
        .project
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .find(|clip| clip.id == clip_id)
        .unwrap()
        .clone()
}

#[test]
fn draft_script_clip_saves_and_reopens_with_text_and_voice() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    let clip_id = add_script(&mut store, 1, 1.5, 3.)?;
    let track_id = store.project.tracks[1].id.clone();
    run(
        &mut store,
        json!({"command":"script_patch","trackId":track_id,"clipId":clip_id,"text":"  Welcome to the harbour.  "}),
    )?;
    run(
        &mut store,
        json!({"command":"project_voice","voice":"af_heart"}),
    )?;
    let file = temporary.path().join("saved").join("script.voximov");
    store.save(&file)?;
    let mut reopened = new_store(&temporary.path().join("other"))?;
    reopened.load(&file)?;
    let clip = script_clip(&reopened, &clip_id);
    assert_eq!(clip.start, 1.5);
    assert_eq!(clip.duration, 3.);
    assert_eq!(clip.asset_id, None);
    assert_eq!(
        clip.script,
        Some(Script {
            text: "Welcome to the harbour.".into(),
            take: None
        })
    );
    assert_eq!(reopened.project.voice.as_deref(), Some("af_heart"));
    Ok(())
}

#[test]
fn script_clip_edges_set_its_slot_without_an_audio_limit() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    let clip_id = add_script(&mut store, 1, 1., 3.)?;
    let track_id = store.project.tracks[1].id.clone();
    let edit = |mode: &str, delta: f64| json!({"command":"clip_edit","trackId":track_id,"clipId":clip_id,"mode":mode,"delta":delta});
    run(&mut store, edit("right", 20.))?;
    assert_eq!(script_clip(&store, &clip_id).duration, 23.);
    run(&mut store, edit("left", -0.5))?;
    let clip = script_clip(&store, &clip_id);
    assert_eq!((clip.start, clip.offset, clip.duration), (0.5, 0., 23.5));
    run(&mut store, edit("right", -100.))?;
    assert_eq!(script_clip(&store, &clip_id).duration, 0.25);
    Ok(())
}

#[test]
fn script_clips_cannot_be_split_or_partially_edited() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    let clip_id = add_script(&mut store, 1, 0., 4.)?;
    let track_id = store.project.tracks[1].id.clone();
    let history = store.history.len();
    run(&mut store, json!({"command":"split_at","position":2.}))?;
    assert_eq!(store.history.len(), history);
    assert_eq!(script_clip(&store, &clip_id).duration, 4.);
    assert!(
        run(
            &mut store,
            json!({"command":"regions_edit","regions":[{"trackId":track_id,"clipId":clip_id,"from":1.,"to":2.}],"delta":0.,"trackOffset":0,"remove":true}),
        )
        .is_err()
    );
    run(
        &mut store,
        json!({"command":"regions_edit","regions":[{"trackId":track_id,"clipId":clip_id,"from":0.,"to":4.}],"delta":2.,"trackOffset":0}),
    )?;
    let moved = script_clip(&store, &clip_id);
    assert_eq!(moved.start, 2.);
    assert!(
        store.project.tracks[1]
            .clips
            .iter()
            .any(|clip| clip.id == clip_id)
    );
    assert!(moved.script.is_some());
    Ok(())
}

fn audio_track_error(result: Result<impl Sized>) -> bool {
    result.is_err_and(|error| error.to_string().contains("error.scriptAudioTrack"))
}

fn draft_clip(text: &str) -> Clip {
    Clip {
        id: id(),
        asset_id: None,
        name: String::new(),
        start: 0.,
        offset: 0.,
        duration: 2.,
        script: Some(Script {
            text: text.into(),
            take: None,
        }),
    }
}

#[test]
fn script_clips_go_on_any_audio_track_but_not_the_video_track() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    assert!(audio_track_error(add_script(&mut store, 0, 0., 2.)));
    assert!(store.project.tracks[0].clips.is_empty());
    for index in [1, 2] {
        let clip_id = add_script(&mut store, index, 0., 2.)?;
        assert!(
            store.project.tracks[index]
                .clips
                .iter()
                .any(|clip| clip.id == clip_id)
        );
    }
    Ok(())
}

#[test]
fn track_speeds_reach_the_interface_exactly_as_chosen() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    let track_id = store.project.tracks[1].id.clone();
    for speed in [0.8, 0.9, 1.1, 1.2, 1.3] {
        run(
            &mut store,
            json!({"command":"track_patch","trackId":track_id,"patch":{"speechSpeed":speed}}),
        )?;
        let snapshot = commands::snapshot(&store);
        assert_eq!(
            snapshot["project"]["tracks"][1]["speechSpeed"].as_f64(),
            Some(speed)
        );
    }
    Ok(())
}

#[test]
fn projects_saved_with_a_voice_track_open_with_plain_audio_tracks() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let store = new_store(temporary.path())?;
    let mut saved = serde_json::to_value(&store.project)?;
    saved["tracks"][1]["kind"] = json!("voice");
    saved["tracks"][1]["name"] = json!("track.voice");
    saved["tracks"][1]
        .as_object_mut()
        .unwrap()
        .remove("speechSpeed");
    let project: Project = serde_json::from_value(saved)?;
    project.validate()?;
    assert_eq!(project.tracks[1].kind, TrackKind::Audio);
    assert_eq!(project.tracks[1].name, "track.voice");
    assert_eq!(project.tracks[1].speech_speed, 1.);
    Ok(())
}

#[test]
fn script_clips_move_and_paste_between_audio_tracks_but_not_onto_video() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    let clip_id = add_script(&mut store, 1, 0., 4.)?;
    let video_id = store.project.tracks[0].id.clone();
    let first_id = store.project.tracks[1].id.clone();
    let project_id = store.project.id.clone();
    run(
        &mut store,
        json!({"command":"regions_edit","regions":[{"trackId":first_id,"clipId":clip_id,"from":0.,"to":4.}],"delta":1.,"trackOffset":1}),
    )?;
    assert!(store.project.tracks[1].clips.is_empty());
    assert!(
        store.project.tracks[2]
            .clips
            .iter()
            .any(|clip| clip.start == 1. && clip.script.is_some())
    );
    let copied = draft_clip("Copied line");
    assert!(audio_track_error(run(
        &mut store,
        json!({"command":"clips_paste","projectId":project_id,"trackId":video_id,"position":5.,"clips":[{"trackOffset":0,"clip":copied}]}),
    )));
    assert!(store.project.tracks[0].clips.is_empty());
    run(
        &mut store,
        json!({"command":"clips_paste","projectId":project_id,"trackId":first_id,"position":5.,"clips":[{"trackOffset":0,"clip":copied}]}),
    )?;
    assert!(
        store.project.tracks[1]
            .clips
            .iter()
            .any(|clip| clip.start == 5. && clip.script.is_some())
    );
    Ok(())
}

#[test]
fn projects_with_script_clips_on_audio_tracks_still_load() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    let legacy = draft_clip("Saved on the music track");
    let mut project = store.project.clone();
    project.tracks[2].clips.push(legacy.clone());
    project.validate()?;
    store.replace(project)?;
    let file = temporary.path().join("saved").join("legacy.justspeak");
    store.save(&file)?;
    let mut reopened = new_store(&temporary.path().join("other"))?;
    reopened.load(&file)?;
    assert_eq!(reopened.project.tracks[2].clips[0].id, legacy.id);
    Mixer::unscaled(&reopened)?.block(&reopened.project, 0, 1024, None, None)?;
    Ok(())
}

#[test]
fn script_clips_are_rejected_on_video_tracks_and_without_matching_audio() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    assert!(add_script(&mut store, 0, 0., 2.).is_err());
    let mut project: Project = store.project.clone();
    project.tracks[1].clips.push(Clip {
        id: id(),
        asset_id: None,
        name: String::new(),
        start: 0.,
        offset: 0.,
        duration: 1.,
        script: None,
    });
    assert!(project.validate().is_err());
    let mut project: Project = store.project.clone();
    project.tracks[1].clips.push(Clip {
        id: id(),
        asset_id: None,
        name: String::new(),
        start: 0.,
        offset: 0.,
        duration: 1.,
        script: Some(Script {
            text: "Hello".into(),
            take: Some(Take {
                text: "Hello".into(),
                voice: "af_heart".into(),
                speed: 1.,
                guessed: vec![],
            }),
        }),
    });
    assert!(project.validate().is_err());
    Ok(())
}

#[test]
fn script_clip_plays_its_take_then_silence_and_returns_to_draft_without_it() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut store = new_store(temporary.path())?;
    let asset_id = import_tone(&mut store, &temporary.path().join("take.wav"), 1)?;
    for track in &mut store.project.tracks {
        track.clips.clear();
    }
    let draft_id = add_script(&mut store, 1, 0., 3.)?;
    let mut mixer = Mixer::unscaled(&store)?;
    let (silent, _) = mixer.block(&store.project, 0, 1024, None, None)?;
    assert!(silent.iter().flatten().all(|sample| *sample == 0.));
    let mut project = store.project.clone();
    let clip = project.tracks[1]
        .clips
        .iter_mut()
        .find(|clip| clip.id == draft_id)
        .unwrap();
    clip.asset_id = Some(asset_id.clone());
    clip.script = Some(Script {
        text: "Hello".into(),
        take: Some(Take {
            text: "Hello".into(),
            voice: "af_heart".into(),
            speed: 1.,
            guessed: vec![],
        }),
    });
    store.replace(project)?;
    let mut mixer = Mixer::unscaled(&store)?;
    let (speech, _) = mixer.block(&store.project, SAMPLE_RATE as u64 / 2, 1024, None, None)?;
    assert!(speech.iter().flatten().any(|sample| *sample != 0.));
    let mut fresh = Mixer::unscaled(&store)?;
    let (after, _) = fresh.block(&store.project, SAMPLE_RATE as u64 * 2, 1024, None, None)?;
    drop(fresh);
    assert!(after.iter().flatten().all(|sample| *sample == 0.));
    drop(mixer);
    run(
        &mut store,
        json!({"command":"asset_remove","assetId":asset_id}),
    )?;
    let clip = script_clip(&store, &draft_id);
    assert_eq!(clip.asset_id, None);
    assert_eq!(clip.script.unwrap().take, None);
    Ok(())
}

#[test]
fn clips_saved_before_scripts_existed_still_load() -> Result<()> {
    let clip: Clip = serde_json::from_value(json!({
        "id": id(),
        "assetId": id(),
        "name": "voice.wav",
        "start": 1.,
        "offset": 0.,
        "duration": 2.
    }))?;
    assert!(clip.asset_id.is_some() && clip.script.is_none());
    assert!(
        !serde_json::to_string(&clip)?.contains("script"),
        "clips without a script keep the original JSON shape"
    );
    Ok(())
}
