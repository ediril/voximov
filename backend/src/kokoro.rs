use crate::voice_model::{self, MODEL, TOKENIZER};
use anyhow::{Context, Result, ensure};
use ort::{environment::Environment, session::Session, value::Tensor};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub const SAMPLE_RATE: u32 = 24_000;
pub const MAX_PHONEMES: usize = 510;
const STYLE_WIDTH: usize = 256;
const AUDIBLE: f32 = 0.003;
const LEAD_SECONDS: f32 = 0.02;
const TAIL_SECONDS: f32 = 0.05;
const SENTENCE_PAUSE_SECONDS: f32 = 0.3;
const PHRASE_PAUSE_SECONDS: f32 = 0.15;

pub struct Kokoro {
    _environment: Environment,
    session: Session,
    vocabulary: HashMap<char, i64>,
    directory: PathBuf,
}

pub fn vocabulary(tokenizer: &str) -> Result<HashMap<char, i64>> {
    let value: serde_json::Value = serde_json::from_str(tokenizer)?;
    let vocabulary: HashMap<char, i64> = value["model"]["vocab"]
        .as_object()
        .context("tokenizer has no vocabulary")?
        .iter()
        .filter_map(|(symbol, id)| {
            let mut chars = symbol.chars();
            let symbol = chars.next()?;
            chars.next().is_none().then_some((symbol, id.as_i64()?))
        })
        .collect();
    ensure!(!vocabulary.is_empty(), "tokenizer has no vocabulary");
    Ok(vocabulary)
}

pub fn token_ids(vocabulary: &HashMap<char, i64>, phonemes: &str) -> Vec<i64> {
    phonemes
        .chars()
        .filter_map(|symbol| vocabulary.get(&symbol).copied())
        .take(MAX_PHONEMES)
        .collect()
}

fn seconds(seconds: f32) -> usize {
    (seconds * SAMPLE_RATE as f32) as usize
}

pub fn audible(samples: &[f32]) -> &[f32] {
    let Some(first) = samples.iter().position(|sample| sample.abs() > AUDIBLE) else {
        return &[];
    };
    let last = samples
        .iter()
        .rposition(|sample| sample.abs() > AUDIBLE)
        .unwrap_or(first);
    let start = first.saturating_sub(seconds(LEAD_SECONDS));
    let end = (last + 1 + seconds(TAIL_SECONDS)).min(samples.len());
    &samples[start..end]
}

pub fn pause_after(chunk: &str, speed: f32) -> usize {
    let pause = if chunk.ends_with(['.', '!', '?']) {
        SENTENCE_PAUSE_SECONDS
    } else {
        PHRASE_PAUSE_SECONDS
    };
    seconds(pause / speed)
}

pub fn join(spoken: &[(&str, Vec<f32>)], speed: f32) -> Vec<f32> {
    let mut samples = vec![];
    let mut previous: Option<&str> = None;
    for (chunk, audio) in spoken {
        let audio = audible(audio);
        if audio.is_empty() {
            continue;
        }
        if let Some(previous) = previous {
            samples.resize(samples.len() + pause_after(previous, speed), 0.);
        }
        samples.extend_from_slice(audio);
        previous = Some(chunk);
    }
    samples
}

fn style(voice_pack: &[u8], tokens: usize) -> Result<Vec<f32>> {
    let rows = voice_pack.len() / (STYLE_WIDTH * 4);
    ensure!(rows > 0, "voice file is too short");
    let row = tokens.min(rows - 1);
    let start = row * STYLE_WIDTH * 4;
    let bytes = voice_pack
        .get(start..start + STYLE_WIDTH * 4)
        .context("voice file is too short")?;
    let (values, _) = bytes.as_chunks::<4>();
    Ok(values.iter().copied().map(f32::from_le_bytes).collect())
}

impl Kokoro {
    pub fn load(directory: &Path) -> Result<Self> {
        let environment = ort::init().with_name("voximov").build()?;
        let session = Session::builder(&environment)?.commit_from_file(directory.join(MODEL))?;
        let vocabulary = vocabulary(&fs::read_to_string(directory.join(TOKENIZER))?)?;
        Ok(Self {
            _environment: environment,
            session,
            vocabulary,
            directory: directory.to_owned(),
        })
    }

    pub fn speak(
        &mut self,
        sentences: &[String],
        voice: &str,
        speed: f32,
        mut progress: impl FnMut(f64) -> bool,
    ) -> Result<Option<Vec<f32>>> {
        let pack = fs::read(self.directory.join(voice_model::voice_path(voice)))?;
        let total: usize = sentences
            .iter()
            .map(|s| s.chars().count())
            .sum::<usize>()
            .max(1);
        let mut done = 0;
        let mut spoken = vec![];
        for sentence in sentences {
            let ids = token_ids(&self.vocabulary, sentence);
            if !ids.is_empty() {
                let audio = self.synthesize(&ids, &style(&pack, ids.len())?, speed)?;
                spoken.push((sentence.as_str(), audio));
            }
            done += sentence.chars().count();
            if !progress(done as f64 / total as f64) {
                return Ok(None);
            }
        }
        Ok(Some(join(&spoken, speed)))
    }

    fn synthesize(&mut self, ids: &[i64], style: &[f32], speed: f32) -> Result<Vec<f32>> {
        let mut padded = Vec::with_capacity(ids.len() + 2);
        padded.push(0);
        padded.extend_from_slice(ids);
        padded.push(0);
        let length = padded.len();
        let outputs = self.session.run(ort::inputs![
            "input_ids" => Tensor::from_array(([1usize, length], padded))?,
            "style" => Tensor::from_array(([1usize, STYLE_WIDTH], style.to_vec()))?,
            "speed" => Tensor::from_array(([1usize], vec![speed]))?,
        ])?;
        let (_, samples) = outputs[0].try_extract_tensor::<f32>()?;
        Ok(samples.to_vec())
    }
}
