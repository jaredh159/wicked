use crate::internal::*;

pub mod images;
mod words;

// reqwest docs: https://docs.rs/reqwest/0.10.7/reqwest/
// async process: https://docs.rs/async-process/latest/async_process/struct.Command.html

#[derive(Debug)]
pub struct TestResult {
  pub is_porn: bool,
  pub prefix: String,
  pub word_score: usize,
  pub num_total_images: usize,
  pub num_images_tested: usize,
}

impl TestResult {
  pub fn new(prefix: &str) -> Self {
    Self {
      is_porn: false,
      prefix: prefix.to_string(),
      word_score: 0,
      num_total_images: 0,
      num_images_tested: 0,
    }
  }
}

#[derive(Debug)]
pub enum DomainResult {
  Unreachable,
  Parked,
  Tested(TestResult),
}

pub async fn domain(domain: &str, conf: &Config, http: &HttpClient) -> DomainResult {
  let prefixes = ["https://www.", "https://", "http://www.", "http://"];
  for prefix in &prefixes {
    match domain_impl(domain, prefix, conf, http).await {
      DomainResult::Unreachable => continue,
      DomainResult::Parked => return DomainResult::Parked,
      DomainResult::Tested(result) => return DomainResult::Tested(result),
    }
  }
  DomainResult::Unreachable
}

pub async fn domain_impl(
  domain: &str,
  prefix: &str,
  words: &Config,
  http: &HttpClient,
) -> DomainResult {
  let url = format!("{prefix}{domain}");

  let Ok(response) = http.get(&url).send().await else {
    log::trace!("GET failed `{url}`: http error - UNREACHABLE");
    return DomainResult::Unreachable;
  };

  if !response.status().is_success() {
    log::trace!(
      "GET failed `{url}`: status={} - UNREACHABLE",
      response.status()
    );
    return DomainResult::Unreachable;
  }

  let Ok(body) = response.text().await else {
    log::error!("GET  fail`{url}`: error getting response text - UNREACHABLE");
    return DomainResult::Unreachable;
  };

  log::trace!("GET success `{url}` body length={}", body.len());
  let content = html::content(&body);

  let mut result = TestResult::new(prefix);
  result.word_score = words::check(&content, words);
  if result.word_score > 250 {
    log::info!("site {url} found to be PORN by WORDS check");
    result.is_porn = true;
    return DomainResult::Tested(result);
  }
  images::check(&url, &content, http, &mut result).await;
  log::trace!("finished checking {url}, porn={}", result.is_porn);
  DomainResult::Tested(result)
}

impl fmt::Display for DomainResult {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Unreachable => write!(f, "Unreachable"),
      Self::Parked => write!(f, "Parked"),
      Self::Tested(result) => write!(f, "Successfully tested:\n  {:?}", result),
    }
  }
}
