mod bootstrap;
mod check;
mod config;
mod db;
mod exec;
mod html;
mod internal;
mod prereqs;
mod stream;

use internal::*;

#[tokio::main]
async fn main() -> Result<()> {
  dotenv::dotenv().ok();

  env_logger::init_from_env(Env::default().default_filter_or("info"));

  let mut args = std::env::args().skip(1);
  let cmd = args
    .next()
    .expect("missing required [cmd] arg (bootstrap|exec|check-domain)");

  let db_client = db::connect().await?;
  let http_client = build_http_client();
  let config = config::load();

  match cmd.as_str() {
    "bootstrap" => bootstrap::run(&db_client).await?,
    "exec" => {
      prereqs::check()?;
      let server_proc = check::images::start_server()?;
      let db = Arc::new(Mutex::new(db_client));
      exec::run(db, &config, &http_client).await?;
      check::images::cleanup(server_proc)?;
    }
    "check-domain" => {
      prereqs::check()?;
      let domain = args.next().expect("missing required [domain] arg");
      let server_proc = check::images::start_server()?;
      let result = check::domain(&domain, &config, &http_client).await;
      check::images::cleanup(server_proc)?;
      println!("\nresult: {result}");
    }
    _ => panic!("unknown command: `{cmd}`"),
  }

  Ok(())
}

fn build_http_client() -> HttpClient {
  reqwest::Client::builder()
    .user_agent(USER_AGENT)
    .redirect(reqwest::redirect::Policy::none())
    .timeout(Duration::from_secs(4))
    .build()
    .unwrap()
}

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
