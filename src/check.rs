use crate::internal::*;

// reqwest docs: https://docs.rs/reqwest/0.10.7/reqwest/
// async process: https://docs.rs/async-process/latest/async_process/struct.Command.html

pub struct DomainResult {
  pub is_porn: bool,
}

// todo: request no follow redirects
// todo: request add short timeout

pub async fn domain(domain: &str, client: &Client) -> Result<DomainResult> {
  let url = format!("https://{domain}");
  let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
  let response = client.get(&url).send().await?;
  if !response.status().is_success() {
    println!("response: {}", response.text().await?);
    panic!("nope");
  }

  let body = response.text().await?;
  let content = html::dom::content(&body);
  println!("content: {:#?}", content);

  Ok(DomainResult { is_porn: false })
}

impl fmt::Display for DomainResult {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "is porn: {}", self.is_porn)
  }
}

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
