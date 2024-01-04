use std::fmt;

use tokio_postgres::Client;

pub struct DomainResult {
  pub is_porn: bool,
}

pub async fn domain(domain: &str, client: &Client) -> DomainResult {
  DomainResult { is_porn: false }
}

impl fmt::Display for DomainResult {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "is porn: {}", self.is_porn)
  }
}
