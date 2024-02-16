use crate::internal::*;

#[derive(Debug, Clone, Default)]
pub struct Config {
  pub sample_size: u32,
  pub parallelism: u32,
  pub word_score_threshold: usize,
  pub total_num_domains: Option<u32>,
  pub title_tag_weight: usize,
  pub h1_tag_weight: usize,
  pub img_tag_alt_weight: usize,
  pub other_text_weight: usize,
  pub link_title_weight: usize,
  pub porn_image_confidence: f32,
  pub hentai_image_confidence: f32,
  pub sexy_image_confidence: f32,
  pub num_porn_images_threshold: u8,
  pub num_sexy_images_threshold: u8,
  pub database_url: String,
  pub raw_domains_filepath: String,
  pub bun_bin_path: String,
  pub image_server_startup_wait_ms: u64,
  pub words: Vec<(String, Regex, usize)>,
}

pub fn load() -> Result<Config> {
  use config::FileFormat as Format;
  let config = config::Config::builder()
    .add_source(config::File::new("config.yaml", Format::Yaml))
    .add_source(config::File::new("config.local.yaml", Format::Yaml))
    .build()?;
  Ok(config.try_deserialize::<FileConfig>()?.try_into()?)
}

// this is the struct we deserialize from .yaml
// but doesn't include the Regex objects
#[derive(serde::Deserialize)]
struct FileConfig {
  pub sample_size: u32,
  pub parallelism: u32,
  pub word_score_threshold: usize,
  pub total_num_domains: Option<u32>,
  pub title_tag_weight: usize,
  pub h1_tag_weight: usize,
  pub other_text_weight: usize,
  pub img_tag_alt_weight: usize,
  pub link_title_weight: usize,
  pub porn_image_confidence: f32,
  pub hentai_image_confidence: f32,
  pub sexy_image_confidence: f32,
  pub num_porn_images_threshold: u8,
  pub num_sexy_images_threshold: u8,
  pub database_url: String,
  pub raw_domains_filepath: String,
  pub bun_bin_path: String,
  pub image_server_startup_wait_ms: u64,
  pub words: Vec<(String, usize)>,
}

impl TryFrom<FileConfig> for Config {
  type Error = String;

  fn try_from(settings: FileConfig) -> std::result::Result<Self, String> {
    let conf = Self {
      sample_size: settings.sample_size,
      parallelism: settings.parallelism,
      word_score_threshold: settings.word_score_threshold,
      total_num_domains: settings.total_num_domains,
      title_tag_weight: settings.title_tag_weight,
      h1_tag_weight: settings.h1_tag_weight,
      img_tag_alt_weight: settings.img_tag_alt_weight,
      other_text_weight: settings.other_text_weight,
      link_title_weight: settings.link_title_weight,
      porn_image_confidence: settings.porn_image_confidence,
      hentai_image_confidence: settings.hentai_image_confidence,
      sexy_image_confidence: settings.sexy_image_confidence,
      num_porn_images_threshold: settings.num_porn_images_threshold,
      num_sexy_images_threshold: settings.num_sexy_images_threshold,
      database_url: settings.database_url,
      raw_domains_filepath: settings.raw_domains_filepath,
      bun_bin_path: settings.bun_bin_path,
      image_server_startup_wait_ms: settings.image_server_startup_wait_ms,
      words: map_regex(settings.words),
    };
    if conf.sample_size < conf.parallelism {
      return Err("Config.sample_size must be >= parallelism".to_string());
    }
    if conf.porn_image_confidence < 0.0 || conf.porn_image_confidence > 1.0 {
      return Err("Config.porn_image_confidence must be between 0.0 and 1.0".to_string());
    }
    if conf.hentai_image_confidence < 0.0 || conf.hentai_image_confidence > 1.0 {
      return Err(
        "Config.hentai_image_confidence must be between 0.0 and 1.0".to_string(),
      );
    }
    if conf.sexy_image_confidence < 0.0 || conf.sexy_image_confidence > 1.0 {
      return Err("Config.sexy_image_confidence must be between 0.0 and 1.0".to_string());
    }
    for (word, _, score) in &conf.words {
      if word.len() < 3 {
        return Err("each Config.words.* must be at least 3 characters".to_string());
      }
      if *score < 1 || *score > 10 {
        return Err("each Config.words. score must be between 1 and 10".to_string());
      }
    }
    Ok(conf)
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
