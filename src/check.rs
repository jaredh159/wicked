use std::time::Duration;

use crate::internal::*;

// reqwest docs: https://docs.rs/reqwest/0.10.7/reqwest/
// async process: https://docs.rs/async-process/latest/async_process/struct.Command.html

// unreachable
// parked
// tested (subdomain, word score, image score, is_porn)

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

pub async fn domain(domain: &str, client: &Client) -> DomainResult {
  let prefixes = ["https://www.", "https://", "http://www.", "http://"];
  for prefix in &prefixes {
    match domain_impl(domain, client, prefix).await {
      DomainResult::Unreachable => continue,
      DomainResult::Parked => return DomainResult::Parked,
      DomainResult::Tested(result) => return DomainResult::Tested(result),
    }
  }
  DomainResult::Unreachable
}

pub async fn domain_impl(domain: &str, client: &Client, prefix: &str) -> DomainResult {
  let url = format!("{prefix}{domain}");
  let client = reqwest::Client::builder()
    .user_agent(USER_AGENT)
    .redirect(reqwest::redirect::Policy::none())
    .timeout(Duration::from_secs(3))
    .build()
    .unwrap();

  let Ok(response) = client.get(&url).send().await else {
    return DomainResult::Unreachable;
  };

  if !response.status().is_success() {
    println!("Request failed: {url}");
    return DomainResult::Unreachable;
  }

  let Ok(body) = response.text().await else {
    return DomainResult::Unreachable;
  };

  let content = html::dom::content(&body);
  // // println!("content: {:#?}", content);

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

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
