use crate::internal::*;

pub mod images;
mod parked;
mod words;

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

pub async fn domain(domain: &str, conf: &Config) -> DomainResult {
  let client = http::client(redirect::Policy::none(), 4);
  let prefixes = ["https://www.", "https://", "http://www.", "http://"];
  for prefix in &prefixes {
    match domain_impl(domain, prefix, conf, &client).await {
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
  config: &Config,
  http: &HttpClient,
) -> DomainResult {
  let url = format!("{prefix}{domain}");
  log::trace!("begin checking url: {url}");

  let response = match http.get(&url).send().await {
    Ok(response) => response,
    Err(err) => {
      log::trace!("http error - UNREACHABLE - {err}");
      return DomainResult::Unreachable;
    }
  };

  if !response.status().is_success() {
    log::trace!(
      "GET failed `{url}`: status={} - UNREACHABLE",
      response.status()
    );
    return DomainResult::Unreachable;
  }

  let body = match response.text().await {
    Ok(body) => body,
    Err(err) => {
      log::debug!("GET fail `{url}`: no response text - UNREACHABLE, err={err}");
      return DomainResult::Unreachable;
    }
  };

  if parked::check(domain, &body) {
    log::debug!("found PARKED: site: {url}");
    return DomainResult::Parked;
  }

  log::trace!("GET success `{url}` body length={}", body.len());
  let mut result = TestResult::new(prefix);
  let content = html::content(&body);
  result.word_score = words::check(&content, config);

  if result.word_score > config.word_score_threshold {
    log::error!("site {url} found to be PORN by WORDS check");
    result.is_porn = true;
    return DomainResult::Tested(result);
  }

  images::check(&url, &content, config, &mut result).await;
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
