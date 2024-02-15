use crate::internal::*;

pub fn client(redirect_policy: redirect::Policy, timeout_secs: u64) -> HttpClient {
  reqwest::Client::builder()
    .user_agent(USER_AGENT)
    .redirect(redirect_policy)
    .timeout(Duration::from_secs(timeout_secs))
    .build()
    .unwrap()
}

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
