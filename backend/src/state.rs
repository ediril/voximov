use crate::{
    cancellation::Cancellation, generator::Generator, session::AudioSession, speech::LocalSpeaker,
    storage::Store, voice_model,
};
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tokio::sync::Notify;

#[derive(Clone)]
pub struct StudioState {
    pub store: Arc<Mutex<Store>>,
    pub audio: Arc<Mutex<AudioSession>>,
    pub shutdown: Arc<Notify>,
    pub connected: Arc<AtomicBool>,
    pub close_requested: Arc<Notify>,
    pub generator: Generator,
    pub cancellation: Cancellation,
}
impl StudioState {
    pub fn new(store: Store) -> Self {
        let cancellation = store.cancellation.clone();
        let speaker = LocalSpeaker::new(
            voice_model::directory(&store.config_directory),
            voice_model::SOURCE,
        );
        let store = Arc::new(Mutex::new(store));
        Self {
            generator: Generator::start(store.clone(), Box::new(speaker)),
            store,
            audio: Arc::new(Mutex::new(AudioSession::new())),
            shutdown: Arc::new(Notify::new()),
            connected: Arc::new(AtomicBool::new(false)),
            close_requested: Arc::new(Notify::new()),
            cancellation,
        }
    }
}
