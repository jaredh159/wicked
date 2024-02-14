use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Config {
  pub title_tag_multiplier: usize,
  pub h1_tag_multiplier: usize,
  pub img_tag_alt_multiplier: usize,
  pub other_text_multiplier: usize,
  pub link_title_multiplier: usize,
  pub words: Vec<(String, Regex, usize)>,
}

#[derive(Deserialize)]
struct ConfigFile {
  pub title_tag_multiplier: usize,
  pub h1_tag_multiplier: usize,
  pub other_text_multiplier: usize,
  pub img_tag_alt_multiplier: usize,
  pub link_title_multiplier: usize,
  pub words: Vec<(String, usize)>,
}

pub fn load() -> Config {
  let file_contents =
    std::fs::read_to_string("words.json").expect("unable to read `words.json`");
  let file: ConfigFile =
    serde_json::from_str(&file_contents).expect("unable to deserialize `words.json`");
  Config {
    title_tag_multiplier: file.title_tag_multiplier,
    h1_tag_multiplier: file.h1_tag_multiplier,
    other_text_multiplier: file.other_text_multiplier,
    img_tag_alt_multiplier: file.img_tag_alt_multiplier,
    link_title_multiplier: file.link_title_multiplier,
    words: map_regex(file.words),
  }
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
