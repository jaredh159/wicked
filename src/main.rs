mod bootstrap;
mod check;
mod db;
mod exec;
mod html;
mod internal;
mod stream;
mod utils;

use internal::*;

#[tokio::main]
async fn main() -> Result<()> {
  dotenv::dotenv().ok();

  let mut args = std::env::args().skip(1);
  let cmd = args
    .next()
    .expect("missing required [cmd] arg (bootstrap|exec|check-domain)");

  let db_client = db::connect().await?;

  let http_client = reqwest::Client::builder()
    .user_agent(USER_AGENT)
    .redirect(reqwest::redirect::Policy::none())
    .timeout(Duration::from_secs(3))
    .build()
    .unwrap();

  match cmd.as_str() {
    "bootstrap" => bootstrap::run(&db_client).await?,
    "exec" => exec::run(Arc::new(Mutex::new(db_client))).await?,
    "check-domain" => {
      let domain = args.next().expect("missing required [domain] arg");
      let result = check::domain(&domain, &http_client).await;
      println!("\nresult: {result}");
    }
    _ => panic!("unknown command: `{cmd}`"),
  }
  Ok(())
}

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
