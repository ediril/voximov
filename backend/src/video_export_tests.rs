use crate::{
    cancellation::{Cancellation, Cancelled},
    ffmpeg::{self, Tool},
    mixer::Mixer,
    model::{Asset, Clip, Project, SAMPLE_RATE, Track, id},
    progress::Progress,
    storage::Store,
    video_export::{
        FrameRate, SEGMENTS_PER_BATCH, VideoExportPlan, VideoFormat, VideoInput, VideoSegment,
        batch_arguments, batches, bitrate, filter_graph, has_video, mux_arguments, plan,
    },
};
use anyhow::Result;
use std::{
    cell::RefCell,
    ffi::OsString,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const RATE: u64 = SAMPLE_RATE as u64;
const FULL_HD: VideoFormat = VideoFormat {
    width: 1920,
    height: 1080,
    frame_rate: FrameRate {
        numerator: 30,
        denominator: 1,
    },
};

fn asset(kind: &str, duration: f64) -> Asset {
    Asset {
        id: id(),
        name: format!("{kind}.mp4"),
        kind: kind.into(),
        size: 0,
        duration,
        frames: 0,
        sample_rate: SAMPLE_RATE,
        waveform: vec![],
        peak_frames: 256,
    }
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

fn project(video_clips: Vec<Clip>, assets: Vec<Asset>, end: f64) -> Project {
    let mut project = Project::new();
    project.tracks[0].clips = video_clips;
    let music = asset("audio", end);
    project.tracks[2].clips.push(clip(&music, 0., 0., end));
    project.assets = assets;
    project.assets.push(music);
    project.validate().unwrap();
    project
}

fn standalone(asset_id: &str) -> Result<VideoInput> {
    Ok(VideoInput {
        path: PathBuf::from(format!("/media/{asset_id}")),
        base: 0,
        length: 100,
    })
}

fn planned(project: &Project) -> Result<VideoExportPlan> {
    plan(project, standalone, |_| Ok(FULL_HD))
}

#[test]
fn gaps_fill_the_start_the_middle_and_the_end_up_to_the_mix_length() -> Result<()> {
    let first = asset("video", 60.);
    let second = asset("video", 60.);
    let project = project(
        vec![clip(&second, 4., 0., 1.), clip(&first, 1., 5., 2.)],
        vec![first.clone(), second.clone()],
        8.,
    );
    let plan = planned(&project)?;
    assert_eq!(plan.length, 8 * RATE);
    assert_eq!(
        plan.inputs,
        vec![standalone(&first.id)?, standalone(&second.id)?]
    );
    assert_eq!(
        plan.segments,
        vec![
            VideoSegment::Gap { length: RATE },
            VideoSegment::Clip {
                input: 0,
                from: 5 * RATE,
                to: 7 * RATE
            },
            VideoSegment::Gap { length: RATE },
            VideoSegment::Clip {
                input: 1,
                from: 0,
                to: RATE
            },
            VideoSegment::Gap { length: 3 * RATE },
        ]
    );
    assert_eq!(
        plan.segments.iter().map(VideoSegment::length).sum::<u64>(),
        plan.length
    );
    Ok(())
}

#[test]
fn clip_positions_stay_on_the_sample_grid_of_the_mix() -> Result<()> {
    let source = asset("video", 60.);
    let project = project(
        vec![clip(&source, 0.123_456_7, 2.000_010_4, 1.5)],
        vec![source.clone()],
        3.000_001,
    );
    let plan = planned(&project)?;
    assert_eq!(plan.length, (3.000_001 * RATE as f64).ceil() as u64);
    assert_eq!(
        plan.segments,
        vec![
            VideoSegment::Gap { length: 5926 },
            VideoSegment::Clip {
                input: 0,
                from: 96000,
                to: 96000 + 72000
            },
            VideoSegment::Gap {
                length: plan.length - 5926 - 72000
            },
        ]
    );
    Ok(())
}

#[test]
fn a_later_clip_wins_an_overlap_from_its_start() -> Result<()> {
    let source = asset("video", 60.);
    let project = project(
        vec![
            clip(&source, 0., 10., 4.),
            clip(&source, 2., 30., 3.),
            clip(&source, 6., 0., 4.),
            clip(&source, 7., 20., 1.),
        ],
        vec![source.clone()],
        10.,
    );
    let plan = planned(&project)?;
    assert_eq!(plan.inputs.len(), 1);
    assert_eq!(
        plan.segments,
        vec![
            VideoSegment::Clip {
                input: 0,
                from: 10 * RATE,
                to: 12 * RATE
            },
            VideoSegment::Clip {
                input: 0,
                from: 30 * RATE,
                to: 33 * RATE
            },
            VideoSegment::Gap { length: RATE },
            VideoSegment::Clip {
                input: 0,
                from: 0,
                to: RATE
            },
            VideoSegment::Clip {
                input: 0,
                from: 20 * RATE,
                to: 21 * RATE
            },
            VideoSegment::Gap { length: 2 * RATE },
        ]
    );
    Ok(())
}

#[test]
fn archived_media_is_read_through_the_subfile_protocol() -> Result<()> {
    let source = asset("video", 60.);
    let project = project(vec![clip(&source, 0., 0., 2.)], vec![source.clone()], 2.);
    let archived = PathBuf::from("/Projects/My film, final.voximov");
    let plan = plan(
        &project,
        |_| {
            Ok(VideoInput {
                path: archived.clone(),
                base: 4096,
                length: 1000,
            })
        },
        |_| Ok(FULL_HD),
    )?;
    assert_eq!(
        plan.inputs[0].url(),
        OsString::from("subfile,,start,4096,end,5096,,:file:/Projects/My film, final.voximov")
    );
    assert_eq!(
        standalone("clip")?.url(),
        OsString::from("file:/media/clip")
    );
    Ok(())
}

#[test]
fn output_format_comes_from_the_first_clip_and_every_clip_is_fitted_to_it() -> Result<()> {
    let small = asset("video", 60.);
    let large = asset("video", 60.);
    let project = project(
        vec![clip(&large, 3., 0., 1.), clip(&small, 1., 0., 1.)],
        vec![large.clone(), small.clone()],
        4.,
    );
    let probed = RefCell::new(Vec::new());
    let small_format = VideoFormat {
        width: 640,
        height: 480,
        frame_rate: FrameRate {
            numerator: 25,
            denominator: 1,
        },
    };
    let plan = plan(&project, standalone, |input| {
        probed.borrow_mut().push(input.clone());
        Ok(small_format)
    })?;
    assert_eq!(probed.into_inner(), vec![standalone(&small.id)?]);
    assert_eq!(plan.format, small_format);
    let batches = batches(&plan);
    assert_eq!(batches.len(), 1);
    let graph = filter_graph(&plan, &batches[0]);
    assert_eq!(graph.matches("scale=640:480:").count(), 2);
    assert_eq!(graph.matches("overlay=x=(W-w)/2:y=(H-h)/2").count(), 2);
    assert_eq!(graph.matches("color=c=black:s=640x480:r=25/1").count(), 4);
    Ok(())
}

#[test]
fn projects_without_video_clips_cannot_export_video() -> Result<()> {
    let music = asset("audio", 5.);
    let mut empty = Project::new();
    empty.tracks[2].clips.push(clip(&music, 0., 0., 5.));
    empty.assets.push(music);
    assert!(!has_video(&empty));
    let error = planned(&empty).unwrap_err();
    assert_eq!(error.to_string(), "[[error.noExportVideo]]");
    let source = asset("video", 5.);
    let mut second_track_only = project(vec![], vec![source.clone()], 5.);
    let mut track = Track::new("video", "track.video");
    track.clips.push(clip(&source, 0., 0., 5.));
    second_track_only.tracks.push(track);
    assert!(!has_video(&second_track_only));
    Ok(())
}

fn known_plan() -> VideoExportPlan {
    VideoExportPlan {
        format: FULL_HD,
        length: 10 * RATE,
        inputs: vec![
            VideoInput {
                path: PathBuf::from("/media/a.mp4"),
                base: 0,
                length: 10,
            },
            VideoInput {
                path: PathBuf::from("/saved/project.voximov"),
                base: 512,
                length: 2048,
            },
        ],
        segments: vec![
            VideoSegment::Clip {
                input: 0,
                from: 257_760,
                to: 257_760 + 2 * RATE,
            },
            VideoSegment::Gap { length: 2 * RATE },
            VideoSegment::Gap { length: 100 },
            VideoSegment::Clip {
                input: 1,
                from: 30 * RATE,
                to: 30 * RATE + 6 * RATE - 100,
            },
        ],
    }
}

#[test]
fn every_segment_is_exactly_its_frame_count_and_the_batch_is_retimed_by_count() {
    let plan = known_plan();
    let batches = batches(&plan);
    assert_eq!(batches.len(), 1);
    assert_eq!(
        filter_graph(&plan, &batches[0]),
        "color=c=black:s=1920x1080:r=30/1,trim=end_frame=60[b0];\
[0:v]setpts=PTS-(0.0005)/TB,trim=end=2.033333,scale=1920:1080:force_original_aspect_ratio=decrease:force_divisible_by=2,setsar=1[c0];\
[b0][c0]overlay=x=(W-w)/2:y=(H-h)/2:eof_action=repeat:repeatlast=1,trim=end_frame=60[s0];\
color=c=black:s=1920x1080:r=30/1,trim=end_frame=60[s1];\
color=c=black:s=1920x1080:r=30/1,trim=end_frame=180[b2];\
[1:v]setpts=PTS-(-0.001583)/TB,trim=end=6.033333,scale=1920:1080:force_original_aspect_ratio=decrease:force_divisible_by=2,setsar=1[c2];\
[b2][c2]overlay=x=(W-w)/2:y=(H-h)/2:eof_action=repeat:repeatlast=1,trim=end_frame=180[s2];\
[s0][s1][s2]concat=n=3:v=1:a=0,setpts=N/(30/1)/TB,format=yuv420p,setsar=1[v]"
    );
}

fn strings(arguments: &[&str]) -> Vec<OsString> {
    arguments.iter().map(OsString::from).collect()
}

#[test]
fn batch_arguments_seek_to_the_keyframe_before_each_clip_and_encode_to_raw_h264() {
    let plan = known_plan();
    let batches = batches(&plan);
    let graph = filter_graph(&plan, &batches[0]);
    assert_eq!(
        batch_arguments(&plan, &batches[0], Path::new("/tmp/batch.h264")),
        strings(&[
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-y",
            "-progress",
            "pipe:1",
            "-nostats",
            "-noaccurate_seek",
            "-an",
            "-ss",
            "5.37",
            "-i",
            "file:/media/a.mp4",
            "-noaccurate_seek",
            "-an",
            "-ss",
            "30",
            "-i",
            "subfile,,start,512,end,2560,,:file:/saved/project.voximov",
            "-filter_complex",
            &graph,
            "-map",
            "[v]",
            "-fps_mode",
            "passthrough",
            "-c:v",
            "h264_videotoolbox",
            "-allow_sw",
            "1",
            "-bf",
            "0",
            "-b:v",
            "8000k",
            "-f",
            "h264",
            "/tmp/batch.h264",
        ])
    );
}

#[test]
fn the_final_step_times_video_by_frame_count_and_adds_the_mix() {
    let mut plan = known_plan();
    plan.format.frame_rate = FrameRate {
        numerator: 30000,
        denominator: 1001,
    };
    assert_eq!(
        mux_arguments(
            &plan,
            Path::new("/tmp/video.h264"),
            Path::new("/tmp/mix.wav"),
            Path::new("/tmp/out.partial")
        ),
        strings(&[
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-y",
            "-progress",
            "pipe:1",
            "-nostats",
            "-f",
            "h264",
            "-r",
            "30000/1001",
            "-i",
            "/tmp/video.h264",
            "-i",
            "/tmp/mix.wav",
            "-map",
            "0:v",
            "-map",
            "1:a",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-video_track_timescale",
            "30000",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
            "/tmp/out.partial",
        ])
    );
}

#[test]
fn segment_frames_follow_the_output_frame_grid_so_cuts_never_drift() {
    let rate = FrameRate {
        numerator: 30000,
        denominator: 1001,
    };
    let plan = VideoExportPlan {
        format: VideoFormat {
            frame_rate: rate,
            ..FULL_HD
        },
        length: 7 * 12_345,
        inputs: vec![standalone("a").unwrap()],
        segments: (0..7)
            .map(|index| VideoSegment::Clip {
                input: 0,
                from: index * 50_000,
                to: index * 50_000 + 12_345,
            })
            .collect(),
    };
    let segments: Vec<_> = batches(&plan).into_iter().flatten().collect();
    let mut next = 0;
    for segment in &segments {
        assert_eq!(segment.first_frame, next);
        next += segment.frames;
    }
    assert_eq!(next, rate.frames_at(plan.length));
    assert_eq!(rate.frames_at(SAMPLE_RATE as u64 * 1001), 30000);
}

#[test]
fn long_timelines_are_encoded_in_small_batches_with_short_command_lines() {
    let long_path = PathBuf::from(format!("/{}/project.voximov", "folder name ".repeat(80)));
    let segments: Vec<_> = (0..1000)
        .flat_map(|index| {
            [
                VideoSegment::Clip {
                    input: index % 3,
                    from: index as u64 * 4000,
                    to: index as u64 * 4000 + 1600,
                },
                VideoSegment::Gap { length: 1600 },
            ]
        })
        .collect();
    let plan = VideoExportPlan {
        format: FULL_HD,
        length: 2000 * 1600,
        inputs: (0..3)
            .map(|index| VideoInput {
                path: long_path.clone(),
                base: index * 1_000_000 + 1,
                length: 1_000_000,
            })
            .collect(),
        segments,
    };
    let batches = batches(&plan);
    assert_eq!(batches.len(), 2000 / SEGMENTS_PER_BATCH);
    let mut frames = 0;
    for batch in &batches {
        assert!(batch.len() <= SEGMENTS_PER_BATCH);
        let arguments = batch_arguments(&plan, batch, Path::new("/tmp/batch.h264"));
        let inputs = arguments
            .iter()
            .filter(|argument| *argument == "-i")
            .count();
        assert!(inputs <= SEGMENTS_PER_BATCH);
        let bytes: usize = arguments.iter().map(|argument| argument.len() + 1).sum();
        assert!(bytes < 64 * 1024, "{bytes}");
        frames += batch.iter().map(|segment| segment.frames).sum::<u64>();
    }
    assert_eq!(frames, FULL_HD.frame_rate.frames_at(plan.length));
}

#[test]
fn bitrate_scales_with_the_output_size() {
    assert_eq!(bitrate(&FULL_HD), 8000);
    let size = |width, height| VideoFormat {
        width,
        height,
        ..FULL_HD
    };
    assert_eq!(bitrate(&size(3840, 2160)), 32000);
    assert_eq!(bitrate(&size(1280, 720)), 3555);
    assert_eq!(bitrate(&size(320, 240)), 1000);
}

#[test]
fn probe_output_gives_an_even_display_size_and_a_standard_frame_rate() -> Result<()> {
    let rotated = r#"{"streams":[{"width":1080,"height":1921,"avg_frame_rate":"2997/100","r_frame_rate":"30000/1001","side_data_list":[{"rotation":-90}]}]}"#;
    assert_eq!(
        ffmpeg::parse_probe(rotated)?,
        VideoFormat {
            width: 1920,
            height: 1080,
            frame_rate: FrameRate {
                numerator: 30000,
                denominator: 1001
            }
        }
    );
    let unknown_nominal =
        r#"{"streams":[{"width":641,"height":480,"avg_frame_rate":"25/1","r_frame_rate":"0/0"}]}"#;
    assert_eq!(
        ffmpeg::parse_probe(unknown_nominal)?,
        VideoFormat {
            width: 640,
            height: 480,
            frame_rate: FrameRate {
                numerator: 25,
                denominator: 1
            }
        }
    );
    let screen_recording = r#"{"streams":[{"width":640,"height":480,"avg_frame_rate":"1253/97","r_frame_rate":"60/1"}]}"#;
    assert_eq!(
        ffmpeg::parse_probe(screen_recording)?.frame_rate.numerator,
        60
    );
    let timebase_rate = r#"{"streams":[{"width":640,"height":480,"avg_frame_rate":"8999/300","r_frame_rate":"90000/1"}]}"#;
    assert_eq!(
        ffmpeg::parse_probe(timebase_rate)?.frame_rate,
        FrameRate {
            numerator: 30,
            denominator: 1
        }
    );
    for invalid in [
        r#"{"streams":[]}"#,
        r#"{"streams":[{"width":0,"height":480,"avg_frame_rate":"25/1"}]}"#,
        r#"{"streams":[{"width":640,"height":480,"avg_frame_rate":"0/0","r_frame_rate":"0/0"}]}"#,
    ] {
        assert_eq!(
            ffmpeg::parse_probe(invalid).unwrap_err().to_string(),
            "[[error.videoFormat]]"
        );
    }
    Ok(())
}

#[test]
fn frame_rates_snap_to_standard_rates_with_exact_track_timescales() {
    let rate = |frames_per_second| ffmpeg::standard_rate(frames_per_second);
    assert_eq!(rate(29.97).to_string(), "30000/1001");
    assert_eq!(rate(1. / 3.).to_string(), "10/1");
    assert_eq!(rate(500.).to_string(), "120/1");
    assert_eq!(rate(24.).to_string(), "24/1");
    assert_eq!(rate(59.94).to_string(), "60000/1001");
    assert_eq!(rate(25.).track_timescale(), 10_000);
    assert_eq!(rate(29.97).track_timescale(), 30_000);
    assert_eq!(rate(24.).track_timescale(), 10_008);
}

#[test]
fn ffmpeg_lookup_prefers_the_override_then_the_bundled_sidecar_then_the_path() {
    let bundle = Path::new("/Applications/Voximov.app/Contents/MacOS");
    let everywhere = |_: &Path| true;
    let nowhere = |_: &Path| false;
    assert_eq!(
        ffmpeg::resolve(
            Tool::Ffmpeg,
            Some("/opt/ffmpeg".into()),
            Some(bundle),
            everywhere
        ),
        PathBuf::from("/opt/ffmpeg")
    );
    assert_eq!(
        ffmpeg::resolve(Tool::Ffprobe, Some("".into()), Some(bundle), everywhere),
        bundle.join("ffprobe")
    );
    assert_eq!(
        ffmpeg::resolve(Tool::Ffmpeg, None, Some(bundle), nowhere),
        PathBuf::from("ffmpeg")
    );
    assert_eq!(
        ffmpeg::resolve(Tool::Ffprobe, None, None, everywhere),
        PathBuf::from("ffprobe")
    );
    assert_eq!(Tool::Ffmpeg.override_variable(), "VOXIMOV_FFMPEG");
    assert_eq!(Tool::Ffprobe.override_variable(), "VOXIMOV_FFPROBE");
}

#[test]
fn progress_lines_report_encoded_time() {
    assert_eq!(
        ffmpeg::progress_fraction("out_time_us=2500000", 10_000_000),
        Some(0.25)
    );
    assert_eq!(
        ffmpeg::progress_fraction("out_time_us=N/A", 10_000_000),
        None
    );
    assert_eq!(ffmpeg::progress_fraction("frame=12", 10_000_000), None);
    assert_eq!(
        ffmpeg::progress_fraction("out_time_us=20000000", 10_000_000),
        Some(1.)
    );
}

#[cfg(unix)]
#[test]
fn a_failing_tool_reports_its_last_error_line() {
    let arguments: Vec<OsString> = [
        "-c",
        "echo 'first problem' >&2; echo '  input.mp4: Invalid data found  ' >&2; echo >&2; exit 3",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    let error = ffmpeg::run(
        Path::new("/bin/sh"),
        &arguments,
        1,
        &Progress::default(),
        &Cancellation::default(),
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        crate::i18n::formatted_message(
            "error.videoExport",
            &[("detail", "input.mp4: Invalid data found".into())]
        )
    );
    let missing = ffmpeg::run(
        Path::new("/nonexistent/ffmpeg"),
        &[],
        1,
        &Progress::default(),
        &Cancellation::default(),
    )
    .unwrap_err();
    assert_eq!(missing.to_string(), "[[error.ffmpegMissing]]");
}

#[cfg(unix)]
fn cancel_after(cancellation: &Cancellation, delay: Duration) {
    let cancellation = cancellation.clone();
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        cancellation.request();
    });
}

#[cfg(unix)]
#[test]
fn cancelling_stops_a_running_tool_without_waiting_for_it() {
    let cancellation = Cancellation::default();
    let _operation = cancellation.begin();
    cancel_after(&cancellation, Duration::from_millis(200));
    let started = Instant::now();
    let error = ffmpeg::run(
        Path::new("/bin/sleep"),
        &[OsString::from("30")],
        1,
        &Progress::default(),
        &cancellation,
    )
    .unwrap_err();
    assert!(error.is::<Cancelled>());
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[cfg(unix)]
#[test]
fn a_stalled_probe_can_be_cancelled_and_times_out() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir()?;
    let stalled = directory.path().join("ffprobe");
    std::fs::write(&stalled, "#!/bin/sh\nsleep 30\n")?;
    std::fs::set_permissions(&stalled, std::fs::Permissions::from_mode(0o755))?;
    let cancellation = Cancellation::default();
    let _operation = cancellation.begin();
    cancel_after(&cancellation, Duration::from_millis(200));
    let started = Instant::now();
    let error =
        ffmpeg::probe(&stalled, std::ffi::OsStr::new("input.mp4"), &cancellation).unwrap_err();
    assert!(error.is::<Cancelled>());
    assert!(started.elapsed() < Duration::from_secs(5));
    let idle = Cancellation::default();
    let error = ffmpeg::supervise(
        Path::new("/bin/sleep"),
        &["30"],
        &idle,
        Some(Duration::from_millis(200)),
        |_| {},
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "[[error.toolTimeout]]");
    Ok(())
}

#[test]
fn cancel_requests_only_reach_a_running_export() {
    let cancellation = Cancellation::default();
    cancellation.request();
    assert!(!cancellation.requested());
    let operation = cancellation.begin();
    cancellation.request();
    assert!(cancellation.requested());
    assert!(cancellation.check().unwrap_err().is::<Cancelled>());
    drop(operation);
    assert!(!cancellation.requested());
    cancellation.request();
    assert!(cancellation.check().is_ok());
}

#[test]
fn loudness_analysis_stops_when_the_export_is_cancelled() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut store = Store::open(
        directory.path().join("config"),
        Some(directory.path().join("work")),
    )?;
    let voice = asset("audio", 10.);
    std::fs::write(store.pcm(&voice.id), vec![0u8; 10 * RATE as usize * 8])?;
    store.project.tracks[1]
        .clips
        .push(clip(&voice, 0., 0., 10.));
    store.project.assets.push(voice);
    assert!(store.project.tracks[1].effects.normalize && !store.project.tracks[1].fx_bypass);
    let operation = store.cancellation.begin();
    store.cancellation.request();
    let error = Mixer::with_selection(&store, None, None).err().unwrap();
    assert!(error.is::<Cancelled>());
    drop(operation);
    assert!(Mixer::with_selection(&store, None, None).is_ok());
    Ok(())
}
