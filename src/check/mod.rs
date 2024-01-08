use crate::internal::*;

mod words;

// reqwest docs: https://docs.rs/reqwest/0.10.7/reqwest/
// async process: https://docs.rs/async-process/latest/async_process/struct.Command.html

#[derive(Debug)]
pub struct TestResult {
  pub is_porn: bool,
  pub prefix: String,
  pub word_score: u32,
  pub num_total_images: u32,
  pub num_images_tested: u32,
  pub image_scores: Vec<u32>,
}

#[derive(Debug)]
pub enum DomainResult {
  Unreachable,
  Parked,
  Tested(TestResult),
}

pub async fn domain(domain: &str, http: &HttpClient) -> DomainResult {
  let prefixes = ["https://www.", "https://", "http://www.", "http://"];
  for prefix in &prefixes {
    match domain_impl(domain, prefix, http).await {
      DomainResult::Unreachable => continue,
      DomainResult::Parked => return DomainResult::Parked,
      DomainResult::Tested(result) => return DomainResult::Tested(result),
    }
  }
  DomainResult::Unreachable
}

pub async fn domain_impl(domain: &str, prefix: &str, http: &HttpClient) -> DomainResult {
  let url = format!("{prefix}{domain}");

  let Ok(response) = http.get(&url).send().await else {
    println!("Http request to `{url}` failed with error");
    return DomainResult::Unreachable;
  };

  if !response.status().is_success() {
    println!(
      "Http request failed to `{url}` failed w/ status code {}",
      response.status()
    );
    return DomainResult::Unreachable;
  }

  let Ok(body) = response.text().await else {
    println!("Http request failed to `{url}` failed to parse body into text");
    return DomainResult::Unreachable;
  };

  let content = html::content(&body);

  DomainResult::Tested(TestResult {
    is_porn: false,
    prefix: prefix.to_string(),
    word_score: 0,
    num_total_images: 0,
    num_images_tested: 0,
    image_scores: vec![],
  })
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
