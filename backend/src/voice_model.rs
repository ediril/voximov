use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

pub const SOURCE: &str = "https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/resolve/1939ad2a8e416c0acfeecc08a694d14ef25f2231";
pub const MODEL: &str = "onnx/model_q8f16.onnx";
pub const TOKENIZER: &str = "tokenizer.json";

#[derive(Clone, Debug, PartialEq)]
pub struct ModelFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

fn file(path: impl Into<String>, size: u64, sha256: &str) -> ModelFile {
    ModelFile {
        path: path.into(),
        size,
        sha256: sha256.into(),
    }
}

const VOICE_SIZE: u64 = 522_240;
const CHUNK_BYTES: u64 = 4 * 1024 * 1024;

pub const VOICES: [(&str, &str); 28] = [
    (
        "af_alloy",
        "c4a6b876047fd7fb472edf4ebd63cfac7c3b958a7cae7c106e8f038ca6308c45",
    ),
    (
        "af_aoede",
        "4a004c33430762e2461eedb2013fad808ef4ab3121f5300f554476caf58d8361",
    ),
    (
        "af_bella",
        "f69d836209b78eb8c66e75e3cda491e26ea838a3674257e9d4e5703cbaf55c8b",
    ),
    (
        "af_heart",
        "d583ccff3cdca2f7fae535cb998ac07e9fcb90f09737b9a41fa2734ec44a8f0b",
    ),
    (
        "af_jessica",
        "a240a5e3c15b43563d6e923bdca8ef5613a23471d9b77653694012435df23bd8",
    ),
    (
        "af_kore",
        "9be5221b6a941c04b561959b8ff0b06e809444dcc4ab7e75a7b23606f691819e",
    ),
    (
        "af_nicole",
        "cd2191ab31b914ed7b318416b0e4440fdf392ddad9106a060819aa600a64f59a",
    ),
    (
        "af_nova",
        "18778272caa0d0eebaea251c35fd635f038434f9eee5e691d02a174bd328414f",
    ),
    (
        "af_river",
        "00a2bcf82b1d86e8f19902ede58c65ccf6c0e43b44b7d74fad54e5d8933c9c30",
    ),
    (
        "af_sarah",
        "4409fbc125afabacc615d94db5398d847006a737b0247d6892b7a9a0007a2f0a",
    ),
    (
        "af_sky",
        "4435255c9744f3f31659e0d714ab7689bf65d9e77ec1cce060f083912614f0b9",
    ),
    (
        "am_adam",
        "162b035ed91cfc48b6046982184c645f72edcdd1b82843347f605d7bf7b15716",
    ),
    (
        "am_echo",
        "3968b92c3c4cd1c4416dbded36c13eaa388a90d5788d02a13e4d781f5f8cf3c3",
    ),
    (
        "am_eric",
        "e8b5be17edd1e3636901ce7598baafe2dc8dd8ff707a0c23bf9e461add7e2832",
    ),
    (
        "am_fenrir",
        "c27989f741f7ee34d273a39d8a595cc0837d35f5ced9a29b7cc162614616df43",
    ),
    (
        "am_liam",
        "52403be32fd047c6a44517cb0bcd6b134f2a18baa73e70ef41651e0eab921ade",
    ),
    (
        "am_michael",
        "1d1f21dd8da39c30705cd4c75d039d265e9bc4a2a93ed09bc9e1b1225eb95ba1",
    ),
    (
        "am_onyx",
        "da5d135b424164916d75a68ffb4c2abce3d7d5ccc82dd1ee6cf447ce286145e6",
    ),
    (
        "am_puck",
        "fcf73c989033e9233e0b98713eca600c8c74dcc1614b37009d5450ff4a2274a0",
    ),
    (
        "am_santa",
        "61150cf726ab6c5ed7a99f90a304f91f5a72c00c592e89ec94e5df11c319227a",
    ),
    (
        "bf_alice",
        "08afa6ba24da61ea5e8efa139e5aadc938d83f0a6da5a900adaf763ac1da5573",
    ),
    (
        "bf_emma",
        "669fe0647f9dd04fcab92f1439a40eeb4c8b4ab1f82e4996fe3d918ce4a63b73",
    ),
    (
        "bf_isabella",
        "3754352c4aaa46d17f27654ab7518d65b62ad6163a0f55a5f4330c2da2c4e94f",
    ),
    (
        "bf_lily",
        "5e0ee32ebe64a467124976b14e69590746f1c4ce41a12b587a50c862edfea335",
    ),
    (
        "bm_daniel",
        "6b3194bbceffb746733cbc22c8f593dd44e401a71d53895a2dca891bc595a1e8",
    ),
    (
        "bm_fable",
        "f889083196807b4adb15e9204252165f503b8d33d3982e681c52443c49d798f1",
    ),
    (
        "bm_george",
        "c4b235a4c1f2cd3b939fed08b899ce9385638b763f7b73a59616c4fc9bd6c9bc",
    ),
    (
        "bm_lewis",
        "b8f671cef828c30e66fdf0b0756a76bba58f6bb3398cbbf27058642acbcedb97",
    ),
];

pub fn is_voice(name: &str) -> bool {
    VOICES.iter().any(|(voice, _)| *voice == name)
}

pub fn is_british(voice: &str) -> bool {
    voice.starts_with('b')
}

pub fn voice_path(voice: &str) -> String {
    format!("voices/{voice}.bin")
}

pub fn manifest() -> Vec<ModelFile> {
    let mut files = vec![
        file(
            MODEL,
            86_033_585,
            "04c658aec1b6008857c2ad10f8c589d4180d0ec427e7e6118ceb487e215c3cd0",
        ),
        file(
            TOKENIZER,
            3_497,
            "77a02c8e164413299b4b4c403b14f8e0e1c1b727db4d46a09d6327b861060a34",
        ),
    ];
    files.extend(
        VOICES
            .iter()
            .map(|(voice, sha256)| file(voice_path(voice), VOICE_SIZE, sha256)),
    );
    files
}

pub fn directory(config_directory: &Path) -> PathBuf {
    config_directory
        .join("voice-model")
        .join("kokoro-v1.0-q8f16")
}

pub fn is_installed(directory: &Path, files: &[ModelFile]) -> bool {
    files.iter().all(|file| {
        fs::metadata(directory.join(&file.path)).is_ok_and(|metadata| metadata.len() == file.size)
    })
}

pub fn install(
    directory: &Path,
    source: &str,
    files: &[ModelFile],
    mut progress: impl FnMut(f64) -> bool,
) -> Result<bool> {
    let total: u64 = files.iter().map(|file| file.size).sum();
    let mut done = 0u64;
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_global(Some(Duration::from_secs(60)))
            .http_status_as_error(false)
            .build(),
    );
    for file in files {
        let target = directory.join(&file.path);
        if fs::metadata(&target).is_ok_and(|metadata| metadata.len() == file.size) {
            done += file.size;
            continue;
        }
        let finished = download(&agent, source, file, &target, |bytes| {
            progress((done + bytes) as f64 / total as f64)
        })?;
        if !finished {
            return Ok(false);
        }
        done += file.size;
    }
    Ok(progress(1.))
}

fn download_failed() -> anyhow::Error {
    anyhow::anyhow!(crate::i18n::message("error.voiceModelDownload"))
}

fn resume(partial: &Path, size: u64, hasher: &mut Sha256) -> Result<u64> {
    match File::open(partial) {
        Ok(mut existing) if existing.metadata()?.len() <= size => {
            Ok(io::copy(&mut existing, hasher)?)
        }
        _ => {
            let _ = fs::remove_file(partial);
            Ok(0)
        }
    }
}

fn download(
    agent: &ureq::Agent,
    source: &str,
    file: &ModelFile,
    target: &Path,
    mut progress: impl FnMut(u64) -> bool,
) -> Result<bool> {
    fs::create_dir_all(target.parent().context("model file has no folder")?)?;
    let partial = target.with_extension("part");
    let mut hasher = Sha256::new();
    let mut received = resume(&partial, file.size, &mut hasher)?;
    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&partial)?;
    let url = format!("{source}/{}", file.path);
    let mut buffer = vec![0u8; 256 * 1024];
    while received < file.size {
        let end = (received + CHUNK_BYTES).min(file.size);
        let response = agent
            .get(&url)
            .header("Range", format!("bytes={received}-{}", end - 1))
            .call()
            .map_err(|_| download_failed())?;
        ensure!(response.status() == 206, download_failed());
        let mut reader = response
            .into_body()
            .into_with_config()
            .limit(end - received + 1)
            .reader();
        loop {
            let read = reader.read(&mut buffer).map_err(|_| download_failed())?;
            if read == 0 {
                break;
            }
            ensure!(received + read as u64 <= end, download_failed());
            hasher.update(&buffer[..read]);
            output.write_all(&buffer[..read])?;
            received += read as u64;
            if !progress(received) {
                return Ok(false);
            }
        }
        ensure!(received == end, download_failed());
    }
    output.sync_all()?;
    drop(output);
    let digest: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if digest != file.sha256 {
        let _ = fs::remove_file(&partial);
        bail!(download_failed());
    }
    fs::rename(&partial, target)?;
    Ok(true)
}
