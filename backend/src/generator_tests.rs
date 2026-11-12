use crate::{
    commands,
    generator::{GenerationEvent, GenerationState, Generator, Job},
    model::id,
    speech::{LocalSpeaker, Speaker, Speech, Stage},
    storage::Store,
    voice_model,
};
use anyhow::Result;
use serde_json::json;
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, channel},
    },
    thread,
    time::Duration,
};

#[derive(Default)]
struct FakeSpeaker {
    requests: Arc<Mutex<Vec<(String, String, f64)>>>,
    sentence_delay: Duration,
    sentences: usize,
}

impl Speaker for FakeSpeaker {
    fn speak(
        &mut self,
        text: &str,
        voice: &str,
        speed: f64,
        report: &mut dyn FnMut(Stage, f64) -> bool,
    ) -> Result<Option<Speech>> {
        self.requests
            .lock()
            .unwrap()
            .push((text.to_owned(), voice.to_owned(), speed));
        if !report(Stage::Downloading, 0.5) {
            return Ok(None);
        }
        for sentence in 0..self.sentences {
            thread::sleep(self.sentence_delay);
            if !report(
                Stage::Generating,
                (sentence + 1) as f64 / self.sentences as f64,
            ) {
                return Ok(None);
            }
        }
        Ok(Some(Speech {
            samples: vec![0.25; 24_000],
            sample_rate: 24_000,
            guessed: vec!["Zorblax".into()],
        }))
    }
}

type Studio = (
    tempfile::TempDir,
    Arc<Mutex<Store>>,
    Generator,
    Receiver<GenerationEvent>,
);

fn studio(speaker: impl Speaker + 'static) -> Result<Studio> {
    let temporary = tempfile::tempdir()?;
    let store = Store::open(
        temporary.path().join("config"),
        Some(temporary.path().join("work")),
    )?;
    let store = Arc::new(Mutex::new(store));
    let generator = Generator::start(store.clone(), Box::new(speaker));
    let (sender, events) = channel();
    let sender = Mutex::new(sender);
    generator.set_sink(move |event| {
        let _ = sender.lock().unwrap().send(event);
    });
    Ok((temporary, store, generator, events))
}

fn script(store: &Arc<Mutex<Store>>, text: &str) -> Result<Job> {
    let mut store = store.lock().unwrap();
    let track_id = store.project.tracks[1].id.clone();
    let clip_id = id();
    commands::execute(
        &mut store,
        &json!({"command":"script_add","trackId":track_id,"clipId":clip_id,"position":0.,"duration":4.}),
    )?;
    commands::execute(
        &mut store,
        &json!({"command":"script_patch","trackId":track_id,"clipId":clip_id,"text":text}),
    )?;
    Ok(Job { clip_id })
}

fn wait_for(events: &Receiver<GenerationEvent>, state: GenerationState) -> Vec<GenerationEvent> {
    let mut seen = vec![];
    loop {
        let event = events
            .recv_timeout(Duration::from_secs(60))
            .expect("generation event");
        let done = event.state == state;
        seen.push(event);
        if done {
            return seen;
        }
    }
}

#[test]
fn generated_speech_becomes_the_clip_take_with_its_guessed_words() -> Result<()> {
    let speaker = FakeSpeaker {
        sentences: 3,
        ..Default::default()
    };
    let requests = speaker.requests.clone();
    let (_temporary, store, generator, events) = studio(speaker)?;
    let job = script(&store, "Welcome to Zorblax at dawn.")?;
    generator.enqueue([job.clone()]);
    let seen = wait_for(&events, GenerationState::Done);
    let states: Vec<GenerationState> = seen.iter().map(|event| event.state).collect();
    assert_eq!(
        states,
        [
            GenerationState::Queued,
            GenerationState::Downloading,
            GenerationState::Generating,
            GenerationState::Generating,
            GenerationState::Generating,
            GenerationState::Done,
        ]
    );
    assert_eq!(seen.last().unwrap().guessed, ["Zorblax"]);
    assert_eq!(
        requests.lock().unwrap()[0],
        ("Welcome to Zorblax at dawn.".into(), "af_heart".into(), 1.)
    );
    let store = store.lock().unwrap();
    let clip = &store.project.tracks[1].clips[0];
    let script = clip.script.as_ref().unwrap();
    let take = script.take.as_ref().unwrap();
    assert_eq!(
        (take.text.as_str(), take.voice.as_str()),
        (script.text.as_str(), "af_heart")
    );
    assert_eq!(take.guessed, ["Zorblax"]);
    let asset = store
        .project
        .assets
        .iter()
        .find(|asset| Some(&asset.id) == clip.asset_id.as_ref())
        .unwrap();
    assert!((asset.duration - 1.).abs() < 0.01, "{}", asset.duration);
    assert_eq!(asset.kind, "audio");
    Ok(())
}

#[test]
fn regenerating_replaces_the_previous_take_with_the_project_voice_and_track_speed() -> Result<()> {
    let speaker = FakeSpeaker::default();
    let requests = speaker.requests.clone();
    let (_temporary, store, generator, events) = studio(speaker)?;
    let job = script(&store, "First version.")?;
    generator.enqueue([job.clone()]);
    wait_for(&events, GenerationState::Done);
    let first = store.lock().unwrap().project.tracks[1].clips[0]
        .asset_id
        .clone()
        .unwrap();
    commands::execute(
        &mut store.lock().unwrap(),
        &json!({"command":"project_voice","voice":"am_michael"}),
    )?;
    let track_id = store.lock().unwrap().project.tracks[1].id.clone();
    commands::execute(
        &mut store.lock().unwrap(),
        &json!({"command":"track_patch","trackId":track_id,"patch":{"speechSpeed":1.25}}),
    )?;
    generator.enqueue([job]);
    wait_for(&events, GenerationState::Done);
    let request = requests.lock().unwrap()[1].clone();
    assert_eq!((request.1.as_str(), request.2), ("am_michael", 1.25));
    let store = store.lock().unwrap();
    let clip = &store.project.tracks[1].clips[0];
    assert_eq!(
        clip.script.as_ref().unwrap().take.as_ref().unwrap().speed,
        1.25
    );
    assert_ne!(clip.asset_id.as_deref(), Some(first.as_str()));
    assert!(store.project.assets.iter().all(|asset| asset.id != first));
    assert_eq!(store.project.assets.len(), 1);
    Ok(())
}

#[test]
fn cancelling_stops_generation_and_clears_the_queue() -> Result<()> {
    let speaker = FakeSpeaker {
        sentences: 40,
        sentence_delay: Duration::from_millis(50),
        ..Default::default()
    };
    let requests = speaker.requests.clone();
    let (_temporary, store, generator, events) = studio(speaker)?;
    let current = script(&store, "A long sentence that takes a while to speak aloud.")?;
    let waiting = script(&store, "This one never starts.")?;
    generator.enqueue([current.clone(), waiting.clone()]);
    loop {
        let event = events.recv_timeout(Duration::from_secs(10)).unwrap();
        if event.state == GenerationState::Generating && event.progress > 0. {
            break;
        }
    }
    generator.cancel();
    let mut cancelled = vec![];
    while cancelled.len() < 2 {
        let event = events.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_ne!(event.state, GenerationState::Done);
        if event.state == GenerationState::Cancelled {
            cancelled.push(event.clip_id);
        }
    }
    cancelled.sort();
    let mut expected = vec![current.clip_id, waiting.clip_id];
    expected.sort();
    assert_eq!(cancelled, expected);
    let store = store.lock().unwrap();
    assert!(
        store.project.tracks[1]
            .clips
            .iter()
            .all(|clip| clip.asset_id.is_none())
    );
    assert!(store.project.assets.is_empty());
    assert_eq!(requests.lock().unwrap().len(), 1);
    Ok(())
}

#[test]
fn a_clip_moved_to_another_track_while_generating_keeps_its_take() -> Result<()> {
    let speaker = FakeSpeaker {
        sentences: 10,
        sentence_delay: Duration::from_millis(50),
        ..Default::default()
    };
    let (_temporary, store, generator, events) = studio(speaker)?;
    let job = script(&store, "Moving while speaking.")?;
    generator.enqueue([job.clone()]);
    loop {
        let event = events.recv_timeout(Duration::from_secs(10)).unwrap();
        if event.state == GenerationState::Generating && event.progress > 0. {
            break;
        }
    }
    {
        let mut store = store.lock().unwrap();
        let track_id = store.project.tracks[1].id.clone();
        commands::execute(
            &mut store,
            &json!({"command":"regions_edit","regions":[{"trackId":track_id,"clipId":job.clip_id,"from":0.,"to":4.}],"delta":0.,"trackOffset":1}),
        )?;
    }
    wait_for(&events, GenerationState::Done);
    let store = store.lock().unwrap();
    let moved = &store.project.tracks[2].clips[0];
    assert_eq!(moved.id, job.clip_id);
    assert!(moved.script.as_ref().unwrap().take.is_some());
    assert!(moved.asset_id.is_some());
    Ok(())
}

#[test]
fn empty_clips_and_unknown_voices_fail_with_a_reason() -> Result<()> {
    let (_temporary, store, generator, events) = studio(FakeSpeaker::default())?;
    let empty = script(&store, "   ")?;
    generator.enqueue([empty]);
    let failure = wait_for(&events, GenerationState::Failed).pop().unwrap();
    assert!(failure.error.unwrap().contains("error.scriptEmpty"));
    let rejected = commands::execute(
        &mut store.lock().unwrap(),
        &json!({"command":"project_voice","voice":"af_bella+af_sky"}),
    );
    assert!(
        rejected
            .unwrap_err()
            .to_string()
            .contains("error.scriptVoice")
    );
    store.lock().unwrap().project.voice = Some("ef_dora".into());
    generator.enqueue([script(&store, "Hello there.")?]);
    let failure = wait_for(&events, GenerationState::Failed).pop().unwrap();
    assert!(failure.error.unwrap().contains("error.scriptVoice"));
    Ok(())
}

#[test]
#[ignore = "downloads the 100 MB voice model; set VOXIMOV_VOICE_MODEL to reuse a folder"]
fn local_kokoro_generates_a_take() -> Result<()> {
    let directory = std::env::var_os("VOXIMOV_VOICE_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("voximov-voice-model"));
    let speaker = LocalSpeaker::new(directory.clone(), voice_model::SOURCE);
    let (_temporary, store, generator, events) = studio(speaker)?;
    let job = script(
        &store,
        "Welcome to Voximov. Record a voiceover, then deploy it to Zorblax.",
    )?;
    generator.enqueue([job]);
    let seen = wait_for(&events, GenerationState::Done);
    let generating = seen
        .iter()
        .filter(|event| event.state == GenerationState::Generating && event.progress > 0.)
        .count();
    assert!(voice_model::is_installed(
        &directory,
        &voice_model::manifest()
    ));
    assert_eq!(seen.last().unwrap().guessed, ["Zorblax", "voiceover"]);
    let store = store.lock().unwrap();
    let asset = &store.project.assets[0];
    println!(
        "duration {:.2}s, {generating} progress events",
        asset.duration
    );
    assert!((3.0..8.).contains(&asset.duration));
    assert!(generating >= 2, "progress arrives per sentence");
    Ok(())
}
