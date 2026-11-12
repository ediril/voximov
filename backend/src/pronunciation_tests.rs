use crate::{
    kokoro::{self, MAX_PHONEMES},
    pronunciation::{BUILT_IN, Pronouncer, guess, parse_entries, respell, sentence_chunks},
};
use std::collections::HashMap;

fn vocabulary() -> HashMap<char, i64> {
    kokoro::vocabulary(include_str!("../../tests/fixtures/kokoro-tokenizer.json")).unwrap()
}

fn unspeakable(phonemes: &str) -> String {
    let vocabulary = vocabulary();
    phonemes
        .chars()
        .filter(|symbol| !vocabulary.contains_key(symbol))
        .collect()
}

#[test]
fn sounds_like_spellings_become_exact_phonemes() {
    assert_eq!(respell("VOX-ih-mov").as_deref(), Some("vˈɑksɪmɑv"));
    assert_eq!(respell("koo-ber-NET-eez").as_deref(), Some("kubəɹnˈɛtiz"));
    assert_eq!(respell("TOW-ree").as_deref(), Some("tˈWɹi"));
    assert_eq!(respell("vox ih mov").as_deref(), Some("vˈɑksɪmɑv"));
    assert_eq!(respell("9000"), None);
    assert_eq!(respell(""), None);
}

#[test]
fn every_built_in_word_parses_into_speakable_phonemes() {
    let lines = BUILT_IN
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let entries = parse_entries(BUILT_IN);
    assert_eq!(entries.len(), lines, "a built-in line failed to parse");
    for (word, phonemes) in entries {
        assert_eq!(
            unspeakable(&phonemes),
            "",
            "{word} has unspeakable phonemes"
        );
    }
}

#[test]
fn words_missing_from_the_dictionary_are_spoken_and_reported() {
    let mut pronouncer = Pronouncer::new();
    let result = pronouncer
        .pronounce(
            "Record a voiceover at the playhead, then ship it to Zorblax with gh.",
            false,
        )
        .unwrap();
    assert_eq!(result.guessed, ["Zorblax", "gh", "playhead", "voiceover"]);
    assert!(result.phonemes.contains("vˈYsˌOvəɹ"));
    assert!(result.phonemes.contains("plˈAhˌɛd"));
    assert!(!result.phonemes.contains('❓'));
    assert_eq!(unspeakable(&result.phonemes), "");
}

#[test]
fn built_in_words_override_the_dictionary_without_being_reported() {
    let mut pronouncer = Pronouncer::new();
    let result = pronouncer
        .pronounce("Voximov exports JSON for Kubernetes.", false)
        .unwrap();
    assert_eq!(result.guessed, Vec::<String>::new());
    assert!(result.phonemes.starts_with("vˈɑksɪmɑv"));
    assert!(result.phonemes.contains("ʤˈAsən"));
    assert!(result.phonemes.contains("kubəɹnˈɛtiz"));
}

#[test]
fn inline_pronunciations_are_kept_as_written() {
    let mut pronouncer = Pronouncer::new();
    let result = pronouncer
        .pronounce(
            "Say [Zorblax](/zˈɔɹbləks/) and [JSON](/ʤˈAsən/) twice.",
            false,
        )
        .unwrap();
    assert!(result.phonemes.contains("zˈɔɹbləks"));
    assert!(result.phonemes.contains("ʤˈAsən"));
    assert_eq!(result.guessed, Vec::<String>::new());
}

#[test]
fn british_voices_use_the_british_dictionary() {
    let mut pronouncer = Pronouncer::new();
    let american = pronouncer.pronounce("tomato", false).unwrap();
    let british = pronouncer.pronounce("tomato", true).unwrap();
    assert_ne!(american.phonemes, british.phonemes);
}

#[test]
fn guesses_use_only_symbols_the_model_can_say() {
    for word in [
        "Zorblax",
        "Quintara",
        "Tauri",
        "xyzzy",
        "Brightwhistle",
        "Yonder",
    ] {
        let phonemes = guess(word).unwrap();
        assert_eq!(unspeakable(&phonemes), "", "{word} guessed as {phonemes}");
    }
    assert_eq!(guess("123"), None);
}

#[test]
fn speech_is_split_into_sentences_that_fit_the_model() {
    assert_eq!(
        sentence_chunks("hˈI. hˌW ɑɹ ju? ɡˈʊd!", MAX_PHONEMES),
        ["hˈI.", "hˌW ɑɹ ju?", "ɡˈʊd!"]
    );
    let long = vec!["wˈʌn tˈu θɹˈi"; 80].join(", ");
    let chunks = sentence_chunks(&long, MAX_PHONEMES);
    assert!(chunks.len() > 1);
    assert!(
        chunks
            .iter()
            .all(|chunk| chunk.chars().count() <= MAX_PHONEMES)
    );
    assert_eq!(
        chunks.join(" ").split_whitespace().count(),
        long.split_whitespace().count()
    );
}
