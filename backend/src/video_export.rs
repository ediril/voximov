use crate::{
    cancellation::{Cancellation, Cancelled},
    ffmpeg::{self, Tool},
    mixer,
    model::{FrameRate, Project, SAMPLE_RATE, id},
    storage::Store,
};
use anyhow::{Result, ensure};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub type Samples = u64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VideoFormat {
    pub width: u32,
    pub height: u32,
    pub frame_rate: FrameRate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VideoInput {
    pub path: PathBuf,
    pub base: u64,
    pub length: u64,
}

impl VideoInput {
    pub fn url(&self) -> OsString {
        let mut url = OsString::from(if self.base == 0 {
            String::new()
        } else {
            format!(
                "subfile,,start,{},end,{},,:",
                self.base,
                self.base + self.length
            )
        });
        url.push("file:");
        url.push(&self.path);
        url
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VideoSegment {
    Clip {
        input: usize,
        from: Samples,
        to: Samples,
    },
    Gap {
        length: Samples,
    },
}

impl VideoSegment {
    pub fn length(&self) -> Samples {
        match self {
            Self::Clip { from, to, .. } => to - from,
            Self::Gap { length } => *length,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VideoExportPlan {
    pub format: VideoFormat,
    pub length: Samples,
    pub inputs: Vec<VideoInput>,
    pub segments: Vec<VideoSegment>,
}

struct PlacedClip<'a> {
    asset_id: &'a str,
    start: Samples,
    end: Samples,
    offset: Samples,
}

fn samples(seconds: f64) -> Samples {
    (seconds * SAMPLE_RATE as f64).round() as Samples
}

fn placed_clips(project: &Project, length: Samples) -> Vec<PlacedClip<'_>> {
    let Some(track) = project.tracks.iter().find(|track| track.kind == "video") else {
        return Vec::new();
    };
    let mut clips: Vec<_> = track
        .clips
        .iter()
        .filter_map(|clip| {
            let asset_id = clip.asset_id.as_deref()?;
            project
                .assets
                .iter()
                .any(|asset| asset.id == asset_id && asset.kind == "video")
                .then_some(())?;
            let start = samples(clip.start);
            Some(PlacedClip {
                asset_id,
                start,
                end: (start + samples(clip.duration)).min(length),
                offset: samples(clip.offset),
            })
        })
        .collect();
    clips.sort_by_key(|clip| clip.start);
    for index in 1..clips.len() {
        let next_start = clips[index].start;
        let previous = &mut clips[index - 1];
        previous.end = previous.end.min(next_start);
    }
    clips.retain(|clip| clip.end > clip.start);
    clips
}

pub fn has_video(project: &Project) -> bool {
    !placed_clips(project, mixer::rendered_frames(project.duration())).is_empty()
}

pub fn plan(
    project: &Project,
    mut locate: impl FnMut(&str) -> Result<VideoInput>,
    probe: impl FnOnce(&VideoInput) -> Result<VideoFormat>,
) -> Result<VideoExportPlan> {
    let length = mixer::rendered_frames(project.duration());
    let clips = placed_clips(project, length);
    ensure!(
        !clips.is_empty(),
        crate::i18n::message("error.noExportVideo")
    );
    let mut inputs = Vec::new();
    let mut input_indices = HashMap::new();
    let mut segments = Vec::new();
    let mut cursor = 0;
    for clip in clips {
        if clip.start > cursor {
            segments.push(VideoSegment::Gap {
                length: clip.start - cursor,
            });
        }
        let input = match input_indices.get(clip.asset_id) {
            Some(index) => *index,
            None => {
                inputs.push(locate(clip.asset_id)?);
                input_indices.insert(clip.asset_id, inputs.len() - 1);
                inputs.len() - 1
            }
        };
        segments.push(VideoSegment::Clip {
            input,
            from: clip.offset,
            to: clip.offset + clip.end - clip.start,
        });
        cursor = clip.end;
    }
    if cursor < length {
        segments.push(VideoSegment::Gap {
            length: length - cursor,
        });
    }
    let format = probe(&inputs[0])?;
    ensure!(
        format.frame_rate.frames_at(length) > 0,
        crate::i18n::message("error.noExportVideo")
    );
    Ok(VideoExportPlan {
        format,
        length,
        inputs,
        segments,
    })
}

fn decimal(value: f64) -> String {
    let text = format!("{value:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn seconds(position: Samples) -> String {
    decimal(position as f64 / SAMPLE_RATE as f64)
}

pub fn bitrate(format: &VideoFormat) -> u64 {
    let pixels = format.width as u64 * format.height as u64;
    (8_000 * pixels / (1920 * 1080)).max(1_000)
}

pub const SEGMENTS_PER_BATCH: usize = 8;
const BOUNDARY_TOLERANCE: f64 = 0.0005;

#[derive(Debug, PartialEq)]
pub struct EncodedSegment<'a> {
    pub segment: &'a VideoSegment,
    pub start: Samples,
    pub first_frame: u64,
    pub frames: u64,
}

fn encoded_segments(plan: &VideoExportPlan) -> Vec<EncodedSegment<'_>> {
    let rate = plan.format.frame_rate;
    let mut cursor = 0;
    plan.segments
        .iter()
        .filter_map(|segment| {
            let start = cursor;
            cursor += segment.length();
            let first_frame = rate.frames_at(start);
            let frames = rate.frames_at(cursor) - first_frame;
            (frames > 0).then_some(EncodedSegment {
                segment,
                start,
                first_frame,
                frames,
            })
        })
        .collect()
}

pub fn batches(plan: &VideoExportPlan) -> Vec<Vec<EncodedSegment<'_>>> {
    let mut batches: Vec<Vec<EncodedSegment>> = Vec::new();
    for segment in encoded_segments(plan) {
        match batches.last_mut() {
            Some(batch) if batch.len() < SEGMENTS_PER_BATCH => batch.push(segment),
            _ => batches.push(vec![segment]),
        }
    }
    batches
}

pub fn filter_graph(plan: &VideoExportPlan, batch: &[EncodedSegment]) -> String {
    let VideoFormat {
        width,
        height,
        frame_rate,
    } = plan.format;
    let black = format!("color=c=black:s={width}x{height}:r={frame_rate}");
    let mut input = 0;
    let mut chains = Vec::new();
    let mut labels = String::new();
    for (index, encoded) in batch.iter().enumerate() {
        let frames = encoded.frames;
        chains.push(match encoded.segment {
            VideoSegment::Clip { .. } => {
                let shift = decimal(
                    frame_rate.seconds(encoded.first_frame)
                        - encoded.start as f64 / SAMPLE_RATE as f64
                        + BOUNDARY_TOLERANCE,
                );
                let end = decimal(frame_rate.seconds(frames + 1));
                format!(
                    "{black},trim=end_frame={frames}[b{index}];[{input}:v]setpts=PTS-({shift})/TB,trim=end={end},scale={width}:{height}:force_original_aspect_ratio=decrease:force_divisible_by=2,setsar=1[c{index}];[b{index}][c{index}]overlay=x=(W-w)/2:y=(H-h)/2:eof_action=repeat:repeatlast=1,trim=end_frame={frames}[s{index}]"
                )
            }
            VideoSegment::Gap { .. } => {
                format!("{black},trim=end_frame={frames}[s{index}]")
            }
        });
        if let VideoSegment::Clip { .. } = encoded.segment {
            input += 1;
        }
        labels.push_str(&format!("[s{index}]"));
    }
    let count = batch.len();
    chains.push(format!(
        "{labels}concat=n={count}:v=1:a=0,setpts=N/({frame_rate})/TB,format=yuv420p,setsar=1[v]"
    ));
    chains.join(";")
}

fn header() -> Vec<OsString> {
    [
        "-hide_banner",
        "-nostdin",
        "-loglevel",
        "error",
        "-y",
        "-progress",
        "pipe:1",
        "-nostats",
    ]
    .map(OsString::from)
    .into()
}

pub fn batch_arguments(
    plan: &VideoExportPlan,
    batch: &[EncodedSegment],
    output: &Path,
) -> Vec<OsString> {
    let mut arguments = header();
    for encoded in batch {
        if let VideoSegment::Clip { input, from, .. } = encoded.segment {
            arguments.extend(
                ["-noaccurate_seek", "-an", "-ss", &seconds(*from), "-i"].map(OsString::from),
            );
            arguments.push(plan.inputs[*input].url());
        }
    }
    arguments.extend(
        [
            "-filter_complex".into(),
            filter_graph(plan, batch),
            "-map".into(),
            "[v]".into(),
            "-fps_mode".into(),
            "passthrough".into(),
            "-c:v".into(),
            "h264_videotoolbox".into(),
            "-allow_sw".into(),
            "1".into(),
            "-bf".into(),
            "0".into(),
            "-b:v".into(),
            format!("{}k", bitrate(&plan.format)),
            "-f".into(),
            "h264".into(),
        ]
        .map(OsString::from),
    );
    arguments.push(output.into());
    arguments
}

pub fn mux_arguments(
    plan: &VideoExportPlan,
    video: &Path,
    audio: &Path,
    output: &Path,
) -> Vec<OsString> {
    let rate = plan.format.frame_rate;
    let mut arguments = header();
    arguments.extend(["-f", "h264", "-r", &rate.to_string(), "-i"].map(OsString::from));
    arguments.push(video.into());
    arguments.push("-i".into());
    arguments.push(audio.into());
    arguments.extend(
        [
            "-map".into(),
            "0:v".into(),
            "-map".into(),
            "1:a".into(),
            "-c:v".into(),
            "copy".into(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "192k".into(),
            "-video_track_timescale".into(),
            rate.track_timescale().to_string(),
            "-movflags".into(),
            "+faststart".into(),
            "-f".into(),
            "mp4".into(),
        ]
        .map(OsString::from),
    );
    arguments.push(output.into());
    arguments
}

pub enum Outcome {
    Saved,
    Cancelled,
}

struct TemporaryFiles(Vec<PathBuf>);

impl Drop for TemporaryFiles {
    fn drop(&mut self) {
        for path in &self.0 {
            let _ = fs::remove_file(path);
        }
    }
}

fn encode_video(
    store: &Store,
    plan: &VideoExportPlan,
    batch_file: &Path,
    video: &Path,
    progress: &crate::progress::Progress,
) -> Result<()> {
    let ffmpeg = ffmpeg::locate(Tool::Ffmpeg);
    let rate = plan.format.frame_rate;
    let total = rate.frames_at(plan.length).max(1);
    let mut joined = fs::File::create(video)?;
    for batch in batches(plan) {
        store.cancellation.check()?;
        let first = batch[0].first_frame;
        let frames: u64 = batch.iter().map(|encoded| encoded.frames).sum();
        ffmpeg::run(
            &ffmpeg,
            &batch_arguments(plan, &batch, batch_file),
            (rate.seconds(frames) * 1_000_000.) as u64,
            &progress.range(first as f64 / total as f64, frames as f64 / total as f64),
            &store.cancellation,
        )?;
        append(batch_file, &mut joined, &store.cancellation)?;
    }
    store.cancellation.check()?;
    joined.sync_all()?;
    Ok(())
}

const COPY_CHUNK: usize = 1 << 20;

pub fn append(
    source: &Path,
    destination: &mut impl Write,
    cancellation: &Cancellation,
) -> Result<()> {
    let mut input = fs::File::open(source)?;
    let mut buffer = vec![0; COPY_CHUNK];
    loop {
        cancellation.check()?;
        let count = input.read(&mut buffer)?;
        if count == 0 {
            return Ok(());
        }
        destination.write_all(&buffer[..count])?;
    }
}

pub fn export(store: &mut Store, destination: &Path) -> Result<Outcome> {
    let ffprobe = ffmpeg::locate(Tool::Ffprobe);
    let plan = plan(
        &store.project,
        |asset_id| {
            let (path, base, length) = store.location(asset_id, false)?;
            Ok(VideoInput { path, base, length })
        },
        |input| ffmpeg::probe(&ffprobe, &input.url(), &store.cancellation),
    );
    let plan = match plan {
        Err(error) if error.is::<Cancelled>() => return Ok(Outcome::Cancelled),
        plan => plan?,
    };
    let partial = |suffix: &str| destination.with_extension(format!("{}.{suffix}", id()));
    let output = partial("partial");
    let audio = partial("partial.wav");
    let batch_file = partial("partial.batch.h264");
    let video = partial("partial.h264");
    let temporary = TemporaryFiles(vec![
        audio.clone(),
        batch_file.clone(),
        video.clone(),
        output.clone(),
    ]);
    let progress = store.progress.clone();
    let result = (|| {
        store.progress = progress.range(0., 0.2);
        let rendered = mixer::render(store, &audio, None, "wav");
        store.progress = progress.clone();
        rendered?;
        encode_video(
            store,
            &plan,
            &batch_file,
            &video,
            &progress.range(0.2, 0.75),
        )?;
        ffmpeg::run(
            &ffmpeg::locate(Tool::Ffmpeg),
            &mux_arguments(&plan, &video, &audio, &output),
            plan.length * 1_000_000 / SAMPLE_RATE as u64,
            &progress.range(0.95, 0.05),
            &store.cancellation,
        )?;
        crate::filesystem::replace_file(&output, destination)
    })();
    drop(temporary);
    match result {
        Ok(()) => Ok(Outcome::Saved),
        Err(error) if error.is::<Cancelled>() => Ok(Outcome::Cancelled),
        Err(error) => Err(error),
    }
}
