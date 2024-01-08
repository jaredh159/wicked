use crate::internal::*;

pub struct Config {
  pub title_tag_multiplier: usize,
  pub h1_tag_multiplier: usize,
  pub other_text_multiplier: usize,
  pub words: Vec<(String, Regex, usize)>,
}

pub fn map_regex(input: Vec<(String, usize)>) -> Vec<(String, Regex, usize)> {
  input
    .into_iter()
    .map(|(word, n)| {
      (
        word.clone(),
        regex::RegexBuilder::new(&format!(r"\b{}\b", &word))
          .case_insensitive(true)
          .build()
          .unwrap_or_else(|_| panic!("unable to create regex from word: `{}`", word)),
        n,
      )
    })
    .collect()
}

#[derive(serde::Deserialize)]
pub struct ConfigFile {
  pub title_tag_multiplier: usize,
  pub h1_tag_multiplier: usize,
  pub other_text_multiplier: usize,
  pub words: Vec<(String, usize)>,
}

pub fn load() -> Config {
  let file_contents =
    std::fs::read_to_string("words.json").expect("unable to read `words.json`");
  let file: ConfigFile =
    serde_json::from_str(&file_contents).expect("unable to deserialize `words.json`");
  assert!(
    file.h1_tag_multiplier <= 10,
    "title_tag_multiplier must be between 0 and 10"
  );
  assert!(
    file.h1_tag_multiplier <= 10,
    "h1_tag_multiplier must be between 0 and 10"
  );
  assert!(
    file.other_text_multiplier <= 10,
    "other_text_multiplier must be between 0 and 10"
  );
  Config {
    title_tag_multiplier: file.title_tag_multiplier,
    h1_tag_multiplier: file.h1_tag_multiplier,
    other_text_multiplier: file.other_text_multiplier,
    words: map_regex(file.words),
  }
}

// base64 encode it
