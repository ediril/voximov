use crate::{
    cancellation::{Cancellation, Cancelled},
    progress::Progress,
    video_export::{FrameRate, VideoFormat},
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::{
    ffi::{OsStr, OsString},
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tool {
    Ffmpeg,
    Ffprobe,
}

impl Tool {
    pub fn program(self) -> &'static str {
        match self {
            Self::Ffmpeg => "ffmpeg",
            Self::Ffprobe => "ffprobe",
        }
    }
    pub fn override_variable(self) -> &'static str {
        match self {
            Self::Ffmpeg => "VOXIMOV_FFMPEG",
            Self::Ffprobe => "VOXIMOV_FFPROBE",
        }
    }
}

pub fn resolve(
    tool: Tool,
    override_path: Option<OsString>,
    executable_directory: Option<&Path>,
    exists: impl Fn(&Path) -> bool,
) -> PathBuf {
    if let Some(path) = override_path.filter(|path| !path.is_empty()) {
        return path.into();
    }
    executable_directory
        .map(|directory| directory.join(tool.program()))
        .filter(|path| exists(path))
        .unwrap_or_else(|| tool.program().into())
}

pub fn locate(tool: Tool) -> PathBuf {
    let executable = std::env::current_exe().ok();
    resolve(
        tool,
        std::env::var_os(tool.override_variable()),
        executable.as_deref().and_then(Path::parent),
        Path::is_file,
    )
}

fn spawn(program: &Path, arguments: &[impl AsRef<OsStr>]) -> Result<Child> {
    Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(crate::i18n::message("error.ffmpegMissing"))
            } else {
                error.into()
            }
        })
}

fn failure(stderr: &str) -> anyhow::Error {
    let detail = stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or_default()
        .chars()
        .take(300)
        .collect::<String>();
    anyhow::anyhow!(crate::i18n::formatted_message(
        "error.videoExport",
        &[("detail", detail)]
    ))
}

fn read_all(mut stream: impl Read) -> String {
    let mut bytes = Vec::new();
    let _ = stream.read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

const STANDARD_RATES: [(u32, u32); 16] = [
    (10, 1),
    (12, 1),
    (15, 1),
    (20, 1),
    (24000, 1001),
    (24, 1),
    (25, 1),
    (30000, 1001),
    (30, 1),
    (48, 1),
    (50, 1),
    (60000, 1001),
    (60, 1),
    (90, 1),
    (100, 1),
    (120, 1),
];

fn rate(text: &str) -> Option<f64> {
    let (numerator, denominator) = text.split_once('/')?;
    let numerator: f64 = numerator.parse().ok()?;
    let denominator: f64 = denominator.parse().ok()?;
    let rate = numerator / denominator;
    (rate.is_finite() && rate > 0.).then_some(rate)
}

pub fn standard_rate(frames_per_second: f64) -> FrameRate {
    let distance = |(numerator, denominator): &(u32, u32)| {
        (frames_per_second * *denominator as f64 / *numerator as f64)
            .ln()
            .abs()
    };
    let (numerator, denominator) = STANDARD_RATES
        .iter()
        .min_by(|left, right| distance(left).total_cmp(&distance(right)))
        .copied()
        .unwrap();
    FrameRate {
        numerator,
        denominator,
    }
}

pub fn parse_probe(output: &str) -> Result<VideoFormat> {
    let value: Value = serde_json::from_str(output)?;
    let stream = &value["streams"][0];
    let dimension = |key: &str| {
        stream[key]
            .as_u64()
            .filter(|value| (2..=16384).contains(value))
            .map(|value| value as u32)
    };
    let invalid = || anyhow::anyhow!(crate::i18n::message("error.videoFormat"));
    let (mut width, mut height) = (
        dimension("width").ok_or_else(invalid)?,
        dimension("height").ok_or_else(invalid)?,
    );
    let rotation = stream["side_data_list"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|data| data["rotation"].as_f64())
        .unwrap_or(0.);
    if (rotation.abs().round() as i64) % 180 == 90 {
        (width, height) = (height, width);
    }
    let nominal = stream["r_frame_rate"]
        .as_str()
        .and_then(rate)
        .filter(|rate| *rate <= 121.);
    let average = stream["avg_frame_rate"].as_str().and_then(rate);
    let frames_per_second = nominal.or(average).ok_or_else(invalid)?;
    Ok(VideoFormat {
        width: width & !1,
        height: height & !1,
        frame_rate: standard_rate(frames_per_second),
    })
}

const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

pub fn probe(ffprobe: &Path, input: &OsStr, cancellation: &Cancellation) -> Result<VideoFormat> {
    let output = Arc::new(Mutex::new(String::new()));
    let collected = output.clone();
    supervise(
        ffprobe,
        &[
            OsStr::new("-v"),
            OsStr::new("error"),
            OsStr::new("-select_streams"),
            OsStr::new("v:0"),
            OsStr::new("-show_entries"),
            OsStr::new("stream=width,height,avg_frame_rate,r_frame_rate:stream_side_data=rotation"),
            OsStr::new("-of"),
            OsStr::new("json"),
            input,
        ],
        cancellation,
        Some(PROBE_TIMEOUT),
        move |line| {
            let mut output = collected.lock().unwrap();
            output.push_str(line);
            output.push('\n');
        },
    )?;
    let output = output.lock().unwrap().clone();
    parse_probe(&output)
}

pub fn progress_fraction(line: &str, total_microseconds: u64) -> Option<f64> {
    let microseconds: f64 = line.trim().strip_prefix("out_time_us=")?.parse().ok()?;
    Some((microseconds / total_microseconds.max(1) as f64).clamp(0., 1.))
}

pub fn supervise(
    program: &Path,
    arguments: &[impl AsRef<OsStr>],
    cancellation: &Cancellation,
    timeout: Option<Duration>,
    mut on_line: impl FnMut(&str) + Send + 'static,
) -> Result<()> {
    cancellation.check()?;
    let mut child = spawn(program, arguments)?;
    let stdout = child.stdout.take().context("stdout")?;
    let stderr = child.stderr.take().context("stderr")?;
    let output_reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            on_line(&line);
        }
    });
    let error_reader = thread::spawn(move || read_all(stderr));
    let started = Instant::now();
    let stop = |child: &mut Child| {
        let _ = child.kill();
        let _ = child.wait();
    };
    let status = loop {
        if cancellation.requested() {
            stop(&mut child);
            let _ = output_reader.join();
            let _ = error_reader.join();
            return Err(Cancelled.into());
        }
        if timeout.is_some_and(|timeout| started.elapsed() > timeout) {
            stop(&mut child);
            let _ = output_reader.join();
            let _ = error_reader.join();
            return Err(anyhow::anyhow!(crate::i18n::message("error.toolTimeout")));
        }
        if let Some(status) = child.try_wait()? {
            break status;
        }
        thread::sleep(Duration::from_millis(50));
    };
    let _ = output_reader.join();
    let errors = error_reader.join().unwrap_or_default();
    if !status.success() {
        return Err(failure(&errors));
    }
    Ok(())
}

pub fn run(
    ffmpeg: &Path,
    arguments: &[OsString],
    total_microseconds: u64,
    progress: &Progress,
    cancellation: &Cancellation,
) -> Result<()> {
    let reporter = progress.clone();
    let label = crate::i18n::message("progress.encodeVideo");
    reporter.report(&label, 0.);
    supervise(ffmpeg, arguments, cancellation, None, move |line| {
        if let Some(fraction) = progress_fraction(line, total_microseconds) {
            reporter.report(&label, fraction);
        }
    })
}
