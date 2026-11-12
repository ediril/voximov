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
        mut progress: impl FnMut(f64) -> bool,
    ) -> Result<Option<Vec<f32>>> {
        let pack = fs::read(self.directory.join(voice_model::voice_path(voice)))?;
        let total: usize = sentences
            .iter()
            .map(|s| s.chars().count())
            .sum::<usize>()
            .max(1);
        let mut done = 0;
        let mut samples = vec![];
        for sentence in sentences {
            let ids = token_ids(&self.vocabulary, sentence);
            if !ids.is_empty() {
                samples.extend(self.synthesize(&ids, &style(&pack, ids.len())?)?);
            }
            done += sentence.chars().count();
            if !progress(done as f64 / total as f64) {
                return Ok(None);
            }
        }
        Ok(Some(samples))
    }

    fn synthesize(&mut self, ids: &[i64], style: &[f32]) -> Result<Vec<f32>> {
        let mut padded = Vec::with_capacity(ids.len() + 2);
        padded.push(0);
        padded.extend_from_slice(ids);
        padded.push(0);
        let length = padded.len();
        let outputs = self.session.run(ort::inputs![
            "input_ids" => Tensor::from_array(([1usize, length], padded))?,
            "style" => Tensor::from_array(([1usize, STYLE_WIDTH], style.to_vec()))?,
            "speed" => Tensor::from_array(([1usize], vec![1.0f32]))?,
        ])?;
        let (_, samples) = outputs[0].try_extract_tensor::<f32>()?;
        Ok(samples.to_vec())
    }
}
