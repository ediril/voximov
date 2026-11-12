use crate::voice_model::{ModelFile, install, is_installed};
use sha2::{Digest, Sha256};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct FakeHost {
    url: String,
    requests: Arc<AtomicUsize>,
    starts: Arc<Mutex<Vec<usize>>>,
}

fn requested_range(header: &str, length: usize) -> Option<(usize, usize)> {
    let (name, value) = header.split_once(':')?;
    if !name.eq_ignore_ascii_case("range") {
        return None;
    }
    let (start, end) = value.trim().strip_prefix("bytes=")?.split_once('-')?;
    let start: usize = start.parse().ok()?;
    let end = end
        .parse::<usize>()
        .map_or(length, |end| end + 1)
        .min(length);
    Some((start, end))
}

fn host(files: Vec<(&'static str, Vec<u8>)>) -> FakeHost {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(AtomicUsize::new(0));
    let starts = Arc::new(Mutex::new(vec![]));
    let (counted, started) = (requests.clone(), starts.clone());
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            counted.fetch_add(1, Ordering::SeqCst);
            let mut line = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            reader.read_line(&mut line).unwrap();
            let path = line.split_whitespace().nth(1).unwrap_or("/").to_owned();
            let body = files.iter().find(|(name, _)| path == format!("/{name}"));
            let mut range = None;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                if header == "\r\n" || header.is_empty() {
                    break;
                }
                if let Some((_, body)) = body {
                    range = range.or(requested_range(&header, body.len()));
                }
            }
            let mut stream = stream;
            match (body, range) {
                (Some((_, body)), Some((start, end))) => {
                    started.lock().unwrap().push(start);
                    let _ = write!(
                        stream,
                        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        end - start
                    );
                    let _ = stream.write_all(&body[start..end]);
                }
                _ => {
                    let _ = write!(
                        stream,
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    );
                }
            }
        }
    });
    FakeHost {
        url,
        requests,
        starts,
    }
}

fn manifest(entries: &[(&str, &[u8])]) -> Vec<ModelFile> {
    entries
        .iter()
        .map(|(path, body)| ModelFile {
            path: path.to_string(),
            size: body.len() as u64,
            sha256: sha256(body),
        })
        .collect()
}

fn leftovers(directory: &Path) -> Vec<String> {
    walk(directory)
        .into_iter()
        .filter(|path| path.ends_with(".part"))
        .collect()
}

fn walk(directory: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return vec![];
    };
    entries
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                walk(&path)
            } else {
                vec![path.display().to_string()]
            }
        })
        .collect()
}

#[test]
fn installs_every_file_once_and_reports_progress() {
    let model = vec![7u8; 600_000];
    let voice = b"voice".to_vec();
    let host = host(vec![
        ("model.onnx", model.clone()),
        ("voices/a.bin", voice.clone()),
    ]);
    let files = manifest(&[("model.onnx", &model), ("voices/a.bin", &voice)]);
    let directory = tempfile::tempdir().unwrap();
    let mut progress = vec![];
    assert!(
        install(directory.path(), &host.url, &files, |value| {
            progress.push(value);
            true
        })
        .unwrap()
    );
    assert!(is_installed(directory.path(), &files));
    assert_eq!(
        std::fs::read(directory.path().join("voices/a.bin")).unwrap(),
        voice
    );
    assert!(progress.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(progress.last(), Some(&1.));
    let requests = host.requests.load(Ordering::SeqCst);
    assert!(install(directory.path(), &host.url, &files, |_| true).unwrap());
    assert_eq!(host.requests.load(Ordering::SeqCst), requests);
}

#[test]
fn a_corrupted_download_is_rejected_and_removed() {
    let expected = b"the real model".to_vec();
    let host = host(vec![("model.onnx", b"the fake model".to_vec())]);
    let files = manifest(&[("model.onnx", &expected)]);
    let directory = tempfile::tempdir().unwrap();
    let error = install(directory.path(), &host.url, &files, |_| true).unwrap_err();
    assert!(error.to_string().contains("error.voiceModelDownload"));
    assert!(!is_installed(directory.path(), &files));
    assert_eq!(leftovers(directory.path()), Vec::<String>::new());
}

#[test]
fn a_cancelled_download_resumes_where_it_stopped() {
    let model: Vec<u8> = (0..9_000_000u32).map(|index| index as u8).collect();
    let host = host(vec![("model.onnx", model.clone())]);
    let files = manifest(&[("model.onnx", &model)]);
    let directory = tempfile::tempdir().unwrap();
    assert!(!install(directory.path(), &host.url, &files, |value| value < 0.5).unwrap());
    assert!(!is_installed(directory.path(), &files));
    let attempted = host.starts.lock().unwrap().len();
    assert!(install(directory.path(), &host.url, &files, |_| true).unwrap());
    assert_eq!(
        std::fs::read(directory.path().join("model.onnx")).unwrap(),
        model
    );
    assert!(host.starts.lock().unwrap()[attempted] >= model.len() / 2);
    assert_eq!(leftovers(directory.path()), Vec::<String>::new());
}

#[test]
fn a_missing_file_fails_the_install() {
    let host = host(vec![]);
    let files = manifest(&[("model.onnx", b"model")]);
    let directory = tempfile::tempdir().unwrap();
    assert!(
        install(directory.path(), &host.url, &files, |_| true)
            .unwrap_err()
            .to_string()
            .contains("error.voiceModelDownload")
    );
}
