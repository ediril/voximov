use crate::{
    kokoro::{self, Kokoro},
    pronunciation::{self, Pronouncer},
    voice_model,
};
use anyhow::{Result, ensure};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stage {
    Downloading,
    Generating,
}

pub struct Speech {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub guessed: Vec<String>,
}

pub trait Speaker: Send {
    fn speak(
        &mut self,
        text: &str,
        voice: &str,
        speed: f64,
        report: &mut dyn FnMut(Stage, f64) -> bool,
    ) -> Result<Option<Speech>>;
}

pub struct LocalSpeaker {
    directory: PathBuf,
    source: String,
    pronouncer: Option<Pronouncer>,
    kokoro: Option<Kokoro>,
}

impl LocalSpeaker {
    pub fn new(directory: PathBuf, source: impl Into<String>) -> Self {
        Self {
            directory,
            source: source.into(),
            pronouncer: None,
            kokoro: None,
        }
    }
}

impl Speaker for LocalSpeaker {
    fn speak(
        &mut self,
        text: &str,
        voice: &str,
        speed: f64,
        report: &mut dyn FnMut(Stage, f64) -> bool,
    ) -> Result<Option<Speech>> {
        let files = voice_model::manifest();
        if !voice_model::is_installed(&self.directory, &files)
            && !voice_model::install(&self.directory, &self.source, &files, |progress| {
                report(Stage::Downloading, progress)
            })?
        {
            return Ok(None);
        }
        if !report(Stage::Generating, 0.) {
            return Ok(None);
        }
        let pronounced = self
            .pronouncer
            .get_or_insert_with(Pronouncer::new)
            .pronounce(text, voice_model::is_british(voice))?;
        let sentences = pronunciation::sentence_chunks(&pronounced.phonemes, kokoro::MAX_PHONEMES);
        ensure!(
            !sentences.is_empty(),
            crate::i18n::message("error.scriptEmpty")
        );
        let kokoro = match &mut self.kokoro {
            Some(kokoro) => kokoro,
            empty => empty.insert(Kokoro::load(&self.directory)?),
        };
        Ok(kokoro
            .speak(&sentences, voice, speed as f32, |progress| {
                report(Stage::Generating, progress)
            })?
            .map(|samples| Speech {
                samples,
                sample_rate: kokoro::SAMPLE_RATE,
                guessed: pronounced.guessed,
            }))
    }
}

pub fn wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data = (samples.len() * 2) as u32;
    let mut file = Vec::with_capacity(44 + samples.len() * 2);
    file.extend(b"RIFF");
    file.extend((36 + data).to_le_bytes());
    file.extend(b"WAVEfmt ");
    file.extend(16u32.to_le_bytes());
    file.extend(1u16.to_le_bytes());
    file.extend(1u16.to_le_bytes());
    file.extend(sample_rate.to_le_bytes());
    file.extend((sample_rate * 2).to_le_bytes());
    file.extend(2u16.to_le_bytes());
    file.extend(16u16.to_le_bytes());
    file.extend(b"data");
    file.extend(data.to_le_bytes());
    for sample in samples {
        file.extend(((sample.clamp(-1., 1.) * i16::MAX as f32) as i16).to_le_bytes());
    }
    file
}
