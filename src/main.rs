mod bootstrap;
mod check;
mod config;
mod db;
mod exec;
mod html;
mod http;
mod internal;
mod prereqs;
mod shared;
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
  let config = config::load();

  match cmd.as_str() {
    "bootstrap" => bootstrap::run(&db_client).await?,
    "exec" => {
      prereqs::check()?;
      let server_proc = check::images::start_server()?;
      let db = Arc::new(Mutex::new(db_client));
      exec::run(db, &config).await?;
      check::images::cleanup(server_proc)?;
    }
    "check-domain" => {
      prereqs::check()?;
      let domain = args.next().expect("missing required [domain] arg");
      let server_proc = check::images::start_server()?;
      let result = check::domain(&domain, &config).await;
      check::images::cleanup(server_proc)?;
      println!("\nresult: {result}");
    }
    _ => panic!("unknown command: `{cmd}`"),
  }

  Ok(())
}
