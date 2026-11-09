use crate::{
    commands,
    ffmpeg::{self, Tool},
    model::{Asset, Clip, SAMPLE_RATE, id},
    progress::Progress,
    storage::Store,
};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

struct Tools {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
}

fn tools(test: &str) -> Option<Tools> {
    let tools = Tools {
        ffmpeg: ffmpeg::locate(Tool::Ffmpeg),
        ffprobe: ffmpeg::locate(Tool::Ffprobe),
    };
    let works = |tool: &Path| {
        Command::new(tool)
            .arg("-version")
            .output()
            .is_ok_and(|output| output.status.success())
    };
    if works(&tools.ffmpeg) && works(&tools.ffprobe) {
        Some(tools)
    } else {
        eprintln!(
            "Skipping {test}: no FFmpeg and FFprobe found (set VOXIMOV_FFMPEG and VOXIMOV_FFPROBE or put them on PATH)"
        );
        None
    }
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/studio-check.mp4")
}

fn generate(tools: &Tools, graph: &str, output: &Path) -> Result<()> {
    let status = Command::new(&tools.ffmpeg)
        .args(["-v", "error", "-y", "-filter_complex", graph])
        .args(["-map", "[v]", "-fps_mode", "passthrough"])
        .args([
            "-c:v",
            "h264_videotoolbox",
            "-allow_sw",
            "1",
            "-b:v",
            "2000k",
        ])
        .args(["-f", "mp4"])
        .arg(output)
        .status()?;
    ensure!(status.success(), "could not generate {}", output.display());
    Ok(())
}

fn colors_video(tools: &Tools, directory: &Path) -> Result<PathBuf> {
    let output = directory.join("colors.mp4");
    generate(
        tools,
        "color=c=red:s=128x72:r=30:d=1[a];color=c=green:s=128x72:r=30:d=1[b];color=c=blue:s=128x72:r=30:d=1[c];[a][b][c]concat=n=3:v=1:a=0,format=yuv420p[v]",
        &output,
    )?;
    Ok(output)
}

fn sparse_video(tools: &Tools, directory: &Path) -> Result<PathBuf> {
    let output = directory.join("sparse.mp4");
    generate(
        tools,
        "color=c=red:s=128x72:r=1:d=1[a];color=c=green:s=128x72:r=1:d=1[b];color=c=blue:s=128x72:r=1:d=1[c];[a][b][c]concat=n=3:v=1:a=0,setpts=N*3/TB,format=yuv420p[v]",
        &output,
    )?;
    Ok(output)
}

fn silent_asset(store: &Store, name: &str, kind: &str, duration: f64) -> Result<Asset> {
    let asset = Asset {
        id: id(),
        name: name.into(),
        kind: kind.into(),
        size: 0,
        duration,
        frames: 0,
        sample_rate: SAMPLE_RATE,
        waveform: vec![],
        peak_frames: 256,
    };
    std::fs::write(store.pcm(&asset.id), [])?;
    Ok(asset)
}

fn clip(asset: &Asset, start: f64, offset: f64, duration: f64) -> Clip {
    Clip {
        id: id(),
        asset_id: Some(asset.id.clone()),
        name: asset.name.clone(),
        start,
        offset,
        duration,
        script: None,
    }
}

fn timeline_store(
    directory: &Path,
    sources: &[(&Path, f64)],
    clips: &[(usize, f64, f64, f64)],
    end: f64,
) -> Result<Store> {
    let mut store = Store::open(directory.join("config"), Some(directory.join("work")))?;
    let mut assets = Vec::new();
    for (path, duration) in sources {
        let asset = silent_asset(&store, "source.mp4", "video", *duration)?;
        std::fs::copy(path, store.source(&asset.id))?;
        assets.push(asset);
    }
    for (source, start, offset, duration) in clips {
        store.project.tracks[0]
            .clips
            .push(clip(&assets[*source], *start, *offset, *duration));
    }
    let music = silent_asset(&store, "music.wav", "audio", end)?;
    store.project.tracks[2]
        .clips
        .push(clip(&music, 0., 0., end));
    store.project.assets = assets;
    store.project.assets.push(music);
    store.project.validate()?;
    Ok(store)
}

fn export(store: &mut Store, destination: &Path) -> Result<Value> {
    commands::execute(store, &json!({"command":"export_video","path":destination}))
}

fn frame_colors(tools: &Tools, file: &Path) -> Result<String> {
    let output = Command::new(&tools.ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(file)
        .args(["-vf", "crop=8:8:(iw-8)/2:(ih-8)/2,scale=1:1"])
        .args([
            "-fps_mode",
            "passthrough",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
        ])
        .arg("pipe:1")
        .output()?;
    ensure!(
        output.status.success(),
        "could not decode {}",
        file.display()
    );
    Ok(output
        .stdout
        .as_chunks::<3>()
        .0
        .iter()
        .map(|pixel| {
            let brightest = *pixel.iter().max().unwrap();
            if brightest < 60 {
                'K'
            } else {
                ['R', 'G', 'B'][pixel.iter().position(|value| *value == brightest).unwrap()]
            }
        })
        .collect())
}

fn sequence(runs: &[(char, usize)]) -> String {
    runs.iter()
        .map(|(color, count)| color.to_string().repeat(*count))
        .collect()
}

fn frame_times(tools: &Tools, file: &Path) -> Result<Vec<f64>> {
    let output = Command::new(&tools.ffprobe)
        .args(["-v", "error", "-select_streams", "v:0"])
        .args(["-show_entries", "packet=pts_time", "-of", "csv=p=0"])
        .arg(file)
        .output()?;
    ensure!(
        output.status.success(),
        "could not probe {}",
        file.display()
    );
    let mut times: Vec<f64> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().trim_end_matches(',').parse().ok())
        .collect();
    times.sort_by(f64::total_cmp);
    Ok(times)
}

fn assert_constant_frame_rate(times: &[f64], frames_per_second: f64) {
    for (index, time) in times.iter().enumerate() {
        let expected = index as f64 / frames_per_second;
        assert!(
            (time - expected).abs() < 1e-4,
            "frame {index} at {time}, expected {expected}"
        );
    }
}

fn streams(tools: &Tools, file: &Path) -> Result<Value> {
    let output = Command::new(&tools.ffprobe)
        .args(["-v", "error", "-count_packets", "-show_entries"])
        .args([
            "stream=codec_type,codec_name,width,height,nb_read_packets:format=duration",
            "-of",
            "json",
        ])
        .arg(file)
        .output()?;
    ensure!(
        output.status.success(),
        "could not probe {}",
        file.display()
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn leftovers(directory: &Path) -> Result<Vec<String>> {
    Ok(std::fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|name| name.contains("partial"))
        .collect())
}

const FRAME: f64 = 1. / 30.;

#[test]
fn one_frame_clips_and_gaps_keep_later_video_in_place() -> Result<()> {
    let Some(tools) = tools("the one-frame segment test") else {
        return Ok(());
    };
    let directory = tempfile::tempdir()?;
    let colors = colors_video(&tools, directory.path())?;
    let mut store = timeline_store(
        directory.path(),
        &[(&colors, 3.)],
        &[
            (0, 0., 0., FRAME),
            (0, 2. * FRAME, 1., 30. * FRAME),
            (0, 32. * FRAME, 2.5, FRAME),
            (0, 34. * FRAME, 0.2, 10. * FRAME),
        ],
        44. * FRAME,
    )?;
    let output = directory.path().join("one-frame.mp4");
    assert_eq!(export(&mut store, &output)?["saved"], true);
    assert_eq!(
        frame_colors(&tools, &output)?,
        sequence(&[('R', 1), ('K', 1), ('G', 30), ('B', 1), ('K', 1), ('R', 10)])
    );
    assert_constant_frame_rate(&frame_times(&tools, &output)?, 30.);
    Ok(())
}

#[test]
fn sparse_variable_frame_rate_footage_shows_the_frame_current_at_each_moment() -> Result<()> {
    let Some(tools) = tools("the sparse footage test") else {
        return Ok(());
    };
    let directory = tempfile::tempdir()?;
    let colors = colors_video(&tools, directory.path())?;
    let sparse = sparse_video(&tools, directory.path())?;
    let mut store = timeline_store(
        directory.path(),
        &[(&colors, 3.), (&sparse, 7.)],
        &[(0, 0., 1., FRAME), (1, FRAME, 1., 5.)],
        FRAME + 5.,
    )?;
    let output = directory.path().join("sparse-cut.mp4");
    assert_eq!(export(&mut store, &output)?["saved"], true);
    assert_eq!(
        frame_colors(&tools, &output)?,
        sequence(&[('G', 1), ('R', 60), ('G', 90)])
    );
    assert_constant_frame_rate(&frame_times(&tools, &output)?, 30.);
    Ok(())
}

#[test]
fn clips_past_the_end_of_the_video_hold_its_last_frame_without_shifting_later_clips() -> Result<()>
{
    let Some(tools) = tools("the past-the-end test") else {
        return Ok(());
    };
    let directory = tempfile::tempdir()?;
    let colors = colors_video(&tools, directory.path())?;
    let mut store = timeline_store(
        directory.path(),
        &[(&colors, 10.)],
        &[(0, 0., 0., 1.), (0, 2., 5., 1.), (0, 3., 1., 1.)],
        4.,
    )?;
    let output = directory.path().join("past-end.mp4");
    assert_eq!(export(&mut store, &output)?["saved"], true);
    assert_eq!(
        frame_colors(&tools, &output)?,
        sequence(&[('R', 30), ('K', 30), ('B', 30), ('G', 30)])
    );
    Ok(())
}

#[test]
fn many_tiny_cuts_export_in_batches_with_exact_frames() -> Result<()> {
    let Some(tools) = tools("the many-cuts test") else {
        return Ok(());
    };
    let directory = tempfile::tempdir()?;
    let clips: Vec<_> = (0..100)
        .map(|index| {
            (
                0,
                index as f64 * 0.12,
                (index * 7 % 130) as f64 * 0.04,
                0.08,
            )
        })
        .collect();
    let mut store = timeline_store(directory.path(), &[(&fixture(), 6.)], &clips, 12.)?;
    let output = directory.path().join("many-cuts.mp4");
    assert_eq!(export(&mut store, &output)?["saved"], true);
    assert!(leftovers(directory.path())?.is_empty());
    let times = frame_times(&tools, &output)?;
    assert_eq!(times.len(), 300);
    assert_constant_frame_rate(&times, 25.);
    Ok(())
}

fn edited_store(directory: &Path) -> Result<Store> {
    let mut store = Store::open(directory.join("config"), Some(directory.join("work")))?;
    commands::import_source(
        &mut store,
        fixture(),
        "video.mp4".into(),
        "video".into(),
        None,
        0.,
    )?;
    let mut project = store.project.clone();
    let video = &mut project.tracks[0];
    let mut tail = video.clips[0].clone();
    video.clips[0].duration = 2.;
    tail.id = id();
    tail.start = 3.;
    tail.offset = 3.5;
    tail.duration = 2.5;
    video.clips.push(tail);
    store.replace(project)?;
    Ok(store)
}

fn export_and_check(store: &mut Store, tools: &Tools, destination: &Path) -> Result<()> {
    let result = export(store, destination)?;
    assert_eq!(result, json!({"saved":true,"path":destination}));
    assert!(leftovers(destination.parent().unwrap())?.is_empty());
    let probed = streams(tools, destination)?;
    let streams = probed["streams"].as_array().unwrap();
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .unwrap();
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .unwrap();
    assert_eq!(video["codec_name"], "h264");
    assert_eq!(
        (video["width"].as_u64(), video["height"].as_u64()),
        (Some(960), Some(540))
    );
    assert_eq!(audio["codec_name"], "aac");
    assert_eq!(video["nb_read_packets"], "138");
    let duration: f64 = probed["format"]["duration"].as_str().unwrap().parse()?;
    assert!((duration - 5.5).abs() < 0.1, "{duration}");
    assert_constant_frame_rate(&frame_times(tools, destination)?, 25.);
    Ok(())
}

#[test]
fn exports_an_edit_through_a_real_ffmpeg_before_and_after_saving() -> Result<()> {
    let Some(tools) = tools("the video export integration test") else {
        return Ok(());
    };
    let temporary = tempfile::tempdir()?;
    let mut store = edited_store(temporary.path())?;
    assert_eq!(store.location(&store.project.assets[0].id, false)?.1, 0);
    export_and_check(&mut store, &tools, &temporary.path().join("Unsaved.mp4"))?;
    let saved = temporary.path().join("saved, copy");
    store.save(&saved.join("Edit 1.voximov"))?;
    assert!(store.location(&store.project.assets[0].id, false)?.1 > 0);
    export_and_check(&mut store, &tools, &saved.join("Edit 1.mp4"))?;
    export_and_check(&mut store, &tools, &saved.join("Edit 1.mp4"))
}

#[test]
fn cancelling_a_video_export_removes_every_partial_file() -> Result<()> {
    let Some(_tools) = tools("the video export cancellation test") else {
        return Ok(());
    };
    for phase in ["progress.renderMix", "progress.encodeVideo"] {
        let temporary = tempfile::tempdir()?;
        let mut store = edited_store(temporary.path())?;
        let destination = temporary.path().join("cancelled.mp4");
        let cancellation = store.cancellation.clone();
        let labels = Arc::new(Mutex::new(Vec::new()));
        let seen = labels.clone();
        store.progress = Progress::new(move |event| {
            if event.label == crate::i18n::message(phase) {
                cancellation.request();
            }
            seen.lock().unwrap().push(event.label);
        });
        let result = export(&mut store, &destination)?;
        assert_eq!(result, json!({"saved":false,"cancelled":true}), "{phase}");
        assert!(!destination.exists());
        assert!(leftovers(temporary.path())?.is_empty(), "{phase}");
        assert!(
            labels
                .lock()
                .unwrap()
                .contains(&crate::i18n::message(phase))
        );
        assert!(!store.cancellation.requested());
    }
    Ok(())
}
