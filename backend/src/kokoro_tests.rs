use crate::kokoro::{SAMPLE_RATE, join};

fn padded_speech(speech_seconds: f32) -> Vec<f32> {
    let frames = |seconds: f32| (seconds * SAMPLE_RATE as f32) as usize;
    let mut samples = vec![0.; frames(0.31)];
    samples.extend((0..frames(speech_seconds)).map(|frame| (frame as f32 * 0.05).sin() * 0.4));
    samples.resize(samples.len() + frames(0.43), 0.);
    samples
}

fn silences(samples: &[f32]) -> (f32, f32, Vec<f32>) {
    let seconds = |frames: usize| frames as f32 / SAMPLE_RATE as f32;
    let audible = |sample: &f32| sample.abs() > 0.003;
    let lead = samples.iter().position(audible).unwrap();
    let tail = samples.len() - 1 - samples.iter().rposition(audible).unwrap();
    let mut gaps = vec![];
    let mut quiet = 0;
    for sample in &samples[lead..samples.len() - tail] {
        if audible(sample) {
            if quiet > SAMPLE_RATE as usize / 10 {
                gaps.push(seconds(quiet));
            }
            quiet = 0;
        } else {
            quiet += 1;
        }
    }
    (seconds(lead), seconds(tail), gaps)
}

#[test]
fn speech_starts_and_ends_promptly_with_short_pauses_between_sentences() {
    let spoken = [
        ("hˈɛloʊ.", padded_speech(0.6)),
        ("ðɪs ɪz ɐ lˈɔŋ fɹˈAz,", padded_speech(0.8)),
        ("ðˈɛn ɐnˈʌðəɹ.", padded_speech(0.5)),
        ("ðə ˈɛnd!", padded_speech(0.4)),
    ];
    let (lead, tail, gaps) = silences(&join(&spoken, 1.));
    assert!(lead <= 0.025, "lead {lead}");
    assert!(tail <= 0.055, "tail {tail}");
    assert_eq!(gaps.len(), 3);
    assert!((0.35..0.4).contains(&gaps[0]), "after a sentence {gaps:?}");
    assert!((0.2..0.25).contains(&gaps[1]), "after a phrase {gaps:?}");
    let (_, _, faster) = silences(&join(&spoken, 1.5));
    assert!(faster[0] < gaps[0] - 0.08, "{faster:?}");
}

#[test]
fn silent_chunks_add_no_pause() {
    let spoken = [
        ("wˈʌn.", padded_speech(0.5)),
        ("…", vec![0.; SAMPLE_RATE as usize]),
        ("tˈu.", padded_speech(0.5)),
    ];
    let (_, _, gaps) = silences(&join(&spoken, 1.));
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0] < 0.4, "{gaps:?}");
}
