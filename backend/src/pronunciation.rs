use anyhow::{Result, anyhow};
use misaki_rs::{G2P, Language, MToken, lexicon::PhonemeEntry};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

const UNKNOWN: char = '❓';
const PRIMARY: char = 'ˈ';
const SECONDARY: char = 'ˌ';
pub const BUILT_IN: &str = include_str!("pronunciations.txt");
static LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]*)\)").expect("link pattern"));

#[derive(Debug, PartialEq)]
pub struct Pronounced {
    pub phonemes: String,
    pub guessed: Vec<String>,
}

pub struct Pronouncer {
    american: G2P,
    british: Option<G2P>,
}

impl Pronouncer {
    pub fn new() -> Self {
        Self {
            american: with_built_ins(G2P::new(Language::EnglishUS)),
            british: None,
        }
    }

    pub fn pronounce(&mut self, text: &str, british: bool) -> Result<Pronounced> {
        let g2p = if british {
            self.british
                .get_or_insert_with(|| with_built_ins(G2P::new(Language::EnglishGB)))
        } else {
            &mut self.american
        };
        pronounce(g2p, text)
    }
}

fn with_built_ins(mut g2p: G2P) -> G2P {
    let mut words = BTreeMap::new();
    for (word, phonemes) in parse_entries(BUILT_IN) {
        add(&mut words, &word, phonemes);
    }
    for (word, phonemes) in words {
        g2p.lexicon
            .golds
            .insert(word, PhonemeEntry::Simple(phonemes));
    }
    g2p
}

fn phonemize(
    g2p: &mut G2P,
    words: &BTreeMap<String, String>,
    text: &str,
) -> Result<(String, Vec<MToken>)> {
    let replaced: Vec<(String, Option<PhonemeEntry>)> = words
        .iter()
        .map(|(word, phonemes)| {
            let entry = PhonemeEntry::Simple(phonemes.clone());
            (word.clone(), g2p.lexicon.golds.insert(word.clone(), entry))
        })
        .collect();
    let result = g2p.g2p(text).map_err(|error| anyhow!("{error}"));
    for (word, previous) in replaced {
        match previous {
            Some(entry) => g2p.lexicon.golds.insert(word, entry),
            None => g2p.lexicon.golds.remove(&word),
        };
    }
    result
}

fn without_brackets(text: &str) -> String {
    text.replace(['[', ']'], "")
}

fn inline_pronunciations(text: &str) -> (String, BTreeMap<String, String>) {
    let mut output = String::with_capacity(text.len());
    let mut words = BTreeMap::new();
    let mut copied = 0;
    for link in LINK.captures_iter(text) {
        let whole = link.get(0).map_or(0..0, |whole| whole.range());
        output.push_str(&without_brackets(&text[copied..whole.start]));
        let word = &link[1];
        match link[2]
            .strip_prefix('/')
            .and_then(|rest| rest.strip_suffix('/'))
        {
            Some(phonemes) if !word.contains(char::is_whitespace) => {
                output.push_str(word);
                add(&mut words, word, phonemes.to_owned());
            }
            _ => output.push_str(&text[whole.clone()]),
        }
        copied = whole.end;
    }
    output.push_str(&without_brackets(&text[copied..]));
    (output, words)
}

const CONTRACTIONS: [(&str, &str); 3] = [("ll", "əl"), ("d", "d"), ("ve", "v")];

fn unresolved(tokens: &[MToken]) -> BTreeSet<String> {
    tokens
        .iter()
        .filter(|token| {
            token
                .phonemes
                .as_deref()
                .is_none_or(|phonemes| phonemes.contains(UNKNOWN))
        })
        .map(|token| token.text.clone())
        .collect()
}

fn standalone(g2p: &G2P, word: &str) -> Option<String> {
    let (phonemes, _) = g2p.g2p(word).ok()?;
    let phonemes = phonemes.trim();
    (!phonemes.is_empty() && !phonemes.contains(UNKNOWN)).then(|| phonemes.to_owned())
}

fn add(words: &mut BTreeMap<String, String>, word: &str, phonemes: String) {
    words.insert(word.to_owned(), phonemes.clone());
    words.insert(word.to_lowercase(), phonemes);
}

fn pronounce(g2p: &mut G2P, text: &str) -> Result<Pronounced> {
    let (text, mut words) = inline_pronunciations(text);
    let (phonemes, tokens) = phonemize(g2p, &words, &text)?;
    let unresolved = unresolved(&tokens);
    if unresolved.is_empty() {
        return Ok(Pronounced {
            phonemes: phonemes.trim().to_owned(),
            guessed: vec![],
        });
    }
    let mut guessed = BTreeSet::new();
    let mut say = |word: &str| {
        let folded = fold(word);
        standalone(g2p, &folded).or_else(|| {
            guessed.insert(word.to_owned());
            compound(g2p, &folded)
                .or_else(|| acronym(g2p, &folded))
                .or_else(|| guess(&folded))
                .or_else(|| spell(g2p, &folded))
        })
    };
    for token in &unresolved {
        let (stem, suffix) = token.split_once(['\'', '’']).unwrap_or((token, ""));
        let contraction = CONTRACTIONS
            .iter()
            .find(|(written, _)| suffix.eq_ignore_ascii_case(written));
        match contraction {
            Some((_, sound)) if stem.chars().all(char::is_alphabetic) => {
                if let Some(phonemes) = say(stem) {
                    add(&mut words, token, phonemes + sound);
                }
            }
            _ => {
                for word in stem.split(|c: char| !c.is_alphabetic()) {
                    if let Some(phonemes) = (!word.is_empty()).then(|| say(word)).flatten() {
                        add(&mut words, word, phonemes);
                    }
                }
            }
        }
    }
    let (phonemes, _) = phonemize(g2p, &words, &text)?;
    Ok(Pronounced {
        phonemes: phonemes.replace(UNKNOWN, "").trim().to_owned(),
        guessed: guessed.into_iter().collect(),
    })
}

fn fold(word: &str) -> String {
    word.nfd().filter(|c| !is_combining_mark(*c)).collect()
}

fn known(g2p: &G2P, word: &str) -> Option<String> {
    g2p.lexicon
        .get_word(word, "NN", None, None)
        .map(|(phonemes, _)| phonemes)
        .filter(|phonemes| !phonemes.is_empty() && !phonemes.contains(UNKNOWN))
}

fn compound(g2p: &G2P, word: &str) -> Option<String> {
    let lower = word.to_lowercase();
    if lower.len() < 6 || !lower.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    let mut best: Vec<Option<Vec<(usize, String)>>> = vec![None; lower.len() + 1];
    best[0] = Some(vec![]);
    for end in 3..=lower.len() {
        for start in (0..=end - 3).rev() {
            let Some(before) = best[start].clone() else {
                continue;
            };
            let Some(part) = known(g2p, &lower[start..end]) else {
                continue;
            };
            let candidate = [before, vec![(end - start, part)]].concat();
            let rank = |parts: &[(usize, String)]| {
                let shortest = parts.iter().map(|(length, _)| *length).min().unwrap_or(0);
                (std::cmp::Reverse(parts.len()), shortest)
            };
            if best[end]
                .as_ref()
                .is_none_or(|current| rank(&candidate) > rank(current))
            {
                best[end] = Some(candidate);
            }
        }
    }
    let parts = best[lower.len()].take().filter(|parts| parts.len() > 1)?;
    Some(
        parts
            .iter()
            .enumerate()
            .map(|(index, (_, part))| {
                if index == 0 {
                    part.clone()
                } else {
                    part.replace(PRIMARY, &SECONDARY.to_string())
                }
            })
            .collect(),
    )
}

fn acronym(g2p: &G2P, word: &str) -> Option<String> {
    let letters = word.chars().all(|c| c.is_ascii_alphabetic());
    let capitals = word.chars().all(|c| c.is_ascii_uppercase());
    let vowelless = !word.to_lowercase().contains(['a', 'e', 'i', 'o', 'u', 'y']);
    if !letters || !((capitals && word.len() <= 6) || (vowelless && word.len() <= 5)) {
        return None;
    }
    spell(g2p, word)
}

fn spell(g2p: &G2P, word: &str) -> Option<String> {
    let spelled: Vec<String> = word
        .to_lowercase()
        .chars()
        .map(|letter| known(g2p, &letter.to_uppercase().to_string()))
        .collect::<Option<_>>()?;
    let last = spelled.len().checked_sub(1)?;
    Some(
        spelled
            .iter()
            .enumerate()
            .map(|(index, letter)| {
                if index == last {
                    letter.clone()
                } else {
                    letter.replace(PRIMARY, &SECONDARY.to_string())
                }
            })
            .collect(),
    )
}

pub fn parse_entries(source: &str) -> Vec<(String, String)> {
    source
        .lines()
        .filter_map(|line| {
            let (word, sounds) = line.split_once('=')?;
            let word = word.trim();
            (!word.is_empty() && !word.starts_with('#'))
                .then(|| Some((word.to_owned(), respell(sounds.trim())?)))?
        })
        .collect()
}

const RESPELLING: [(&str, &str, &str); 52] = [
    ("eye", "I", "I"),
    ("igh", "I", "I"),
    ("air", "ɛɹ", "ɛɹ"),
    ("are", "ɛɹ", "ɛɹ"),
    ("ear", "ɪɹ", "ɪɹ"),
    ("eer", "ɪɹ", "ɪɹ"),
    ("oor", "ʊɹ", "ʊɹ"),
    ("our", "ɔɹ", "ɔɹ"),
    ("tch", "ʧ", "ʧ"),
    ("ow", "W", "W"),
    ("oy", "Y", "Y"),
    ("oi", "Y", "Y"),
    ("ay", "A", "A"),
    ("ai", "A", "A"),
    ("ee", "i", "i"),
    ("ea", "i", "i"),
    ("oo", "u", "u"),
    ("ew", "ju", "ju"),
    ("oh", "O", "O"),
    ("oe", "O", "O"),
    ("ah", "ɑ", "ɑ"),
    ("aw", "ɔ", "ɔ"),
    ("au", "ɔ", "ɔ"),
    ("uu", "ʊ", "ʊ"),
    ("uh", "ʌ", "ə"),
    ("ar", "ɑɹ", "ɑɹ"),
    ("or", "ɔɹ", "ɔɹ"),
    ("er", "ɜɹ", "əɹ"),
    ("ur", "ɜɹ", "əɹ"),
    ("ir", "ɜɹ", "əɹ"),
    ("eh", "ɛ", "ɛ"),
    ("ih", "ɪ", "ɪ"),
    ("ch", "ʧ", "ʧ"),
    ("sh", "ʃ", "ʃ"),
    ("zh", "ʒ", "ʒ"),
    ("th", "θ", "θ"),
    ("dh", "ð", "ð"),
    ("ng", "ŋ", "ŋ"),
    ("nk", "ŋk", "ŋk"),
    ("ck", "k", "k"),
    ("ph", "f", "f"),
    ("wh", "w", "w"),
    ("a", "æ", "æ"),
    ("e", "ɛ", "ɛ"),
    ("i", "ɪ", "ɪ"),
    ("o", "ɑ", "ɑ"),
    ("u", "ʌ", "ə"),
    ("j", "ʤ", "ʤ"),
    ("g", "ɡ", "ɡ"),
    ("r", "ɹ", "ɹ"),
    ("x", "ks", "ks"),
    ("c", "k", "k"),
];

const VOWEL_SOUNDS: &str = "æɑɛɪʌʊəiuɔAIOWYɜ";

fn insert_stress(phonemes: &str, mark: char) -> String {
    match phonemes.find(|c| VOWEL_SOUNDS.contains(c)) {
        Some(index) => format!("{}{mark}{}", &phonemes[..index], &phonemes[index..]),
        None => phonemes.to_owned(),
    }
}

fn respell_syllable(syllable: &str, stressed: bool) -> Option<String> {
    let mut rest = syllable;
    let mut phonemes = String::new();
    let mut previous = None;
    while let Some(first) = rest.chars().next() {
        if previous == Some(first) && "bdfhklmnpqstvwz".contains(first) {
            rest = &rest[1..];
            continue;
        }
        previous = Some(first);
        if first == 'y' {
            let vowel_follows = rest[1..].starts_with(['a', 'e', 'i', 'o', 'u']);
            phonemes.push_str(if vowel_follows { "j" } else { "I" });
            rest = &rest[1..];
            continue;
        }
        if let Some((pattern, strong, weak)) = RESPELLING
            .iter()
            .find(|(pattern, _, _)| rest.starts_with(pattern))
        {
            phonemes.push_str(if stressed { strong } else { weak });
            rest = &rest[pattern.len()..];
            continue;
        }
        if !"bdfhklmnpqstvwz".contains(first) {
            return None;
        }
        phonemes.push(if first == 'q' { 'k' } else { first });
        rest = &rest[1..];
    }
    Some(phonemes)
}

pub fn respell(sounds: &str) -> Option<String> {
    let syllables: Vec<&str> = sounds
        .split(['-', ' '])
        .filter(|syllable| !syllable.is_empty())
        .collect();
    if syllables.is_empty() {
        return None;
    }
    let marked = |syllable: &&str| {
        syllable.chars().any(|c| c.is_ascii_uppercase())
            && !syllable.chars().any(|c| c.is_ascii_lowercase())
    };
    let any_marked = syllables.iter().any(marked);
    let mut seen_primary = false;
    let mut phonemes = String::new();
    for (index, syllable) in syllables.iter().enumerate() {
        let stressed = if any_marked {
            marked(syllable)
        } else {
            index == 0
        };
        let sound = respell_syllable(&syllable.to_lowercase(), stressed)?;
        phonemes.push_str(&if stressed {
            let mark = if seen_primary { SECONDARY } else { PRIMARY };
            seen_primary = true;
            insert_stress(&sound, mark)
        } else {
            sound
        });
    }
    Some(phonemes)
}

fn is_vowel(c: char) -> bool {
    "aeiou".contains(c)
}

const SPELLING: [(&str, &str); 44] = [
    ("tion", "ʃən"),
    ("sion", "ʒən"),
    ("ture", "ʧəɹ"),
    ("igh", "I"),
    ("tch", "ʧ"),
    ("sch", "sk"),
    ("ph", "f"),
    ("sh", "ʃ"),
    ("ch", "ʧ"),
    ("th", "θ"),
    ("ck", "k"),
    ("qu", "kw"),
    ("ng", "ŋ"),
    ("wh", "w"),
    ("gh", ""),
    ("ee", "i"),
    ("ea", "i"),
    ("ie", "i"),
    ("ei", "A"),
    ("ay", "A"),
    ("ai", "A"),
    ("oa", "O"),
    ("oo", "u"),
    ("ou", "W"),
    ("ow", "O"),
    ("oi", "Y"),
    ("oy", "Y"),
    ("au", "ɔ"),
    ("aw", "ɔ"),
    ("ew", "ju"),
    ("eu", "ju"),
    ("ue", "u"),
    ("ui", "u"),
    ("ar", "ɑɹ"),
    ("or", "ɔɹ"),
    ("er", "əɹ"),
    ("ir", "ɜɹ"),
    ("ur", "ɜɹ"),
    ("j", "ʤ"),
    ("g", "ɡ"),
    ("r", "ɹ"),
    ("x", "ks"),
    ("q", "k"),
    ("y", "ɪ"),
];

fn vowel_sound(letter: char, long: bool) -> &'static str {
    match (letter, long) {
        ('a', true) => "A",
        ('e', true) => "i",
        ('i', true) => "I",
        ('o', true) => "O",
        ('u', true) => "u",
        ('a', false) => "æ",
        ('e', false) => "ɛ",
        ('i', false) => "ɪ",
        ('o', false) => "ɑ",
        _ => "ʌ",
    }
}

pub fn guess(word: &str) -> Option<String> {
    let letters: Vec<char> = word.to_lowercase().chars().collect();
    if letters.is_empty() || !letters.iter().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    let text: String = letters.iter().collect();
    let has_vowel = letters.iter().any(|&c| is_vowel(c) || c == 'y');
    let mut phonemes = String::new();
    let mut index = 0;
    while index < letters.len() {
        let c = letters[index];
        let rest = &text[index..];
        let next = letters.get(index + 1).copied();
        let after = letters.get(index + 2).copied();
        let at_end = index + 1 == letters.len();
        if index > 0 && letters[index - 1] == c && !is_vowel(c) {
            index += 1;
            continue;
        }
        if c == 'e' && at_end && letters[..index].iter().any(|&l| is_vowel(l) || l == 'y') {
            break;
        }
        if c == 'y' {
            let sound = if index == 0 && next.is_some_and(is_vowel) {
                "j"
            } else if at_end {
                if letters[..index].iter().any(|&l| is_vowel(l)) {
                    "i"
                } else {
                    "I"
                }
            } else {
                "ɪ"
            };
            phonemes.push_str(sound);
            index += 1;
            continue;
        }
        if let Some((pattern, sound)) = SPELLING.iter().find(|(pattern, _)| {
            rest.starts_with(pattern)
                && !(pattern.len() == 2
                    && pattern.ends_with('r')
                    && letters.get(index + 2).is_some_and(|&n| is_vowel(n)))
        }) {
            phonemes.push_str(sound);
            index += pattern.len();
            continue;
        }
        if c == 'c' {
            phonemes.push(if next.is_some_and(|n| "eiy".contains(n)) {
                's'
            } else {
                'k'
            });
            index += 1;
            continue;
        }
        if is_vowel(c) {
            let magic_e = next.is_some_and(|n| !is_vowel(n))
                && after == Some('e')
                && index + 3 == letters.len();
            let open = at_end
                || (next.is_some_and(|n| !is_vowel(n) && n != 'x')
                    && after.is_some_and(|a| is_vowel(a) || a == 'y'));
            phonemes.push_str(if c == 'i' && at_end && index > 0 {
                "i"
            } else {
                vowel_sound(c, magic_e || open)
            });
            index += 1;
            continue;
        }
        phonemes.push(c);
        index += 1;
    }
    if !has_vowel {
        return None;
    }
    Some(insert_stress(&phonemes, PRIMARY))
}

pub fn sentence_chunks(phonemes: &str, limit: usize) -> Vec<String> {
    let mut chunks = vec![];
    let mut current = String::new();
    for c in phonemes.chars() {
        current.push(c);
        if ".!?".contains(c) {
            chunks.push(std::mem::take(&mut current));
        }
    }
    chunks.push(current);
    chunks
        .into_iter()
        .map(|chunk| chunk.trim().to_owned())
        .filter(|chunk| chunk.chars().any(char::is_alphabetic))
        .flat_map(|chunk| split_long(&chunk, limit))
        .collect()
}

fn split_long(chunk: &str, limit: usize) -> Vec<String> {
    if chunk.chars().count() <= limit {
        return vec![chunk.to_owned()];
    }
    let chars: Vec<char> = chunk.chars().collect();
    let window = &chars[..limit];
    let cut = window
        .iter()
        .rposition(|c| ",;:—".contains(*c))
        .or_else(|| window.iter().rposition(|c| *c == ' '))
        .map_or(limit, |index| index + 1);
    let head: String = chars[..cut].iter().collect();
    let tail: String = chars[cut..].iter().collect();
    let mut parts = vec![head.trim().to_owned()];
    parts.extend(split_long(tail.trim(), limit));
    parts
}
